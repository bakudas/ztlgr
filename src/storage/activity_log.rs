use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::error::Result;

/// Types of activities that can be logged.
#[derive(Debug, Clone, PartialEq)]
pub enum ActivityKind {
    Sync,
    Create,
    Update,
    Delete,
    Import,
    Index,
    Ingest,
}

impl ActivityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActivityKind::Sync => "sync",
            ActivityKind::Create => "create",
            ActivityKind::Update => "update",
            ActivityKind::Delete => "delete",
            ActivityKind::Import => "import",
            ActivityKind::Index => "index",
            ActivityKind::Ingest => "ingest",
        }
    }
}

impl std::fmt::Display for ActivityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single activity log entry.
#[derive(Debug, Clone)]
pub struct ActivityEntry {
    pub kind: ActivityKind,
    pub summary: String,
    pub details: Vec<String>,
}

impl ActivityEntry {
    pub fn new(kind: ActivityKind, summary: impl Into<String>) -> Self {
        Self {
            kind,
            summary: summary.into(),
            details: Vec::new(),
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    /// Format this entry as a markdown log block.
    pub fn to_markdown(&self) -> String {
        let now = Utc::now().format("%Y-%m-%d %H:%M");
        let mut out = format!("## [{}] {} | {}\n", now, self.kind, self.summary);

        for detail in &self.details {
            out.push_str(&format!("- {}\n", detail));
        }

        out
    }
}

/// Append-only activity log writer for a vault.
///
/// Writes to `.ztlgr/log.md` inside the vault. Creates the file with a
/// header if it does not exist. Each entry is appended at the end.
pub struct ActivityLog {
    log_path: PathBuf,
}

impl ActivityLog {
    pub fn new(vault_path: &Path) -> Self {
        Self {
            log_path: vault_path.join(".ztlgr").join("log.md"),
        }
    }

    /// Returns the path to the log file.
    pub fn path(&self) -> &Path {
        &self.log_path
    }

    /// Ensure the log file exists. If it does not, create it with a header.
    fn ensure_file(&self) -> Result<()> {
        if !self.log_path.exists() {
            if let Some(parent) = self.log_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let header = "# Activity Log\n\n> Auto-maintained by ztlgr. Do not edit manually.\n\n";
            std::fs::write(&self.log_path, header)?;
        }
        Ok(())
    }

    /// Append an activity entry to the log.
    pub fn append(&self, entry: &ActivityEntry) -> Result<()> {
        self.ensure_file()?;

        let markdown = entry.to_markdown();

        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.log_path)
            .map_err(crate::error::ZtlgrError::Io)?;

        writeln!(file, "{}", markdown)?;

        tracing::info!("Logged activity: {} | {}", entry.kind, entry.summary);
        Ok(())
    }

    /// Read the full log content.
    pub fn read(&self) -> Result<String> {
        if self.log_path.exists() {
            Ok(std::fs::read_to_string(&self.log_path)?)
        } else {
            Ok(String::new())
        }
    }

    /// Convenience: log a sync operation.
    pub fn log_sync(
        &self,
        created_files: usize,
        imported_notes: usize,
        synced: usize,
    ) -> Result<()> {
        let entry = ActivityEntry::new(ActivityKind::Sync, "Full grimoire sync")
            .with_detail(format!("Files created: {}", created_files))
            .with_detail(format!("Notes imported: {}", imported_notes))
            .with_detail(format!("Notes synced: {}", synced));
        self.append(&entry)
    }

    /// Convenience: log a note creation.
    pub fn log_create(&self, title: &str, note_type: &str) -> Result<()> {
        let entry = ActivityEntry::new(ActivityKind::Create, format!("Created \"{}\"", title))
            .with_detail(format!("Type: {}", note_type));
        self.append(&entry)
    }

    /// Convenience: log a note deletion.
    pub fn log_delete(&self, title: &str) -> Result<()> {
        let entry = ActivityEntry::new(ActivityKind::Delete, format!("Deleted \"{}\"", title));
        self.append(&entry)
    }

    /// Convenience: log an import operation.
    pub fn log_import(&self, imported: usize, failed: usize) -> Result<()> {
        let entry = ActivityEntry::new(ActivityKind::Import, "Import from directory")
            .with_detail(format!("Imported: {}", imported))
            .with_detail(format!("Failed: {}", failed));
        self.append(&entry)
    }

    /// Convenience: log an index regeneration.
    pub fn log_index(&self, total_notes: usize) -> Result<()> {
        let entry = ActivityEntry::new(ActivityKind::Index, "Index regenerated")
            .with_detail(format!("Total notes indexed: {}", total_notes));
        self.append(&entry)
    }

    /// Convenience: log a source ingest operation.
    pub fn log_ingest(&self, title: &str, file_path: &str) -> Result<()> {
        let entry = ActivityEntry::new(ActivityKind::Ingest, format!("Ingested \"{}\"", title))
            .with_detail(format!("File: {}", file_path));
        self.append(&entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (ActivityLog, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let vault_path = temp_dir.path();
        std::fs::create_dir_all(vault_path.join(".ztlgr")).unwrap();
        let log = ActivityLog::new(vault_path);
        (log, temp_dir)
    }

    // =====================================================================
    // ActivityEntry tests
    // =====================================================================

    #[test]
    fn test_activity_entry_to_markdown() {
        let entry = ActivityEntry::new(ActivityKind::Sync, "Full sync")
            .with_detail("Files: 10")
            .with_detail("Notes: 5");
        let md = entry.to_markdown();

        assert!(md.contains("] sync | Full sync"));
        assert!(md.contains("- Files: 10"));
        assert!(md.contains("- Notes: 5"));
    }

    #[test]
    fn test_activity_entry_to_markdown_no_details() {
        let entry = ActivityEntry::new(ActivityKind::Create, "Created note");
        let md = entry.to_markdown();

        assert!(md.contains("] create | Created note"));
        assert!(!md.contains("- "));
    }

    #[test]
    fn test_activity_kind_display() {
        assert_eq!(format!("{}", ActivityKind::Sync), "sync");
        assert_eq!(format!("{}", ActivityKind::Create), "create");
        assert_eq!(format!("{}", ActivityKind::Update), "update");
        assert_eq!(format!("{}", ActivityKind::Delete), "delete");
        assert_eq!(format!("{}", ActivityKind::Import), "import");
        assert_eq!(format!("{}", ActivityKind::Index), "index");
        assert_eq!(format!("{}", ActivityKind::Ingest), "ingest");
    }

    // =====================================================================
    // ActivityLog basic operations
    // =====================================================================

    #[test]
    fn test_log_path() {
        let (log, temp) = setup();
        assert_eq!(log.path(), temp.path().join(".ztlgr").join("log.md"));
    }

    #[test]
    fn test_read_nonexistent_log() {
        let temp_dir = TempDir::new().unwrap();
        let log = ActivityLog::new(temp_dir.path());
        let content = log.read().unwrap();
        assert!(content.is_empty());
    }

    #[test]
    fn test_append_creates_file_with_header() {
        let (log, _temp) = setup();

        let entry = ActivityEntry::new(ActivityKind::Sync, "Test");
        log.append(&entry).unwrap();

        let content = log.read().unwrap();
        assert!(content.starts_with("# Activity Log\n"));
        assert!(content.contains("Auto-maintained by ztlgr"));
    }

    #[test]
    fn test_append_adds_entry() {
        let (log, _temp) = setup();

        let entry = ActivityEntry::new(ActivityKind::Create, "Created note A");
        log.append(&entry).unwrap();

        let content = log.read().unwrap();
        assert!(content.contains("create | Created note A"));
    }

    #[test]
    fn test_append_is_additive() {
        let (log, _temp) = setup();

        log.append(&ActivityEntry::new(ActivityKind::Create, "Note A"))
            .unwrap();
        log.append(&ActivityEntry::new(ActivityKind::Create, "Note B"))
            .unwrap();

        let content = log.read().unwrap();
        assert!(content.contains("Note A"));
        assert!(content.contains("Note B"));
    }

    #[test]
    fn test_append_preserves_existing_content() {
        let (log, _temp) = setup();

        // Write first entry
        log.append(&ActivityEntry::new(ActivityKind::Sync, "First"))
            .unwrap();
        let first_content = log.read().unwrap();

        // Write second entry
        log.append(&ActivityEntry::new(ActivityKind::Sync, "Second"))
            .unwrap();
        let second_content = log.read().unwrap();

        // Second content should contain everything from first plus new
        assert!(second_content.starts_with(&first_content.trim_end()));
        assert!(second_content.contains("Second"));
    }

    // =====================================================================
    // Convenience method tests
    // =====================================================================

    #[test]
    fn test_log_sync() {
        let (log, _temp) = setup();
        log.log_sync(3, 5, 10).unwrap();

        let content = log.read().unwrap();
        assert!(content.contains("sync | Full grimoire sync"));
        assert!(content.contains("Files created: 3"));
        assert!(content.contains("Notes imported: 5"));
        assert!(content.contains("Notes synced: 10"));
    }

    #[test]
    fn test_log_create() {
        let (log, _temp) = setup();
        log.log_create("My Note", "permanent").unwrap();

        let content = log.read().unwrap();
        assert!(content.contains("create | Created \"My Note\""));
        assert!(content.contains("Type: permanent"));
    }

    #[test]
    fn test_log_delete() {
        let (log, _temp) = setup();
        log.log_delete("Old Note").unwrap();

        let content = log.read().unwrap();
        assert!(content.contains("delete | Deleted \"Old Note\""));
    }

    #[test]
    fn test_log_import() {
        let (log, _temp) = setup();
        log.log_import(15, 2).unwrap();

        let content = log.read().unwrap();
        assert!(content.contains("import | Import from directory"));
        assert!(content.contains("Imported: 15"));
        assert!(content.contains("Failed: 2"));
    }

    #[test]
    fn test_log_index() {
        let (log, _temp) = setup();
        log.log_index(42).unwrap();

        let content = log.read().unwrap();
        assert!(content.contains("index | Index regenerated"));
        assert!(content.contains("Total notes indexed: 42"));
    }

    #[test]
    fn test_log_ingest() {
        let (log, _temp) = setup();
        log.log_ingest("My Article", "raw/my-article-abc12345.md")
            .unwrap();

        let content = log.read().unwrap();
        assert!(content.contains("ingest | Ingested \"My Article\""));
        assert!(content.contains("File: raw/my-article-abc12345.md"));
    }

    // =====================================================================
    // Edge cases
    // =====================================================================

    #[test]
    fn test_ensure_file_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        // Don't create .ztlgr beforehand -- ensure_file should create it
        let log = ActivityLog::new(temp_dir.path());

        log.append(&ActivityEntry::new(ActivityKind::Sync, "Test"))
            .unwrap();

        assert!(log.path().exists());
    }

    #[test]
    fn test_multiple_entry_types() {
        let (log, _temp) = setup();

        log.log_sync(1, 2, 3).unwrap();
        log.log_create("Note", "permanent").unwrap();
        log.log_delete("Old").unwrap();
        log.log_import(10, 0).unwrap();
        log.log_index(50).unwrap();
        log.log_ingest("Paper", "raw/paper.pdf").unwrap();

        let content = log.read().unwrap();

        // All entries present
        assert!(content.contains("sync |"));
        assert!(content.contains("create |"));
        assert!(content.contains("delete |"));
        assert!(content.contains("import |"));
        assert!(content.contains("index |"));
        assert!(content.contains("ingest |"));
    }
}
