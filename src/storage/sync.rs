use std::path::PathBuf;

use crate::db::Database;
use crate::error::Result;
use crate::note::Note;
use crate::storage::{Format, MarkdownStorage, OrgStorage, Storage};

pub struct FileSync {
    vault_path: PathBuf,
    format: Format,
    database: Database,
}

impl FileSync {
    pub fn new(vault_path: PathBuf, format: Format, database: Database) -> Self {
        Self {
            vault_path,
            format,
            database,
        }
    }

    pub fn sync_to_file(&self, note: &Note) -> Result<PathBuf> {
        let storage: Box<dyn Storage> = match self.format {
            Format::Markdown => Box::new(MarkdownStorage::new()),
            Format::Org => Box::new(OrgStorage::new()),
        };

        // Determine file path based on note type and zettel ID
        let subdir = match &note.note_type {
            crate::note::NoteType::Daily => "daily",
            crate::note::NoteType::Fleeting => "inbox",
            crate::note::NoteType::Literature { .. } => "literature",
            crate::note::NoteType::Permanent => "permanent",
            crate::note::NoteType::Reference { .. } => "reference",
            crate::note::NoteType::Index => "index",
        };

        let filename = note
            .zettel_id
            .as_ref()
            .map(|z| format!("{}-{}", z.as_str(), self.sanitize_filename(&note.title)))
            .unwrap_or_else(|| self.sanitize_filename(&note.title));

        let path =
            self.vault_path
                .join(subdir)
                .join(format!("{}.{}", filename, self.format.extension()));

        // Write note to file
        storage.write_note(note, &path)?;

        tracing::info!("Synced note {} to {}", note.id, path.display());

        Ok(path)
    }

    pub fn sync_from_file(&self, path: &std::path::Path) -> Result<Note> {
        let storage: Box<dyn Storage> = match self.format {
            Format::Markdown => Box::new(MarkdownStorage::new()),
            Format::Org => Box::new(OrgStorage::new()),
        };

        let note = storage.read_note(path)?;

        // Update or create in database
        if self.database.get_note(&note.id)?.is_some() {
            self.database.update_note(&note)?;
        } else {
            self.database.create_note(&note)?;
        }

        tracing::info!("Synced file {} to database", path.display());

        Ok(note)
    }

    pub fn full_sync(&self) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        // Get all notes from database
        let db_notes = self.database.list_notes(1000, 0)?;

        // Get all files in vault
        let files = self.list_vault_files()?;

        // Build sets for comparison
        let db_note_ids: std::collections::HashSet<_> =
            db_notes.iter().map(|n| n.id.as_str().to_string()).collect();

        let file_note_ids: std::collections::HashSet<_> = files
            .iter()
            .filter_map(|path| {
                let storage: Box<dyn Storage> = match self.format {
                    Format::Markdown => Box::new(MarkdownStorage::new()),
                    Format::Org => Box::new(OrgStorage::new()),
                };
                storage
                    .read_note(path)
                    .ok()
                    .map(|n| n.id.as_str().to_string())
            })
            .collect();

        // Notes only in database (need to create files)
        for note in &db_notes {
            if !file_note_ids.contains(note.id.as_str()) {
                self.sync_to_file(note)?;
                result.created_files += 1;
            }
        }

        // Notes only in files (need to import to database)
        for path in &files {
            let storage: Box<dyn Storage> = match self.format {
                Format::Markdown => Box::new(MarkdownStorage::new()),
                Format::Org => Box::new(OrgStorage::new()),
            };

            if let Ok(note) = storage.read_note(path) {
                if !db_note_ids.contains(note.id.as_str()) {
                    self.database.create_note(&note)?;
                    result.imported_notes += 1;
                }
            }
        }

        // Notes in both (check for updates)
        for note in &db_notes {
            if file_note_ids.contains(note.id.as_str()) {
                // Compare timestamps and sync newer to older
                // For now, just mark as synced
                result.synced += 1;
            }
        }

        Ok(result)
    }

    fn list_vault_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

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
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| Format::from_extension(ext) == Some(self.format))
                .unwrap_or(false)
            {
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }

    fn sanitize_filename(&self, name: &str) -> String {
        // Remove or replace invalid characters
        let mut filename = name.to_string();

        // Replace spaces with hyphens
        filename = filename.replace(' ', "-");

        // Remove invalid characters
        filename.retain(|c| c.is_alphanumeric() || c == '-' || c == '_');

        // Limit length
        if filename.len() > 100 {
            filename = filename[..100].to_string();
        }

        filename
    }
}

#[derive(Debug, Default)]
pub struct SyncResult {
    pub created_files: usize,
    pub imported_notes: usize,
    pub synced: usize,
}
