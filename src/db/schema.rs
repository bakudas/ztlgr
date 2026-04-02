use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::{Result as ZResult, ZtlgrError};
use crate::note::{Note, NoteId, NoteType, ZettelId};

pub use rusqlite;

pub struct Database {
    conn: Arc<Mutex<rusqlite::Connection>>,
    path: PathBuf,
}

impl Database {
    pub fn new(path: &Path) -> ZResult<Self> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ZtlgrError::Io(e))?;
        }

        let conn = rusqlite::Connection::open(path).map_err(ZtlgrError::Database)?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            path: path.to_path_buf(),
        };

        db.initialize()?;

        Ok(db)
    }

    fn initialize(&self) -> ZResult<()> {
        let conn = self.conn.lock();

        // Create tables
        conn.execute_batch(include_str!("../schema.sql"))
            .map_err(ZtlgrError::Database)?;

        Ok(())
    }

    // Note operations
    pub fn create_note(&self, note: &Note) -> ZResult<NoteId> {
        let conn = self.conn.lock();

        let id = NoteId::new();
        let now = chrono::Utc::now();

        conn.execute(
            "INSERT INTO notes (id, title, content, note_type, zettel_id, parent_id, source, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                id.as_str(),
                note.title,
                note.content,
                note.note_type.as_str(),
                note.zettel_id.as_ref().map(|z| z.as_str()),
                note.parent_id.as_ref().map(|p: &NoteId| p.as_str()),
                note.source,
                serde_json::to_string(&note.metadata).map_err(|e| ZtlgrError::Serialization(e))?,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ]
        ).map_err(ZtlgrError::Database)?;

        Ok(id)
    }

    pub fn get_note(&self, id: &NoteId) -> ZResult<Option<Note>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, title, content, note_type, zettel_id, parent_id, source, metadata, created_at, updated_at
             FROM notes WHERE id = ?1 AND status = 'active'"
        ).map_err(ZtlgrError::Database)?;

        let note = stmt
            .query_row(rusqlite::params![id.as_str()], |row| {
                Ok(Note {
                    id: NoteId::parse(&row.get::<_, String>(0)?).unwrap(),
                    title: row.get(1)?,
                    content: row.get(2)?,
                    note_type: NoteType::from_str(&row.get::<_, String>(3)?).unwrap(),
                    zettel_id: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| ZettelId::parse(&s).ok()),
                    parent_id: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| NoteId::parse(&s).ok()),
                    source: row.get(6)?,
                    metadata: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                })
            })
            .ok();

        Ok(note)
    }

    pub fn update_note(&self, note: &Note) -> ZResult<()> {
        let conn = self.conn.lock();

        let now = chrono::Utc::now();

        conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, note_type = ?3, zettel_id = ?4, parent_id = ?5, source = ?6, metadata = ?7, updated_at = ?8
             WHERE id = ?9",
            rusqlite::params![
                note.title,
                note.content,
                note.note_type.as_str(),
                note.zettel_id.as_ref().map(|z| z.as_str()),
                note.parent_id.as_ref().map(|p: &NoteId| p.as_str()),
                note.source,
                serde_json::to_string(&note.metadata).map_err(|e| ZtlgrError::Serialization(e))?,
                now.to_rfc3339(),
                note.id.as_str(),
            ]
        ).map_err(ZtlgrError::Database)?;

        Ok(())
    }

    pub fn delete_note(&self, id: &NoteId) -> ZResult<()> {
        let conn = self.conn.lock();

        conn.execute(
            "UPDATE notes SET status = 'deleted', updated_at = ?1 WHERE id = ?2",
            rusqlite::params![chrono::Utc::now().to_rfc3339(), id.as_str()],
        )
        .map_err(ZtlgrError::Database)?;

        Ok(())
    }

    pub fn list_notes(&self, limit: usize, offset: usize) -> ZResult<Vec<Note>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, title, content, note_type, zettel_id, parent_id, source, metadata, created_at, updated_at
             FROM notes
             WHERE status = 'active'
             ORDER BY updated_at DESC
             LIMIT ?1 OFFSET ?2"
        ).map_err(ZtlgrError::Database)?;

        let notes = stmt
            .query_map(rusqlite::params![limit as i32, offset as i32], |row| {
                Ok(Note {
                    id: NoteId::parse(&row.get::<_, String>(0)?).unwrap(),
                    title: row.get(1)?,
                    content: row.get(2)?,
                    note_type: NoteType::from_str(&row.get::<_, String>(3)?).unwrap(),
                    zettel_id: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| ZettelId::parse(&s).ok()),
                    parent_id: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| NoteId::parse(&s).ok()),
                    source: row.get(6)?,
                    metadata: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                })
            })
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(notes)
    }

    pub fn search_notes(&self, query: &str, limit: usize) -> ZResult<Vec<Note>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, n.content, n.note_type, n.zettel_id, n.parent_id, n.source, n.metadata, n.created_at, n.updated_at
             FROM notes n
             JOIN notes_fts fts ON n.id = fts.id
             WHERE notes_fts MATCH ?1
             ORDER BY bm25(notes_fts)
             LIMIT ?2"
        ).map_err(ZtlgrError::Database)?;

        let notes = stmt
            .query_map(rusqlite::params![query, limit as i32], |row| {
                Ok(Note {
                    id: NoteId::parse(&row.get::<_, String>(0)?).unwrap(),
                    title: row.get(1)?,
                    content: row.get(2)?,
                    note_type: NoteType::from_str(&row.get::<_, String>(3)?).unwrap(),
                    zettel_id: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| ZettelId::parse(&s).ok()),
                    parent_id: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| NoteId::parse(&s).ok()),
                    source: row.get(6)?,
                    metadata: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                })
            })
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(notes)
    }

    pub fn create_link(
        &self,
        source_id: &NoteId,
        target_id: &NoteId,
        link_type: &str,
        context: Option<&str>,
    ) -> ZResult<()> {
        let conn = self.conn.lock();

        conn.execute(
            "INSERT OR IGNORE INTO links (source_id, target_id, link_type, context)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![source_id.as_str(), target_id.as_str(), link_type, context],
        )
        .map_err(ZtlgrError::Database)?;

        Ok(())
    }

    pub fn create_tag(&self, note_id: &NoteId, tag: &str) -> ZResult<()> {
        let conn = self.conn.lock();

        // Insert or get tag
        conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            rusqlite::params![tag],
        )
        .map_err(ZtlgrError::Database)?;

        let tag_id: i64 = conn
            .query_row(
                "SELECT id FROM tags WHERE name = ?1",
                rusqlite::params![tag],
                |row| row.get(0),
            )
            .map_err(ZtlgrError::Database)?;

        // Link note to tag
        conn.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![note_id.as_str(), tag_id],
        )
        .map_err(ZtlgrError::Database)?;

        Ok(())
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }
}
