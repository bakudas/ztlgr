use std::fmt;
use thiserror::Error;

pub mod fuzzy;
pub mod parser;
pub mod validator;

/// Represents different link format styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkFormat {
    /// Wiki-style: [[note-id]] or [[note-id|label]]
    Wiki,
    /// Markdown: [label](note-id)
    Markdown,
    /// Org-mode: [[id]] or [[id][label]]
    OrgMode,
}

impl fmt::Display for LinkFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkFormat::Wiki => write!(f, "Wiki"),
            LinkFormat::Markdown => write!(f, "Markdown"),
            LinkFormat::OrgMode => write!(f, "Org-Mode"),
        }
    }
}

/// Represents the target of a link
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkTarget {
    /// Internal link to a note by ID
    NoteId(String),
    /// External URL (http, https, ftp, mailto, etc.)
    ExternalUrl(String),
}

impl fmt::Display for LinkTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkTarget::NoteId(id) => write!(f, "NoteId({})", id),
            LinkTarget::ExternalUrl(url) => write!(f, "URL({})", url),
        }
    }
}

/// Position information for a link in the source text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkPosition {
    /// Line number (0-indexed)
    pub line: usize,
    /// Starting column (0-indexed)
    pub start_col: usize,
    /// Ending column (0-indexed)
    pub end_col: usize,
}

impl fmt::Display for LinkPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}..{}", self.line, self.start_col, self.end_col)
    }
}

/// Complete link information extracted from text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkInfo {
    /// The detected link format
    pub format: LinkFormat,
    /// What the link points to
    pub target: LinkTarget,
    /// Human-readable label (or empty string if none)
    pub label: String,
    /// Original raw text of the link (e.g., "[[note-id|label]]")
    pub raw: String,
    /// Position in source text
    pub position: LinkPosition,
}

impl fmt::Display for LinkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LinkInfo {{ format: {}, target: {}, label: \"{}\", pos: {} }}",
            self.format, self.target, self.label, self.position
        )
    }
}

/// Statistics about links in a document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkStats {
    /// Total number of links found
    pub total_links: usize,
    /// Number of valid links
    pub valid_links: usize,
    /// Number of invalid/malformed links
    pub invalid_links: usize,
    /// Number of external URL links
    pub external_links: usize,
    /// Number of internal note ID links
    pub internal_links: usize,
}

impl fmt::Display for LinkStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LinkStats {{ total: {}, valid: {}, invalid: {}, external: {}, internal: {} }}",
            self.total_links,
            self.valid_links,
            self.invalid_links,
            self.external_links,
            self.internal_links
        )
    }
}

impl LinkStats {
    /// Create empty stats (all zeros)
    pub fn new() -> Self {
        Self {
            total_links: 0,
            valid_links: 0,
            invalid_links: 0,
            external_links: 0,
            internal_links: 0,
        }
    }
}

impl Default for LinkStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for link parsing operations
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LinkError {
    #[error("Invalid link format: {0}")]
    InvalidFormat(String),

    #[error("Empty link target")]
    EmptyTarget,

    #[error("Malformed link syntax: {0}")]
    MalformedSyntax(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Result type for link operations
pub type Result<T> = std::result::Result<T, LinkError>;

// Re-export parser module's public items
pub use fuzzy::fuzzy_match;
pub use parser::parse_link;
pub use validator::{LinkValidator, ValidatedLink};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_format_display() {
        assert_eq!(LinkFormat::Wiki.to_string(), "Wiki");
        assert_eq!(LinkFormat::Markdown.to_string(), "Markdown");
        assert_eq!(LinkFormat::OrgMode.to_string(), "Org-Mode");
    }

    #[test]
    fn test_link_target_display() {
        let note_id = LinkTarget::NoteId("note-123".to_string());
        assert!(note_id.to_string().contains("note-123"));

        let url = LinkTarget::ExternalUrl("https://example.com".to_string());
        assert!(url.to_string().contains("https://example.com"));
    }

    #[test]
    fn test_link_position_display() {
        let pos = LinkPosition {
            line: 5,
            start_col: 10,
            end_col: 25,
        };
        assert_eq!(pos.to_string(), "5:10..25");
    }

    #[test]
    fn test_link_info_display() {
        let info = LinkInfo {
            format: LinkFormat::Wiki,
            target: LinkTarget::NoteId("test-note".to_string()),
            label: "Test Label".to_string(),
            raw: "[[test-note|Test Label]]".to_string(),
            position: LinkPosition {
                line: 0,
                start_col: 0,
                end_col: 24,
            },
        };
        let display = info.to_string();
        assert!(display.contains("Wiki"));
        assert!(display.contains("test-note"));
        assert!(display.contains("Test Label"));
    }

    #[test]
    fn test_link_stats_default() {
        let stats = LinkStats::default();
        assert_eq!(stats.total_links, 0);
        assert_eq!(stats.valid_links, 0);
        assert_eq!(stats.invalid_links, 0);
        assert_eq!(stats.external_links, 0);
        assert_eq!(stats.internal_links, 0);
    }

    #[test]
    fn test_link_error_display() {
        let err = LinkError::InvalidFormat("test error".to_string());
        assert!(err.to_string().contains("Invalid link format"));

        let err2 = LinkError::EmptyTarget;
        assert!(err2.to_string().contains("Empty link target"));
    }
}
