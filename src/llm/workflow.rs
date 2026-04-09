use std::path::{Path, PathBuf};

use crate::config::LlmConfig;
use crate::error::{Result, ZtlgrError};

use super::context::ContextBuilder;
use super::create_provider;
use super::provider::{LlmProvider, LlmRequest, LlmResponse};
use super::usage::UsageTracker;

/// Result of running a workflow step that produces LLM output.
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    /// The LLM-generated content.
    pub content: String,
    /// The model that produced this result.
    pub model: String,
    /// Number of prompt tokens used.
    pub prompt_tokens: u32,
    /// Number of completion tokens used.
    pub completion_tokens: u32,
    /// Estimated cost in USD.
    pub estimated_cost_usd: f64,
    /// The operation name (for logging).
    pub operation: String,
}

/// Core workflow engine that orchestrates LLM calls within the grimoire.
///
/// Combines an LLM provider, context builder, and usage tracker. Each
/// workflow (ingest, query, lint) uses `WorkflowEngine` to build prompts,
/// call the LLM, and record usage.
///
/// Manual `Debug` impl because `Box<dyn LlmProvider>` is not `Debug`.
pub struct WorkflowEngine {
    provider: Box<dyn LlmProvider>,
    tracker: UsageTracker,
    vault_path: PathBuf,
    config: LlmConfig,
}

impl std::fmt::Debug for WorkflowEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowEngine")
            .field("provider", &self.provider.name())
            .field("model", &self.provider.model())
            .field("vault_path", &self.vault_path)
            .finish()
    }
}

impl WorkflowEngine {
    /// Create a new workflow engine from config and vault path.
    ///
    /// Returns an error if the LLM provider cannot be created (e.g.
    /// missing API key for cloud providers).
    pub fn new(config: &LlmConfig, vault_path: &Path) -> Result<Self> {
        if !config.enabled {
            return Err(ZtlgrError::Llm(
                "LLM is disabled. Enable it in config with [llm] enabled = true".to_string(),
            ));
        }

        let provider = create_provider(config)?;

        Ok(Self {
            provider,
            tracker: UsageTracker::new(),
            vault_path: vault_path.to_path_buf(),
            config: config.clone(),
        })
    }

    /// Build a context (system prompt) loaded with grimoire skills and
    /// a specific workflow.
    pub fn build_context(&self, workflow_name: &str) -> Result<ContextBuilder> {
        let builder = ContextBuilder::new()
            .load_skills(&self.vault_path)?
            .load_workflow(&self.vault_path, workflow_name)?;

        Ok(builder)
    }

    /// Build a context with skills, a workflow, AND a template.
    pub fn build_context_with_template(
        &self,
        workflow_name: &str,
        template_name: &str,
    ) -> Result<ContextBuilder> {
        let builder = ContextBuilder::new()
            .load_skills(&self.vault_path)?
            .load_workflow(&self.vault_path, workflow_name)?
            .load_template(&self.vault_path, template_name)?;

        Ok(builder)
    }

    /// Execute a single LLM call with a system prompt and user message.
    ///
    /// Records usage in the tracker and returns the result.
    pub async fn execute(
        &mut self,
        system_prompt: &str,
        user_prompt: &str,
        operation: &str,
    ) -> Result<WorkflowResult> {
        let mut request = LlmRequest::with_system(system_prompt, user_prompt);

        // Apply config overrides
        if self.config.max_tokens > 0 {
            request = request.max_tokens(self.config.max_tokens);
        }
        if self.config.temperature >= 0.0 {
            request = request.temperature(self.config.temperature);
        }

        let response: LlmResponse = self.provider.complete(&request).await?;

        // Track usage
        let cost =
            super::usage::estimate_cost(self.provider.name(), &response.model, &response.usage);

        self.tracker.record(
            self.provider.name(),
            &response.model,
            &response.usage,
            operation,
        );

        Ok(WorkflowResult {
            content: response.content,
            model: response.model,
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            estimated_cost_usd: cost,
            operation: operation.to_string(),
        })
    }

    /// Flush the usage tracker to the activity log and return a summary.
    pub fn finish(&mut self) -> Result<String> {
        let summary = self.tracker.summary();
        self.tracker.write_to_log(&self.vault_path)?;
        self.tracker.clear();
        Ok(summary)
    }

    /// Get the current usage summary without flushing.
    pub fn usage_summary(&self) -> String {
        self.tracker.summary()
    }

    /// Get the number of LLM calls made so far.
    pub fn call_count(&self) -> usize {
        self.tracker.call_count()
    }

    /// Get the total estimated cost so far.
    pub fn total_cost_usd(&self) -> f64 {
        self.tracker.total_cost_usd()
    }

    /// Get the provider name.
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    /// Get the model name.
    pub fn model_name(&self) -> &str {
        self.provider.model()
    }

    /// Get a reference to the vault path.
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    /// Get a reference to the config.
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }

    /// Check if the LLM provider is reachable.
    pub async fn health_check(&self) -> Result<bool> {
        self.provider.health_check().await
    }
}

/// Validate that LLM is enabled and return a helpful error if not.
pub fn require_llm_enabled(config: &LlmConfig) -> Result<()> {
    if !config.enabled {
        return Err(ZtlgrError::Llm(
            "LLM features require [llm] enabled = true in your config.\n\
             Configure a provider:\n  \
             [llm]\n  \
             enabled = true\n  \
             provider = \"ollama\"   # or \"openai\", \"anthropic\"\n  \
             model = \"llama3\"\n\
             \n\
             For local (free) operation, install Ollama: https://ollama.ai"
                .to_string(),
        ));
    }
    Ok(())
}

/// Read source file content from the raw/ directory.
///
/// Given a vault path and a relative file path (e.g. "raw/article-abc123.md"),
/// reads the full content. Returns an error if the file doesn't exist.
pub fn read_source_content(vault_path: &Path, relative_path: &str) -> Result<String> {
    let full_path = vault_path.join(relative_path);

    if !full_path.exists() {
        return Err(ZtlgrError::SourceNotFound(format!(
            "Source file not found: {}",
            full_path.display()
        )));
    }

    std::fs::read_to_string(&full_path).map_err(|e| {
        ZtlgrError::Ingest(format!(
            "Failed to read source file {}: {}",
            full_path.display(),
            e
        ))
    })
}

/// Truncate content to fit within a token budget.
///
/// Uses the rough heuristic of ~4 chars per token. Truncates at a
/// paragraph boundary when possible, adding a "[truncated]" marker.
pub fn truncate_to_token_budget(content: &str, max_tokens: u32) -> String {
    let max_chars = (max_tokens as usize) * 4;

    if content.len() <= max_chars {
        return content.to_string();
    }

    // Try to truncate at a paragraph boundary
    let truncated = &content[..max_chars];
    if let Some(last_para) = truncated.rfind("\n\n") {
        format!(
            "{}\n\n[... truncated, {} chars omitted ...]",
            &truncated[..last_para],
            content.len() - last_para
        )
    } else if let Some(last_newline) = truncated.rfind('\n') {
        format!(
            "{}\n\n[... truncated, {} chars omitted ...]",
            &truncated[..last_newline],
            content.len() - last_newline
        )
    } else {
        format!(
            "{}\n\n[... truncated, {} chars omitted ...]",
            truncated,
            content.len() - max_chars
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- WorkflowResult ---

    #[test]
    fn test_workflow_result_fields() {
        let result = WorkflowResult {
            content: "Summary of article".to_string(),
            model: "llama3".to_string(),
            prompt_tokens: 500,
            completion_tokens: 200,
            estimated_cost_usd: 0.0,
            operation: "ingest".to_string(),
        };

        assert_eq!(result.content, "Summary of article");
        assert_eq!(result.model, "llama3");
        assert_eq!(result.prompt_tokens, 500);
        assert_eq!(result.completion_tokens, 200);
        assert_eq!(result.estimated_cost_usd, 0.0);
        assert_eq!(result.operation, "ingest");
    }

    // --- require_llm_enabled ---

    #[test]
    fn test_require_llm_enabled_true() {
        let config = LlmConfig {
            enabled: true,
            ..Default::default()
        };
        assert!(require_llm_enabled(&config).is_ok());
    }

    #[test]
    fn test_require_llm_enabled_false() {
        let config = LlmConfig {
            enabled: false,
            ..Default::default()
        };
        let err = require_llm_enabled(&config).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("enabled = true"));
        assert!(msg.contains("ollama"));
    }

    // --- WorkflowEngine::new ---

    #[test]
    fn test_workflow_engine_new_disabled() {
        let temp = TempDir::new().unwrap();
        let config = LlmConfig {
            enabled: false,
            ..Default::default()
        };

        let result = WorkflowEngine::new(&config, temp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }

    #[test]
    fn test_workflow_engine_new_ollama() {
        let temp = TempDir::new().unwrap();
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };

        let engine = WorkflowEngine::new(&config, temp.path()).unwrap();
        assert_eq!(engine.provider_name(), "ollama");
        assert_eq!(engine.model_name(), "llama3");
        assert_eq!(engine.call_count(), 0);
        assert_eq!(engine.total_cost_usd(), 0.0);
    }

    #[test]
    fn test_workflow_engine_vault_path() {
        let temp = TempDir::new().unwrap();
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };

        let engine = WorkflowEngine::new(&config, temp.path()).unwrap();
        assert_eq!(engine.vault_path(), temp.path());
    }

    #[test]
    fn test_workflow_engine_config() {
        let temp = TempDir::new().unwrap();
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "custom-model".to_string(),
            max_tokens: 8192,
            temperature: 0.3,
            ..Default::default()
        };

        let engine = WorkflowEngine::new(&config, temp.path()).unwrap();
        assert_eq!(engine.config().model, "custom-model");
        assert_eq!(engine.config().max_tokens, 8192);
    }

    #[test]
    fn test_workflow_engine_usage_summary_empty() {
        let temp = TempDir::new().unwrap();
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };

        let engine = WorkflowEngine::new(&config, temp.path()).unwrap();
        assert_eq!(engine.usage_summary(), "No LLM usage recorded.");
    }

    // --- build_context ---

    #[test]
    fn test_build_context_no_skills() {
        let temp = TempDir::new().unwrap();
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };

        let engine = WorkflowEngine::new(&config, temp.path()).unwrap();
        let ctx = engine.build_context("ingest").unwrap();

        // No .skills/ dir, so context should be empty
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_build_context_with_skills() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        std::fs::create_dir_all(skills_dir.join("workflows")).unwrap();
        std::fs::write(skills_dir.join("README.md"), "# My Grimoire").unwrap();
        std::fs::write(skills_dir.join("conventions.md"), "Use wiki-links.").unwrap();
        std::fs::write(
            skills_dir.join("workflows/ingest.md"),
            "1. Read source\n2. Summarize\n3. Create note",
        )
        .unwrap();

        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };

        let engine = WorkflowEngine::new(&config, temp.path()).unwrap();
        let ctx = engine.build_context("ingest").unwrap();

        assert_eq!(ctx.section_count(), 3); // README + conventions + workflow
        let prompt = ctx.build();
        assert!(prompt.contains("My Grimoire"));
        assert!(prompt.contains("wiki-links"));
        assert!(prompt.contains("Summarize"));
    }

    #[test]
    fn test_build_context_with_template() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        std::fs::create_dir_all(skills_dir.join("workflows")).unwrap();
        std::fs::create_dir_all(skills_dir.join("templates")).unwrap();
        std::fs::write(
            skills_dir.join("workflows/ingest.md"),
            "Ingest workflow steps",
        )
        .unwrap();
        std::fs::write(
            skills_dir.join("templates/source-summary.md"),
            "# {{title}}\n\nSummary template",
        )
        .unwrap();

        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };

        let engine = WorkflowEngine::new(&config, temp.path()).unwrap();
        let ctx = engine
            .build_context_with_template("ingest", "source-summary")
            .unwrap();

        // workflow + template (no README/conventions to load)
        assert_eq!(ctx.section_count(), 2);
        let prompt = ctx.build();
        assert!(prompt.contains("Ingest workflow"));
        assert!(prompt.contains("Summary template"));
    }

    // --- read_source_content ---

    #[test]
    fn test_read_source_content_success() {
        let temp = TempDir::new().unwrap();
        let raw_dir = temp.path().join("raw");
        std::fs::create_dir_all(&raw_dir).unwrap();
        std::fs::write(raw_dir.join("article.md"), "# Article\n\nContent here").unwrap();

        let content = read_source_content(temp.path(), "raw/article.md").unwrap();
        assert_eq!(content, "# Article\n\nContent here");
    }

    #[test]
    fn test_read_source_content_not_found() {
        let temp = TempDir::new().unwrap();
        let result = read_source_content(temp.path(), "raw/nonexistent.md");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    // --- truncate_to_token_budget ---

    #[test]
    fn test_truncate_short_content() {
        let content = "Short text";
        let result = truncate_to_token_budget(content, 100);
        assert_eq!(result, "Short text");
    }

    #[test]
    fn test_truncate_at_paragraph_boundary() {
        let content = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph that is longer.";
        // With 10 tokens = ~40 chars, should truncate at a paragraph boundary
        let result = truncate_to_token_budget(content, 10);
        assert!(result.contains("First paragraph."));
        assert!(result.contains("[... truncated"));
    }

    #[test]
    fn test_truncate_at_line_boundary() {
        let content = "Line one\nLine two\nLine three that is really quite long";
        // With 5 tokens = ~20 chars
        let result = truncate_to_token_budget(content, 5);
        assert!(result.contains("[... truncated"));
    }

    #[test]
    fn test_truncate_no_boundary() {
        let content = "a".repeat(100);
        let result = truncate_to_token_budget(&content, 5);
        // 5 tokens = ~20 chars, no newlines to break at
        assert!(result.contains("[... truncated"));
        assert!(result.len() < content.len() + 50); // truncated + marker
    }

    #[test]
    fn test_truncate_exact_budget() {
        let content = "a".repeat(400); // 400 chars = ~100 tokens
        let result = truncate_to_token_budget(&content, 100);
        assert_eq!(result, content); // Should not truncate
    }

    // --- finish (write to log) ---

    #[test]
    fn test_workflow_engine_finish_empty() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".ztlgr")).unwrap();

        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };

        let mut engine = WorkflowEngine::new(&config, temp.path()).unwrap();
        let summary = engine.finish().unwrap();
        assert_eq!(summary, "No LLM usage recorded.");
    }

    // --- ProviderKind integration ---

    #[test]
    fn test_workflow_engine_openai_requires_key() {
        let temp = TempDir::new().unwrap();
        let config = LlmConfig {
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_key_env: String::new(),
            ..Default::default()
        };

        let result = WorkflowEngine::new(&config, temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_engine_unknown_provider() {
        let temp = TempDir::new().unwrap();
        let config = LlmConfig {
            enabled: true,
            provider: "unknown".to_string(),
            model: "test".to_string(),
            ..Default::default()
        };

        let result = WorkflowEngine::new(&config, temp.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown LLM provider"));
    }
}
