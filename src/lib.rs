pub mod cli;
pub mod config;
pub mod db;
pub mod error;
pub mod graph;
pub mod link;
pub mod llm;
pub mod note;
pub mod setup;
pub mod skills;
pub mod source;
pub mod storage;
pub mod ui;

// Re-export commonly used types
pub use config::Config;
pub use db::Database;
pub use error::{Result, ZtlgrError};
pub use link::{LinkError, LinkFormat, LinkInfo, LinkTarget};
pub use llm::{ContextBuilder, LlmProvider, LlmRequest, LlmResponse, ProviderKind};
pub use note::{Note, NoteId, NoteType, ZettelId};
pub use skills::Skills;
pub use source::{Source, SourceId};
pub use storage::{Format, MarkdownStorage, OrgStorage, Storage, Vault};

// Version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::tempdir;

    #[test]
    fn test_markdown_save_creates_file() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let note_path = temp_dir.path().join("test_note.md");

        let note = Note {
            id: NoteId::new(),
            title: "Test Note".to_string(),
            content: "This is test content".to_string(),
            note_type: NoteType::Fleeting,
            zettel_id: None,
            parent_id: None,
            source: None,
            metadata: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let storage = MarkdownStorage::new();
        let result = storage.write_note(&note, &note_path);

        assert!(result.is_ok(), "Failed to write note: {:?}", result);
        assert!(note_path.exists(), "Note file was not created");

        let content = std::fs::read_to_string(&note_path).expect("Failed to read written file");
        assert!(
            content.contains("Test Note"),
            "File doesn't contain note title"
        );
        assert!(
            content.contains("This is test content"),
            "File doesn't contain note content"
        );
    }

    #[test]
    fn test_markdown_save_preserves_content() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let note_path = temp_dir.path().join("content_test.md");

        let original_content = "Line 1\nLine 2\nLine 3";
        let note = Note {
            id: NoteId::new(),
            title: "Multi-line Note".to_string(),
            content: original_content.to_string(),
            note_type: NoteType::Permanent,
            zettel_id: None,
            parent_id: None,
            source: None,
            metadata: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let storage = MarkdownStorage::new();
        storage
            .write_note(&note, &note_path)
            .expect("Failed to write note");

        let content = std::fs::read_to_string(&note_path).expect("Failed to read written file");
        assert!(
            content.contains(original_content),
            "Content not preserved. Original: {}, Found: {}",
            original_content,
            content
        );
    }

    #[test]
    fn test_markdown_save_creates_parent_dirs() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let nested_path = temp_dir
            .path()
            .join("a")
            .join("b")
            .join("c")
            .join("note.md");

        let note = Note {
            id: NoteId::new(),
            title: "Nested Note".to_string(),
            content: "Nested content".to_string(),
            note_type: NoteType::Fleeting,
            zettel_id: None,
            parent_id: None,
            source: None,
            metadata: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let storage = MarkdownStorage::new();
        let result = storage.write_note(&note, &nested_path);

        assert!(result.is_ok(), "Failed to write note to nested path");
        assert!(nested_path.exists(), "Nested file was not created");
        assert!(
            nested_path.parent().unwrap().exists(),
            "Parent directories were not created"
        );
    }
}
