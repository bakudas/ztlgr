use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::db::Database;
use crate::error::{Result, ZtlgrError};
use crate::source::Source;
use crate::storage::ActivityLog;

/// Result of a single ingest operation.
#[derive(Debug)]
pub struct IngestResult {
    pub source: Source,
    /// True if the file was newly ingested, false if it was a duplicate.
    pub is_new: bool,
}

/// Ingests source files into the `raw/` directory of a vault.
///
/// The ingest pipeline:
/// 1. Reads the source file
/// 2. Computes SHA-256 hash for deduplication
/// 3. Copies file to `raw/` directory (preserving extension)
/// 4. Registers source in the database
/// 5. Logs the activity
pub struct Ingester {
    vault_path: PathBuf,
    database: Database,
}

impl Ingester {
    pub fn new(vault_path: PathBuf, database: Database) -> Self {
        Self {
            vault_path,
            database,
        }
    }

    /// The path to the `raw/` directory in the vault.
    pub fn raw_dir(&self) -> PathBuf {
        self.vault_path.join("raw")
    }

    /// Ingest a local file into the vault.
    ///
    /// The file is copied to `raw/` with a sanitized filename. If a source
    /// with the same content hash already exists, the operation returns
    /// the existing source with `is_new = false`.
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the file to ingest
    /// * `title` - Optional title override (defaults to filename stem)
    pub fn ingest_file(&self, source_path: &Path, title: Option<&str>) -> Result<IngestResult> {
        // Validate source file exists
        if !source_path.exists() {
            return Err(ZtlgrError::Ingest(format!(
                "source file not found: {}",
                source_path.display()
            )));
        }

        if !source_path.is_file() {
            return Err(ZtlgrError::Ingest(format!(
                "source path is not a file: {}",
                source_path.display()
            )));
        }

        // Read file content and compute hash
        let content = fs::read(source_path)?;
        let hash = compute_sha256(&content);

        // Check for duplicate
        if let Some(existing) = self.database.find_source_by_hash(&hash)? {
            return Ok(IngestResult {
                source: existing,
                is_new: false,
            });
        }

        // Determine title
        let title = title.map(|t| t.to_string()).unwrap_or_else(|| {
            source_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string()
        });

        // Determine destination filename
        let dest_filename = destination_filename(source_path, &hash);
        let raw_dir = self.raw_dir();
        fs::create_dir_all(&raw_dir)?;
        let dest_path = raw_dir.join(&dest_filename);

        // Copy file to raw/
        fs::copy(source_path, &dest_path)?;

        let file_size = content.len() as u64;
        let relative_path = format!("raw/{}", dest_filename);
        let mime_type = guess_mime_type(source_path);

        // Create source record
        let mut source = Source::new(&title, &hash, &relative_path, file_size)
            .with_origin(source_path.display().to_string());

        if let Some(mime) = mime_type {
            source = source.with_mime_type(mime);
        }

        // Register in database
        self.database.create_source(&source)?;

        // Log the activity
        let activity_log = ActivityLog::new(&self.vault_path);
        let _ = activity_log.log_ingest(&title, &relative_path);

        Ok(IngestResult {
            source,
            is_new: true,
        })
    }
}

/// Compute SHA-256 hash of content, returned as hex string.
pub fn compute_sha256(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Build a sanitized destination filename for the raw/ directory.
///
/// Format: `{sanitized_stem}-{short_hash}.{ext}`
fn destination_filename(source_path: &Path, hash: &str) -> String {
    let stem = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = source_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("bin");

    let sanitized_stem = sanitize_filename::sanitize(stem);
    let short_hash = &hash[..8.min(hash.len())];

    format!("{}-{}.{}", sanitized_stem, short_hash, ext)
}

/// Guess MIME type from file extension.
fn guess_mime_type(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    let mime = match ext.as_str() {
        "md" | "markdown" => "text/markdown",
        "txt" => "text/plain",
        "pdf" => "application/pdf",
        "html" | "htm" => "text/html",
        "org" => "text/org",
        "json" => "application/json",
        "yaml" | "yml" => "text/yaml",
        "csv" => "text/csv",
        "epub" => "application/epub+zip",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => return None,
    };
    Some(mime.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_vault() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let vault_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(vault_path.join(".ztlgr")).unwrap();
        fs::create_dir_all(vault_path.join("raw")).unwrap();
        (temp_dir, vault_path)
    }

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    // =====================================================================
    // compute_sha256 tests
    // =====================================================================

    #[test]
    fn test_compute_sha256_deterministic() {
        let hash1 = compute_sha256(b"hello world");
        let hash2 = compute_sha256(b"hello world");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_sha256_different_content() {
        let hash1 = compute_sha256(b"hello");
        let hash2 = compute_sha256(b"world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_sha256_known_value() {
        // SHA-256 of empty string
        let hash = compute_sha256(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_compute_sha256_length() {
        let hash = compute_sha256(b"test");
        assert_eq!(hash.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }

    // =====================================================================
    // destination_filename tests
    // =====================================================================

    #[test]
    fn test_destination_filename_basic() {
        let path = PathBuf::from("article.md");
        let name = destination_filename(&path, "abcdef0123456789");
        assert_eq!(name, "article-abcdef01.md");
    }

    #[test]
    fn test_destination_filename_preserves_extension() {
        let path = PathBuf::from("paper.pdf");
        let name = destination_filename(&path, "0123456789abcdef");
        assert_eq!(name, "paper-01234567.pdf");
    }

    #[test]
    fn test_destination_filename_no_extension() {
        let path = PathBuf::from("README");
        let name = destination_filename(&path, "aabbccdd");
        assert_eq!(name, "README-aabbccdd.bin");
    }

    #[test]
    fn test_destination_filename_sanitizes() {
        let path = PathBuf::from("bad/file:name?.md");
        let name = destination_filename(&path, "abcdef01");
        // sanitize_filename strips special chars
        assert!(name.ends_with(".md"));
        assert!(name.contains("abcdef01"));
    }

    // =====================================================================
    // guess_mime_type tests
    // =====================================================================

    #[test]
    fn test_guess_mime_markdown() {
        assert_eq!(
            guess_mime_type(Path::new("file.md")),
            Some("text/markdown".to_string())
        );
    }

    #[test]
    fn test_guess_mime_pdf() {
        assert_eq!(
            guess_mime_type(Path::new("paper.pdf")),
            Some("application/pdf".to_string())
        );
    }

    #[test]
    fn test_guess_mime_html() {
        assert_eq!(
            guess_mime_type(Path::new("page.html")),
            Some("text/html".to_string())
        );
    }

    #[test]
    fn test_guess_mime_unknown() {
        assert_eq!(guess_mime_type(Path::new("data.xyz")), None);
    }

    #[test]
    fn test_guess_mime_no_extension() {
        assert_eq!(guess_mime_type(Path::new("Makefile")), None);
    }

    #[test]
    fn test_guess_mime_case_insensitive() {
        assert_eq!(
            guess_mime_type(Path::new("FILE.PDF")),
            Some("application/pdf".to_string())
        );
    }

    // =====================================================================
    // Ingester tests
    // =====================================================================

    #[test]
    fn test_ingest_file_success() {
        let (_temp, vault_path) = setup_vault();
        let db = Database::new(&vault_path.join(".ztlgr").join("vault.db")).unwrap();
        let ingester = Ingester::new(vault_path.clone(), db);

        let source_file = create_test_file(_temp.path(), "article.md", "# My Article\n\nContent");
        let result = ingester.ingest_file(&source_file, None).unwrap();

        assert!(result.is_new);
        assert_eq!(result.source.title, "article");
        assert!(result.source.file_path.starts_with("raw/"));
        assert!(result.source.file_path.ends_with(".md"));
        assert_eq!(result.source.file_size, 21);

        // File should exist in raw/
        let dest = vault_path.join(&result.source.file_path);
        assert!(dest.exists());
        assert_eq!(
            fs::read_to_string(&dest).unwrap(),
            "# My Article\n\nContent"
        );
    }

    #[test]
    fn test_ingest_file_with_title_override() {
        let (_temp, vault_path) = setup_vault();
        let db = Database::new(&vault_path.join(".ztlgr").join("vault.db")).unwrap();
        let ingester = Ingester::new(vault_path, db);

        let source_file = create_test_file(_temp.path(), "a.md", "content");
        let result = ingester
            .ingest_file(&source_file, Some("Custom Title"))
            .unwrap();

        assert!(result.is_new);
        assert_eq!(result.source.title, "Custom Title");
    }

    #[test]
    fn test_ingest_file_deduplication() {
        let (_temp, vault_path) = setup_vault();
        let db = Database::new(&vault_path.join(".ztlgr").join("vault.db")).unwrap();
        let ingester = Ingester::new(vault_path, db);

        let file1 = create_test_file(_temp.path(), "first.md", "same content");
        let file2 = create_test_file(_temp.path(), "second.md", "same content");

        let result1 = ingester.ingest_file(&file1, None).unwrap();
        assert!(result1.is_new);

        let result2 = ingester.ingest_file(&file2, None).unwrap();
        assert!(!result2.is_new);
        assert_eq!(result1.source.content_hash, result2.source.content_hash);
    }

    #[test]
    fn test_ingest_file_nonexistent_fails() {
        let (_temp, vault_path) = setup_vault();
        let db = Database::new(&vault_path.join(".ztlgr").join("vault.db")).unwrap();
        let ingester = Ingester::new(vault_path, db);

        let result = ingester.ingest_file(Path::new("/nonexistent/file.md"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_ingest_directory_fails() {
        let (_temp, vault_path) = setup_vault();
        let db = Database::new(&vault_path.join(".ztlgr").join("vault.db")).unwrap();
        let ingester = Ingester::new(vault_path, db);

        let result = ingester.ingest_file(_temp.path(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_ingest_file_records_origin() {
        let (_temp, vault_path) = setup_vault();
        let db = Database::new(&vault_path.join(".ztlgr").join("vault.db")).unwrap();
        let ingester = Ingester::new(vault_path, db);

        let source_file = create_test_file(_temp.path(), "doc.md", "text");
        let result = ingester.ingest_file(&source_file, None).unwrap();

        assert!(result.source.origin.is_some());
        let origin = result.source.origin.unwrap();
        assert!(origin.contains("doc.md"));
    }

    #[test]
    fn test_ingest_file_detects_mime() {
        let (_temp, vault_path) = setup_vault();
        let db = Database::new(&vault_path.join(".ztlgr").join("vault.db")).unwrap();
        let ingester = Ingester::new(vault_path, db);

        let source_file = create_test_file(_temp.path(), "paper.pdf", "fake pdf");
        let result = ingester.ingest_file(&source_file, None).unwrap();

        assert_eq!(result.source.mime_type.as_deref(), Some("application/pdf"));
    }

    #[test]
    fn test_ingest_creates_raw_dir_if_missing() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(vault_path.join(".ztlgr")).unwrap();
        // Don't create raw/ -- ingester should create it

        let db = Database::new(&vault_path.join(".ztlgr").join("vault.db")).unwrap();
        let ingester = Ingester::new(vault_path.clone(), db);

        let source_file = create_test_file(temp_dir.path(), "note.md", "content");
        let result = ingester.ingest_file(&source_file, None).unwrap();

        assert!(result.is_new);
        assert!(vault_path.join("raw").exists());
    }

    #[test]
    fn test_ingest_file_logs_activity() {
        let (_temp, vault_path) = setup_vault();
        let db = Database::new(&vault_path.join(".ztlgr").join("vault.db")).unwrap();
        let ingester = Ingester::new(vault_path.clone(), db);

        let source_file = create_test_file(_temp.path(), "logged.md", "content");
        ingester.ingest_file(&source_file, None).unwrap();

        let log_path = vault_path.join(".ztlgr").join("log.md");
        assert!(log_path.exists());
        let log_content = fs::read_to_string(&log_path).unwrap();
        assert!(log_content.contains("ingest |"));
    }
}
