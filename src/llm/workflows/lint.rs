use std::path::Path;

use crate::config::LlmConfig;
use crate::db::Database;
use crate::error::Result;
use crate::llm::workflow::{truncate_to_token_budget, WorkflowEngine};
use crate::storage::activity_log::{ActivityEntry, ActivityKind, ActivityLog};

/// A single issue found during linting.
#[derive(Debug, Clone)]
pub struct LintIssue {
    /// Category of the issue (orphan, broken_link, stale, short, unprocessed_source).
    pub category: String,
    /// Human-readable description of the issue.
    pub description: String,
    /// The note title or source path affected.
    pub affected: String,
    /// LLM-suggested fix (if LLM was used).
    pub suggestion: Option<String>,
}

/// Full lint report for a grimoire.
#[derive(Debug, Clone)]
pub struct LintReport {
    /// Issues found during local analysis.
    pub issues: Vec<LintIssue>,
    /// LLM-generated analysis and suggestions (if LLM was used).
    pub llm_analysis: Option<String>,
    /// The LLM model used (if any).
    pub model: Option<String>,
    /// Total tokens consumed by LLM (0 if no LLM used).
    pub total_tokens: u32,
    /// Estimated cost in USD.
    pub estimated_cost_usd: f64,
}

impl LintReport {
    /// Number of issues found.
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    /// True if no issues were found.
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }

    /// Count issues by category.
    pub fn count_by_category(&self, category: &str) -> usize {
        self.issues
            .iter()
            .filter(|i| i.category == category)
            .count()
    }

    /// Format the report as markdown.
    pub fn to_markdown(&self) -> String {
        let mut out = String::from("# Wiki Lint Report\n\n");

        out.push_str("## Summary\n\n");
        out.push_str(&format!("- Total issues: {}\n", self.issue_count()));
        out.push_str(&format!(
            "- Orphan notes: {}\n",
            self.count_by_category("orphan")
        ));
        out.push_str(&format!(
            "- Short notes: {}\n",
            self.count_by_category("short")
        ));
        out.push_str(&format!(
            "- Unprocessed sources: {}\n",
            self.count_by_category("unprocessed_source")
        ));
        out.push('\n');

        if !self.issues.is_empty() {
            out.push_str("## Issues\n\n");

            for issue in &self.issues {
                out.push_str(&format!("### [{}] {}\n\n", issue.category, issue.affected));
                out.push_str(&format!("{}\n\n", issue.description));
                if let Some(suggestion) = &issue.suggestion {
                    out.push_str(&format!("**Suggestion:** {}\n\n", suggestion));
                }
            }
        }

        if let Some(analysis) = &self.llm_analysis {
            out.push_str("## LLM Analysis\n\n");
            out.push_str(analysis);
            out.push('\n');
        }

        out
    }
}

/// Orchestrates the wiki lint workflow.
///
/// Performs local analysis first (orphan notes, short notes, unprocessed
/// sources), then optionally sends the findings to an LLM for deeper
/// analysis and suggestions.
pub struct LintWorkflow;

impl LintWorkflow {
    /// Run local-only lint (no LLM required).
    ///
    /// Checks for:
    /// - Orphan notes (notes with no inbound links, excluding daily/index)
    /// - Short notes (< 100 chars content, excluding fleeting)
    /// - Unprocessed sources (sources in `raw/` with no literature note)
    pub fn local_lint(vault_path: &Path, db: &Database) -> Result<LintReport> {
        let mut issues = Vec::new();

        // Check for orphan notes
        find_orphan_notes(db, &mut issues)?;

        // Check for short notes
        find_short_notes(db, &mut issues)?;

        // Check for unprocessed sources
        find_unprocessed_sources(vault_path, db, &mut issues)?;

        // Log the lint
        let activity_log = ActivityLog::new(vault_path);
        let entry = ActivityEntry::new(
            ActivityKind::Llm,
            format!("Local lint: {} issues found", issues.len()),
        );
        let _ = activity_log.append(&entry);

        Ok(LintReport {
            issues,
            llm_analysis: None,
            model: None,
            total_tokens: 0,
            estimated_cost_usd: 0.0,
        })
    }

    /// Run full lint with LLM analysis.
    ///
    /// First performs local analysis, then sends the findings to the LLM
    /// for deeper suggestions (cross-references, contradictions, etc.).
    pub async fn full_lint(
        config: &LlmConfig,
        vault_path: &Path,
        db: &Database,
    ) -> Result<LintReport> {
        // Start with local lint
        let mut report = Self::local_lint(vault_path, db)?;

        // Build the workflow engine
        let mut engine = WorkflowEngine::new(config, vault_path)?;

        // Gather grimoire overview for context
        let grimoire_summary = build_grimoire_summary(db)?;

        // Build system prompt from .skills/
        let context = engine.build_context("lint")?;
        let system_prompt = if context.is_empty() {
            default_lint_system_prompt()
        } else {
            context.build()
        };

        // Build user prompt with local lint results + grimoire overview
        let local_report_md = report.to_markdown();
        let max_content_tokens = config.max_tokens.saturating_sub(2000).max(1000);
        let truncated_summary = truncate_to_token_budget(&grimoire_summary, max_content_tokens / 2);
        let truncated_report = truncate_to_token_budget(&local_report_md, max_content_tokens / 2);

        let user_prompt = format!(
            "## Grimoire Overview\n\n{}\n\n\
             ## Local Lint Results\n\n{}\n\n\
             ## Instructions\n\n\
             Analyze the grimoire health based on the overview and local lint results.\n\
             Provide:\n\
             1. Assessment of overall grimoire health\n\
             2. Specific suggestions for each issue found\n\
             3. Cross-reference opportunities (notes that should link to each other)\n\
             4. Any patterns or concerns you notice\n\n\
             Format as markdown. Use [[wiki-links]] for note references.",
            truncated_summary, truncated_report,
        );

        // Execute the LLM call
        let result = engine.execute(&system_prompt, &user_prompt, "lint").await?;

        report.llm_analysis = Some(result.content);
        report.model = Some(result.model);
        report.total_tokens = result.prompt_tokens + result.completion_tokens;
        report.estimated_cost_usd = result.estimated_cost_usd;

        // Flush usage
        let _ = engine.finish();

        // Log the full lint
        let activity_log = ActivityLog::new(vault_path);
        let entry = ActivityEntry::new(
            ActivityKind::Llm,
            format!(
                "Full lint: {} issues, LLM analysis complete",
                report.issues.len()
            ),
        )
        .with_detail(format!(
            "Model: {}",
            report.model.as_deref().unwrap_or("unknown")
        ))
        .with_detail(format!("Tokens: {}", report.total_tokens));
        let _ = activity_log.append(&entry);

        Ok(report)
    }
}

/// Find notes that have no inbound links (orphans).
///
/// Excludes daily and index notes which are entry points by nature.
fn find_orphan_notes(db: &Database, issues: &mut Vec<LintIssue>) -> Result<()> {
    let notes = db.list_notes(1000, 0)?;

    for note in &notes {
        // Skip daily and index notes — they're entry points
        let type_str = note.note_type.as_str();
        if type_str == "daily" || type_str == "index" {
            continue;
        }

        // Check if note has any inbound links (backlinks)
        let backlinks = db.get_backlinks(&note.id)?;
        if backlinks.is_empty() {
            issues.push(LintIssue {
                category: "orphan".to_string(),
                description: format!(
                    "Note '{}' ({}) has no inbound links from other notes.",
                    note.title, type_str,
                ),
                affected: note.title.clone(),
                suggestion: None,
            });
        }
    }

    Ok(())
}

/// Find notes with very short content (likely incomplete).
///
/// Excludes fleeting notes which are intentionally brief.
fn find_short_notes(db: &Database, issues: &mut Vec<LintIssue>) -> Result<()> {
    let notes = db.list_notes(1000, 0)?;

    for note in &notes {
        // Skip fleeting notes — they're intentionally brief
        if note.note_type.as_str() == "fleeting" {
            continue;
        }

        // Strip frontmatter for content length check
        let content = strip_frontmatter(&note.content);
        if content.len() < 100 && !content.is_empty() {
            issues.push(LintIssue {
                category: "short".to_string(),
                description: format!(
                    "Note '{}' has very short content ({} chars). Consider expanding or merging.",
                    note.title,
                    content.len(),
                ),
                affected: note.title.clone(),
                suggestion: None,
            });
        }
    }

    Ok(())
}

/// Find sources in `raw/` that have no corresponding literature note.
fn find_unprocessed_sources(
    _vault_path: &Path,
    db: &Database,
    issues: &mut Vec<LintIssue>,
) -> Result<()> {
    let sources = db.list_sources(1000, 0)?;
    let lit_notes = db.list_notes_by_type("literature", 1000, 0)?;

    // Build a set of source paths referenced by literature notes
    let referenced_sources: std::collections::HashSet<String> = lit_notes
        .iter()
        .filter_map(|note| {
            if let crate::note::NoteType::Literature { source } = &note.note_type {
                if !source.is_empty() {
                    return Some(source.clone());
                }
            }
            // Also check the note.source field
            note.source.clone()
        })
        .collect();

    for source in &sources {
        if !referenced_sources.contains(&source.file_path) {
            issues.push(LintIssue {
                category: "unprocessed_source".to_string(),
                description: format!(
                    "Source '{}' ({}) has no corresponding literature note.",
                    source.title, source.file_path,
                ),
                affected: source.file_path.clone(),
                suggestion: Some(format!(
                    "Run `ztlgr ingest --process` to create a literature note for '{}'",
                    source.title,
                )),
            });
        }
    }

    Ok(())
}

/// Build a summary of the grimoire for LLM context.
fn build_grimoire_summary(db: &Database) -> Result<String> {
    let total_notes = db.count_notes()?;
    let total_links = db.count_links()?;
    let total_sources = db.count_sources()?;

    let mut summary = format!(
        "Grimoire statistics:\n\
         - Total notes: {}\n\
         - Total links: {}\n\
         - Total sources: {}\n\n",
        total_notes, total_links, total_sources,
    );

    // List note titles by type for overview
    for note_type in &[
        "permanent",
        "literature",
        "reference",
        "fleeting",
        "daily",
        "index",
    ] {
        let notes = db.list_notes_by_type(note_type, 20, 0)?;
        if !notes.is_empty() {
            summary.push_str(&format!("### {} notes ({}):\n", note_type, notes.len()));
            for note in &notes {
                let preview: String = note
                    .content
                    .lines()
                    .find(|l| !l.starts_with('#') && !l.starts_with("---") && !l.trim().is_empty())
                    .unwrap_or("")
                    .chars()
                    .take(80)
                    .collect();
                summary.push_str(&format!("- [[{}]]: {}\n", note.title, preview));
            }
            summary.push('\n');
        }
    }

    Ok(summary)
}

/// Strip YAML frontmatter from note content for length checking.
fn strip_frontmatter(content: &str) -> &str {
    if let Some(rest) = content.strip_prefix("---") {
        if let Some(end) = rest.find("---") {
            let after = end + 3;
            return rest.get(after..).unwrap_or("").trim();
        }
    }
    content
}

/// Default system prompt when no `.skills/` directory is configured.
fn default_lint_system_prompt() -> String {
    "You are a wiki quality analyst for a personal Zettelkasten grimoire.\n\n\
     Your task is to analyze the grimoire's health and suggest improvements.\n\n\
     Guidelines:\n\
     - Identify issues: orphan notes, missing cross-references, stale content\n\
     - Suggest specific fixes using [[wiki-links]] notation\n\
     - Look for notes that should be connected but aren't\n\
     - Flag potential contradictions between notes\n\
     - Recommend structural improvements\n\
     - Be specific and actionable in your suggestions\n\
     - Use markdown formatting"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::Note;
    use tempfile::TempDir;

    // --- LintIssue ---

    #[test]
    fn test_lint_issue_fields() {
        let issue = LintIssue {
            category: "orphan".to_string(),
            description: "Note has no links".to_string(),
            affected: "My Note".to_string(),
            suggestion: Some("Link from [[Index]]".to_string()),
        };

        assert_eq!(issue.category, "orphan");
        assert_eq!(issue.affected, "My Note");
        assert!(issue.suggestion.is_some());
    }

    // --- LintReport ---

    #[test]
    fn test_lint_report_empty() {
        let report = LintReport {
            issues: vec![],
            llm_analysis: None,
            model: None,
            total_tokens: 0,
            estimated_cost_usd: 0.0,
        };

        assert!(report.is_clean());
        assert_eq!(report.issue_count(), 0);
    }

    #[test]
    fn test_lint_report_with_issues() {
        let report = LintReport {
            issues: vec![
                LintIssue {
                    category: "orphan".to_string(),
                    description: "desc".to_string(),
                    affected: "Note A".to_string(),
                    suggestion: None,
                },
                LintIssue {
                    category: "orphan".to_string(),
                    description: "desc".to_string(),
                    affected: "Note B".to_string(),
                    suggestion: None,
                },
                LintIssue {
                    category: "short".to_string(),
                    description: "desc".to_string(),
                    affected: "Note C".to_string(),
                    suggestion: None,
                },
            ],
            llm_analysis: None,
            model: None,
            total_tokens: 0,
            estimated_cost_usd: 0.0,
        };

        assert!(!report.is_clean());
        assert_eq!(report.issue_count(), 3);
        assert_eq!(report.count_by_category("orphan"), 2);
        assert_eq!(report.count_by_category("short"), 1);
        assert_eq!(report.count_by_category("broken_link"), 0);
    }

    #[test]
    fn test_lint_report_to_markdown() {
        let report = LintReport {
            issues: vec![LintIssue {
                category: "orphan".to_string(),
                description: "No inbound links".to_string(),
                affected: "Lonely Note".to_string(),
                suggestion: Some("Link from [[Index]]".to_string()),
            }],
            llm_analysis: Some("The grimoire looks healthy overall.".to_string()),
            model: Some("llama3".to_string()),
            total_tokens: 500,
            estimated_cost_usd: 0.0,
        };

        let md = report.to_markdown();
        assert!(md.contains("# Wiki Lint Report"));
        assert!(md.contains("Total issues: 1"));
        assert!(md.contains("Orphan notes: 1"));
        assert!(md.contains("[orphan] Lonely Note"));
        assert!(md.contains("No inbound links"));
        assert!(md.contains("Link from [[Index]]"));
        assert!(md.contains("LLM Analysis"));
        assert!(md.contains("grimoire looks healthy"));
    }

    #[test]
    fn test_lint_report_to_markdown_clean() {
        let report = LintReport {
            issues: vec![],
            llm_analysis: None,
            model: None,
            total_tokens: 0,
            estimated_cost_usd: 0.0,
        };

        let md = report.to_markdown();
        assert!(md.contains("Total issues: 0"));
        assert!(!md.contains("## Issues"));
        assert!(!md.contains("## LLM Analysis"));
    }

    // --- strip_frontmatter ---

    #[test]
    fn test_strip_frontmatter_with_frontmatter() {
        let content = "---\ntitle: Test\n---\n\nActual content here";
        assert_eq!(strip_frontmatter(content), "Actual content here");
    }

    #[test]
    fn test_strip_frontmatter_without() {
        let content = "Just plain content";
        assert_eq!(strip_frontmatter(content), "Just plain content");
    }

    #[test]
    fn test_strip_frontmatter_incomplete() {
        let content = "---\nunclosed frontmatter";
        assert_eq!(strip_frontmatter(content), "---\nunclosed frontmatter");
    }

    #[test]
    fn test_strip_frontmatter_empty_body() {
        let content = "---\ntitle: Test\n---\n";
        let result = strip_frontmatter(content);
        assert!(result.is_empty());
    }

    // --- default_lint_system_prompt ---

    #[test]
    fn test_default_lint_prompt() {
        let prompt = default_lint_system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("Zettelkasten"));
        assert!(prompt.contains("wiki-links"));
    }

    // --- build_grimoire_summary ---

    #[test]
    fn test_build_grimoire_summary_empty_db() {
        let temp = TempDir::new().unwrap();
        let db = Database::new(&temp.path().join("vault.db")).unwrap();

        let summary = build_grimoire_summary(&db).unwrap();
        assert!(summary.contains("Total notes: 0"));
        assert!(summary.contains("Total links: 0"));
        assert!(summary.contains("Total sources: 0"));
    }

    #[test]
    fn test_build_grimoire_summary_with_notes() {
        let temp = TempDir::new().unwrap();
        let db = Database::new(&temp.path().join("vault.db")).unwrap();

        let note = Note::new("Test Note".to_string(), "Some content here.".to_string());
        db.create_note(&note).unwrap();

        let summary = build_grimoire_summary(&db).unwrap();
        assert!(summary.contains("Total notes: 1"));
        assert!(summary.contains("permanent notes"));
        assert!(summary.contains("[[Test Note]]"));
    }

    // --- LintWorkflow::local_lint ---

    #[test]
    fn test_local_lint_empty_grimoire() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".ztlgr")).unwrap();
        let db = Database::new(&temp.path().join(".ztlgr").join("vault.db")).unwrap();

        let report = LintWorkflow::local_lint(temp.path(), &db).unwrap();
        assert!(report.is_clean());
        assert!(report.llm_analysis.is_none());
    }

    #[test]
    fn test_local_lint_finds_short_notes() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".ztlgr")).unwrap();
        let db = Database::new(&temp.path().join(".ztlgr").join("vault.db")).unwrap();

        // Create a short permanent note
        let note = Note::new("Short Note".to_string(), "Too short.".to_string());
        db.create_note(&note).unwrap();

        let report = LintWorkflow::local_lint(temp.path(), &db).unwrap();
        assert!(report.count_by_category("short") >= 1);

        let short_issue = report
            .issues
            .iter()
            .find(|i| i.category == "short")
            .unwrap();
        assert!(short_issue.affected.contains("Short Note"));
    }

    #[test]
    fn test_local_lint_skips_fleeting_short_notes() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".ztlgr")).unwrap();
        let db = Database::new(&temp.path().join(".ztlgr").join("vault.db")).unwrap();

        // Create a short fleeting note (should not be flagged)
        let note = Note::new("Quick Thought".to_string(), "Brief.".to_string())
            .with_type(crate::note::NoteType::Fleeting);
        db.create_note(&note).unwrap();

        let report = LintWorkflow::local_lint(temp.path(), &db).unwrap();
        let short_issues: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.category == "short" && i.affected == "Quick Thought")
            .collect();
        assert!(short_issues.is_empty());
    }

    #[test]
    fn test_local_lint_finds_unprocessed_sources() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".ztlgr")).unwrap();
        std::fs::create_dir_all(temp.path().join("raw")).unwrap();
        let db = Database::new(&temp.path().join(".ztlgr").join("vault.db")).unwrap();

        // Create a source with no literature note
        let source = crate::source::Source::new("Article", "hash123", "raw/article.md", 1000);
        db.create_source(&source).unwrap();

        let report = LintWorkflow::local_lint(temp.path(), &db).unwrap();
        assert!(report.count_by_category("unprocessed_source") >= 1);

        let src_issue = report
            .issues
            .iter()
            .find(|i| i.category == "unprocessed_source")
            .unwrap();
        assert!(src_issue.affected.contains("raw/article.md"));
        assert!(src_issue.suggestion.is_some());
    }

    #[test]
    fn test_local_lint_no_false_positive_for_processed_source() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".ztlgr")).unwrap();
        let db = Database::new(&temp.path().join(".ztlgr").join("vault.db")).unwrap();

        // Create a source
        let source = crate::source::Source::new("Article", "hash123", "raw/article.md", 1000);
        db.create_source(&source).unwrap();

        // Create a literature note referencing this source.
        // NOTE: NoteType::Literature { source } inner field is lost during DB
        // round-trip (always reconstructed as empty string). The `note.source`
        // field (DB column) is what persists, so we set that too.
        let mut note = Note::new(
            "Literature: Article".to_string(),
            "Summary of article.".to_string(),
        )
        .with_type(crate::note::NoteType::Literature {
            source: "raw/article.md".to_string(),
        });
        note.source = Some("raw/article.md".to_string());
        db.create_note(&note).unwrap();

        let report = LintWorkflow::local_lint(temp.path(), &db).unwrap();
        let unprocessed: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.category == "unprocessed_source")
            .collect();
        assert!(unprocessed.is_empty());
    }

    // --- full_lint (error cases) ---

    #[test]
    fn test_full_lint_disabled_llm() {
        let config = LlmConfig {
            enabled: false,
            ..Default::default()
        };

        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".ztlgr")).unwrap();
        let db = Database::new(&temp.path().join(".ztlgr").join("vault.db")).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(LintWorkflow::full_lint(&config, temp.path(), &db));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }
}
