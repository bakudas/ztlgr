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
            .and_then(|ext| Self::from_extension(ext))
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
        let gitignore = r#"
# Ignore database (but not config)
.ztlgr/vault.db
.ztlgr/cache/

# Keep config
!.ztlgr/config.toml

# Ignore Obsidian workspace (if using Obsidian)
.obsidian/workspace*

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
            .unwrap_or("My Vault");
        let extension = self.format.extension();
        let readme = format!(
            r#"# {0}

A Zettelkasten vault created with ztlgr.

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
# Open this vault
ztlgr {0}

# Create a new note
ztlgr new --type permanent "My Note Title"

# Import existing notes
ztlgr import --recursive

# Sync with database
ztlgr sync
```

## Backup

Your notes are stored as {1} files in this directory.
The SQLite database (`.ztlgr/vault.db`) is just an index.

To backup:
1. Use git: `git init && git add . && git commit -m "Initial commit"`
2. Or copy the entire directory

## Compatibility

This vault is compatible with:
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
}
