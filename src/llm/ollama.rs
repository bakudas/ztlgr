use std::future::Future;
use std::pin::Pin;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::LlmConfig;
use crate::error::{Result, ZtlgrError};

use super::provider::{LlmProvider, LlmRequest, LlmResponse, TokenUsage};

const DEFAULT_OLLAMA_BASE: &str = "http://localhost:11434";

/// Ollama provider for local LLM inference.
///
/// Ollama runs models locally with zero network dependency and no API key.
/// This is the default (local-first) provider for ztlgr.
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl OllamaProvider {
    /// Create a new Ollama provider from config.
    pub fn new(config: &LlmConfig) -> Self {
        let base_url = if config.api_base.is_empty() {
            DEFAULT_OLLAMA_BASE.to_string()
        } else {
            config.api_base.trim_end_matches('/').to_string()
        };

        Self {
            client: Client::new(),
            base_url,
            model: config.model.clone(),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
        }
    }

    /// Create a provider with explicit parameters (useful for testing).
    pub fn with_params(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            model: model.into(),
            max_tokens: 4096,
            temperature: 0.7,
        }
    }

    /// Build the Ollama chat API request body.
    fn build_request_body(&self, request: &LlmRequest) -> OllamaChatRequest {
        let messages: Vec<OllamaMessage> = request
            .messages
            .iter()
            .map(|m| OllamaMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
            })
            .collect();

        let max_tokens = request.max_tokens.unwrap_or(self.max_tokens);
        let temperature = request.temperature.unwrap_or(self.temperature);

        OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            options: OllamaOptions {
                num_predict: max_tokens,
                temperature,
            },
        }
    }

    /// Parse the Ollama chat API response.
    fn parse_response(body: OllamaChatResponse) -> LlmResponse {
        let usage = if let Some(prompt_tokens) = body.prompt_eval_count {
            let completion_tokens = body.eval_count.unwrap_or(0);
            TokenUsage::new(prompt_tokens, completion_tokens)
        } else {
            TokenUsage::default()
        };

        LlmResponse {
            content: body.message.content,
            model: body.model,
            usage,
        }
    }
}

impl LlmProvider for OllamaProvider {
    fn complete(
        &self,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse>> + Send + '_>> {
        let body = self.build_request_body(request);
        let url = format!("{}/api/chat", self.base_url);

        Box::pin(async move {
            let resp = self
                .client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| ZtlgrError::LlmProvider(format!("Ollama request failed: {}", e)))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown error".to_string());
                return Err(ZtlgrError::LlmProvider(format!(
                    "Ollama returned {}: {}",
                    status, text
                )));
            }

            let chat_resp: OllamaChatResponse = resp
                .json()
                .await
                .map_err(|e| ZtlgrError::LlmProvider(format!("Failed to parse response: {}", e)))?;

            Ok(Self::parse_response(chat_resp))
        })
    }

    fn name(&self) -> &str {
        "ollama"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn health_check(&self) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>> {
        let url = format!("{}/api/tags", self.base_url);

        Box::pin(async move {
            match self.client.get(&url).send().await {
                Ok(resp) => Ok(resp.status().is_success()),
                Err(_) => Ok(false),
            }
        })
    }
}

// --- Ollama API types (private) ---

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    num_predict: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    model: String,
    message: OllamaResponseMessage,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::provider::Message;

    #[test]
    fn test_ollama_provider_new_defaults() {
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            api_base: String::new(),
            api_key_env: String::new(),
            max_tokens: 4096,
            temperature: 0.7,
        };

        let provider = OllamaProvider::new(&config);
        assert_eq!(provider.base_url, "http://localhost:11434");
        assert_eq!(provider.model, "llama3");
        assert_eq!(provider.max_tokens, 4096);
        assert!((provider.temperature - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ollama_provider_custom_base() {
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "mistral".to_string(),
            api_base: "http://192.168.1.100:11434/".to_string(),
            api_key_env: String::new(),
            max_tokens: 2048,
            temperature: 0.5,
        };

        let provider = OllamaProvider::new(&config);
        // Trailing slash should be stripped
        assert_eq!(provider.base_url, "http://192.168.1.100:11434");
        assert_eq!(provider.model, "mistral");
    }

    #[test]
    fn test_ollama_provider_with_params() {
        let provider = OllamaProvider::with_params("http://localhost:11434", "codellama");
        assert_eq!(provider.base_url, "http://localhost:11434");
        assert_eq!(provider.model, "codellama");
        assert_eq!(provider.max_tokens, 4096);
    }

    #[test]
    fn test_ollama_provider_name() {
        let provider = OllamaProvider::with_params("http://localhost:11434", "llama3");
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_ollama_provider_model() {
        let provider = OllamaProvider::with_params("http://localhost:11434", "llama3");
        assert_eq!(provider.model(), "llama3");
    }

    #[test]
    fn test_ollama_build_request_body_simple() {
        let provider = OllamaProvider::with_params("http://localhost:11434", "llama3");
        let request = LlmRequest::simple("Hello");

        let body = provider.build_request_body(&request);

        assert_eq!(body.model, "llama3");
        assert_eq!(body.messages.len(), 1);
        assert_eq!(body.messages[0].role, "user");
        assert_eq!(body.messages[0].content, "Hello");
        assert!(!body.stream);
        assert_eq!(body.options.num_predict, 4096);
        assert!((body.options.temperature - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ollama_build_request_body_with_system() {
        let provider = OllamaProvider::with_params("http://localhost:11434", "llama3");
        let request = LlmRequest::with_system("Be concise.", "Explain monads.");

        let body = provider.build_request_body(&request);

        assert_eq!(body.messages.len(), 2);
        assert_eq!(body.messages[0].role, "system");
        assert_eq!(body.messages[0].content, "Be concise.");
        assert_eq!(body.messages[1].role, "user");
    }

    #[test]
    fn test_ollama_build_request_body_overrides() {
        let provider = OllamaProvider::with_params("http://localhost:11434", "llama3");
        let request = LlmRequest::simple("test").max_tokens(1024).temperature(0.2);

        let body = provider.build_request_body(&request);

        assert_eq!(body.options.num_predict, 1024);
        assert!((body.options.temperature - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ollama_parse_response_with_usage() {
        let raw = OllamaChatResponse {
            model: "llama3".to_string(),
            message: OllamaResponseMessage {
                content: "Hello!".to_string(),
            },
            prompt_eval_count: Some(10),
            eval_count: Some(5),
        };

        let resp = OllamaProvider::parse_response(raw);
        assert_eq!(resp.content, "Hello!");
        assert_eq!(resp.model, "llama3");
        assert_eq!(resp.usage.prompt_tokens, 10);
        assert_eq!(resp.usage.completion_tokens, 5);
        assert_eq!(resp.usage.total_tokens, 15);
    }

    #[test]
    fn test_ollama_parse_response_without_usage() {
        let raw = OllamaChatResponse {
            model: "llama3".to_string(),
            message: OllamaResponseMessage {
                content: "Hi!".to_string(),
            },
            prompt_eval_count: None,
            eval_count: None,
        };

        let resp = OllamaProvider::parse_response(raw);
        assert_eq!(resp.content, "Hi!");
        assert_eq!(resp.usage.prompt_tokens, 0);
        assert_eq!(resp.usage.completion_tokens, 0);
        assert_eq!(resp.usage.total_tokens, 0);
    }

    #[test]
    fn test_ollama_build_request_multi_turn() {
        let provider = OllamaProvider::with_params("http://localhost:11434", "llama3");
        let request = LlmRequest::with_system("system", "first")
            .add_message(Message::assistant("response"))
            .add_message(Message::user("followup"));

        let body = provider.build_request_body(&request);

        assert_eq!(body.messages.len(), 4);
        assert_eq!(body.messages[0].role, "system");
        assert_eq!(body.messages[1].role, "user");
        assert_eq!(body.messages[2].role, "assistant");
        assert_eq!(body.messages[3].role, "user");
    }
}
