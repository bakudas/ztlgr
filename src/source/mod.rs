pub mod convert;
pub mod ingest;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::note::utils::Id;

/// Marker type for source IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMarker;

/// A unique identifier for a source document in the `raw/` directory.
pub type SourceId = Id<SourceMarker>;

/// Represents an ingested source document stored in the `raw/` directory.
///
/// Sources are immutable reference material (PDFs, articles, book excerpts)
/// that serve as the origin for literature notes. They are tracked in the
/// database for deduplication (via content hash) and indexed in `index.md`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: SourceId,
    pub title: String,
    /// Where the source came from (file path, URL, etc.)
    pub origin: Option<String>,
    /// SHA-256 hash of the file content for deduplication.
    pub content_hash: String,
    pub ingested_at: DateTime<Utc>,
    /// Relative path inside the vault (e.g. `raw/article.md`).
    pub file_path: String,
    /// File size in bytes.
    pub file_size: u64,
    /// MIME type if known (e.g. `text/markdown`, `application/pdf`).
    pub mime_type: Option<String>,
    /// Arbitrary JSON metadata.
    pub metadata: Option<String>,
}

impl Source {
    /// Create a new source with the required fields.
    pub fn new(
        title: impl Into<String>,
        content_hash: impl Into<String>,
        file_path: impl Into<String>,
        file_size: u64,
    ) -> Self {
        Self {
            id: SourceId::new(),
            title: title.into(),
            origin: None,
            content_hash: content_hash.into(),
            ingested_at: Utc::now(),
            file_path: file_path.into(),
            file_size,
            mime_type: None,
            metadata: None,
        }
    }

    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    pub fn with_mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_id_is_unique() {
        let a = SourceId::new();
        let b = SourceId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn test_source_id_parse() {
        let id = SourceId::parse("abc-123").expect("Failed to parse");
        assert_eq!(id.as_str(), "abc-123");
    }

    #[test]
    fn test_source_id_empty_fails() {
        let result = SourceId::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_source_new() {
        let source = Source::new("My Article", "sha256:abc", "raw/article.md", 1024);
        assert_eq!(source.title, "My Article");
        assert_eq!(source.content_hash, "sha256:abc");
        assert_eq!(source.file_path, "raw/article.md");
        assert_eq!(source.file_size, 1024);
        assert!(source.origin.is_none());
        assert!(source.mime_type.is_none());
        assert!(source.metadata.is_none());
    }

    #[test]
    fn test_source_with_origin() {
        let source = Source::new("Article", "hash", "raw/a.md", 100)
            .with_origin("https://example.com/article");
        assert_eq!(
            source.origin.as_deref(),
            Some("https://example.com/article")
        );
    }

    #[test]
    fn test_source_with_mime_type() {
        let source =
            Source::new("Paper", "hash", "raw/paper.pdf", 5000).with_mime_type("application/pdf");
        assert_eq!(source.mime_type.as_deref(), Some("application/pdf"));
    }

    #[test]
    fn test_source_builder_chain() {
        let source = Source::new("Doc", "h", "raw/doc.md", 200)
            .with_origin("/home/user/doc.md")
            .with_mime_type("text/markdown");
        assert_eq!(source.title, "Doc");
        assert!(source.origin.is_some());
        assert!(source.mime_type.is_some());
    }

    #[test]
    fn test_source_serialization() {
        let source = Source::new("Test", "hash123", "raw/test.md", 42);
        let json = serde_json::to_string(&source).expect("Failed to serialize");
        assert!(json.contains("\"title\":\"Test\""));
        assert!(json.contains("\"content_hash\":\"hash123\""));
    }

    #[test]
    fn test_source_deserialization() {
        let source = Source::new("Test", "hash123", "raw/test.md", 42);
        let json = serde_json::to_string(&source).expect("Failed to serialize");
        let deserialized: Source = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.title, "Test");
        assert_eq!(deserialized.content_hash, "hash123");
        assert_eq!(deserialized.file_path, "raw/test.md");
        assert_eq!(deserialized.file_size, 42);
    }
}
