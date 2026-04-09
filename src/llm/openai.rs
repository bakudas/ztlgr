use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::LlmConfig;
use crate::error::{Result, ZtlgrError};

use super::provider::{LlmProvider, LlmRequest, LlmResponse, TokenUsage};

const DEFAULT_OPENAI_BASE: &str = "https://api.openai.com/v1";

/// OpenAI provider for GPT-4o, o3, and other OpenAI models.
///
/// Requires an API key via the environment variable named in `api_key_env`.
pub struct OpenAiProvider {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
    max_tokens: u32,
    temperature: f32,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider from config.
    ///
    /// Reads the API key from the environment variable specified in
    /// `config.api_key_env`. Returns an error if the env var is missing.
    pub fn new(config: &LlmConfig) -> Result<Self> {
        let api_key = Self::resolve_api_key(&config.api_key_env)?;

        let base_url = if config.api_base.is_empty() {
            DEFAULT_OPENAI_BASE.to_string()
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
                "OpenAI requires api_key_env to be set in [llm] config".to_string(),
            ));
        }

        std::env::var(env_var_name).map_err(|_| {
            ZtlgrError::LlmProvider(format!(
                "Environment variable '{}' not set (required for OpenAI API key)",
                env_var_name
            ))
        })
    }

    /// Build the OpenAI chat completions request body.
    fn build_request_body(&self, request: &LlmRequest) -> OpenAiChatRequest {
        let messages: Vec<OpenAiMessage> = request
            .messages
            .iter()
            .map(|m| OpenAiMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
            })
            .collect();

        let max_tokens = request.max_tokens.unwrap_or(self.max_tokens);
        let temperature = request.temperature.unwrap_or(self.temperature);

        OpenAiChatRequest {
            model: self.model.clone(),
            messages,
            max_tokens,
            temperature,
        }
    }

    /// Parse the OpenAI chat completions response.
    fn parse_response(body: OpenAiChatResponse) -> Result<LlmResponse> {
        let choice = body
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| ZtlgrError::LlmProvider("OpenAI returned no choices".to_string()))?;

        let usage = if let Some(u) = body.usage {
            TokenUsage::new(u.prompt_tokens, u.completion_tokens)
        } else {
            TokenUsage::default()
        };

        Ok(LlmResponse {
            content: choice.message.content,
            model: body.model,
            usage,
        })
    }
}

impl LlmProvider for OpenAiProvider {
    fn complete(
        &self,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse>> + Send + '_>> {
        let body = self.build_request_body(request);
        let url = format!("{}/chat/completions", self.base_url);

        Box::pin(async move {
            let resp = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| ZtlgrError::LlmProvider(format!("OpenAI request failed: {}", e)))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown error".to_string());
                return Err(ZtlgrError::LlmProvider(format!(
                    "OpenAI returned {}: {}",
                    status, text
                )));
            }

            let chat_resp: OpenAiChatResponse = resp
                .json()
                .await
                .map_err(|e| ZtlgrError::LlmProvider(format!("Failed to parse response: {}", e)))?;

            Self::parse_response(chat_resp)
        })
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn health_check(&self) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>> {
        let url = format!("{}/models", self.base_url);

        Box::pin(async move {
            match self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .send()
                .await
            {
                Ok(resp) => Ok(resp.status().is_success()),
                Err(_) => Ok(false),
            }
        })
    }
}

// --- OpenAI API types (private) ---

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::provider::Message;

    #[test]
    fn test_openai_provider_with_params() {
        let provider =
            OpenAiProvider::with_params("https://api.openai.com/v1", "gpt-4o", "sk-test123");
        assert_eq!(provider.base_url, "https://api.openai.com/v1");
        assert_eq!(provider.model, "gpt-4o");
        assert_eq!(provider.api_key, "sk-test123");
    }

    #[test]
    fn test_openai_provider_name() {
        let provider = OpenAiProvider::with_params("http://test", "gpt-4o", "key");
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_openai_provider_model() {
        let provider = OpenAiProvider::with_params("http://test", "gpt-4o-mini", "key");
        assert_eq!(provider.model(), "gpt-4o-mini");
    }

    #[test]
    fn test_openai_resolve_api_key_empty_env_name() {
        let result = OpenAiProvider::resolve_api_key("");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("api_key_env"));
    }

    #[test]
    fn test_openai_resolve_api_key_missing_env() {
        let result = OpenAiProvider::resolve_api_key("ZTLGR_TEST_NONEXISTENT_KEY_12345");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("ZTLGR_TEST_NONEXISTENT_KEY_12345"));
    }

    #[test]
    fn test_openai_resolve_api_key_set_env() {
        // Set a temporary env var for this test
        std::env::set_var("ZTLGR_TEST_OPENAI_KEY", "sk-test-value");
        let result = OpenAiProvider::resolve_api_key("ZTLGR_TEST_OPENAI_KEY");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "sk-test-value");
        std::env::remove_var("ZTLGR_TEST_OPENAI_KEY");
    }

    #[test]
    fn test_openai_new_missing_key() {
        let config = LlmConfig {
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_base: String::new(),
            api_key_env: String::new(),
            max_tokens: 4096,
            temperature: 0.7,
        };
        let result = OpenAiProvider::new(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_openai_new_with_env_key() {
        std::env::set_var("ZTLGR_TEST_OPENAI_NEW", "sk-from-env");
        let config = LlmConfig {
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_base: String::new(),
            api_key_env: "ZTLGR_TEST_OPENAI_NEW".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        };
        let result = OpenAiProvider::new(&config);
        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.base_url, "https://api.openai.com/v1");
        assert_eq!(provider.api_key, "sk-from-env");
        std::env::remove_var("ZTLGR_TEST_OPENAI_NEW");
    }

    #[test]
    fn test_openai_custom_base_url() {
        let config = LlmConfig {
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_base: "https://custom-proxy.example.com/v1/".to_string(),
            api_key_env: "ZTLGR_TEST_OPENAI_CUSTOM".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        };
        std::env::set_var("ZTLGR_TEST_OPENAI_CUSTOM", "sk-custom");
        let provider = OpenAiProvider::new(&config).unwrap();
        assert_eq!(provider.base_url, "https://custom-proxy.example.com/v1");
        std::env::remove_var("ZTLGR_TEST_OPENAI_CUSTOM");
    }

    #[test]
    fn test_openai_build_request_body() {
        let provider = OpenAiProvider::with_params("http://test", "gpt-4o", "key");
        let request = LlmRequest::with_system("Be concise.", "What is Rust?");

        let body = provider.build_request_body(&request);

        assert_eq!(body.model, "gpt-4o");
        assert_eq!(body.messages.len(), 2);
        assert_eq!(body.messages[0].role, "system");
        assert_eq!(body.messages[1].role, "user");
        assert_eq!(body.max_tokens, 4096);
    }

    #[test]
    fn test_openai_build_request_overrides() {
        let provider = OpenAiProvider::with_params("http://test", "gpt-4o", "key");
        let request = LlmRequest::simple("test").max_tokens(512).temperature(0.1);

        let body = provider.build_request_body(&request);

        assert_eq!(body.max_tokens, 512);
        assert!((body.temperature - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_openai_parse_response_success() {
        let raw = OpenAiChatResponse {
            model: "gpt-4o-2024-05-13".to_string(),
            choices: vec![OpenAiChoice {
                message: OpenAiResponseMessage {
                    content: "Hello there!".to_string(),
                },
            }],
            usage: Some(OpenAiUsage {
                prompt_tokens: 20,
                completion_tokens: 10,
            }),
        };

        let resp = OpenAiProvider::parse_response(raw).unwrap();
        assert_eq!(resp.content, "Hello there!");
        assert_eq!(resp.model, "gpt-4o-2024-05-13");
        assert_eq!(resp.usage.prompt_tokens, 20);
        assert_eq!(resp.usage.completion_tokens, 10);
        assert_eq!(resp.usage.total_tokens, 30);
    }

    #[test]
    fn test_openai_parse_response_no_choices() {
        let raw = OpenAiChatResponse {
            model: "gpt-4o".to_string(),
            choices: vec![],
            usage: None,
        };

        let result = OpenAiProvider::parse_response(raw);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no choices"));
    }

    #[test]
    fn test_openai_parse_response_no_usage() {
        let raw = OpenAiChatResponse {
            model: "gpt-4o".to_string(),
            choices: vec![OpenAiChoice {
                message: OpenAiResponseMessage {
                    content: "Response".to_string(),
                },
            }],
            usage: None,
        };

        let resp = OpenAiProvider::parse_response(raw).unwrap();
        assert_eq!(resp.usage.total_tokens, 0);
    }

    #[test]
    fn test_openai_build_request_multi_turn() {
        let provider = OpenAiProvider::with_params("http://test", "gpt-4o", "key");
        let request = LlmRequest::with_system("system", "hello")
            .add_message(Message::assistant("hi"))
            .add_message(Message::user("how are you?"));

        let body = provider.build_request_body(&request);

        assert_eq!(body.messages.len(), 4);
        assert_eq!(body.messages[0].role, "system");
        assert_eq!(body.messages[1].role, "user");
        assert_eq!(body.messages[2].role, "assistant");
        assert_eq!(body.messages[3].role, "user");
    }
}
