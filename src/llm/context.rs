use std::path::Path;

use crate::error::Result;
use crate::skills::Skills;

/// Assembles LLM context (system prompt) from `.skills/` files and
/// optional wiki page content.
///
/// The context builder reads the skills directory to produce a system
/// prompt that instructs the LLM about the grimoire's conventions,
/// domain knowledge, and workflows. Additional context (e.g. relevant
/// wiki pages, note content) can be appended.
pub struct ContextBuilder {
    /// Sections of the system prompt, accumulated during building.
    sections: Vec<ContextSection>,
}

/// A labeled section of the system prompt.
#[derive(Debug, Clone)]
pub struct ContextSection {
    pub label: String,
    pub content: String,
}

impl ContextSection {
    pub fn new(label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            content: content.into(),
        }
    }
}

impl ContextBuilder {
    /// Create a new empty context builder.
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Load context from the `.skills/` directory in the vault.
    ///
    /// Reads conventions, domain, priorities, and the README to build
    /// a comprehensive system prompt. Silently skips any files that
    /// don't exist.
    pub fn load_skills(mut self, vault_path: &Path) -> Result<Self> {
        let skills = Skills::new(vault_path);

        if !skills.exists() {
            return Ok(self);
        }

        // README provides high-level orientation
        if let Ok(readme) = skills.readme() {
            self.sections
                .push(ContextSection::new("Grimoire Overview", readme));
        }

        // Conventions define naming, format, and style rules
        if let Ok(conventions) = skills.conventions() {
            self.sections
                .push(ContextSection::new("Conventions", conventions));
        }

        // Domain context
        if let Ok(domain) = skills.read_file("context/domain.md") {
            self.sections
                .push(ContextSection::new("Domain Knowledge", domain));
        }

        // Priorities
        if let Ok(priorities) = skills.read_file("context/priorities.md") {
            self.sections
                .push(ContextSection::new("Current Priorities", priorities));
        }

        Ok(self)
    }

    /// Load a specific workflow from `.skills/workflows/`.
    ///
    /// This adds task-specific instructions (e.g. "ingest", "query", "lint")
    /// to the context.
    pub fn load_workflow(mut self, vault_path: &Path, workflow_name: &str) -> Result<Self> {
        let skills = Skills::new(vault_path);
        let path = format!("workflows/{}.md", workflow_name);

        if let Ok(content) = skills.read_file(&path) {
            self.sections.push(ContextSection::new(
                format!("Workflow: {}", workflow_name),
                content,
            ));
        }

        Ok(self)
    }

    /// Load a specific template from `.skills/templates/`.
    pub fn load_template(mut self, vault_path: &Path, template_name: &str) -> Result<Self> {
        let skills = Skills::new(vault_path);
        let path = format!("templates/{}.md", template_name);

        if let Ok(content) = skills.read_file(&path) {
            self.sections.push(ContextSection::new(
                format!("Template: {}", template_name),
                content,
            ));
        }

        Ok(self)
    }

    /// Add arbitrary context (e.g. note content, search results).
    pub fn add_section(mut self, label: impl Into<String>, content: impl Into<String>) -> Self {
        self.sections.push(ContextSection::new(label, content));
        self
    }

    /// Return the accumulated sections.
    pub fn sections(&self) -> &[ContextSection] {
        &self.sections
    }

    /// Return the number of sections.
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// Check if the builder has any content.
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// Build the final system prompt string.
    ///
    /// Each section is separated by a header line for clarity.
    pub fn build(&self) -> String {
        if self.sections.is_empty() {
            return String::new();
        }

        let mut parts = Vec::with_capacity(self.sections.len());

        for section in &self.sections {
            parts.push(format!("## {}\n\n{}", section.label, section.content));
        }

        parts.join("\n\n---\n\n")
    }

    /// Estimate the token count of the built prompt.
    ///
    /// Uses a rough heuristic of ~4 characters per token. This is an
    /// approximation -- actual token counts vary by model and tokenizer.
    pub fn estimate_tokens(&self) -> u32 {
        let total_chars: usize = self
            .sections
            .iter()
            .map(|s| s.label.len() + s.content.len() + 10) // +10 for separators
            .sum();

        (total_chars as f64 / 4.0).ceil() as u32
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_context_builder_new_is_empty() {
        let builder = ContextBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.section_count(), 0);
    }

    #[test]
    fn test_context_builder_default() {
        let builder = ContextBuilder::default();
        assert!(builder.is_empty());
    }

    #[test]
    fn test_context_section_new() {
        let section = ContextSection::new("Label", "Content");
        assert_eq!(section.label, "Label");
        assert_eq!(section.content, "Content");
    }

    #[test]
    fn test_add_section() {
        let builder = ContextBuilder::new()
            .add_section("Test", "Some content")
            .add_section("More", "More content");

        assert_eq!(builder.section_count(), 2);
        assert!(!builder.is_empty());
    }

    #[test]
    fn test_build_empty() {
        let builder = ContextBuilder::new();
        assert_eq!(builder.build(), "");
    }

    #[test]
    fn test_build_single_section() {
        let builder = ContextBuilder::new().add_section("Title", "Body text here.");
        let result = builder.build();

        assert!(result.contains("## Title"));
        assert!(result.contains("Body text here."));
        // Single section should not have a separator
        assert!(!result.contains("---"));
    }

    #[test]
    fn test_build_multiple_sections() {
        let builder = ContextBuilder::new()
            .add_section("First", "Content A")
            .add_section("Second", "Content B");
        let result = builder.build();

        assert!(result.contains("## First"));
        assert!(result.contains("Content A"));
        assert!(result.contains("---"));
        assert!(result.contains("## Second"));
        assert!(result.contains("Content B"));
    }

    #[test]
    fn test_sections_accessor() {
        let builder = ContextBuilder::new()
            .add_section("A", "a-content")
            .add_section("B", "b-content");

        let sections = builder.sections();
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].label, "A");
        assert_eq!(sections[1].label, "B");
    }

    #[test]
    fn test_estimate_tokens_empty() {
        let builder = ContextBuilder::new();
        assert_eq!(builder.estimate_tokens(), 0);
    }

    #[test]
    fn test_estimate_tokens_nonzero() {
        let builder = ContextBuilder::new().add_section("T", "a".repeat(400));
        let tokens = builder.estimate_tokens();
        // ~410 chars + 10 overhead = 420 chars / 4 = 105 tokens
        assert!(tokens > 0);
        assert!(tokens < 200);
    }

    #[test]
    fn test_load_skills_nonexistent_vault() {
        let temp = TempDir::new().unwrap();
        let vault_path = temp.path().join("nonexistent");

        let builder = ContextBuilder::new().load_skills(&vault_path).unwrap();
        // No .skills/ dir means no sections loaded
        assert!(builder.is_empty());
    }

    #[test]
    fn test_load_skills_with_skills_dir() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        // Create a README.md
        std::fs::write(
            skills_dir.join("README.md"),
            "# My Grimoire\nA test grimoire.",
        )
        .unwrap();

        // Create conventions.md
        std::fs::write(
            skills_dir.join("conventions.md"),
            "Use lowercase filenames.",
        )
        .unwrap();

        let builder = ContextBuilder::new().load_skills(temp.path()).unwrap();

        assert_eq!(builder.section_count(), 2);
        assert_eq!(builder.sections()[0].label, "Grimoire Overview");
        assert_eq!(builder.sections()[1].label, "Conventions");
    }

    #[test]
    fn test_load_skills_with_context_files() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        std::fs::create_dir_all(skills_dir.join("context")).unwrap();

        std::fs::write(
            skills_dir.join("context/domain.md"),
            "Domain: software engineering",
        )
        .unwrap();

        std::fs::write(
            skills_dir.join("context/priorities.md"),
            "Priority: documentation",
        )
        .unwrap();

        let builder = ContextBuilder::new().load_skills(temp.path()).unwrap();

        assert_eq!(builder.section_count(), 2);
        assert_eq!(builder.sections()[0].label, "Domain Knowledge");
        assert_eq!(builder.sections()[1].label, "Current Priorities");
    }

    #[test]
    fn test_load_workflow() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        std::fs::create_dir_all(skills_dir.join("workflows")).unwrap();

        std::fs::write(
            skills_dir.join("workflows/ingest.md"),
            "Steps for ingesting a source.",
        )
        .unwrap();

        let builder = ContextBuilder::new()
            .load_workflow(temp.path(), "ingest")
            .unwrap();

        assert_eq!(builder.section_count(), 1);
        assert_eq!(builder.sections()[0].label, "Workflow: ingest");
        assert!(builder.sections()[0].content.contains("ingesting"));
    }

    #[test]
    fn test_load_workflow_missing() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        std::fs::create_dir_all(skills_dir.join("workflows")).unwrap();

        let builder = ContextBuilder::new()
            .load_workflow(temp.path(), "nonexistent")
            .unwrap();

        // Missing workflow should be silently skipped
        assert!(builder.is_empty());
    }

    #[test]
    fn test_load_template() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        std::fs::create_dir_all(skills_dir.join("templates")).unwrap();

        std::fs::write(
            skills_dir.join("templates/source-summary.md"),
            "# Source Summary\nTitle: {{title}}",
        )
        .unwrap();

        let builder = ContextBuilder::new()
            .load_template(temp.path(), "source-summary")
            .unwrap();

        assert_eq!(builder.section_count(), 1);
        assert_eq!(builder.sections()[0].label, "Template: source-summary");
    }

    #[test]
    fn test_chained_loading() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        std::fs::create_dir_all(skills_dir.join("workflows")).unwrap();
        std::fs::create_dir_all(skills_dir.join("templates")).unwrap();

        std::fs::write(skills_dir.join("README.md"), "Overview").unwrap();
        std::fs::write(skills_dir.join("workflows/query.md"), "Query steps").unwrap();
        std::fs::write(
            skills_dir.join("templates/entity-page.md"),
            "Entity template",
        )
        .unwrap();

        let builder = ContextBuilder::new()
            .load_skills(temp.path())
            .unwrap()
            .load_workflow(temp.path(), "query")
            .unwrap()
            .load_template(temp.path(), "entity-page")
            .unwrap()
            .add_section("User Query", "What is Rust?");

        assert_eq!(builder.section_count(), 4);

        let prompt = builder.build();
        assert!(prompt.contains("Overview"));
        assert!(prompt.contains("Query steps"));
        assert!(prompt.contains("Entity template"));
        assert!(prompt.contains("What is Rust?"));
    }

    #[test]
    fn test_build_output_format() {
        let builder = ContextBuilder::new()
            .add_section("A", "Alpha")
            .add_section("B", "Beta");

        let output = builder.build();

        // Should have headers
        assert!(output.starts_with("## A"));
        // Should have separator between sections
        assert!(output.contains("\n\n---\n\n## B"));
    }
}
