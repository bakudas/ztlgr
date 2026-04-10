use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::LlmConfig;
use crate::error::{Result, ZtlgrError};

use super::provider::{LlmProvider, LlmRequest, LlmResponse, Role, TokenUsage};

const DEFAULT_ANTHROPIC_BASE: &str = "https://api.anthropic.com";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Anthropic provider for Claude models.
///
/// Requires an API key via the environment variable named in `api_key_env`.
/// Uses the Anthropic Messages API format where the system prompt is a
/// top-level field rather than a message.
pub struct AnthropicProvider {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
    max_tokens: u32,
    temperature: f32,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider from config.
    ///
    /// Reads the API key from the environment variable specified in
    /// `config.api_key_env`. Returns an error if the env var is missing.
    pub fn new(config: &LlmConfig) -> Result<Self> {
        let api_key = Self::resolve_api_key(&config.api_key_env)?;

        let base_url = if config.api_base.is_empty() {
            DEFAULT_ANTHROPIC_BASE.to_string()
        } else {
            config.api_base.trim_end_matches('/').to_string()
        };

        Ok(Self {
            client: Client::new(),
            base_url,
            model: config.model.clone(),
            api_key,
            max_tokens: config.max_tokens,
            temperature: config.temperature,
        })
    }

    /// Create a provider with explicit parameters (useful for testing).
    pub fn with_params(
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            model: model.into(),
            api_key: api_key.into(),
            max_tokens: 4096,
            temperature: 0.7,
        }
    }

    /// Resolve the API key from the named environment variable.
    fn resolve_api_key(env_var_name: &str) -> Result<String> {
        if env_var_name.is_empty() {
            return Err(ZtlgrError::LlmProvider(
                "Anthropic requires api_key_env to be set in [llm] config".to_string(),
            ));
        }

        std::env::var(env_var_name).map_err(|_| {
            ZtlgrError::LlmProvider(format!(
                "Environment variable '{}' not set (required for Anthropic API key)",
                env_var_name
            ))
        })
    }

    /// Build the Anthropic Messages API request body.
    ///
    /// Anthropic uses a different format than OpenAI: the system prompt
    /// is a top-level field, not a message. We extract it from the messages
    /// and send non-system messages in the `messages` array.
    fn build_request_body(&self, request: &LlmRequest) -> AnthropicMessagesRequest {
        let mut system = None;
        let mut messages = Vec::new();

        for msg in &request.messages {
            if msg.role == Role::System {
                // Anthropic takes system as a top-level field.
                // If multiple system messages exist, concatenate them.
                match &mut system {
                    Some(existing) => {
                        *existing = format!("{}\n\n{}", existing, msg.content);
                    }
                    None => {
                        system = Some(msg.content.clone());
                    }
                }
            } else {
                messages.push(AnthropicMessage {
                    role: msg.role.to_string(),
                    content: msg.content.clone(),
                });
            }
        }

        let max_tokens = request.max_tokens.unwrap_or(self.max_tokens);
        let temperature = request.temperature.unwrap_or(self.temperature);

        AnthropicMessagesRequest {
            model: self.model.clone(),
            messages,
            system,
            max_tokens,
            temperature,
        }
    }

    /// Parse the Anthropic Messages API response.
    fn parse_response(body: AnthropicMessagesResponse) -> Result<LlmResponse> {
        // Collect all text content blocks into a single string
        let content: String = body
            .content
            .iter()
            .filter_map(|block| {
                if block.content_type == "text" {
                    Some(block.text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        if content.is_empty() {
            return Err(ZtlgrError::LlmProvider(
                "Anthropic returned no text content blocks".to_string(),
            ));
        }

        let usage = TokenUsage::new(body.usage.input_tokens, body.usage.output_tokens);

        Ok(LlmResponse {
            content,
            model: body.model,
            usage,
        })
    }
}

impl LlmProvider for AnthropicProvider {
    fn complete(
        &self,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse>> + Send + '_>> {
        let body = self.build_request_body(request);
        let url = format!("{}/v1/messages", self.base_url);

        Box::pin(async move {
            let resp = self
                .client
                .post(&url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", ANTHROPIC_API_VERSION)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| ZtlgrError::LlmProvider(format!("Anthropic request failed: {}", e)))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown error".to_string());
                return Err(ZtlgrError::LlmProvider(format!(
                    "Anthropic returned {}: {}",
                    status, text
                )));
            }

            let msg_resp: AnthropicMessagesResponse = resp
                .json()
                .await
                .map_err(|e| ZtlgrError::LlmProvider(format!("Failed to parse response: {}", e)))?;

            Self::parse_response(msg_resp)
        })
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn health_check(&self) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>> {
        // Anthropic doesn't have a simple health endpoint, so we try listing models.
        // A 401 means the key is wrong but the service is up; a connection error means down.
        let url = format!("{}/v1/messages", self.base_url);

        Box::pin(async move {
            match self
                .client
                .post(&url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", ANTHROPIC_API_VERSION)
                .header("Content-Type", "application/json")
                .body("{}")
                .send()
                .await
            {
                Ok(_) => Ok(true), // Any response means the service is reachable
                Err(_) => Ok(false),
            }
        })
    }
}

// --- Anthropic API types (private) ---

#[derive(Debug, Serialize)]
struct AnthropicMessagesRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicMessagesResponse {
    model: String,
    content: Vec<AnthropicContentBlock>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::provider::Message;

    #[test]
    fn test_anthropic_provider_with_params() {
        let provider = AnthropicProvider::with_params(
            "https://api.anthropic.com",
            "claude-sonnet-4-20250514",
            "sk-ant-test",
        );
        assert_eq!(provider.base_url, "https://api.anthropic.com");
        assert_eq!(provider.model, "claude-sonnet-4-20250514");
        assert_eq!(provider.api_key, "sk-ant-test");
    }

    #[test]
    fn test_anthropic_provider_name() {
        let provider =
            AnthropicProvider::with_params("http://test", "claude-sonnet-4-20250514", "key");
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_anthropic_provider_model() {
        let provider = AnthropicProvider::with_params("http://test", "claude-haiku", "key");
        assert_eq!(provider.model(), "claude-haiku");
    }

    #[test]
    fn test_anthropic_resolve_api_key_empty() {
        let result = AnthropicProvider::resolve_api_key("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("api_key_env"));
    }

    #[test]
    fn test_anthropic_resolve_api_key_missing() {
        let result = AnthropicProvider::resolve_api_key("ZTLGR_TEST_ANTHROPIC_MISSING_99999");
        assert!(result.is_err());
    }

    #[test]
    fn test_anthropic_resolve_api_key_set() {
        std::env::set_var("ZTLGR_TEST_ANTHROPIC_KEY", "sk-ant-value");
        let result = AnthropicProvider::resolve_api_key("ZTLGR_TEST_ANTHROPIC_KEY");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "sk-ant-value");
        std::env::remove_var("ZTLGR_TEST_ANTHROPIC_KEY");
    }

    #[test]
    fn test_anthropic_new_missing_key() {
        let config = LlmConfig {
            enabled: true,
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_base: String::new(),
            api_key_env: String::new(),
            max_tokens: 4096,
            temperature: 0.7,
        };
        assert!(AnthropicProvider::new(&config).is_err());
    }

    #[test]
    fn test_anthropic_new_with_env_key() {
        std::env::set_var("ZTLGR_TEST_ANTHROPIC_NEW", "sk-ant-from-env");
        let config = LlmConfig {
            enabled: true,
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_base: String::new(),
            api_key_env: "ZTLGR_TEST_ANTHROPIC_NEW".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        };
        let provider = AnthropicProvider::new(&config).unwrap();
        assert_eq!(provider.base_url, "https://api.anthropic.com");
        assert_eq!(provider.api_key, "sk-ant-from-env");
        std::env::remove_var("ZTLGR_TEST_ANTHROPIC_NEW");
    }

    #[test]
    fn test_anthropic_build_request_simple() {
        let provider =
            AnthropicProvider::with_params("http://test", "claude-sonnet-4-20250514", "key");
        let request = LlmRequest::simple("What is Rust?");

        let body = provider.build_request_body(&request);

        assert_eq!(body.model, "claude-sonnet-4-20250514");
        assert!(body.system.is_none());
        assert_eq!(body.messages.len(), 1);
        assert_eq!(body.messages[0].role, "user");
        assert_eq!(body.messages[0].content, "What is Rust?");
    }

    #[test]
    fn test_anthropic_build_request_with_system() {
        let provider =
            AnthropicProvider::with_params("http://test", "claude-sonnet-4-20250514", "key");
        let request = LlmRequest::with_system("Be concise.", "Explain monads.");

        let body = provider.build_request_body(&request);

        // System should be extracted as top-level field
        assert_eq!(body.system, Some("Be concise.".to_string()));
        // Only the user message should remain in messages
        assert_eq!(body.messages.len(), 1);
        assert_eq!(body.messages[0].role, "user");
        assert_eq!(body.messages[0].content, "Explain monads.");
    }

    #[test]
    fn test_anthropic_build_request_multiple_system_messages() {
        let provider =
            AnthropicProvider::with_params("http://test", "claude-sonnet-4-20250514", "key");
        let request = LlmRequest {
            messages: vec![
                Message::system("First instruction."),
                Message::system("Second instruction."),
                Message::user("Hello"),
            ],
            max_tokens: None,
            temperature: None,
        };

        let body = provider.build_request_body(&request);

        // Multiple system messages should be concatenated
        assert_eq!(
            body.system,
            Some("First instruction.\n\nSecond instruction.".to_string())
        );
        assert_eq!(body.messages.len(), 1);
    }

    #[test]
    fn test_anthropic_build_request_overrides() {
        let provider =
            AnthropicProvider::with_params("http://test", "claude-sonnet-4-20250514", "key");
        let request = LlmRequest::simple("test").max_tokens(512).temperature(0.1);

        let body = provider.build_request_body(&request);

        assert_eq!(body.max_tokens, 512);
        assert!((body.temperature - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_anthropic_parse_response_single_block() {
        let raw = AnthropicMessagesResponse {
            model: "claude-sonnet-4-20250514".to_string(),
            content: vec![AnthropicContentBlock {
                content_type: "text".to_string(),
                text: "Hello!".to_string(),
            }],
            usage: AnthropicUsage {
                input_tokens: 15,
                output_tokens: 5,
            },
        };

        let resp = AnthropicProvider::parse_response(raw).unwrap();
        assert_eq!(resp.content, "Hello!");
        assert_eq!(resp.model, "claude-sonnet-4-20250514");
        assert_eq!(resp.usage.prompt_tokens, 15);
        assert_eq!(resp.usage.completion_tokens, 5);
        assert_eq!(resp.usage.total_tokens, 20);
    }

    #[test]
    fn test_anthropic_parse_response_multiple_blocks() {
        let raw = AnthropicMessagesResponse {
            model: "claude-sonnet-4-20250514".to_string(),
            content: vec![
                AnthropicContentBlock {
                    content_type: "text".to_string(),
                    text: "Part one. ".to_string(),
                },
                AnthropicContentBlock {
                    content_type: "text".to_string(),
                    text: "Part two.".to_string(),
                },
            ],
            usage: AnthropicUsage {
                input_tokens: 10,
                output_tokens: 8,
            },
        };

        let resp = AnthropicProvider::parse_response(raw).unwrap();
        assert_eq!(resp.content, "Part one. Part two.");
    }

    #[test]
    fn test_anthropic_parse_response_empty_content() {
        let raw = AnthropicMessagesResponse {
            model: "claude-sonnet-4-20250514".to_string(),
            content: vec![],
            usage: AnthropicUsage {
                input_tokens: 10,
                output_tokens: 0,
            },
        };

        let result = AnthropicProvider::parse_response(raw);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no text content"));
    }

    #[test]
    fn test_anthropic_parse_response_non_text_blocks_skipped() {
        let raw = AnthropicMessagesResponse {
            model: "claude-sonnet-4-20250514".to_string(),
            content: vec![
                AnthropicContentBlock {
                    content_type: "tool_use".to_string(),
                    text: String::new(),
                },
                AnthropicContentBlock {
                    content_type: "text".to_string(),
                    text: "Actual text.".to_string(),
                },
            ],
            usage: AnthropicUsage {
                input_tokens: 10,
                output_tokens: 5,
            },
        };

        let resp = AnthropicProvider::parse_response(raw).unwrap();
        assert_eq!(resp.content, "Actual text.");
    }

    #[test]
    fn test_anthropic_build_request_multi_turn() {
        let provider =
            AnthropicProvider::with_params("http://test", "claude-sonnet-4-20250514", "key");
        let request = LlmRequest::with_system("system", "hello")
            .add_message(Message::assistant("hi"))
            .add_message(Message::user("followup"));

        let body = provider.build_request_body(&request);

        assert_eq!(body.system, Some("system".to_string()));
        assert_eq!(body.messages.len(), 3);
        assert_eq!(body.messages[0].role, "user");
        assert_eq!(body.messages[1].role, "assistant");
        assert_eq!(body.messages[2].role, "user");
    }

    #[test]
    fn test_anthropic_custom_base_url() {
        std::env::set_var("ZTLGR_TEST_ANTHROPIC_CUSTOM", "sk-ant-custom");
        let config = LlmConfig {
            enabled: true,
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_base: "https://proxy.example.com/".to_string(),
            api_key_env: "ZTLGR_TEST_ANTHROPIC_CUSTOM".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        };
        let provider = AnthropicProvider::new(&config).unwrap();
        assert_eq!(provider.base_url, "https://proxy.example.com");
        std::env::remove_var("ZTLGR_TEST_ANTHROPIC_CUSTOM");
    }
}
