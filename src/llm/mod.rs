pub mod anthropic;
pub mod context;
pub mod ollama;
pub mod openai;
pub mod provider;
pub mod usage;
pub mod workflow;
pub mod workflows;

use crate::config::LlmConfig;
use crate::error::{Result, ZtlgrError};

pub use context::ContextBuilder;
pub use provider::{LlmProvider, LlmRequest, LlmResponse, Message, Role, TokenUsage};
pub use usage::UsageTracker;
pub use workflow::{WorkflowEngine, WorkflowResult};
pub use workflows::{
    IngestProcessResult, IngestWorkflow, LintReport, LintWorkflow, QueryResult, QueryWorkflow,
};

/// Supported LLM provider backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Ollama,
    OpenAi,
    Anthropic,
}

impl std::str::FromStr for ProviderKind {
    type Err = ZtlgrError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(ProviderKind::Ollama),
            "openai" => Ok(ProviderKind::OpenAi),
            "anthropic" => Ok(ProviderKind::Anthropic),
            other => Err(ZtlgrError::Llm(format!(
                "Unknown LLM provider: '{}'. Supported: ollama, openai, anthropic",
                other
            ))),
        }
    }
}

impl ProviderKind {
    /// Return the canonical name of this provider.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::Ollama => "ollama",
            ProviderKind::OpenAi => "openai",
            ProviderKind::Anthropic => "anthropic",
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Create an LLM provider from the application config.
///
/// Returns `Err` if the config specifies an unknown provider or if required
/// configuration (e.g. API key env var) is missing.
///
/// # Example
///
/// ```no_run
/// # use ztlgr::config::LlmConfig;
/// # use ztlgr::llm::create_provider;
/// let config = LlmConfig::default();
/// let provider = create_provider(&config).unwrap();
/// assert_eq!(provider.name(), "ollama");
/// ```
pub fn create_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>> {
    let kind: ProviderKind = config.provider.parse()?;

    match kind {
        ProviderKind::Ollama => Ok(Box::new(ollama::OllamaProvider::new(config))),
        ProviderKind::OpenAi => Ok(Box::new(openai::OpenAiProvider::new(config)?)),
        ProviderKind::Anthropic => Ok(Box::new(anthropic::AnthropicProvider::new(config)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ProviderKind tests ---

    #[test]
    fn test_provider_kind_from_str_ollama() {
        assert_eq!(
            "ollama".parse::<ProviderKind>().unwrap(),
            ProviderKind::Ollama
        );
    }

    #[test]
    fn test_provider_kind_from_str_openai() {
        assert_eq!(
            "openai".parse::<ProviderKind>().unwrap(),
            ProviderKind::OpenAi
        );
    }

    #[test]
    fn test_provider_kind_from_str_anthropic() {
        assert_eq!(
            "anthropic".parse::<ProviderKind>().unwrap(),
            ProviderKind::Anthropic
        );
    }

    #[test]
    fn test_provider_kind_from_str_case_insensitive() {
        assert_eq!(
            "Ollama".parse::<ProviderKind>().unwrap(),
            ProviderKind::Ollama
        );
        assert_eq!(
            "OPENAI".parse::<ProviderKind>().unwrap(),
            ProviderKind::OpenAi
        );
        assert_eq!(
            "Anthropic".parse::<ProviderKind>().unwrap(),
            ProviderKind::Anthropic
        );
    }

    #[test]
    fn test_provider_kind_from_str_unknown() {
        let result = "gemini".parse::<ProviderKind>();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown LLM provider"));
        assert!(err.contains("gemini"));
    }

    #[test]
    fn test_provider_kind_as_str() {
        assert_eq!(ProviderKind::Ollama.as_str(), "ollama");
        assert_eq!(ProviderKind::OpenAi.as_str(), "openai");
        assert_eq!(ProviderKind::Anthropic.as_str(), "anthropic");
    }

    #[test]
    fn test_provider_kind_display() {
        assert_eq!(ProviderKind::Ollama.to_string(), "ollama");
        assert_eq!(ProviderKind::OpenAi.to_string(), "openai");
        assert_eq!(ProviderKind::Anthropic.to_string(), "anthropic");
    }

    #[test]
    fn test_provider_kind_equality() {
        assert_eq!(ProviderKind::Ollama, ProviderKind::Ollama);
        assert_ne!(ProviderKind::Ollama, ProviderKind::OpenAi);
    }

    #[test]
    fn test_provider_kind_copy() {
        let kind = ProviderKind::Ollama;
        let copied = kind;
        assert_eq!(kind, copied); // Both still valid -- Copy trait
    }

    // --- Factory function tests ---

    #[test]
    fn test_create_provider_ollama() {
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            api_base: String::new(),
            api_key_env: String::new(),
            max_tokens: 4096,
            temperature: 0.7,
        };

        let provider = create_provider(&config).unwrap();
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.model(), "llama3");
    }

    #[test]
    fn test_create_provider_openai_requires_key() {
        let config = LlmConfig {
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_base: String::new(),
            api_key_env: String::new(), // No key env var
            max_tokens: 4096,
            temperature: 0.7,
        };

        let result = create_provider(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_provider_openai_with_key() {
        std::env::set_var("ZTLGR_TEST_CREATE_OPENAI", "sk-test");
        let config = LlmConfig {
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_base: String::new(),
            api_key_env: "ZTLGR_TEST_CREATE_OPENAI".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        };

        let provider = create_provider(&config).unwrap();
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.model(), "gpt-4o");
        std::env::remove_var("ZTLGR_TEST_CREATE_OPENAI");
    }

    #[test]
    fn test_create_provider_anthropic_requires_key() {
        let config = LlmConfig {
            enabled: true,
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_base: String::new(),
            api_key_env: String::new(),
            max_tokens: 4096,
            temperature: 0.7,
        };

        assert!(create_provider(&config).is_err());
    }

    #[test]
    fn test_create_provider_anthropic_with_key() {
        std::env::set_var("ZTLGR_TEST_CREATE_ANTHROPIC", "sk-ant-test");
        let config = LlmConfig {
            enabled: true,
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_base: String::new(),
            api_key_env: "ZTLGR_TEST_CREATE_ANTHROPIC".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        };

        let provider = create_provider(&config).unwrap();
        assert_eq!(provider.name(), "anthropic");
        std::env::remove_var("ZTLGR_TEST_CREATE_ANTHROPIC");
    }

    #[test]
    fn test_create_provider_unknown() {
        let config = LlmConfig {
            enabled: true,
            provider: "google".to_string(),
            model: "gemini".to_string(),
            api_base: String::new(),
            api_key_env: String::new(),
            max_tokens: 4096,
            temperature: 0.7,
        };

        let result = create_provider(&config);
        assert!(result.is_err());
        // Verify the error message mentions the unknown provider
        let err_msg = match result {
            Err(e) => e.to_string(),
            Ok(_) => panic!("Expected error"),
        };
        assert!(err_msg.contains("Unknown LLM provider"));
    }
}
