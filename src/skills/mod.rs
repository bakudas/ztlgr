pub mod generator;

use std::path::{Path, PathBuf};

use crate::error::{Result, ZtlgrError};

/// Expected files in the .skills/ directory tree.
pub const SKILLS_FILES: &[&str] = &[
    "README.md",
    "conventions.md",
    "workflows/ingest.md",
    "workflows/query.md",
    "workflows/lint.md",
    "workflows/maintain.md",
    "templates/source-summary.md",
    "templates/entity-page.md",
    "templates/comparison.md",
    "templates/index-entry.md",
    "context/domain.md",
    "context/priorities.md",
];

/// Subdirectories under .skills/
pub const SKILLS_DIRS: &[&str] = &["workflows", "templates", "context"];

/// Represents the .skills/ directory of a vault.
///
/// Provides loading, validation, and file access for the LLM schema layer.
#[derive(Debug, Clone)]
pub struct Skills {
    /// Root path to the .skills/ directory
    pub root: PathBuf,
}

/// Result of validating a .skills/ directory.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationReport {
    /// Files that exist and are non-empty
    pub present: Vec<String>,
    /// Files that are expected but missing
    pub missing: Vec<String>,
    /// Files that exist but are empty (0 bytes)
    pub empty: Vec<String>,
}

impl ValidationReport {
    /// Returns true if all expected files are present and non-empty.
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.empty.is_empty()
    }

    /// Total number of issues (missing + empty).
    pub fn issue_count(&self) -> usize {
        self.missing.len() + self.empty.len()
    }
}

impl Skills {
    /// Create a new Skills handle for the given vault path.
    ///
    /// The `vault_path` should be the root of the vault; `.skills/` is appended.
    pub fn new(vault_path: &Path) -> Self {
        Self {
            root: vault_path.join(".skills"),
        }
    }

    /// Whether the .skills/ directory exists at all.
    pub fn exists(&self) -> bool {
        self.root.is_dir()
    }

    /// Validate the .skills/ directory: check which files are present, missing, or empty.
    pub fn validate(&self) -> ValidationReport {
        let mut present = Vec::new();
        let mut missing = Vec::new();
        let mut empty = Vec::new();

        for &file in SKILLS_FILES {
            let path = self.root.join(file);
            if path.exists() {
                match std::fs::metadata(&path) {
                    Ok(meta) if meta.len() > 0 => {
                        present.push(file.to_string());
                    }
                    Ok(_) => {
                        empty.push(file.to_string());
                    }
                    Err(_) => {
                        missing.push(file.to_string());
                    }
                }
            } else {
                missing.push(file.to_string());
            }
        }

        ValidationReport {
            present,
            missing,
            empty,
        }
    }

    /// Read a specific skills file. Returns an error if the file doesn't exist.
    pub fn read_file(&self, relative_path: &str) -> Result<String> {
        let path = self.root.join(relative_path);
        if !path.exists() {
            return Err(ZtlgrError::Skills(format!(
                "skills file not found: {}",
                relative_path
            )));
        }
        std::fs::read_to_string(&path)
            .map_err(|e| ZtlgrError::Skills(format!("failed to read {}: {}", relative_path, e)))
    }

    /// Read the README.md (entry point for LLM agents).
    pub fn readme(&self) -> Result<String> {
        self.read_file("README.md")
    }

    /// Read conventions.md.
    pub fn conventions(&self) -> Result<String> {
        self.read_file("conventions.md")
    }

    /// Read a workflow file by name (e.g. "ingest", "query", "lint", "maintain").
    pub fn workflow(&self, name: &str) -> Result<String> {
        self.read_file(&format!("workflows/{}.md", name))
    }

    /// Read a template file by name (e.g. "source-summary", "entity-page").
    pub fn template(&self, name: &str) -> Result<String> {
        self.read_file(&format!("templates/{}.md", name))
    }

    /// Read a context file by name (e.g. "domain", "priorities").
    pub fn context(&self, name: &str) -> Result<String> {
        self.read_file(&format!("context/{}.md", name))
    }

    /// List all files that exist in the .skills/ directory (recursively).
    pub fn list_files(&self) -> Result<Vec<String>> {
        if !self.exists() {
            return Err(ZtlgrError::Skills(
                ".skills/ directory does not exist".to_string(),
            ));
        }

        let mut files = Vec::new();
        Self::collect_files(&self.root, &self.root, &mut files)?;
        files.sort();
        Ok(files)
    }

    fn collect_files(base: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| ZtlgrError::Skills(format!("failed to read directory: {}", e)))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| ZtlgrError::Skills(format!("failed to read entry: {}", e)))?;
            let path = entry.path();
            if path.is_dir() {
                Self::collect_files(base, &path, out)?;
            } else if let Ok(rel) = path.strip_prefix(base) {
                out.push(rel.to_string_lossy().to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_skills_dir(temp: &TempDir) -> PathBuf {
        let vault_path = temp.path().to_path_buf();
        let skills_root = vault_path.join(".skills");
        std::fs::create_dir_all(skills_root.join("workflows")).unwrap();
        std::fs::create_dir_all(skills_root.join("templates")).unwrap();
        std::fs::create_dir_all(skills_root.join("context")).unwrap();
        vault_path
    }

    // =====================================================================
    // Skills::new and exists
    // =====================================================================

    #[test]
    fn test_skills_new_sets_root_path() {
        let skills = Skills::new(Path::new("/some/vault"));
        assert_eq!(skills.root, PathBuf::from("/some/vault/.skills"));
    }

    #[test]
    fn test_skills_exists_false_when_no_dir() {
        let temp = TempDir::new().unwrap();
        let skills = Skills::new(temp.path());
        assert!(!skills.exists());
    }

    #[test]
    fn test_skills_exists_true_when_dir_present() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        let skills = Skills::new(&vault_path);
        assert!(skills.exists());
    }

    // =====================================================================
    // Validation
    // =====================================================================

    #[test]
    fn test_validate_empty_skills_dir() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        let skills = Skills::new(&vault_path);

        let report = skills.validate();
        assert_eq!(report.missing.len(), SKILLS_FILES.len());
        assert!(report.present.is_empty());
        assert!(report.empty.is_empty());
        assert!(!report.is_complete());
        assert_eq!(report.issue_count(), SKILLS_FILES.len());
    }

    #[test]
    fn test_validate_complete_skills_dir() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        let skills_root = vault_path.join(".skills");

        // Write all expected files with content
        for &file in SKILLS_FILES {
            let path = skills_root.join(file);
            std::fs::write(&path, "# Content\n\nSome text.").unwrap();
        }

        let skills = Skills::new(&vault_path);
        let report = skills.validate();
        assert_eq!(report.present.len(), SKILLS_FILES.len());
        assert!(report.missing.is_empty());
        assert!(report.empty.is_empty());
        assert!(report.is_complete());
        assert_eq!(report.issue_count(), 0);
    }

    #[test]
    fn test_validate_partial_skills_dir() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        let skills_root = vault_path.join(".skills");

        // Write only README.md and conventions.md
        std::fs::write(skills_root.join("README.md"), "# README").unwrap();
        std::fs::write(skills_root.join("conventions.md"), "# Conventions").unwrap();

        let skills = Skills::new(&vault_path);
        let report = skills.validate();
        assert_eq!(report.present.len(), 2);
        assert_eq!(report.missing.len(), SKILLS_FILES.len() - 2);
        assert!(report.empty.is_empty());
        assert!(!report.is_complete());
    }

    #[test]
    fn test_validate_detects_empty_files() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        let skills_root = vault_path.join(".skills");

        // Write README with content, conventions as empty
        std::fs::write(skills_root.join("README.md"), "# README").unwrap();
        std::fs::write(skills_root.join("conventions.md"), "").unwrap();

        let skills = Skills::new(&vault_path);
        let report = skills.validate();
        assert_eq!(report.present, vec!["README.md"]);
        assert_eq!(report.empty, vec!["conventions.md"]);
        assert!(!report.is_complete());
    }

    // =====================================================================
    // File reading
    // =====================================================================

    #[test]
    fn test_read_file_success() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        let skills_root = vault_path.join(".skills");
        std::fs::write(skills_root.join("README.md"), "# Hello").unwrap();

        let skills = Skills::new(&vault_path);
        let content = skills.read_file("README.md").unwrap();
        assert_eq!(content, "# Hello");
    }

    #[test]
    fn test_read_file_not_found() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        let skills = Skills::new(&vault_path);

        let result = skills.read_file("nonexistent.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_readme_convenience() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        std::fs::write(
            vault_path.join(".skills").join("README.md"),
            "# Skills README",
        )
        .unwrap();

        let skills = Skills::new(&vault_path);
        let content = skills.readme().unwrap();
        assert!(content.contains("Skills README"));
    }

    #[test]
    fn test_conventions_convenience() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        std::fs::write(
            vault_path.join(".skills").join("conventions.md"),
            "# Wiki Conventions",
        )
        .unwrap();

        let skills = Skills::new(&vault_path);
        let content = skills.conventions().unwrap();
        assert!(content.contains("Wiki Conventions"));
    }

    #[test]
    fn test_workflow_convenience() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        std::fs::write(
            vault_path.join(".skills").join("workflows/ingest.md"),
            "# Ingest",
        )
        .unwrap();

        let skills = Skills::new(&vault_path);
        let content = skills.workflow("ingest").unwrap();
        assert!(content.contains("Ingest"));
    }

    #[test]
    fn test_template_convenience() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        std::fs::write(
            vault_path
                .join(".skills")
                .join("templates/source-summary.md"),
            "# Template",
        )
        .unwrap();

        let skills = Skills::new(&vault_path);
        let content = skills.template("source-summary").unwrap();
        assert!(content.contains("Template"));
    }

    #[test]
    fn test_context_convenience() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        std::fs::write(
            vault_path.join(".skills").join("context/domain.md"),
            "# Domain",
        )
        .unwrap();

        let skills = Skills::new(&vault_path);
        let content = skills.context("domain").unwrap();
        assert!(content.contains("Domain"));
    }

    // =====================================================================
    // list_files
    // =====================================================================

    #[test]
    fn test_list_files_returns_sorted() {
        let temp = TempDir::new().unwrap();
        let vault_path = create_skills_dir(&temp);
        let skills_root = vault_path.join(".skills");

        std::fs::write(skills_root.join("README.md"), "readme").unwrap();
        std::fs::write(skills_root.join("conventions.md"), "conv").unwrap();
        std::fs::write(skills_root.join("workflows/ingest.md"), "ingest").unwrap();

        let skills = Skills::new(&vault_path);
        let files = skills.list_files().unwrap();
        assert_eq!(files.len(), 3);
        // Should be sorted
        assert!(files[0] <= files[1]);
        assert!(files[1] <= files[2]);
    }

    #[test]
    fn test_list_files_no_skills_dir() {
        let temp = TempDir::new().unwrap();
        let skills = Skills::new(temp.path());
        let result = skills.list_files();
        assert!(result.is_err());
    }

    // =====================================================================
    // ValidationReport
    // =====================================================================

    #[test]
    fn test_validation_report_is_complete() {
        let report = ValidationReport {
            present: vec!["a".into(), "b".into()],
            missing: vec![],
            empty: vec![],
        };
        assert!(report.is_complete());
        assert_eq!(report.issue_count(), 0);
    }

    #[test]
    fn test_validation_report_not_complete_missing() {
        let report = ValidationReport {
            present: vec!["a".into()],
            missing: vec!["b".into()],
            empty: vec![],
        };
        assert!(!report.is_complete());
        assert_eq!(report.issue_count(), 1);
    }

    #[test]
    fn test_validation_report_not_complete_empty() {
        let report = ValidationReport {
            present: vec!["a".into()],
            missing: vec![],
            empty: vec!["b".into()],
        };
        assert!(!report.is_complete());
        assert_eq!(report.issue_count(), 1);
    }
}
