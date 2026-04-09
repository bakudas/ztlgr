pub mod importer;
pub mod markdown;
pub mod org;
pub mod organization;
pub mod sync;
pub mod watcher;

pub use importer::FileImporter;
pub use markdown::MarkdownStorage;
pub use org::OrgStorage;
pub use organization::{FolderStats, NoteOrganizer};
pub use sync::FileSync;
pub use watcher::FileWatcher;

use crate::error::Result;
use crate::note::Note;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Standard vault directory names
pub const VAULT_DIRS: &[&str] = &[
    "daily",
    "inbox",
    "literature",
    "permanent",
    "reference",
    "index",
    "attachments",
];

/// Trait for file-based storage backends
pub trait Storage: Send + Sync {
    /// Get the file extension for this storage format
    fn extension(&self) -> &str;

    /// Read a note from a file
    fn read_note(&self, path: &Path) -> Result<Note>;

    /// Write a note to a file
    fn write_note(&self, note: &Note, path: &Path) -> Result<()>;

    /// Parse frontmatter/metadata from file
    fn parse_metadata(&self, content: &str) -> Result<crate::note::Metadata>;

    /// Render note content with metadata
    fn render(&self, note: &Note) -> Result<String>;

    /// Extract links from content
    fn extract_links(&self, content: &str) -> Result<Vec<Link>>;

    /// Extract tags from content
    fn extract_tags(&self, content: &str) -> Result<Vec<String>>;
}

#[derive(Debug, Clone)]
pub struct Link {
    pub source_note_id: String,
    pub target: String,
    pub link_type: LinkType,
    pub display_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinkType {
    Wiki,
    Markdown,
    Org,
    Tag,
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkType::Wiki => write!(f, "wiki"),
            LinkType::Markdown => write!(f, "markdown"),
            LinkType::Org => write!(f, "org"),
            LinkType::Tag => write!(f, "tag"),
        }
    }
}

/// Storage format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Markdown,
    Org,
}

impl Format {
    pub fn extension(&self) -> &'static str {
        match self {
            Format::Markdown => "md",
            Format::Org => "org",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => Some(Format::Markdown),
            "org" => Some(Format::Org),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }
}

/// Vault structure
#[derive(Debug, Clone)]
pub struct Vault {
    /// Path to vault directory
    pub path: PathBuf,

    /// Format (all files in vault use same format)
    pub format: Format,

    /// Subdirectories (optional)
    pub directories: VaultDirectories,
}

#[derive(Debug, Clone)]
pub struct VaultDirectories {
    /// Daily notes directory
    pub daily: PathBuf,

    /// Fleeting notes directory
    pub fleeting: PathBuf,

    /// Literature notes directory
    pub literature: PathBuf,

    /// Permanent notes directory
    pub permanent: PathBuf,

    /// Reference notes directory
    pub reference: PathBuf,

    /// Index/structure notes directory
    pub index: PathBuf,

    /// Attachments directory
    pub attachments: PathBuf,
}

impl Default for VaultDirectories {
    fn default() -> Self {
        Self {
            daily: PathBuf::from("daily"),
            fleeting: PathBuf::from("inbox"),
            literature: PathBuf::from("literature"),
            permanent: PathBuf::from("permanent"),
            reference: PathBuf::from("reference"),
            index: PathBuf::from("index"),
            attachments: PathBuf::from("attachments"),
        }
    }
}

impl Vault {
    pub fn new(path: PathBuf, format: Format) -> Self {
        Self {
            path,
            format,
            directories: VaultDirectories::default(),
        }
    }

    pub fn initialize(&self) -> Result<()> {
        // Create directory structure
        std::fs::create_dir_all(&self.path)?;
        std::fs::create_dir_all(self.path.join(&self.directories.daily))?;
        std::fs::create_dir_all(self.path.join(&self.directories.fleeting))?;
        std::fs::create_dir_all(self.path.join(&self.directories.literature))?;
        std::fs::create_dir_all(self.path.join(&self.directories.permanent))?;
        std::fs::create_dir_all(self.path.join(&self.directories.reference))?;
        std::fs::create_dir_all(self.path.join(&self.directories.index))?;
        std::fs::create_dir_all(self.path.join(&self.directories.attachments))?;

        // Create .obsidian directory (for compatibility)
        std::fs::create_dir_all(self.path.join(".obsidian"))?;

        // Create .ztlgr directory (for database and config)
        std::fs::create_dir_all(self.path.join(".ztlgr"))?;

        // Create .gitignore
        let gitignore = r#"# ztlgr grimoire -- generated by ztlgr

# Database and cache (regenerated from files)
.ztlgr/vault.db
.ztlgr/vault.db-wal
.ztlgr/vault.db-shm
.ztlgr/cache/

# Keep config and log
!.ztlgr/config.toml
!.ztlgr/log.md

# Keep .skills/ (LLM schema)
!.skills/**

# Obsidian workspace state (if using Obsidian)
.obsidian/workspace*
.obsidian/workspace.json
.obsidian/workspace-mobile.json

# OS artifacts
.DS_Store
Thumbs.db
Desktop.ini

# Don't ignore notes
!**/*.md
!**/*.org
"#;
        std::fs::write(self.path.join(".gitignore"), gitignore)?;

        // Create README
        let vault_name = self
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("My Grimoire");
        let extension = self.format.extension();
        let readme = format!(
            r#"# {0}

A Zettelkasten grimoire created with ztlgr.

Your notes live as plain {1} files on your machine. No cloud, no telemetry, no lock-in.

## Structure

- `daily/` - Daily notes
- `inbox/` - Fleeting notes for quick capture
- `literature/` - Notes from books, articles, etc.
- `permanent/` - Permanent knowledge notes
- `reference/` - Reference notes with external sources
- `index/` - Index/structure notes (MOCs)
- `attachments/` - Images and other attachments

## Usage

```bash
# Open this grimoire
ztlgr open {0}

# Search notes
ztlgr search "query" --vault {0}

# Sync with database
ztlgr sync --vault {0}
```

## Backup

Your notes are stored as {1} files in this directory.
The SQLite database (`.ztlgr/vault.db`) is just an index -- it is regenerated from files.

## Compatibility

This grimoire is compatible with:
- Obsidian (Markdown mode)
- Foam
- Logseq
- Any Markdown editor
"#,
            vault_name, extension
        );
        std::fs::write(self.path.join("README.md"), readme)?;

        tracing::info!("Initialized vault at {:?}", self.path);
        Ok(())
    }

    pub fn note_path(&self, note: &Note) -> PathBuf {
        let subdir = match &note.note_type {
            crate::note::NoteType::Daily => &self.directories.daily,
            crate::note::NoteType::Fleeting => &self.directories.fleeting,
            crate::note::NoteType::Literature { .. } => &self.directories.literature,
            crate::note::NoteType::Permanent => &self.directories.permanent,
            crate::note::NoteType::Reference { .. } => &self.directories.reference,
            crate::note::NoteType::Index => &self.directories.index,
        };

        let filename = note
            .zettel_id
            .as_ref()
            .map(|z| format!("{}-{}", z.as_str(), note.title))
            .unwrap_or_else(|| note.title.clone());

        let filename = sanitize_filename::sanitize(filename);

        self.path
            .join(subdir)
            .join(format!("{}.{}", filename, self.format.extension()))
    }

    pub fn database_path(&self) -> PathBuf {
        self.path.join(".ztlgr").join("vault.db")
    }

    pub fn config_path(&self) -> PathBuf {
        self.path.join(".ztlgr").join("config.toml")
    }

    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.join(".ztlgr").exists()
    }

    /// Check whether the `git` binary is available on the system.
    pub fn is_git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Initialize a git repository in the vault directory and make an initial commit.
    ///
    /// Returns `Ok(true)` if git was initialized, `Ok(false)` if git is not available
    /// (skipped silently), or `Err` if git commands failed.
    pub fn git_init(&self) -> Result<bool> {
        if !Self::is_git_available() {
            tracing::info!("git not found, skipping repository initialization");
            return Ok(false);
        }

        let run = |args: &[&str]| -> Result<()> {
            let output = Command::new("git")
                .args(args)
                .current_dir(&self.path)
                .output()
                .map_err(|e| crate::error::ZtlgrError::Git(format!("failed to run git: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(crate::error::ZtlgrError::Git(format!(
                    "git {} failed: {}",
                    args.join(" "),
                    stderr.trim()
                )));
            }
            Ok(())
        };

        run(&["init"])?;
        run(&["add", "."])?;
        run(&["commit", "-m", "Initialize ztlgr grimoire", "--allow-empty"])?;

        tracing::info!("Initialized git repository at {:?}", self.path);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_vault_initialization() {
        let dir = tempdir().unwrap();
        let vault = Vault::new(dir.path().to_path_buf(), Format::Markdown);

        vault.initialize().unwrap();

        assert!(vault.exists());
        assert!(vault.path.join(".ztlgr").exists());
        assert!(vault.path.join("permanent").exists());
    }

    #[test]
    fn test_gitignore_content() {
        let dir = tempdir().unwrap();
        let vault = Vault::new(dir.path().to_path_buf(), Format::Markdown);
        vault.initialize().unwrap();

        let gitignore = std::fs::read_to_string(vault.path.join(".gitignore")).unwrap();
        // Must ignore WAL/SHM files
        assert!(gitignore.contains("vault.db-wal"));
        assert!(gitignore.contains("vault.db-shm"));
        // Must ignore OS artifacts
        assert!(gitignore.contains(".DS_Store"));
        assert!(gitignore.contains("Thumbs.db"));
        // Must ignore Obsidian workspace
        assert!(gitignore.contains(".obsidian/workspace"));
        // Must keep .skills/
        assert!(gitignore.contains("!.skills/**"));
        // Must keep config
        assert!(gitignore.contains("!.ztlgr/config.toml"));
    }

    #[test]
    fn test_readme_uses_grimoire_terminology() {
        let dir = tempdir().unwrap();
        let vault_path = dir.path().join("my_grimoire");
        let vault = Vault::new(vault_path, Format::Markdown);
        vault.initialize().unwrap();

        let readme = std::fs::read_to_string(vault.path.join("README.md")).unwrap();
        assert!(readme.contains("grimoire"));
        assert!(!readme.contains("My Vault"));
    }

    #[test]
    fn test_is_git_available() {
        // On most dev machines git is installed; this just verifies the function doesn't panic
        let _available = Vault::is_git_available();
    }

    #[test]
    fn test_git_init_creates_repo() {
        if !Vault::is_git_available() {
            // Skip test if git is not installed
            return;
        }

        let dir = tempdir().unwrap();
        let vault = Vault::new(dir.path().to_path_buf(), Format::Markdown);
        vault.initialize().unwrap();

        let result = vault.git_init().unwrap();
        assert!(result, "git_init should return true when git is available");

        // Verify .git directory was created
        assert!(vault.path.join(".git").exists());

        // Verify at least one commit exists
        let output = Command::new("git")
            .args(["log", "--oneline", "-1"])
            .current_dir(&vault.path)
            .output()
            .unwrap();
        let log = String::from_utf8_lossy(&output.stdout);
        assert!(log.contains("Initialize ztlgr grimoire"));
    }

    #[test]
    fn test_git_init_on_nonexistent_dir_fails() {
        if !Vault::is_git_available() {
            return;
        }

        let vault = Vault::new(
            PathBuf::from("/nonexistent/path/for/test"),
            Format::Markdown,
        );
        let result = vault.git_init();
        assert!(result.is_err());
    }
}
