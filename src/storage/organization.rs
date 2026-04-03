use crate::error::Result;
use crate::note::{Note, NoteType};
use std::path::{Path, PathBuf};

/// Organizes notes into a folder structure by note type
pub struct NoteOrganizer;

impl NoteOrganizer {
    /// Get the folder name for a given note type
    pub fn folder_for_note_type(note_type: &NoteType) -> &'static str {
        match note_type {
            NoteType::Daily => "daily",
            NoteType::Fleeting => "fleeting",
            NoteType::Literature { .. } => "literature",
            NoteType::Permanent => "permanent",
            NoteType::Reference { .. } => "reference",
            NoteType::Index => "index",
        }
    }

    /// Create all necessary folder directories
    pub fn create_folders(vault_path: &Path) -> Result<()> {
        let folders = [
            "daily",
            "fleeting",
            "literature",
            "permanent",
            "reference",
            "index",
        ];

        for folder in &folders {
            let folder_path = vault_path.join(folder);
            std::fs::create_dir_all(&folder_path)?;
        }

        Ok(())
    }

    /// Check if all required folders exist
    pub fn folders_exist(vault_path: &Path) -> bool {
        let folders = [
            "daily",
            "fleeting",
            "literature",
            "permanent",
            "reference",
            "index",
        ];
        folders.iter().all(|folder| {
            let folder_path = vault_path.join(folder);
            folder_path.exists() && folder_path.is_dir()
        })
    }

    /// Get the full path for a note in the vault
    pub fn get_note_path(vault_path: &Path, note: &Note, format: &str) -> PathBuf {
        let folder = Self::folder_for_note_type(&note.note_type);
        let filename = format!("{}.{}", sanitize_filename::sanitize(&note.title), format);
        vault_path.join(folder).join(filename)
    }

    /// Move a note to the appropriate folder based on its type
    pub fn organize_note(vault_path: &Path, note: &Note, format: &str) -> Result<()> {
        let target_path = Self::get_note_path(vault_path, note, format);

        // Create parent directory if it doesn't exist
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(())
    }

    /// List all notes in a specific folder
    pub fn list_notes_in_folder(
        vault_path: &Path,
        folder: &str,
        format: &str,
    ) -> Result<Vec<PathBuf>> {
        let folder_path = vault_path.join(folder);

        if !folder_path.exists() {
            return Ok(Vec::new());
        }

        let mut notes = Vec::new();
        for entry in std::fs::read_dir(&folder_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == format {
                        notes.push(path);
                    }
                }
            }
        }

        Ok(notes)
    }

    /// Get all notes across all folders
    pub fn list_all_notes(vault_path: &Path, format: &str) -> Result<Vec<PathBuf>> {
        let folders = [
            "daily",
            "fleeting",
            "literature",
            "permanent",
            "reference",
            "index",
        ];
        let mut all_notes = Vec::new();

        for folder in &folders {
            let mut folder_notes = Self::list_notes_in_folder(vault_path, folder, format)?;
            all_notes.append(&mut folder_notes);
        }

        Ok(all_notes)
    }

    /// Get folder statistics
    pub fn get_folder_stats(vault_path: &Path, format: &str) -> Result<FolderStats> {
        let folders = [
            "daily",
            "fleeting",
            "literature",
            "permanent",
            "reference",
            "index",
        ];
        let mut stats = FolderStats::default();

        for folder in &folders {
            let notes = Self::list_notes_in_folder(vault_path, folder, format)?;
            match *folder {
                "daily" => stats.daily_count = notes.len(),
                "fleeting" => stats.fleeting_count = notes.len(),
                "literature" => stats.literature_count = notes.len(),
                "permanent" => stats.permanent_count = notes.len(),
                "reference" => stats.reference_count = notes.len(),
                "index" => stats.index_count = notes.len(),
                _ => {}
            }
            stats.total_count += notes.len();
        }

        Ok(stats)
    }
}

/// Statistics about folder organization
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FolderStats {
    pub daily_count: usize,
    pub fleeting_count: usize,
    pub literature_count: usize,
    pub permanent_count: usize,
    pub reference_count: usize,
    pub index_count: usize,
    pub total_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ============================================================================
    // FOLDER HELPER TESTS
    // ============================================================================

    #[test]
    fn test_folder_for_note_type_daily() {
        assert_eq!(
            NoteOrganizer::folder_for_note_type(&NoteType::Daily),
            "daily"
        );
    }

    #[test]
    fn test_folder_for_note_type_fleeting() {
        assert_eq!(
            NoteOrganizer::folder_for_note_type(&NoteType::Fleeting),
            "fleeting"
        );
    }

    #[test]
    fn test_folder_for_note_type_permanent() {
        assert_eq!(
            NoteOrganizer::folder_for_note_type(&NoteType::Permanent),
            "permanent"
        );
    }

    #[test]
    fn test_folder_for_note_type_literature() {
        assert_eq!(
            NoteOrganizer::folder_for_note_type(&NoteType::Literature {
                source: "Book".to_string()
            }),
            "literature"
        );
    }

    #[test]
    fn test_folder_for_note_type_reference() {
        assert_eq!(
            NoteOrganizer::folder_for_note_type(&NoteType::Reference { url: None }),
            "reference"
        );
    }

    #[test]
    fn test_folder_for_note_type_index() {
        assert_eq!(
            NoteOrganizer::folder_for_note_type(&NoteType::Index),
            "index"
        );
    }

    // ============================================================================
    // FOLDER CREATION TESTS
    // ============================================================================

    #[test]
    fn test_create_folders() {
        let dir = tempdir().unwrap();
        let result = NoteOrganizer::create_folders(dir.path());

        assert!(result.is_ok());
        assert!(dir.path().join("daily").exists());
        assert!(dir.path().join("fleeting").exists());
        assert!(dir.path().join("literature").exists());
        assert!(dir.path().join("permanent").exists());
        assert!(dir.path().join("reference").exists());
        assert!(dir.path().join("index").exists());
    }

    #[test]
    fn test_folders_exist_all_present() {
        let dir = tempdir().unwrap();
        NoteOrganizer::create_folders(dir.path()).unwrap();

        assert!(NoteOrganizer::folders_exist(dir.path()));
    }

    #[test]
    fn test_folders_exist_missing_some() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("daily")).unwrap();
        std::fs::create_dir_all(dir.path().join("permanent")).unwrap();

        assert!(!NoteOrganizer::folders_exist(dir.path()));
    }

    #[test]
    fn test_folders_exist_none_present() {
        let dir = tempdir().unwrap();

        assert!(!NoteOrganizer::folders_exist(dir.path()));
    }

    // ============================================================================
    // NOTE PATH TESTS
    // ============================================================================

    #[test]
    fn test_get_note_path_daily() {
        let dir = tempdir().unwrap();
        let note = Note::new("My Daily Note".to_string(), "Content".to_string())
            .with_type(NoteType::Daily);

        let path = NoteOrganizer::get_note_path(dir.path(), &note, "md");

        assert!(path.starts_with(dir.path().join("daily")));
        assert!(path.to_string_lossy().contains("My Daily Note"));
        assert!(path.to_string_lossy().ends_with(".md"));
    }

    #[test]
    fn test_get_note_path_permanent() {
        let dir = tempdir().unwrap();
        let note = Note::new("Permanent Idea".to_string(), "Content".to_string())
            .with_type(NoteType::Permanent);

        let path = NoteOrganizer::get_note_path(dir.path(), &note, "md");

        assert!(path.starts_with(dir.path().join("permanent")));
        assert!(path.to_string_lossy().contains("Permanent Idea"));
    }

    #[test]
    fn test_get_note_path_with_special_chars() {
        let dir = tempdir().unwrap();
        let note = Note::new("Note (2024) [Draft]".to_string(), "Content".to_string())
            .with_type(NoteType::Daily);

        let path = NoteOrganizer::get_note_path(dir.path(), &note, "md");

        // Sanitized filename should not contain invalid chars
        assert!(path.starts_with(dir.path().join("daily")));
        assert!(path.to_string_lossy().ends_with(".md"));
    }

    #[test]
    fn test_get_note_path_org_format() {
        let dir = tempdir().unwrap();
        let note =
            Note::new("My Note".to_string(), "Content".to_string()).with_type(NoteType::Permanent);

        let path = NoteOrganizer::get_note_path(dir.path(), &note, "org");

        assert!(path.to_string_lossy().ends_with(".org"));
    }

    // ============================================================================
    // ORGANIZE NOTE TESTS
    // ============================================================================

    #[test]
    fn test_organize_note_creates_folder() {
        let dir = tempdir().unwrap();
        let note =
            Note::new("My Note".to_string(), "Content".to_string()).with_type(NoteType::Daily);

        let result = NoteOrganizer::organize_note(dir.path(), &note, "md");

        assert!(result.is_ok());
        assert!(dir.path().join("daily").exists());
    }

    #[test]
    fn test_organize_note_various_types() {
        let dir = tempdir().unwrap();
        let types = vec![
            NoteType::Daily,
            NoteType::Fleeting,
            NoteType::Permanent,
            NoteType::Literature {
                source: "Book".to_string(),
            },
            NoteType::Reference { url: None },
            NoteType::Index,
        ];

        for note_type in types {
            let note =
                Note::new("Test Note".to_string(), "Content".to_string()).with_type(note_type);
            let result = NoteOrganizer::organize_note(dir.path(), &note, "md");
            assert!(result.is_ok());
        }
    }

    // ============================================================================
    // LIST NOTES TESTS
    // ============================================================================

    #[test]
    fn test_list_notes_in_folder_empty() {
        let dir = tempdir().unwrap();
        NoteOrganizer::create_folders(dir.path()).unwrap();

        let notes = NoteOrganizer::list_notes_in_folder(dir.path(), "daily", "md").unwrap();

        assert_eq!(notes.len(), 0);
    }

    #[test]
    fn test_list_notes_in_folder_with_files() {
        let dir = tempdir().unwrap();
        NoteOrganizer::create_folders(dir.path()).unwrap();

        // Create some test files
        std::fs::write(dir.path().join("daily").join("note1.md"), "content1").unwrap();
        std::fs::write(dir.path().join("daily").join("note2.md"), "content2").unwrap();
        std::fs::write(dir.path().join("daily").join("note3.txt"), "content3").unwrap(); // Wrong format

        let notes = NoteOrganizer::list_notes_in_folder(dir.path(), "daily", "md").unwrap();

        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn test_list_notes_in_folder_nonexistent() {
        let dir = tempdir().unwrap();

        let notes = NoteOrganizer::list_notes_in_folder(dir.path(), "daily", "md").unwrap();

        assert_eq!(notes.len(), 0);
    }

    #[test]
    fn test_list_notes_in_folder_filters_format() {
        let dir = tempdir().unwrap();
        NoteOrganizer::create_folders(dir.path()).unwrap();

        std::fs::write(dir.path().join("daily").join("note1.md"), "content1").unwrap();
        std::fs::write(dir.path().join("daily").join("note2.org"), "content2").unwrap();

        let md_notes = NoteOrganizer::list_notes_in_folder(dir.path(), "daily", "md").unwrap();
        let org_notes = NoteOrganizer::list_notes_in_folder(dir.path(), "daily", "org").unwrap();

        assert_eq!(md_notes.len(), 1);
        assert_eq!(org_notes.len(), 1);
    }

    // ============================================================================
    // LIST ALL NOTES TESTS
    // ============================================================================

    #[test]
    fn test_list_all_notes_empty() {
        let dir = tempdir().unwrap();
        NoteOrganizer::create_folders(dir.path()).unwrap();

        let notes = NoteOrganizer::list_all_notes(dir.path(), "md").unwrap();

        assert_eq!(notes.len(), 0);
    }

    #[test]
    fn test_list_all_notes_across_folders() {
        let dir = tempdir().unwrap();
        NoteOrganizer::create_folders(dir.path()).unwrap();

        // Create notes in different folders
        std::fs::write(dir.path().join("daily").join("daily1.md"), "content").unwrap();
        std::fs::write(dir.path().join("daily").join("daily2.md"), "content").unwrap();
        std::fs::write(dir.path().join("permanent").join("perm1.md"), "content").unwrap();
        std::fs::write(dir.path().join("literature").join("lit1.md"), "content").unwrap();

        let notes = NoteOrganizer::list_all_notes(dir.path(), "md").unwrap();

        assert_eq!(notes.len(), 4);
    }

    #[test]
    fn test_list_all_notes_only_matching_format() {
        let dir = tempdir().unwrap();
        NoteOrganizer::create_folders(dir.path()).unwrap();

        std::fs::write(dir.path().join("daily").join("note1.md"), "content").unwrap();
        std::fs::write(dir.path().join("daily").join("note2.org"), "content").unwrap();
        std::fs::write(dir.path().join("permanent").join("note3.md"), "content").unwrap();

        let md_notes = NoteOrganizer::list_all_notes(dir.path(), "md").unwrap();

        assert_eq!(md_notes.len(), 2);
    }

    // ============================================================================
    // FOLDER STATS TESTS
    // ============================================================================

    #[test]
    fn test_get_folder_stats_empty() {
        let dir = tempdir().unwrap();
        NoteOrganizer::create_folders(dir.path()).unwrap();

        let stats = NoteOrganizer::get_folder_stats(dir.path(), "md").unwrap();

        assert_eq!(stats.daily_count, 0);
        assert_eq!(stats.fleeting_count, 0);
        assert_eq!(stats.permanent_count, 0);
        assert_eq!(stats.total_count, 0);
    }

    #[test]
    fn test_get_folder_stats_with_notes() {
        let dir = tempdir().unwrap();
        NoteOrganizer::create_folders(dir.path()).unwrap();

        std::fs::write(dir.path().join("daily").join("daily1.md"), "content").unwrap();
        std::fs::write(dir.path().join("daily").join("daily2.md"), "content").unwrap();
        std::fs::write(dir.path().join("permanent").join("perm1.md"), "content").unwrap();
        std::fs::write(dir.path().join("literature").join("lit1.md"), "content").unwrap();
        std::fs::write(dir.path().join("literature").join("lit2.md"), "content").unwrap();

        let stats = NoteOrganizer::get_folder_stats(dir.path(), "md").unwrap();

        assert_eq!(stats.daily_count, 2);
        assert_eq!(stats.permanent_count, 1);
        assert_eq!(stats.literature_count, 2);
        assert_eq!(stats.fleeting_count, 0);
        assert_eq!(stats.reference_count, 0);
        assert_eq!(stats.index_count, 0);
        assert_eq!(stats.total_count, 5);
    }

    #[test]
    fn test_get_folder_stats_all_folders() {
        let dir = tempdir().unwrap();
        NoteOrganizer::create_folders(dir.path()).unwrap();

        // Add one note to each folder
        std::fs::write(dir.path().join("daily").join("daily.md"), "content").unwrap();
        std::fs::write(dir.path().join("fleeting").join("fleeting.md"), "content").unwrap();
        std::fs::write(dir.path().join("permanent").join("permanent.md"), "content").unwrap();
        std::fs::write(
            dir.path().join("literature").join("literature.md"),
            "content",
        )
        .unwrap();
        std::fs::write(dir.path().join("reference").join("reference.md"), "content").unwrap();
        std::fs::write(dir.path().join("index").join("index.md"), "content").unwrap();

        let stats = NoteOrganizer::get_folder_stats(dir.path(), "md").unwrap();

        assert_eq!(stats.daily_count, 1);
        assert_eq!(stats.fleeting_count, 1);
        assert_eq!(stats.permanent_count, 1);
        assert_eq!(stats.literature_count, 1);
        assert_eq!(stats.reference_count, 1);
        assert_eq!(stats.index_count, 1);
        assert_eq!(stats.total_count, 6);
    }

    // ============================================================================
    // FOLDER STATS STRUCT TESTS
    // ============================================================================

    #[test]
    fn test_folder_stats_default() {
        let stats = FolderStats::default();

        assert_eq!(stats.daily_count, 0);
        assert_eq!(stats.fleeting_count, 0);
        assert_eq!(stats.permanent_count, 0);
        assert_eq!(stats.total_count, 0);
    }

    #[test]
    fn test_folder_stats_equality() {
        let stats1 = FolderStats {
            daily_count: 5,
            fleeting_count: 3,
            literature_count: 2,
            permanent_count: 10,
            reference_count: 1,
            index_count: 2,
            total_count: 23,
        };

        let stats2 = FolderStats {
            daily_count: 5,
            fleeting_count: 3,
            literature_count: 2,
            permanent_count: 10,
            reference_count: 1,
            index_count: 2,
            total_count: 23,
        };

        assert_eq!(stats1, stats2);
    }
}
