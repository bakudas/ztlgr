use regex::Regex;
use std::sync::OnceLock;

use super::{LinkError, LinkFormat, LinkInfo, LinkPosition, LinkTarget, Result};

/// Get or compile the Wiki link regex pattern
/// Format: [[note-id]] or [[note-id|label]]
fn wiki_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"^\[\[([^\]|]+)(?:\|([^\]]*))?\]\]$").expect("Invalid wiki regex")
    })
}

/// Get or compile the Markdown link regex pattern
/// Format: [label](note-id or url)
fn markdown_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| Regex::new(r"^\[([^\]]*)\]\(([^)]*)\)$").expect("Invalid markdown regex"))
}

/// Get or compile the Org-mode link regex pattern
/// Format: [[id]] or [[id][label]] or [[url][label]]
fn orgmode_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"^\[\[([^\]]+)\](?:\[([^\]]*)\])?\]$").expect("Invalid orgmode regex")
    })
}

/// Detect if a string is a URL
/// Returns true for http://, https://, ftp://, or mailto: links
pub fn is_url(s: &str) -> bool {
    let s = s.trim();
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("mailto:")
}

/// Parse a Wiki-style link: [[note-id]] or [[note-id|label]]
///
/// # Arguments
/// * `text` - The raw link text (should include the [[ ]] brackets)
/// * `position` - The position information for this link
///
/// # Returns
/// LinkInfo if parsing succeeds
pub fn parse_wiki_link(text: &str, position: LinkPosition) -> Result<LinkInfo> {
    let text = text.trim();
    let caps = wiki_pattern()
        .captures(text)
        .ok_or_else(|| LinkError::MalformedSyntax(text.to_string()))?;

    let target_str = caps
        .get(1)
        .map(|m| m.as_str().trim())
        .ok_or(LinkError::EmptyTarget)?;

    if target_str.is_empty() {
        return Err(LinkError::EmptyTarget);
    }

    let label = caps
        .get(2)
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();

    let target = if is_url(target_str) {
        LinkTarget::ExternalUrl(target_str.to_string())
    } else {
        LinkTarget::NoteId(target_str.to_string())
    };

    Ok(LinkInfo {
        format: LinkFormat::Wiki,
        target,
        label,
        raw: text.to_string(),
        position,
    })
}

/// Parse a Markdown-style link: [label](note-id or url)
///
/// # Arguments
/// * `text` - The raw link text (should include the [ ] and ( ) brackets)
/// * `position` - The position information for this link
///
/// # Returns
/// LinkInfo if parsing succeeds
pub fn parse_markdown_link(text: &str, position: LinkPosition) -> Result<LinkInfo> {
    let text = text.trim();
    let caps = markdown_pattern()
        .captures(text)
        .ok_or_else(|| LinkError::MalformedSyntax(text.to_string()))?;

    let label = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");

    let target_str = caps
        .get(2)
        .map(|m| m.as_str().trim())
        .ok_or(LinkError::EmptyTarget)?;

    if target_str.is_empty() {
        return Err(LinkError::EmptyTarget);
    }

    let target = if is_url(target_str) {
        LinkTarget::ExternalUrl(target_str.to_string())
    } else {
        LinkTarget::NoteId(target_str.to_string())
    };

    Ok(LinkInfo {
        format: LinkFormat::Markdown,
        target,
        label: label.to_string(),
        raw: text.to_string(),
        position,
    })
}

/// Parse an Org-mode-style link: [[id]] or [[id][label]]
///
/// # Arguments
/// * `text` - The raw link text (should include the [[ ]] brackets)
/// * `position` - The position information for this link
///
/// # Returns
/// LinkInfo if parsing succeeds
pub fn parse_orgmode_link(text: &str, position: LinkPosition) -> Result<LinkInfo> {
    let text = text.trim();
    let caps = orgmode_pattern()
        .captures(text)
        .ok_or_else(|| LinkError::MalformedSyntax(text.to_string()))?;

    let target_str = caps
        .get(1)
        .map(|m| m.as_str().trim())
        .ok_or(LinkError::EmptyTarget)?;

    if target_str.is_empty() {
        return Err(LinkError::EmptyTarget);
    }

    let label = caps
        .get(2)
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();

    let target = if is_url(target_str) {
        LinkTarget::ExternalUrl(target_str.to_string())
    } else {
        LinkTarget::NoteId(target_str.to_string())
    };

    Ok(LinkInfo {
        format: LinkFormat::OrgMode,
        target,
        label,
        raw: text.to_string(),
        position,
    })
}

/// Parse a link in any supported format
///
/// Attempts to auto-detect the link format by trying:
/// 1. Markdown (most specific syntax)
/// 2. Wiki (pipe separator is distinctive)
/// 3. Org-mode (most permissive syntax)
///
/// # Arguments
/// * `text` - The raw link text
/// * `position` - The position information for this link
///
/// # Returns
/// LinkInfo with detected format if any parser succeeds
pub fn parse_link(text: &str, position: LinkPosition) -> Result<LinkInfo> {
    let text = text.trim();

    // Try Markdown first (most specific: [label](target))
    if let Ok(link) = parse_markdown_link(text, position) {
        return Ok(link);
    }

    // Try Wiki (distinctive pipe separator: [[target|label]])
    if let Ok(link) = parse_wiki_link(text, position) {
        return Ok(link);
    }

    // Try Org-mode last (most permissive: [[target]] or [[target][label]])
    if let Ok(link) = parse_orgmode_link(text, position) {
        return Ok(link);
    }

    Err(LinkError::InvalidFormat(text.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(line: usize, start: usize, end: usize) -> LinkPosition {
        LinkPosition {
            line,
            start_col: start,
            end_col: end,
        }
    }

    // ============ Wiki Link Tests (6 tests) ============

    #[test]
    fn test_wiki_parse_basic() {
        let result = parse_wiki_link("[[note-id]]", pos(0, 0, 11));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.format, LinkFormat::Wiki);
        assert_eq!(info.target, LinkTarget::NoteId("note-id".to_string()));
        assert_eq!(info.label, "");
    }

    #[test]
    fn test_wiki_parse_with_label() {
        let result = parse_wiki_link("[[note-id|My Label]]", pos(0, 0, 20));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.format, LinkFormat::Wiki);
        assert_eq!(info.target, LinkTarget::NoteId("note-id".to_string()));
        assert_eq!(info.label, "My Label");
    }

    #[test]
    fn test_wiki_parse_with_whitespace() {
        let result = parse_wiki_link("[[  note-id  |  Label  ]]", pos(0, 0, 26));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.target, LinkTarget::NoteId("note-id".to_string()));
        assert_eq!(info.label, "Label");
    }

    #[test]
    fn test_wiki_parse_external_url() {
        let result = parse_wiki_link("[[https://example.com]]", pos(0, 0, 23));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(
            info.target,
            LinkTarget::ExternalUrl("https://example.com".to_string())
        );
    }

    #[test]
    fn test_wiki_parse_url_with_label() {
        let result = parse_wiki_link("[[https://example.com|Example Site]]", pos(0, 0, 36));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(
            info.target,
            LinkTarget::ExternalUrl("https://example.com".to_string())
        );
        assert_eq!(info.label, "Example Site");
    }

    #[test]
    fn test_wiki_parse_malformed() {
        let result = parse_wiki_link("[[unclosed", pos(0, 0, 10));
        assert!(result.is_err());
        match result.unwrap_err() {
            LinkError::MalformedSyntax(_) => (),
            _ => panic!("Expected MalformedSyntax error"),
        }
    }

    // ============ Markdown Link Tests (6 tests) ============

    #[test]
    fn test_markdown_parse_basic() {
        let result = parse_markdown_link("[label](note-id)", pos(0, 0, 15));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.format, LinkFormat::Markdown);
        assert_eq!(info.target, LinkTarget::NoteId("note-id".to_string()));
        assert_eq!(info.label, "label");
    }

    #[test]
    fn test_markdown_parse_external_url() {
        let result = parse_markdown_link("[Example](https://example.com)", pos(0, 0, 29));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(
            info.target,
            LinkTarget::ExternalUrl("https://example.com".to_string())
        );
        assert_eq!(info.label, "Example");
    }

    #[test]
    fn test_markdown_parse_empty_label() {
        let result = parse_markdown_link("[](note-id)", pos(0, 0, 11));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.label, "");
        assert_eq!(info.target, LinkTarget::NoteId("note-id".to_string()));
    }

    #[test]
    fn test_markdown_parse_empty_target() {
        let result = parse_markdown_link("[label]()", pos(0, 0, 9));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), LinkError::EmptyTarget);
    }

    #[test]
    fn test_markdown_parse_with_spaces() {
        let result = parse_markdown_link("[  my label  ](  note-id  )", pos(0, 0, 27));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.label, "my label");
        assert_eq!(info.target, LinkTarget::NoteId("note-id".to_string()));
    }

    #[test]
    fn test_markdown_parse_url_with_query() {
        let result =
            parse_markdown_link("[Search](https://example.com/search?q=test)", pos(0, 0, 43));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(
            info.target,
            LinkTarget::ExternalUrl("https://example.com/search?q=test".to_string())
        );
    }

    // ============ Org-Mode Link Tests (6 tests) ============

    #[test]
    fn test_orgmode_parse_basic() {
        let result = parse_orgmode_link("[[note-id]]", pos(0, 0, 11));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.format, LinkFormat::OrgMode);
        assert_eq!(info.target, LinkTarget::NoteId("note-id".to_string()));
        assert_eq!(info.label, "");
    }

    #[test]
    fn test_orgmode_parse_with_label() {
        let result = parse_orgmode_link("[[note-id][My Label]]", pos(0, 0, 21));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.target, LinkTarget::NoteId("note-id".to_string()));
        assert_eq!(info.label, "My Label");
    }

    #[test]
    fn test_orgmode_parse_external_url() {
        let result = parse_orgmode_link("[[https://example.com]]", pos(0, 0, 23));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(
            info.target,
            LinkTarget::ExternalUrl("https://example.com".to_string())
        );
    }

    #[test]
    fn test_orgmode_parse_empty_label() {
        let result = parse_orgmode_link("[[note-id][]]", pos(0, 0, 13));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.label, "");
        assert_eq!(info.target, LinkTarget::NoteId("note-id".to_string()));
    }

    #[test]
    fn test_orgmode_parse_empty_target() {
        // The regex won't match [[]] at all since it requires at least one char in first group
        let result = parse_orgmode_link("[[]][label]", pos(0, 0, 11));
        assert!(result.is_err());
        match result.unwrap_err() {
            LinkError::MalformedSyntax(_) => (),
            _ => panic!("Expected MalformedSyntax for empty target"),
        }
    }

    #[test]
    fn test_orgmode_parse_url_with_label() {
        let result = parse_orgmode_link("[[https://example.com][Click Here]]", pos(0, 0, 35));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(
            info.target,
            LinkTarget::ExternalUrl("https://example.com".to_string())
        );
        assert_eq!(info.label, "Click Here");
    }

    // ============ Generic Parser Tests (3 tests) ============

    #[test]
    fn test_parse_link_detects_markdown() {
        let result = parse_link("[label](note-id)", pos(0, 0, 15));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.format, LinkFormat::Markdown);
    }

    #[test]
    fn test_parse_link_detects_orgmode() {
        let result = parse_link("[[note-id][label]]", pos(0, 0, 18));
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.format, LinkFormat::OrgMode);
    }

    #[test]
    fn test_parse_link_detects_wiki() {
        // Use pipe separator which is uniquely wiki syntax
        let result = parse_link("[[note-id|label]]", pos(0, 0, 17));
        assert!(result.is_ok());
        let info = result.unwrap();
        // This should be detected as Wiki since the pipe separator is wiki-specific
        assert_eq!(info.format, LinkFormat::Wiki);
    }

    // ============ URL Detection Tests (2 tests) ============

    #[test]
    fn test_is_url_valid_protocols() {
        assert!(is_url("http://example.com"));
        assert!(is_url("https://example.com"));
        assert!(is_url("ftp://example.com"));
        assert!(is_url("mailto:test@example.com"));
    }

    #[test]
    fn test_is_url_local_ids() {
        assert!(!is_url("note-id"));
        assert!(!is_url("my-document"));
        assert!(!is_url("file-123"));
    }

    // ============ Additional Coverage Tests ============

    #[test]
    fn test_wiki_empty_target() {
        let result = parse_wiki_link("[[]]", pos(0, 0, 4));
        assert!(result.is_err());
    }

    #[test]
    fn test_markdown_invalid_format() {
        let result = parse_markdown_link("[label", pos(0, 0, 6));
        assert!(result.is_err());
    }

    #[test]
    fn test_orgmode_invalid_format() {
        let result = parse_orgmode_link("[[note-id]", pos(0, 0, 10));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_link_invalid_format() {
        let result = parse_link("not a link", pos(0, 0, 10));
        assert!(result.is_err());
        match result.unwrap_err() {
            LinkError::InvalidFormat(_) => (),
            _ => panic!("Expected InvalidFormat error"),
        }
    }
}
