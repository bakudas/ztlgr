use crate::db::Database;
use crate::error::{Result, ZtlgrError};
use crate::note::Note;
use crate::storage::{Format, MarkdownStorage, OrgStorage, Storage};
use std::path::Path;

pub struct FileImporter {
    vault_path: std::path::PathBuf,
    format: Format,
    database: Database,
}

impl FileImporter {
    pub fn new(vault_path: std::path::PathBuf, format: Format, database: Database) -> Self {
        Self {
            vault_path,
            format,
            database,
        }
    }

    pub fn import_all(&self) -> Result<ImportResult> {
        let mut result = ImportResult::default();

        let storage: Box<dyn Storage> = match self.format {
            Format::Markdown => Box::new(MarkdownStorage::new()),
            Format::Org => Box::new(OrgStorage::new()),
        };

        for entry in walkdir::WalkDir::new(&self.vault_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip hidden directories
            if path.components().any(|c| {
                c.as_os_str()
                    .to_str()
                    .map(|s| s.starts_with('.'))
                    .unwrap_or(false)
            }) {
                continue;
            }

            // Check if it's a note file
            if !self.is_note_file(path) {
                continue;
            }

            match self.import_file(path, storage.as_ref()) {
                Ok(note) => {
                    result.imported += 1;
                    result.notes.push(note.id.as_str().to_string());
                }
                Err(e) => {
                    result.failed += 1;
                    result.errors.push(format!("{}: {}", path.display(), e));
                }
            }
        }

        result.scan_directories(&self.vault_path)?;

        self.database.rebuild_fts()?;

        Ok(result)
    }

    pub fn import_file(&self, path: &Path, storage: &dyn Storage) -> Result<Note> {
        // Read note from file
        let note = storage.read_note(path)?;

        // Check if note already exists
        if let Some(_existing) = self.database.get_note(&note.id)? {
            // Update existing note
            let note_id = note.id.clone();
            self.database.update_note(&note)?;
            tracing::info!("Updated note {} from {}", note_id, path.display());
        } else {
            // Create new note
            let note_id = self.database.create_note(&note)?;
            tracing::info!("Imported note {} from {}", note_id, path.display());
        }

        // Extract and create links
        let links = storage.extract_links(&note.content)?;
        for link in links {
            // Resolve link target
            if let Ok(Some(target_note)) = self.resolve_link(&link.target) {
                self.database.create_link(
                    &note.id,
                    &target_note.id,
                    &link.link_type.to_string(),
                    link.display_text.as_deref(),
                )?;
            }
        }

        // Extract and create tags
        let tags = storage.extract_tags(&note.content)?;
        for tag in tags {
            self.database.create_tag(&note.id, &tag)?;
        }

        Ok(note)
    }

    fn is_note_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| Format::from_extension(ext) == Some(self.format))
            .unwrap_or(false)
    }

    fn resolve_link(&self, link: &str) -> Result<Option<Note>> {
        let results = self.database.search_notes(link, 1)?;

        Ok(results.into_iter().next())
    }
}

#[derive(Debug, Default)]
pub struct ImportResult {
    pub imported: usize,
    pub failed: usize,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

impl ImportResult {
    fn scan_directories(&mut self, vault_path: &Path) -> Result<()> {
        // Create missing directories
        let dirs = [
            "daily",
            "inbox",
            "literature",
            "permanent",
            "reference",
            "index",
            "attachments",
        ];

        for dir in dirs {
            std::fs::create_dir_all(vault_path.join(dir)).map_err(ZtlgrError::Io)?;
        }

        Ok(())
    }
}
