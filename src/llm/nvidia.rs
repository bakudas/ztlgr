use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::LlmConfig;
use crate::error::{Result, ZtlgrError};

use super::provider::{LlmProvider, LlmRequest, LlmResponse, TokenUsage};

const DEFAULT_NVIDIA_BASE: &str = "https://integrate.api.nvidia.com/v1";

/// NVIDIA NIM API provider.
pub struct NvidiaProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl NvidiaProvider {
    pub fn new(config: &LlmConfig) -> Result<Self> {
        let api_key = std::env::var(&config.api_key_env).map_err(|_| {
            ZtlgrError::LlmProvider(format!(
                "Environment variable '{}' not set (required for NVIDIA API key)",
                config.api_key_env
            ))
        })?;

        let base_url = if config.api_base.is_empty() {
            DEFAULT_NVIDIA_BASE.to_string()
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

    fn build_request(&self, request: &LlmRequest) -> NvidiaRequest {
        let messages: Vec<NvidiaMessage> = request
            .messages
            .iter()
            .map(|msg| NvidiaMessage {
                role: match msg.role {
                    super::provider::Role::System => "system".to_string(),
                    super::provider::Role::User => "user".to_string(),
                    super::provider::Role::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        NvidiaRequest {
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
        request: &NvidiaRequest,
    ) -> Result<NvidiaResponse> {
        let url = format!("{}/chat/completions", base_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(request)
            .send()
            .await
            .map_err(|e| ZtlgrError::LlmProvider(format!("NVIDIA API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ZtlgrError::LlmProvider(format!(
                "NVIDIA API error ({}): {}",
                status, body
            )));
        }

        response
            .json()
            .await
            .map_err(|e| ZtlgrError::LlmProvider(format!("Failed to parse NVIDIA response: {}", e)))
    }

    fn parse_response(response: NvidiaResponse) -> Result<LlmResponse> {
        let choice = response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| ZtlgrError::LlmProvider("NVIDIA returned no choices".to_string()))?;

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

impl LlmProvider for NvidiaProvider {
    fn complete(
        &self,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse>> + Send + '_>> {
        let nvidia_request = self.build_request(request);
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        Box::pin(async move {
            let response =
                Self::send_request(&client, &base_url, &api_key, &nvidia_request).await?;
            Self::parse_response(response)
        })
    }

    fn name(&self) -> &str {
        "nvidia"
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
                    ZtlgrError::LlmProvider(format!("NVIDIA health check failed: {}", e))
                })?;
            Ok(response.status().is_success())
        })
    }
}

#[derive(Serialize)]
struct NvidiaRequest {
    model: String,
    messages: Vec<NvidiaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct NvidiaMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct NvidiaResponse {
    choices: Vec<NvidiaChoice>,
    model: String,
    usage: Option<NvidiaUsage>,
}

#[derive(Deserialize)]
struct NvidiaChoice {
    message: NvidiaMessageResponse,
}

#[derive(Deserialize)]
struct NvidiaMessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct NvidiaUsage {
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
            provider: "nvidia".to_string(),
            model: "meta/llama-3.1-8b-instruct".to_string(),
            api_base: String::new(),
            api_key_env: "NVIDIA_API_KEY".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
        }
    }

    #[test]
    fn test_nvidia_provider_name() {
        std::env::set_var("NVIDIA_API_KEY", "test-key");
        let config = test_config();
        let provider = NvidiaProvider::new(&config).unwrap();
        assert_eq!(provider.name(), "nvidia");
        assert_eq!(provider.model(), "meta/llama-3.1-8b-instruct");
    }

    #[test]
    fn test_nvidia_request_building() {
        std::env::set_var("NVIDIA_API_KEY", "test-key");
        let config = test_config();
        let provider = NvidiaProvider::new(&config).unwrap();

        let request = LlmRequest {
            messages: vec![Message {
                role: Role::User,
                content: "Hello".to_string(),
            }],
            max_tokens: None,
            temperature: None,
        };

        let nv_req = provider.build_request(&request);
        assert_eq!(nv_req.model, "meta/llama-3.1-8b-instruct");
        assert_eq!(nv_req.messages.len(), 1);
        assert!(nv_req.max_tokens.is_some());
    }

    #[test]
    fn test_nvidia_missing_api_key() {
        std::env::remove_var("NVIDIA_KEY");
        let config = LlmConfig {
            api_key_env: "NVIDIA_KEY".to_string(),
            ..test_config()
        };
        let result = NvidiaProvider::new(&config);
        assert!(result.is_err());
    }
}
