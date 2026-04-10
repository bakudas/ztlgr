use std::fmt;
use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Role in a conversation message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
        }
    }
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Token usage statistics from a completion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt (input).
    pub prompt_tokens: u32,
    /// Tokens generated (output).
    pub completion_tokens: u32,
    /// Total tokens consumed.
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

/// Request to an LLM provider.
#[derive(Debug, Clone)]
pub struct LlmRequest {
    /// Conversation messages (may include system, user, assistant turns).
    pub messages: Vec<Message>,
    /// Maximum tokens to generate. If `None`, uses provider/config default.
    pub max_tokens: Option<u32>,
    /// Sampling temperature. If `None`, uses provider/config default.
    pub temperature: Option<f32>,
}

impl LlmRequest {
    /// Create a simple single-turn request with a user message.
    pub fn simple(prompt: impl Into<String>) -> Self {
        Self {
            messages: vec![Message::user(prompt)],
            max_tokens: None,
            temperature: None,
        }
    }

    /// Create a request with a system prompt and user message.
    pub fn with_system(system: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            messages: vec![Message::system(system), Message::user(prompt)],
            max_tokens: None,
            temperature: None,
        }
    }

    /// Set the maximum tokens to generate.
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set the sampling temperature.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Add a message to the conversation.
    pub fn add_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }
}

/// Response from an LLM provider.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// The generated text content.
    pub content: String,
    /// The model that produced this response.
    pub model: String,
    /// Token usage statistics (may be zeros if provider doesn't report).
    pub usage: TokenUsage,
}

/// Trait for LLM provider implementations.
///
/// Each provider (Ollama, OpenAI, Anthropic) implements this trait.
/// The `complete` method sends a request and returns a response.
/// Providers are constructed from an `LlmConfig` and hold their own
/// HTTP client and configuration.
pub trait LlmProvider: Send + Sync {
    /// Send a completion request and return the response.
    fn complete(
        &self,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse>> + Send + '_>>;

    /// Return the provider name (e.g. "ollama", "openai", "anthropic").
    fn name(&self) -> &str;

    /// Return the configured model name.
    fn model(&self) -> &str;

    /// Check if the provider is reachable / properly configured.
    /// Returns `Ok(true)` if healthy, `Ok(false)` or `Err` otherwise.
    fn health_check(&self) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_display() {
        assert_eq!(Role::System.to_string(), "system");
        assert_eq!(Role::User.to_string(), "user");
        assert_eq!(Role::Assistant.to_string(), "assistant");
    }

    #[test]
    fn test_role_equality() {
        assert_eq!(Role::System, Role::System);
        assert_ne!(Role::User, Role::Assistant);
    }

    #[test]
    fn test_message_system() {
        let msg = Message::system("You are a helpful assistant.");
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.content, "You are a helpful assistant.");
    }

    #[test]
    fn test_message_user() {
        let msg = Message::user("Hello, world!");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello, world!");
    }

    #[test]
    fn test_message_assistant() {
        let msg = Message::assistant("Hi there!");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "Hi there!");
    }

    #[test]
    fn test_token_usage_new() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_token_usage_default() {
        let usage = TokenUsage::default();
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.completion_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn test_llm_request_simple() {
        let req = LlmRequest::simple("What is Rust?");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, Role::User);
        assert_eq!(req.messages[0].content, "What is Rust?");
        assert!(req.max_tokens.is_none());
        assert!(req.temperature.is_none());
    }

    #[test]
    fn test_llm_request_with_system() {
        let req = LlmRequest::with_system("Be concise.", "Explain monads.");
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0].role, Role::System);
        assert_eq!(req.messages[0].content, "Be concise.");
        assert_eq!(req.messages[1].role, Role::User);
        assert_eq!(req.messages[1].content, "Explain monads.");
    }

    #[test]
    fn test_llm_request_builder_max_tokens() {
        let req = LlmRequest::simple("test").max_tokens(1024);
        assert_eq!(req.max_tokens, Some(1024));
    }

    #[test]
    fn test_llm_request_builder_temperature() {
        let req = LlmRequest::simple("test").temperature(0.5);
        assert_eq!(req.temperature, Some(0.5));
    }

    #[test]
    fn test_llm_request_builder_chained() {
        let req = LlmRequest::with_system("system", "user")
            .max_tokens(2048)
            .temperature(0.3)
            .add_message(Message::assistant("previous"))
            .add_message(Message::user("followup"));

        assert_eq!(req.messages.len(), 4);
        assert_eq!(req.max_tokens, Some(2048));
        assert_eq!(req.temperature, Some(0.3));
        assert_eq!(req.messages[2].role, Role::Assistant);
        assert_eq!(req.messages[3].role, Role::User);
    }

    #[test]
    fn test_llm_response_fields() {
        let resp = LlmResponse {
            content: "Hello!".to_string(),
            model: "llama3".to_string(),
            usage: TokenUsage::new(10, 5),
        };
        assert_eq!(resp.content, "Hello!");
        assert_eq!(resp.model, "llama3");
        assert_eq!(resp.usage.total_tokens, 15);
    }

    #[test]
    fn test_role_serialization() {
        let json = serde_json::to_string(&Role::User).unwrap();
        assert_eq!(json, "\"user\"");

        let role: Role = serde_json::from_str("\"system\"").unwrap();
        assert_eq!(role, Role::System);
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("test");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"test\""));

        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, Role::User);
        assert_eq!(deserialized.content, "test");
    }

    #[test]
    fn test_token_usage_serialization() {
        let usage = TokenUsage::new(100, 200);
        let json = serde_json::to_string(&usage).unwrap();
        let deserialized: TokenUsage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.prompt_tokens, 100);
        assert_eq!(deserialized.completion_tokens, 200);
        assert_eq!(deserialized.total_tokens, 300);
    }
}
