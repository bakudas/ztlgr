use regex::Regex;
use std::sync::OnceLock;

/// Get wiki link regex
fn wiki_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]*))?]]").expect("Invalid wiki regex"))
}

/// Get markdown link regex
fn markdown_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\[([^\]]*)\]\(([^)]*)\)").expect("Invalid markdown regex"))
}

/// Detects all links on a line and finds if cursor is on one
/// Returns the target if cursor is on a link
pub fn detect_link_at_cursor(line: &str, col: usize) -> Option<String> {
    // Check wiki-style links first: [[target]] or [[target|label]]
    for cap in wiki_regex().captures_iter(line) {
        if let Some(full_match) = cap.get(0) {
            let start = full_match.start();
            let end = full_match.end();

            // Check if cursor is on this link
            if col >= start && col <= end {
                // Extract the target (before the pipe)
                if let Some(target) = cap.get(1) {
                    return Some(target.as_str().trim().to_string());
                }
            }
        }
    }

    // Check markdown-style links: [label](target)
    for cap in markdown_regex().captures_iter(line) {
        if let Some(full_match) = cap.get(0) {
            let start = full_match.start();
            let end = full_match.end();

            // Check if cursor is on this link
            if col >= start && col <= end {
                // Extract the URL/ID (in parentheses)
                if let Some(target) = cap.get(2) {
                    return Some(target.as_str().trim().to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_wiki_link() {
        let line = "Check this [[note-123]] out";
        // Cursor on the 'n' in note-123
        let col = 17;
        assert_eq!(
            detect_link_at_cursor(line, col),
            Some("note-123".to_string())
        );
    }

    #[test]
    fn test_detect_wiki_link_with_label() {
        let line = "See [[note-123|my note]] for details";
        // Cursor on the 'n' in note-123
        let col = 9;
        assert_eq!(
            detect_link_at_cursor(line, col),
            Some("note-123".to_string())
        );
    }

    #[test]
    fn test_detect_markdown_link() {
        let line = "Go to [label](note-456) now";
        // Cursor on the 'n' in note-456
        let col = 16;
        assert_eq!(
            detect_link_at_cursor(line, col),
            Some("note-456".to_string())
        );
    }

    #[test]
    fn test_no_link_at_cursor() {
        let line = "Just some regular text here";
        assert_eq!(detect_link_at_cursor(line, 5), None);
    }

    #[test]
    fn test_detect_external_url() {
        let line = "Visit [site](https://example.com) anytime";
        // Cursor on the 'e' in example
        let col = 25;
        assert_eq!(
            detect_link_at_cursor(line, col),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_cursor_at_link_start() {
        let line = "Check [[note-123]] out";
        // Cursor at the start of the link
        let col = 6;
        assert_eq!(
            detect_link_at_cursor(line, col),
            Some("note-123".to_string())
        );
    }

    #[test]
    fn test_cursor_at_link_end() {
        let line = "Check [[note-123]] out";
        // Cursor at the end of the link
        let col = 18;
        assert_eq!(
            detect_link_at_cursor(line, col),
            Some("note-123".to_string())
        );
    }
}
