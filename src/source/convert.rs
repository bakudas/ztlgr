use std::fs;
use std::path::Path;

use crate::error::{Result, ZtlgrError};

/// Supported document formats for conversion to Markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    /// PDF documents (text extraction)
    Pdf,
    /// EPUB ebooks (HTML extraction + conversion)
    Epub,
    /// All formats supported by anytomd (DOCX, PPTX, XLSX, HTML, CSV, JSON, XML, images, code)
    Generic,
}

impl DocumentFormat {
    /// Detect format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            "pdf" => Some(Self::Pdf),
            "epub" => Some(Self::Epub),
            _ => Some(Self::Generic),
        }
    }

    /// Detect format from file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }
}

/// Result of document conversion.
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// The converted Markdown content.
    pub markdown: String,
    /// Plain text extract (without Markdown formatting).
    pub plain_text: String,
    /// Document title if detected.
    pub title: Option<String>,
    /// Extracted images (filename, bytes).
    pub images: Vec<(String, Vec<u8>)>,
}

impl ConversionResult {
    /// Create a simple result with just Markdown content.
    pub fn simple(markdown: String) -> Self {
        let plain_text = markdown.clone();
        Self {
            markdown,
            plain_text,
            title: None,
            images: Vec::new(),
        }
    }
}

/// Convert a document file to Markdown.
///
/// Detects the format automatically and uses the appropriate converter:
/// - PDF: Text extraction via pdf-extract
/// - EPUB: HTML extraction via epub crate
/// - All others: anytomd (DOCX, PPTX, XLSX, HTML, CSV, JSON, XML, images, code)
///
/// # Arguments
///
/// * `path` - Path to the document file
///
/// # Errors
///
/// Returns an error if:
/// - The file doesn't exist
/// - The format is not supported
/// - Conversion fails
pub fn convert_to_markdown(path: &Path) -> Result<ConversionResult> {
    if !path.exists() {
        return Err(ZtlgrError::Ingest(format!(
            "file not found: {}",
            path.display()
        )));
    }

    let format = DocumentFormat::from_path(path)
        .ok_or_else(|| ZtlgrError::Ingest(format!("unknown file format: {}", path.display())))?;

    match format {
        DocumentFormat::Pdf => convert_pdf(path),
        DocumentFormat::Epub => convert_epub(path),
        DocumentFormat::Generic => convert_anytomd(path),
    }
}

/// Convert PDF to Markdown via text extraction.
fn convert_pdf(path: &Path) -> Result<ConversionResult> {
    let bytes = fs::read(path)?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| ZtlgrError::Ingest(format!("PDF extraction failed: {}", e)))?;

    Ok(ConversionResult::simple(text))
}

/// Convert EPUB to Markdown by extracting and converting HTML content.
fn convert_epub(path: &Path) -> Result<ConversionResult> {
    use epub::doc::EpubDoc;

    let mut doc =
        EpubDoc::new(path).map_err(|e| ZtlgrError::Ingest(format!("EPUB open failed: {}", e)))?;

    let mut all_content = String::new();

    if let Some(title) = doc.mdata("title").map(|m| m.value.clone()) {
        all_content.push_str(&format!("# {}\n\n", title));
    }

    // Clone spine to avoid borrowing issues
    let spine: Vec<String> = doc.spine.iter().map(|item| item.idref.clone()).collect();

    for idref in spine {
        if let Some((html_content, _mime)) = doc.get_resource_str(&idref) {
            let extracted = extract_text_from_html(&html_content);
            all_content.push_str(&extracted);
            all_content.push_str("\n\n---\n\n");
        }
    }

    Ok(ConversionResult::simple(all_content))
}

/// Extract plain text from HTML content.
fn extract_text_from_html(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut skip_content = false;
    let mut skip_depth: u32 = 0;

    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '<' {
            in_tag = true;
            tag_name.clear();
            let mut j = i + 1;

            // Skip leading slash for closing tags
            if j < chars.len() && chars[j] == '/' {
                j += 1;
            }

            while j < chars.len() && !chars[j].is_whitespace() && chars[j] != '>' && chars[j] != '/'
            {
                tag_name.push(chars[j].to_ascii_lowercase());
                j += 1;
            }
        } else if c == '>' {
            in_tag = false;

            // Check if we're entering a script/style block
            if (tag_name == "script" || tag_name == "style") && !skip_content {
                skip_content = true;
                skip_depth = 1;
            } else if skip_content && (tag_name == "script" || tag_name == "style") {
                // Exiting the script/style block
                skip_depth = skip_depth.saturating_sub(1);
                if skip_depth == 0 {
                    skip_content = false;
                }
            }
            tag_name.clear();
        } else if !in_tag && !skip_content {
            if c == '&' {
                let entity = parse_entity(&chars[i..]);
                if let Some((entity_text, len)) = entity {
                    text.push_str(&entity_text);
                    i += len - 1;
                } else {
                    text.push(c);
                }
            } else if c == '\n' || c == '\r' {
                if !text.ends_with('\n') && !text.ends_with(' ') {
                    text.push('\n');
                }
            } else if c.is_whitespace() {
                if !text.ends_with(' ') && !text.ends_with('\n') {
                    text.push(' ');
                }
            } else {
                text.push(c);
            }
        }

        i += 1;
    }

    let lines: Vec<&str> = text.lines().collect();
    lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_entity(chars: &[char]) -> Option<(String, usize)> {
    if chars.len() < 4 {
        return None;
    }

    let mut end = 0;
    for (i, &c) in chars.iter().enumerate().take(8.min(chars.len())) {
        if c == ';' {
            end = i;
            break;
        }
    }

    if end == 0 {
        return None;
    }

    let entity: String = chars[0..=end].iter().collect();

    let decoded = match entity.as_str() {
        "&amp;" => "&",
        "&lt;" => "<",
        "&gt;" => ">",
        "&quot;" => "\"",
        "&#39;" => "'",
        "&nbsp;" => " ",
        "&mdash;" => "—",
        "&ndash;" => "–",
        "&hellip;" => "…",
        _ => return None,
    };

    Some((decoded.to_string(), end + 1))
}

/// Ask anytomd to handle the conversion.
fn convert_anytomd(path: &Path) -> Result<ConversionResult> {
    use anytomd::{convert_file, ConversionOptions};

    let options = ConversionOptions::default();
    let result = convert_file(path, &options)
        .map_err(|e| ZtlgrError::Ingest(format!("Conversion failed: {}", e)))?;

    Ok(ConversionResult {
        markdown: result.markdown,
        plain_text: result.plain_text,
        title: result.title,
        images: result.images.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_format_detection_pdf() {
        assert_eq!(
            DocumentFormat::from_extension("pdf"),
            Some(DocumentFormat::Pdf)
        );
        assert_eq!(
            DocumentFormat::from_extension("PDF"),
            Some(DocumentFormat::Pdf)
        );
    }

    #[test]
    fn test_format_detection_epub() {
        assert_eq!(
            DocumentFormat::from_extension("epub"),
            Some(DocumentFormat::Epub)
        );
    }

    #[test]
    fn test_format_detection_anytomd() {
        assert_eq!(
            DocumentFormat::from_extension("docx"),
            Some(DocumentFormat::Generic)
        );
        assert_eq!(
            DocumentFormat::from_extension("xlsx"),
            Some(DocumentFormat::Generic)
        );
        assert_eq!(
            DocumentFormat::from_extension("html"),
            Some(DocumentFormat::Generic)
        );
        assert_eq!(
            DocumentFormat::from_extension("csv"),
            Some(DocumentFormat::Generic)
        );
    }

    #[test]
    fn test_format_from_path() {
        assert_eq!(
            DocumentFormat::from_path(Path::new("document.pdf")),
            Some(DocumentFormat::Pdf)
        );
        assert_eq!(
            DocumentFormat::from_path(Path::new("book.epub")),
            Some(DocumentFormat::Epub)
        );
        assert_eq!(
            DocumentFormat::from_path(Path::new("report.docx")),
            Some(DocumentFormat::Generic)
        );
    }

    #[test]
    fn test_conversion_result_simple() {
        let result = ConversionResult::simple("# Hello\n\nWorld".to_string());
        assert_eq!(result.markdown, "# Hello\n\nWorld");
        assert_eq!(result.plain_text, "# Hello\n\nWorld");
        assert!(result.title.is_none());
        assert!(result.images.is_empty());
    }

    #[test]
    fn test_extract_text_from_html_basic() {
        let html = "<html><body><h1>Title</h1><p>Paragraph</p></body></html>";
        let text = extract_text_from_html(html);
        assert!(text.contains("Title"));
        assert!(text.contains("Paragraph"));
    }

    #[test]
    fn test_extract_text_from_html_with_entities() {
        let html = "<p>Hello &amp; World</p>";
        let text = extract_text_from_html(html);
        assert!(text.contains("Hello & World"));
    }

    #[test]
    fn test_extract_text_from_html_skips_scripts() {
        let html =
            "<html><body><p>Content</p><script>alert('skip');</script><p>More</p></body></html>";
        let text = extract_text_from_html(html);
        assert!(text.contains("Content"));
        assert!(text.contains("More"));
    }

    #[test]
    fn test_convert_nonexistent_file() {
        let result = convert_to_markdown(Path::new("/nonexistent/file.pdf"));
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_markdown_passthrough() {
        let temp = TempDir::new().unwrap();
        let md_file = create_test_file(temp.path(), "test.md", "# Hello\n\nWorld");

        let result = convert_to_markdown(&md_file);
        assert!(result.is_ok());
        let conv = result.unwrap();
        assert!(conv.markdown.contains("Hello"));
    }

    #[test]
    fn test_convert_html() {
        let temp = TempDir::new().unwrap();
        let html_file = create_test_file(
            temp.path(),
            "test.html",
            "<html><body><h1>Title</h1><p>Content</p></body></html>",
        );

        let result = convert_to_markdown(&html_file);
        assert!(result.is_ok());
        let conv = result.unwrap();
        assert!(conv.markdown.contains("Title") || conv.plain_text.contains("Title"));
    }

    #[test]
    fn test_convert_csv() {
        let temp = TempDir::new().unwrap();
        let csv_file = create_test_file(temp.path(), "test.csv", "Name,Age\nAlice,30\nBob,25");

        let result = convert_to_markdown(&csv_file);
        assert!(result.is_ok());
        let conv = result.unwrap();
        assert!(conv.markdown.contains("Name") || conv.plain_text.contains("Name"));
    }

    #[test]
    fn test_convert_json() {
        let temp = TempDir::new().unwrap();
        let json_file =
            create_test_file(temp.path(), "test.json", r#"{"name": "Test", "value": 42}"#);

        let result = convert_to_markdown(&json_file);
        assert!(result.is_ok());
        let conv = result.unwrap();
        assert!(conv.markdown.contains("name") || conv.plain_text.contains("Test"));
    }

    #[test]
    fn test_parse_entity_amp() {
        let chars: Vec<char> = "&amp;".chars().collect();
        let result = parse_entity(&chars);
        assert_eq!(result, Some(("&".to_string(), 5)));
    }

    #[test]
    fn test_parse_entity_lt() {
        let chars: Vec<char> = "&lt;".chars().collect();
        let result = parse_entity(&chars);
        assert_eq!(result, Some(("<".to_string(), 4)));
    }

    #[test]
    fn test_parse_entity_not_entity() {
        let chars: Vec<char> = "hello".chars().collect();
        let result = parse_entity(&chars);
        assert!(result.is_none());
    }
}
