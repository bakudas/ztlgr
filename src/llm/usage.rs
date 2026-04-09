use std::path::Path;

use chrono::Utc;

use crate::error::Result;
use crate::storage::activity_log::{ActivityEntry, ActivityKind, ActivityLog};

use super::provider::TokenUsage;

/// Tracks token usage and estimated costs for LLM operations.
///
/// Records each LLM call with its provider, model, token counts, and
/// estimated cost. Can write summaries to the activity log.
pub struct UsageTracker {
    records: Vec<UsageRecord>,
}

/// A single LLM usage record.
#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub provider: String,
    pub model: String,
    pub usage: TokenUsage,
    pub estimated_cost_usd: f64,
    pub operation: String,
    pub timestamp: chrono::DateTime<Utc>,
}

impl UsageTracker {
    /// Create a new empty usage tracker.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Record a new LLM usage event.
    pub fn record(
        &mut self,
        provider: &str,
        model: &str,
        usage: &TokenUsage,
        operation: impl Into<String>,
    ) {
        let estimated_cost = estimate_cost(provider, model, usage);

        self.records.push(UsageRecord {
            provider: provider.to_string(),
            model: model.to_string(),
            usage: usage.clone(),
            estimated_cost_usd: estimated_cost,
            operation: operation.into(),
            timestamp: Utc::now(),
        });
    }

    /// Get all usage records.
    pub fn records(&self) -> &[UsageRecord] {
        &self.records
    }

    /// Total tokens across all records.
    pub fn total_tokens(&self) -> u32 {
        self.records.iter().map(|r| r.usage.total_tokens).sum()
    }

    /// Total prompt tokens across all records.
    pub fn total_prompt_tokens(&self) -> u32 {
        self.records.iter().map(|r| r.usage.prompt_tokens).sum()
    }

    /// Total completion tokens across all records.
    pub fn total_completion_tokens(&self) -> u32 {
        self.records.iter().map(|r| r.usage.completion_tokens).sum()
    }

    /// Total estimated cost in USD.
    pub fn total_cost_usd(&self) -> f64 {
        self.records.iter().map(|r| r.estimated_cost_usd).sum()
    }

    /// Number of LLM calls recorded.
    pub fn call_count(&self) -> usize {
        self.records.len()
    }

    /// Format a usage summary as a string.
    pub fn summary(&self) -> String {
        if self.records.is_empty() {
            return "No LLM usage recorded.".to_string();
        }

        format!(
            "{} LLM call(s): {} tokens ({}p + {}c), est. ${:.4}",
            self.call_count(),
            self.total_tokens(),
            self.total_prompt_tokens(),
            self.total_completion_tokens(),
            self.total_cost_usd(),
        )
    }

    /// Write the current usage session to the grimoire's activity log.
    ///
    /// Creates an entry under the `Llm` activity kind with token and
    /// cost details.
    pub fn write_to_log(&self, vault_path: &Path) -> Result<()> {
        if self.records.is_empty() {
            return Ok(());
        }

        let log = ActivityLog::new(vault_path);

        let mut entry = ActivityEntry::new(ActivityKind::Llm, self.summary());

        for record in &self.records {
            entry = entry.with_detail(format!(
                "[{}] {} | {} tokens ({}p + {}c) | ${:.4} | {}",
                record.provider,
                record.model,
                record.usage.total_tokens,
                record.usage.prompt_tokens,
                record.usage.completion_tokens,
                record.estimated_cost_usd,
                record.operation,
            ));
        }

        log.append(&entry)
    }

    /// Clear all records (e.g. after writing to log).
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate the cost in USD for a given usage.
///
/// Uses approximate per-million-token pricing. These are rough estimates
/// and will drift as providers update pricing. Good enough for tracking
/// and budgeting.
pub fn estimate_cost(provider: &str, model: &str, usage: &TokenUsage) -> f64 {
    let (input_per_m, output_per_m) = model_pricing(provider, model);

    let input_cost = (usage.prompt_tokens as f64 / 1_000_000.0) * input_per_m;
    let output_cost = (usage.completion_tokens as f64 / 1_000_000.0) * output_per_m;

    input_cost + output_cost
}

/// Return (input_price_per_million, output_price_per_million) in USD.
///
/// Returns (0.0, 0.0) for Ollama (local, free) and unknown models.
fn model_pricing(provider: &str, model: &str) -> (f64, f64) {
    match provider {
        "ollama" => (0.0, 0.0), // Local models are free

        "openai" => match model {
            m if m.starts_with("gpt-4o-mini") => (0.15, 0.60),
            m if m.starts_with("gpt-4o") => (2.50, 10.00),
            m if m.starts_with("gpt-4-turbo") => (10.00, 30.00),
            m if m.starts_with("gpt-4") => (30.00, 60.00),
            m if m.starts_with("gpt-3.5") => (0.50, 1.50),
            m if m.starts_with("o3-mini") => (1.10, 4.40),
            m if m.starts_with("o3") => (10.00, 40.00),
            m if m.starts_with("o1-mini") => (3.00, 12.00),
            m if m.starts_with("o1") => (15.00, 60.00),
            _ => (0.0, 0.0), // Unknown model
        },

        "anthropic" => match model {
            m if m.contains("opus") => (15.00, 75.00),
            m if m.contains("sonnet") => (3.00, 15.00),
            m if m.contains("haiku") => (0.25, 1.25),
            _ => (0.0, 0.0),
        },

        _ => (0.0, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_tracker_new_empty() {
        let tracker = UsageTracker::new();
        assert_eq!(tracker.call_count(), 0);
        assert_eq!(tracker.total_tokens(), 0);
        assert_eq!(tracker.total_cost_usd(), 0.0);
        assert!(tracker.records().is_empty());
    }

    #[test]
    fn test_usage_tracker_default() {
        let tracker = UsageTracker::default();
        assert!(tracker.records().is_empty());
    }

    #[test]
    fn test_usage_tracker_record() {
        let mut tracker = UsageTracker::new();
        let usage = TokenUsage::new(100, 50);

        tracker.record("ollama", "llama3", &usage, "test query");

        assert_eq!(tracker.call_count(), 1);
        assert_eq!(tracker.total_tokens(), 150);
        assert_eq!(tracker.total_prompt_tokens(), 100);
        assert_eq!(tracker.total_completion_tokens(), 50);
    }

    #[test]
    fn test_usage_tracker_multiple_records() {
        let mut tracker = UsageTracker::new();

        tracker.record("ollama", "llama3", &TokenUsage::new(100, 50), "query 1");
        tracker.record("openai", "gpt-4o", &TokenUsage::new(200, 100), "query 2");

        assert_eq!(tracker.call_count(), 2);
        assert_eq!(tracker.total_tokens(), 450);
        assert_eq!(tracker.total_prompt_tokens(), 300);
        assert_eq!(tracker.total_completion_tokens(), 150);
    }

    #[test]
    fn test_usage_tracker_clear() {
        let mut tracker = UsageTracker::new();
        tracker.record("ollama", "llama3", &TokenUsage::new(100, 50), "test");

        assert_eq!(tracker.call_count(), 1);

        tracker.clear();
        assert_eq!(tracker.call_count(), 0);
        assert_eq!(tracker.total_tokens(), 0);
    }

    #[test]
    fn test_summary_empty() {
        let tracker = UsageTracker::new();
        assert_eq!(tracker.summary(), "No LLM usage recorded.");
    }

    #[test]
    fn test_summary_with_records() {
        let mut tracker = UsageTracker::new();
        tracker.record("ollama", "llama3", &TokenUsage::new(100, 50), "test");

        let summary = tracker.summary();
        assert!(summary.contains("1 LLM call(s)"));
        assert!(summary.contains("150 tokens"));
        assert!(summary.contains("100p"));
        assert!(summary.contains("50c"));
    }

    #[test]
    fn test_usage_record_fields() {
        let mut tracker = UsageTracker::new();
        tracker.record("openai", "gpt-4o", &TokenUsage::new(500, 200), "summarize");

        let record = &tracker.records()[0];
        assert_eq!(record.provider, "openai");
        assert_eq!(record.model, "gpt-4o");
        assert_eq!(record.usage.prompt_tokens, 500);
        assert_eq!(record.usage.completion_tokens, 200);
        assert_eq!(record.operation, "summarize");
    }

    // --- Cost estimation tests ---

    #[test]
    fn test_ollama_cost_is_zero() {
        let usage = TokenUsage::new(10000, 5000);
        let cost = estimate_cost("ollama", "llama3", &usage);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_ollama_any_model_free() {
        let usage = TokenUsage::new(10000, 5000);
        assert_eq!(estimate_cost("ollama", "mistral", &usage), 0.0);
        assert_eq!(estimate_cost("ollama", "codellama", &usage), 0.0);
    }

    #[test]
    fn test_openai_gpt4o_cost() {
        // gpt-4o: $2.50/M input, $10.00/M output
        let usage = TokenUsage::new(1_000_000, 1_000_000);
        let cost = estimate_cost("openai", "gpt-4o", &usage);
        assert!((cost - 12.50).abs() < 0.001);
    }

    #[test]
    fn test_openai_gpt4o_mini_cost() {
        // gpt-4o-mini: $0.15/M input, $0.60/M output
        let usage = TokenUsage::new(1_000_000, 1_000_000);
        let cost = estimate_cost("openai", "gpt-4o-mini", &usage);
        assert!((cost - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_openai_gpt4o_specific_version() {
        // gpt-4o-2024-05-13 should match gpt-4o prefix
        let usage = TokenUsage::new(1000, 500);
        let cost = estimate_cost("openai", "gpt-4o-2024-05-13", &usage);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_anthropic_sonnet_cost() {
        // claude-sonnet: $3.00/M input, $15.00/M output
        let usage = TokenUsage::new(1_000_000, 1_000_000);
        let cost = estimate_cost("anthropic", "claude-sonnet-4-20250514", &usage);
        assert!((cost - 18.0).abs() < 0.001);
    }

    #[test]
    fn test_anthropic_haiku_cost() {
        // claude-haiku: $0.25/M input, $1.25/M output
        let usage = TokenUsage::new(1_000_000, 1_000_000);
        let cost = estimate_cost("anthropic", "claude-haiku", &usage);
        assert!((cost - 1.50).abs() < 0.001);
    }

    #[test]
    fn test_anthropic_opus_cost() {
        // claude-opus: $15.00/M input, $75.00/M output
        let usage = TokenUsage::new(1_000_000, 1_000_000);
        let cost = estimate_cost("anthropic", "claude-opus", &usage);
        assert!((cost - 90.0).abs() < 0.001);
    }

    #[test]
    fn test_unknown_provider_zero_cost() {
        let usage = TokenUsage::new(10000, 5000);
        assert_eq!(estimate_cost("unknown", "whatever", &usage), 0.0);
    }

    #[test]
    fn test_unknown_model_zero_cost() {
        let usage = TokenUsage::new(10000, 5000);
        assert_eq!(estimate_cost("openai", "unknown-model", &usage), 0.0);
    }

    #[test]
    fn test_zero_usage_zero_cost() {
        let usage = TokenUsage::new(0, 0);
        assert_eq!(estimate_cost("openai", "gpt-4o", &usage), 0.0);
    }

    #[test]
    fn test_model_pricing_ollama() {
        let (input, output) = model_pricing("ollama", "llama3");
        assert_eq!(input, 0.0);
        assert_eq!(output, 0.0);
    }

    #[test]
    fn test_model_pricing_openai_o3() {
        let (input, output) = model_pricing("openai", "o3");
        assert!((input - 10.0).abs() < 0.001);
        assert!((output - 40.0).abs() < 0.001);
    }

    #[test]
    fn test_model_pricing_openai_o3_mini() {
        let (input, output) = model_pricing("openai", "o3-mini");
        assert!((input - 1.10).abs() < 0.001);
        assert!((output - 4.40).abs() < 0.001);
    }

    #[test]
    fn test_tracker_cost_accumulation() {
        let mut tracker = UsageTracker::new();

        // Ollama call (free)
        tracker.record("ollama", "llama3", &TokenUsage::new(1000, 500), "q1");
        // OpenAI call
        tracker.record("openai", "gpt-4o-mini", &TokenUsage::new(1000, 500), "q2");

        assert_eq!(tracker.call_count(), 2);
        // Only the OpenAI call should have cost
        let cost = tracker.total_cost_usd();
        assert!(cost > 0.0);
        // First record should have zero cost
        assert_eq!(tracker.records()[0].estimated_cost_usd, 0.0);
    }

    #[test]
    fn test_write_to_log() {
        let temp = tempfile::TempDir::new().unwrap();
        let ztlgr_dir = temp.path().join(".ztlgr");
        std::fs::create_dir_all(&ztlgr_dir).unwrap();

        let mut tracker = UsageTracker::new();
        tracker.record("ollama", "llama3", &TokenUsage::new(100, 50), "test query");

        let result = tracker.write_to_log(temp.path());
        assert!(result.is_ok());

        // Verify log file was created
        let log_path = ztlgr_dir.join("log.md");
        assert!(log_path.exists());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("LLM call"));
        assert!(content.contains("ollama"));
        assert!(content.contains("llama3"));
    }

    #[test]
    fn test_write_to_log_empty_tracker() {
        let temp = tempfile::TempDir::new().unwrap();
        let ztlgr_dir = temp.path().join(".ztlgr");
        std::fs::create_dir_all(&ztlgr_dir).unwrap();

        let tracker = UsageTracker::new();
        let result = tracker.write_to_log(temp.path());
        assert!(result.is_ok());

        // No log should be written for empty tracker
        let log_path = ztlgr_dir.join("log.md");
        assert!(!log_path.exists());
    }
}
