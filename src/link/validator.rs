use regex::Regex;
use std::sync::OnceLock;

use super::{LinkFormat, LinkInfo, LinkPosition, LinkTarget};
use crate::db::Database;
use crate::note::NoteId;

/// Regex patterns for extracting links from text
fn wiki_link_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]*))?\]\]").expect("Invalid wiki regex")
    })
}

fn markdown_link_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\[([^\]]*)\]\(([^)]*)\)").expect("Invalid markdown regex"))
}

fn orgmode_link_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"\[\[([^\]]+)\](?:\[([^\]]*)\])?\]").expect("Invalid orgmode regex")
    })
}

/// Represents a link found in text with validation status
#[derive(Debug, Clone)]
pub struct ValidatedLink {
    pub info: LinkInfo,
    /// true if the link target exists in the database
    pub is_valid: bool,
}

/// Link validator for highlighting and validation
pub struct LinkValidator;

impl LinkValidator {
    /// Extract all links from a line of text
    ///
    /// Returns a vector of validated links with their positions
    pub fn extract_links(text: &str, line_num: usize, db: &Database) -> Vec<ValidatedLink> {
        let mut links = Vec::new();

        // Extract wiki-style links: [[note-id]] or [[note-id|label]]
        for cap in wiki_link_pattern().captures_iter(text) {
            if let Some(full_match) = cap.get(0) {
                let start_col = full_match.start();
                let end_col = full_match.end();
                let target_str = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");

                if !target_str.is_empty() {
                    let label = cap
                        .get(2)
                        .map(|m| m.as_str().trim().to_string())
                        .unwrap_or_default();
                    let target = LinkTarget::NoteId(target_str.to_string());

                    let info = LinkInfo {
                        format: LinkFormat::Wiki,
                        target: target.clone(),
                        label,
                        raw: full_match.as_str().to_string(),
                        position: LinkPosition {
                            line: line_num,
                            start_col,
                            end_col,
                        },
                    };

                    let is_valid = Self::validate_target(&target, db);
                    links.push(ValidatedLink { info, is_valid });
                }
            }
        }

        // Extract markdown-style links: [label](note-id or url)
        for cap in markdown_link_pattern().captures_iter(text) {
            if let Some(full_match) = cap.get(0) {
                let start_col = full_match.start();
                let end_col = full_match.end();
                let target_str = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                let label = cap
                    .get(1)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default();

                if !target_str.is_empty() {
                    let target = if target_str.starts_with("http://")
                        || target_str.starts_with("https://")
                        || target_str.starts_with("ftp://")
                        || target_str.starts_with("mailto:")
                    {
                        LinkTarget::ExternalUrl(target_str.to_string())
                    } else {
                        LinkTarget::NoteId(target_str.to_string())
                    };

                    let info = LinkInfo {
                        format: LinkFormat::Markdown,
                        target: target.clone(),
                        label,
                        raw: full_match.as_str().to_string(),
                        position: LinkPosition {
                            line: line_num,
                            start_col,
                            end_col,
                        },
                    };

                    let is_valid = Self::validate_target(&target, db);
                    links.push(ValidatedLink { info, is_valid });
                }
            }
        }

        // Extract org-mode links: [[id]] or [[id][label]]
        for cap in orgmode_link_pattern().captures_iter(text) {
            if let Some(full_match) = cap.get(0) {
                let start_col = full_match.start();
                let end_col = full_match.end();
                let target_str = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let label = cap
                    .get(2)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default();

                if !target_str.is_empty() {
                    let target = if target_str.starts_with("http://")
                        || target_str.starts_with("https://")
                        || target_str.starts_with("ftp://")
                        || target_str.starts_with("mailto:")
                    {
                        LinkTarget::ExternalUrl(target_str.to_string())
                    } else {
                        LinkTarget::NoteId(target_str.to_string())
                    };

                    let info = LinkInfo {
                        format: LinkFormat::OrgMode,
                        target: target.clone(),
                        label,
                        raw: full_match.as_str().to_string(),
                        position: LinkPosition {
                            line: line_num,
                            start_col,
                            end_col,
                        },
                    };

                    let is_valid = Self::validate_target(&target, db);
                    links.push(ValidatedLink { info, is_valid });
                }
            }
        }

        links
    }

    /// Extract all links from multiline text
    pub fn extract_all_links(text: &str, db: &Database) -> Vec<ValidatedLink> {
        let mut all_links = Vec::new();

        for (line_num, line) in text.lines().enumerate() {
            let line_links = Self::extract_links(line, line_num, db);
            all_links.extend(line_links);
        }

        all_links
    }

    /// Check if a link target exists in the database
    pub fn validate_target(target: &LinkTarget, db: &Database) -> bool {
        match target {
            LinkTarget::NoteId(id_str) => {
                // Create a NoteId from the string using From trait
                let note_id: NoteId = id_str.as_str().into();

                // Check if note with this ID exists
                match db.get_note(&note_id) {
                    Ok(Some(_note)) => true, // Note exists and not deleted
                    Ok(None) => {
                        // Try searching by title or partial match
                        db.search_notes(id_str, 5)
                            .map(|results| !results.is_empty())
                            .unwrap_or(false)
                    }
                    Err(_) => false,
                }
            }
            LinkTarget::ExternalUrl(_) => true, // External URLs are always considered valid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wiki_links() {
        let text = "Check out [[note-123]] and [[other-note|custom label]]";
        // Note: This test won't have database access in unit tests
        // We'll test link extraction logic only
        let wiki_matches: Vec<_> = wiki_link_pattern().captures_iter(text).collect();
        assert_eq!(wiki_matches.len(), 2);

        let first = wiki_matches[0].get(1).unwrap().as_str();
        assert_eq!(first, "note-123");

        let second = wiki_matches[1].get(1).unwrap().as_str();
        assert_eq!(second, "other-note");
    }

    #[test]
    fn test_extract_markdown_links() {
        let text = "See [label](note-123) and [another](https://example.com)";
        let md_matches: Vec<_> = markdown_link_pattern().captures_iter(text).collect();
        assert_eq!(md_matches.len(), 2);

        let first = md_matches[0].get(2).unwrap().as_str();
        assert_eq!(first, "note-123");

        let second = md_matches[1].get(2).unwrap().as_str();
        assert_eq!(second, "https://example.com");
    }

    #[test]
    fn test_extract_orgmode_links() {
        let text = "See [[note-123]] and [[note-456][label]]";
        let org_matches: Vec<_> = orgmode_link_pattern().captures_iter(text).collect();
        assert_eq!(org_matches.len(), 2);

        let first = org_matches[0].get(1).unwrap().as_str();
        assert_eq!(first, "note-123");

        let second = org_matches[1].get(1).unwrap().as_str();
        assert_eq!(second, "note-456");
    }

    #[test]
    fn test_link_position_calculation() {
        let text = "Start [[note-123]] middle [[note-456]] end";
        let wiki_matches: Vec<_> = wiki_link_pattern().captures_iter(text).collect();

        let first_start = wiki_matches[0].get(0).unwrap().start();
        let first_end = wiki_matches[0].get(0).unwrap().end();
        assert_eq!(first_start, 6);
        assert_eq!(first_end, 18);

        let second_start = wiki_matches[1].get(0).unwrap().start();
        let second_end = wiki_matches[1].get(0).unwrap().end();
        assert_eq!(second_start, 26);
        assert_eq!(second_end, 38);
    }
}
