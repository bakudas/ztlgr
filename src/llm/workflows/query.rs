use std::path::Path;

use crate::config::LlmConfig;
use crate::db::Database;
use crate::error::{Result, ZtlgrError};
use crate::llm::workflow::{truncate_to_token_budget, WorkflowEngine};
use crate::storage::activity_log::{ActivityEntry, ActivityKind, ActivityLog};

/// Result of a query workflow.
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// The LLM-generated answer.
    pub answer: String,
    /// The original question.
    pub question: String,
    /// Titles of notes consulted for the answer.
    pub consulted_notes: Vec<String>,
    /// The LLM model used.
    pub model: String,
    /// Tokens consumed.
    pub total_tokens: u32,
    /// Estimated cost in USD.
    pub estimated_cost_usd: f64,
}

/// Orchestrates the LLM-powered query workflow.
///
/// Given a question, this workflow:
/// 1. Searches the grimoire using FTS5 for relevant notes
/// 2. Reads the index for topic orientation
/// 3. Builds context from matching notes
/// 4. Sends everything to the LLM for synthesis
/// 5. Logs the query
pub struct QueryWorkflow;

impl QueryWorkflow {
    /// Ask a question and get an answer synthesized from the grimoire's content.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - LLM is disabled in config
    /// - The LLM call fails
    /// - Database operations fail
    pub async fn ask(
        config: &LlmConfig,
        vault_path: &Path,
        db: &Database,
        question: &str,
    ) -> Result<QueryResult> {
        if question.trim().is_empty() {
            return Err(ZtlgrError::Llm("Question cannot be empty".to_string()));
        }

        // Build the workflow engine
        let mut engine = WorkflowEngine::new(config, vault_path)?;

        // Search for relevant notes using FTS5
        let search_results = db.search_notes(question, 10).unwrap_or_default();

        // Read the index for orientation
        let index_path = vault_path.join(".ztlgr").join("index.md");
        let index_content = std::fs::read_to_string(&index_path).unwrap_or_default();

        // Build context from search results
        let mut consulted_notes: Vec<String> = Vec::new();
        let mut notes_context = String::new();

        for note in &search_results {
            consulted_notes.push(note.title.clone());
            notes_context.push_str(&format!(
                "### [[{}]] ({})\n\n{}\n\n---\n\n",
                note.title,
                note.note_type.as_str(),
                note.content,
            ));
        }

        // Truncate notes context to fit token budget
        let max_context_tokens = config.max_tokens.saturating_sub(2000).max(1000);
        let truncated_context = truncate_to_token_budget(&notes_context, max_context_tokens);

        // Build system prompt from .skills/
        let context_builder = engine.build_context("query")?;
        let system_prompt = if context_builder.is_empty() {
            default_query_system_prompt()
        } else {
            context_builder.build()
        };

        // Build user prompt
        let user_prompt = build_user_prompt(
            question,
            &truncated_context,
            &index_content,
            consulted_notes.len(),
        );

        // Execute the LLM call
        let result = engine
            .execute(&system_prompt, &user_prompt, "query")
            .await?;

        // Flush usage
        let _ = engine.finish();

        // Log the query
        let activity_log = ActivityLog::new(vault_path);
        let entry = ActivityEntry::new(
            ActivityKind::Llm,
            format!(
                "Query: \"{}\" ({} notes consulted)",
                truncate_question(question, 60),
                consulted_notes.len()
            ),
        )
        .with_detail(format!("Model: {}", result.model))
        .with_detail(format!(
            "Tokens: {} ({}p + {}c)",
            result.prompt_tokens + result.completion_tokens,
            result.prompt_tokens,
            result.completion_tokens,
        ));
        let _ = activity_log.append(&entry);

        Ok(QueryResult {
            answer: result.content,
            question: question.to_string(),
            consulted_notes,
            model: result.model,
            total_tokens: result.prompt_tokens + result.completion_tokens,
            estimated_cost_usd: result.estimated_cost_usd,
        })
    }
}

/// Build the user prompt for the query workflow.
fn build_user_prompt(
    question: &str,
    notes_context: &str,
    index_content: &str,
    note_count: usize,
) -> String {
    let mut prompt = format!("## Question\n\n{}\n\n", question);

    if !index_content.is_empty() {
        let index_excerpt = truncate_to_token_budget(index_content, 500);
        prompt.push_str(&format!(
            "## Grimoire Index (for orientation)\n\n{}\n\n",
            index_excerpt
        ));
    }

    if note_count > 0 {
        prompt.push_str(&format!(
            "## Relevant Notes ({} found)\n\n{}\n\n",
            note_count, notes_context
        ));
    } else {
        prompt.push_str(
            "## Note: No relevant notes found in the grimoire\n\n\
             The grimoire does not contain notes directly matching this question.\n\
             Please answer based on what you know, but clearly indicate that the\n\
             grimoire doesn't cover this topic.\n\n",
        );
    }

    prompt.push_str(
        "## Instructions\n\n\
         Answer the question using the grimoire's content as the primary source.\n\
         - Use [[wiki-links]] to cite specific notes\n\
         - If the grimoire doesn't have enough information, say so clearly\n\
         - Prefer wiki content over general knowledge\n\
         - Format the answer as markdown\n",
    );

    prompt
}

/// Truncate a question for display in logs.
fn truncate_question(question: &str, max_len: usize) -> String {
    if question.len() <= max_len {
        question.to_string()
    } else {
        format!("{}...", &question[..max_len])
    }
}

/// Default system prompt when no `.skills/` directory is configured.
fn default_query_system_prompt() -> String {
    "You are a knowledge assistant for a personal wiki (Zettelkasten grimoire).\n\n\
     Your task is to answer questions using the wiki's accumulated knowledge.\n\n\
     Guidelines:\n\
     - Synthesize information from the provided wiki notes\n\
     - Cite sources using [[wiki-links]] (note titles in double brackets)\n\
     - If the wiki doesn't cover a topic, clearly state this\n\
     - Prefer factual, well-sourced answers\n\
     - Use markdown formatting for structure\n\
     - If notes contradict each other, acknowledge the contradiction\n\
     - Do not fabricate information or citations"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- truncate_question ---

    #[test]
    fn test_truncate_question_short() {
        assert_eq!(truncate_question("Hello?", 60), "Hello?");
    }

    #[test]
    fn test_truncate_question_long() {
        let long = "a".repeat(100);
        let result = truncate_question(&long, 60);
        assert_eq!(result.len(), 63); // 60 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_question_exact() {
        let exact = "a".repeat(60);
        assert_eq!(truncate_question(&exact, 60), exact);
    }

    // --- build_user_prompt ---

    #[test]
    fn test_build_user_prompt_with_notes() {
        let prompt = build_user_prompt(
            "What is Rust?",
            "### [[Rust Notes]]\n\nRust is a language.",
            "# Index\n- Rust Notes",
            1,
        );

        assert!(prompt.contains("What is Rust?"));
        assert!(prompt.contains("Relevant Notes (1 found)"));
        assert!(prompt.contains("Rust Notes"));
        assert!(prompt.contains("Grimoire Index"));
    }

    #[test]
    fn test_build_user_prompt_no_notes() {
        let prompt = build_user_prompt("Obscure topic?", "", "", 0);

        assert!(prompt.contains("Obscure topic?"));
        assert!(prompt.contains("No relevant notes found"));
        assert!(!prompt.contains("Grimoire Index"));
    }

    #[test]
    fn test_build_user_prompt_no_index() {
        let prompt = build_user_prompt("Q?", "### [[Note]]\n\nContent", "", 1);

        assert!(!prompt.contains("Grimoire Index"));
        assert!(prompt.contains("Relevant Notes"));
    }

    #[test]
    fn test_build_user_prompt_has_instructions() {
        let prompt = build_user_prompt("Q?", "", "", 0);
        assert!(prompt.contains("[[wiki-links]]"));
        assert!(prompt.contains("Instructions"));
    }

    // --- default_query_system_prompt ---

    #[test]
    fn test_default_query_prompt() {
        let prompt = default_query_system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("Zettelkasten"));
        assert!(prompt.contains("wiki-links"));
    }

    // --- QueryResult fields ---

    #[test]
    fn test_query_result_fields() {
        let result = QueryResult {
            answer: "Rust is a systems language.".to_string(),
            question: "What is Rust?".to_string(),
            consulted_notes: vec!["Rust Notes".to_string()],
            model: "llama3".to_string(),
            total_tokens: 300,
            estimated_cost_usd: 0.0,
        };

        assert_eq!(result.answer, "Rust is a systems language.");
        assert_eq!(result.question, "What is Rust?");
        assert_eq!(result.consulted_notes.len(), 1);
        assert_eq!(result.model, "llama3");
    }

    // --- QueryWorkflow::ask (error cases) ---

    #[test]
    fn test_query_workflow_disabled_llm() {
        let config = LlmConfig {
            enabled: false,
            ..Default::default()
        };

        let temp = tempfile::TempDir::new().unwrap();
        let db = Database::new(&temp.path().join("vault.db")).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(QueryWorkflow::ask(
            &config,
            temp.path(),
            &db,
            "What is Rust?",
        ));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }

    #[test]
    fn test_query_workflow_empty_question() {
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };

        let temp = tempfile::TempDir::new().unwrap();
        let db = Database::new(&temp.path().join("vault.db")).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(QueryWorkflow::ask(&config, temp.path(), &db, ""));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_query_workflow_whitespace_question() {
        let config = LlmConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            ..Default::default()
        };

        let temp = tempfile::TempDir::new().unwrap();
        let db = Database::new(&temp.path().join("vault.db")).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(QueryWorkflow::ask(&config, temp.path(), &db, "   "));

        assert!(result.is_err());
    }
}
