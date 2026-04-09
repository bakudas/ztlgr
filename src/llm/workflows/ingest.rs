use std::path::Path;

use crate::config::LlmConfig;
use crate::db::Database;
use crate::error::Result;
use crate::llm::workflow::{read_source_content, truncate_to_token_budget, WorkflowEngine};
use crate::note::{Note, NoteType};
use crate::storage::activity_log::{ActivityEntry, ActivityKind, ActivityLog};
use crate::storage::IndexGenerator;

/// Result of LLM-powered ingest processing.
#[derive(Debug, Clone)]
pub struct IngestProcessResult {
    /// The literature note created from the source.
    pub literature_note_title: String,
    /// The LLM-generated content for the literature note.
    pub literature_note_content: String,
    /// The note ID assigned to the literature note.
    pub note_id: String,
    /// Relative path of the source in `raw/`.
    pub source_path: String,
    /// The LLM model used.
    pub model: String,
    /// Tokens consumed.
    pub total_tokens: u32,
    /// Estimated cost in USD.
    pub estimated_cost_usd: f64,
}

/// Orchestrates the LLM-powered ingest workflow.
///
/// Given an already-ingested source file (in `raw/`), this workflow:
/// 1. Reads the source content
/// 2. Builds context from `.skills/` (ingest workflow + source-summary template)
/// 3. Sends the source to the LLM for summarization
/// 4. Creates a literature note in the database and writes it to disk
/// 5. Regenerates the index and logs the activity
pub struct IngestWorkflow;

impl IngestWorkflow {
    /// Process an already-ingested source file with the LLM.
    ///
    /// The `source_relative_path` should be something like `"raw/article-abc12345.md"`.
    /// The `source_title` is used as the basis for the literature note title.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - LLM is disabled in config
    /// - The source file cannot be read
    /// - The LLM call fails
    /// - Database operations fail
    pub async fn process(
        config: &LlmConfig,
        vault_path: &Path,
        db: &Database,
        source_relative_path: &str,
        source_title: &str,
    ) -> Result<IngestProcessResult> {
        // Build the workflow engine
        let mut engine = WorkflowEngine::new(config, vault_path)?;

        // Read source content from raw/
        let raw_content = read_source_content(vault_path, source_relative_path)?;

        // Truncate if needed to stay within token budget
        // Reserve ~2000 tokens for system prompt, leave rest for source content
        let max_content_tokens = config.max_tokens.saturating_sub(2000).max(1000);
        let content_for_llm = truncate_to_token_budget(&raw_content, max_content_tokens);

        // Build system prompt from .skills/
        let context = engine.build_context_with_template("ingest", "source-summary")?;
        let system_prompt = if context.is_empty() {
            default_ingest_system_prompt()
        } else {
            context.build()
        };

        // Build user prompt with the source content
        let user_prompt = format!(
            "Process the following source material and create a literature note summary.\n\n\
             Source title: {}\n\
             Source path: {}\n\n\
             ---\n\n\
             {}\n\n\
             ---\n\n\
             Create a comprehensive literature note. Include:\n\
             1. Key takeaways (3-5 bullet points)\n\
             2. Detailed summary\n\
             3. Notable quotes (if any)\n\
             4. Connections to potential topics\n\n\
             Format as markdown. Use [[wiki-links]] for concepts that deserve their own pages.",
            source_title, source_relative_path, content_for_llm,
        );

        // Execute the LLM call
        let result = engine
            .execute(&system_prompt, &user_prompt, "ingest")
            .await?;

        // Create the literature note
        let note_title = format!("Literature: {}", source_title);
        let note_content = format!(
            "---\ntype: literature\nsource: {}\n---\n\n{}",
            source_relative_path, result.content
        );

        let note =
            Note::new(note_title.clone(), note_content.clone()).with_type(NoteType::Literature {
                source: source_relative_path.to_string(),
            });

        let note_id = db.create_note(&note)?;

        // Write the note file to disk
        let note_filename = sanitize_title_for_filename(source_title);
        let note_path = vault_path
            .join("literature")
            .join(format!("{}.md", note_filename));

        if let Some(parent) = note_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&note_path, &note_content)?;

        // Regenerate index
        let generator = IndexGenerator::new(db);
        let _ = generator.write_index(vault_path);

        // Flush engine usage to activity log
        let _ = engine.finish();

        // Log the ingest processing
        let activity_log = ActivityLog::new(vault_path);
        let entry = ActivityEntry::new(
            ActivityKind::Llm,
            format!(
                "Processed source '{}' → literature note '{}'",
                source_title, note_title
            ),
        )
        .with_detail(format!("Source: {}", source_relative_path))
        .with_detail(format!("Note: literature/{}.md", note_filename))
        .with_detail(format!("Model: {}", result.model))
        .with_detail(format!(
            "Tokens: {} ({}p + {}c)",
            result.prompt_tokens + result.completion_tokens,
            result.prompt_tokens,
            result.completion_tokens,
        ));
        let _ = activity_log.append(&entry);

        Ok(IngestProcessResult {
            literature_note_title: note_title,
            literature_note_content: note_content,
            note_id: note_id.as_str().to_string(),
            source_path: source_relative_path.to_string(),
            model: result.model,
            total_tokens: result.prompt_tokens + result.completion_tokens,
            estimated_cost_usd: result.estimated_cost_usd,
        })
    }
}

/// Sanitize a title for use as a filename.
///
/// Converts to lowercase, replaces spaces with hyphens, strips non-alphanumeric chars.
fn sanitize_title_for_filename(title: &str) -> String {
    let sanitized: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse multiple hyphens
    let mut result = String::with_capacity(sanitized.len());
    let mut prev_hyphen = false;
    for c in sanitized.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    // Trim leading/trailing hyphens
    result.trim_matches('-').to_string()
}

/// Default system prompt when no `.skills/` directory is configured.
fn default_ingest_system_prompt() -> String {
    "You are a knowledge management assistant for a personal wiki (Zettelkasten).\n\n\
     Your task is to process source material and create literature notes.\n\n\
     Guidelines:\n\
     - Extract key takeaways as bullet points\n\
     - Write a detailed but concise summary\n\
     - Identify notable quotes\n\
     - Suggest connections to broader topics using [[wiki-links]]\n\
     - Use markdown formatting\n\
     - Be factual and faithful to the source material\n\
     - Do not invent information not present in the source"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- sanitize_title_for_filename ---

    #[test]
    fn test_sanitize_simple_title() {
        assert_eq!(sanitize_title_for_filename("My Article"), "my-article");
    }

    #[test]
    fn test_sanitize_special_chars() {
        let result = sanitize_title_for_filename("Hello, World! (2024)");
        assert_eq!(result, "hello-world-2024");
        assert!(!result.contains("--"));
    }

    #[test]
    fn test_sanitize_preserves_numbers() {
        assert_eq!(
            sanitize_title_for_filename("Chapter 3 Notes"),
            "chapter-3-notes"
        );
    }

    #[test]
    fn test_sanitize_collapses_hyphens() {
        assert_eq!(
            sanitize_title_for_filename("foo - bar -- baz"),
            "foo-bar-baz"
        );
    }

    #[test]
    fn test_sanitize_trims_hyphens() {
        assert_eq!(
            sanitize_title_for_filename("--leading and trailing--"),
            "leading-and-trailing"
        );
    }

    #[test]
    fn test_sanitize_empty() {
        assert_eq!(sanitize_title_for_filename(""), "");
    }

    #[test]
    fn test_sanitize_underscores() {
        assert_eq!(
            sanitize_title_for_filename("my_great_article"),
            "my-great-article"
        );
    }

    // --- default_ingest_system_prompt ---

    #[test]
    fn test_default_system_prompt_not_empty() {
        let prompt = default_ingest_system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("Zettelkasten"));
        assert!(prompt.contains("wiki-links"));
    }

    // --- IngestProcessResult fields ---

    #[test]
    fn test_ingest_process_result_fields() {
        let result = IngestProcessResult {
            literature_note_title: "Literature: Test".to_string(),
            literature_note_content: "content".to_string(),
            note_id: "abc-123".to_string(),
            source_path: "raw/test.md".to_string(),
            model: "llama3".to_string(),
            total_tokens: 500,
            estimated_cost_usd: 0.0,
        };

        assert_eq!(result.literature_note_title, "Literature: Test");
        assert_eq!(result.source_path, "raw/test.md");
        assert_eq!(result.model, "llama3");
        assert_eq!(result.total_tokens, 500);
    }

    // --- IngestWorkflow::process (error cases, no real LLM) ---

    #[test]
    fn test_ingest_workflow_disabled_llm() {
        let config = LlmConfig {
            enabled: false,
            ..Default::default()
        };

        let temp = tempfile::TempDir::new().unwrap();
        let db = Database::new(&temp.path().join("vault.db")).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(IngestWorkflow::process(
            &config,
            temp.path(),
            &db,
            "raw/test.md",
            "Test",
        ));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }

    #[test]
    fn test_ingest_workflow_missing_source() {
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };

        let temp = tempfile::TempDir::new().unwrap();
        let db = Database::new(&temp.path().join("vault.db")).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(IngestWorkflow::process(
            &config,
            temp.path(),
            &db,
            "raw/nonexistent.md",
            "Missing",
        ));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
