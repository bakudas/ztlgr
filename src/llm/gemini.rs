use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::LlmConfig;
use crate::error::{Result, ZtlgrError};

use super::provider::{LlmProvider, LlmRequest, LlmResponse, TokenUsage};

const DEFAULT_GEMINI_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Google Gemini API provider.
pub struct GeminiProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl GeminiProvider {
    pub fn new(config: &LlmConfig) -> Result<Self> {
        let api_key = std::env::var(&config.api_key_env).map_err(|_| {
            ZtlgrError::LlmProvider(format!(
                "Environment variable '{}' not set (required for Gemini API key)",
                config.api_key_env
            ))
        })?;

        let base_url = if config.api_base.is_empty() {
            DEFAULT_GEMINI_BASE.to_string()
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

    fn build_request(&self, request: &LlmRequest) -> GeminiRequest {
        let contents: Vec<GeminiContent> = request
            .messages
            .iter()
            .map(|msg| GeminiContent {
                role: match msg.role {
                    super::provider::Role::System => "user".to_string(),
                    super::provider::Role::User => "user".to_string(),
                    super::provider::Role::Assistant => "model".to_string(),
                },
                parts: vec![GeminiPart {
                    text: msg.content.clone(),
                }],
            })
            .collect();

        GeminiRequest {
            contents,
            generation_config: Some(GeminiGenerationConfig {
                max_output_tokens: Some(self.max_tokens),
                temperature: Some(self.temperature),
            }),
        }
    }

    async fn send_request(
        client: &Client,
        base_url: &str,
        api_key: &str,
        model: &str,
        request: &GeminiRequest,
    ) -> Result<GeminiResponse> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            base_url, model, api_key
        );

        let response =
            client.post(&url).json(request).send().await.map_err(|e| {
                ZtlgrError::LlmProvider(format!("Gemini API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ZtlgrError::LlmProvider(format!(
                "Gemini API error ({}): {}",
                status, body
            )));
        }

        response
            .json()
            .await
            .map_err(|e| ZtlgrError::LlmProvider(format!("Failed to parse Gemini response: {}", e)))
    }

    fn parse_response(response: GeminiResponse, model: &str) -> Result<LlmResponse> {
        let candidate =
            response.candidates.into_iter().next().ok_or_else(|| {
                ZtlgrError::LlmProvider("Gemini returned no candidates".to_string())
            })?;

        let content = candidate
            .content
            .parts
            .into_iter()
            .next()
            .map(|p| p.text)
            .unwrap_or_default();

        let usage = response.usage_metadata.map_or(
            TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            |m| TokenUsage {
                prompt_tokens: m.prompt_token_count,
                completion_tokens: m.candidates_token_count,
                total_tokens: m.total_token_count,
            },
        );

        Ok(LlmResponse {
            content,
            model: model.to_string(),
            usage,
        })
    }
}

impl LlmProvider for GeminiProvider {
    fn complete(
        &self,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse>> + Send + '_>> {
        let gemini_request = self.build_request(request);
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let client = self.client.clone();

        Box::pin(async move {
            let response =
                Self::send_request(&client, &base_url, &api_key, &model, &gemini_request).await?;
            Self::parse_response(response, &model)
        })
    }

    fn name(&self) -> &str {
        "gemini"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn health_check(&self) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>> {
        let url = format!(
            "{}/models/{}?key={}",
            self.base_url, self.model, self.api_key
        );
        let client = self.client.clone();

        Box::pin(async move {
            let response = client.get(&url).send().await.map_err(|e| {
                ZtlgrError::LlmProvider(format!("Gemini health check failed: {}", e))
            })?;
            Ok(response.status().is_success())
        })
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(default)]
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
}

#[derive(Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

#[derive(Deserialize)]
struct GeminiPartResponse {
    text: String,
}

#[derive(Deserialize)]
struct GeminiUsageMetadata {
    prompt_token_count: u32,
    candidates_token_count: u32,
    total_token_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::provider::{LlmRequest, Message, Role};

    fn test_config() -> LlmConfig {
        LlmConfig {
            enabled: true,
            provider: "gemini".to_string(),
            model: "gemini-2.0-flash".to_string(),
            api_base: String::new(),
            api_key_env: "GOOGLE_API_KEY".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
        }
    }

    #[test]
    fn test_gemini_provider_name() {
        std::env::set_var("GOOGLE_API_KEY", "test-key");
        let config = test_config();
        let provider = GeminiProvider::new(&config).unwrap();
        assert_eq!(provider.name(), "gemini");
        assert_eq!(provider.model(), "gemini-2.0-flash");
    }

    #[test]
    fn test_gemini_request_building() {
        std::env::set_var("GOOGLE_API_KEY", "test-key");
        let config = test_config();
        let provider = GeminiProvider::new(&config).unwrap();

        let request = LlmRequest {
            messages: vec![
                Message {
                    role: Role::System,
                    content: "You are helpful.".to_string(),
                },
                Message {
                    role: Role::User,
                    content: "Hello".to_string(),
                },
            ],
            max_tokens: None,
            temperature: None,
        };

        let gemini_req = provider.build_request(&request);
        assert_eq!(gemini_req.contents.len(), 2);
        assert!(gemini_req.generation_config.is_some());
    }

    #[test]
    fn test_gemini_missing_api_key() {
        std::env::remove_var("GOOGLE_API_KEY");
        let config = test_config();
        let result = GeminiProvider::new(&config);
        assert!(result.is_err());
    }
}
