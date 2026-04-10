use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::LlmConfig;
use crate::error::{Result, ZtlgrError};

use super::provider::{LlmProvider, LlmRequest, LlmResponse, TokenUsage};

const DEFAULT_OPENROUTER_BASE: &str = "https://openrouter.ai/api/v1";

/// OpenRouter API provider (supports multiple model providers).
pub struct OpenRouterProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl OpenRouterProvider {
    pub fn new(config: &LlmConfig) -> Result<Self> {
        let api_key = std::env::var(&config.api_key_env).map_err(|_| {
            ZtlgrError::LlmProvider(format!(
                "Environment variable '{}' not set (required for OpenRouter API key)",
                config.api_key_env
            ))
        })?;

        let base_url = if config.api_base.is_empty() {
            DEFAULT_OPENROUTER_BASE.to_string()
        } else {
            config.api_base.trim_end_matches('/').to_string()
        };

        Ok(Self {
            client: Client::new(),
            base_url,
            api_key,
            model: config.model.clone(),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
        })
    }

    fn build_request(&self, request: &LlmRequest) -> OpenRouterRequest {
        let messages: Vec<OpenRouterMessage> = request
            .messages
            .iter()
            .map(|msg| OpenRouterMessage {
                role: match msg.role {
                    super::provider::Role::System => "system".to_string(),
                    super::provider::Role::User => "user".to_string(),
                    super::provider::Role::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        OpenRouterRequest {
            model: self.model.clone(),
            messages,
            max_tokens: Some(self.max_tokens),
            temperature: Some(self.temperature),
        }
    }

    async fn send_request(
        client: &Client,
        base_url: &str,
        api_key: &str,
        request: &OpenRouterRequest,
    ) -> Result<OpenRouterResponse> {
        let url = format!("{}/chat/completions", base_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://github.com/ztlgr/ztlgr")
            .header("X-Title", "ztlgr")
            .json(request)
            .send()
            .await
            .map_err(|e| {
                ZtlgrError::LlmProvider(format!("OpenRouter API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ZtlgrError::LlmProvider(format!(
                "OpenRouter API error ({}): {}",
                status, body
            )));
        }

        response.json().await.map_err(|e| {
            ZtlgrError::LlmProvider(format!("Failed to parse OpenRouter response: {}", e))
        })
    }

    fn parse_response(response: OpenRouterResponse) -> Result<LlmResponse> {
        let choice =
            response.choices.into_iter().next().ok_or_else(|| {
                ZtlgrError::LlmProvider("OpenRouter returned no choices".to_string())
            })?;

        let usage = response.usage.map_or(
            TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            |u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            },
        );

        Ok(LlmResponse {
            content: choice.message.content,
            model: response.model,
            usage,
        })
    }
}

impl LlmProvider for OpenRouterProvider {
    fn complete(
        &self,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse>> + Send + '_>> {
        let openrouter_request = self.build_request(request);
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        Box::pin(async move {
            let response =
                Self::send_request(&client, &base_url, &api_key, &openrouter_request).await?;
            Self::parse_response(response)
        })
    }

    fn name(&self) -> &str {
        "openrouter"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn health_check(&self) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>> {
        let url = format!("{}/models", self.base_url);
        let client = self.client.clone();
        let api_key = self.api_key.clone();

        Box::pin(async move {
            let response = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| {
                    ZtlgrError::LlmProvider(format!("OpenRouter health check failed: {}", e))
                })?;
            Ok(response.status().is_success())
        })
    }
}

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
    model: String,
    usage: Option<OpenRouterUsage>,
}

#[derive(Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessageResponse,
}

#[derive(Deserialize)]
struct OpenRouterMessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::provider::{LlmRequest, Message, Role};

    fn test_config() -> LlmConfig {
        LlmConfig {
            enabled: true,
            provider: "openrouter".to_string(),
            model: "anthropic/claude-3.5-sonnet".to_string(),
            api_base: String::new(),
            api_key_env: "OPENROUTER_API_KEY".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
        }
    }

    #[test]
    fn test_openrouter_provider_name() {
        std::env::set_var("OPENROUTER_API_KEY", "test-key");
        let config = test_config();
        let provider = OpenRouterProvider::new(&config).unwrap();
        assert_eq!(provider.name(), "openrouter");
        assert_eq!(provider.model(), "anthropic/claude-3.5-sonnet");
    }

    #[test]
    fn test_openrouter_request_building() {
        std::env::set_var("OPENROUTER_API_KEY", "test-key");
        let config = test_config();
        let provider = OpenRouterProvider::new(&config).unwrap();

        let request = LlmRequest {
            messages: vec![Message {
                role: Role::User,
                content: "Hello".to_string(),
            }],
            max_tokens: None,
            temperature: None,
        };

        let or_req = provider.build_request(&request);
        assert_eq!(or_req.model, "anthropic/claude-3.5-sonnet");
        assert_eq!(or_req.messages.len(), 1);
        assert!(or_req.max_tokens.is_some());
    }

    #[test]
    fn test_openrouter_missing_api_key() {
        std::env::remove_var("OPENROUTER_KEY");
        let config = LlmConfig {
            api_key_env: "OPENROUTER_KEY".to_string(),
            ..test_config()
        };
        let result = OpenRouterProvider::new(&config);
        assert!(result.is_err());
    }
}
