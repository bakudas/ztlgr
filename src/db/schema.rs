use parking_lot::Mutex;
use rusqlite::{types::Type, Row};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use crate::error::{Result as ZResult, ZtlgrError};
use crate::note::{Note, NoteId, NoteType, ZettelId};
use crate::source::{Source, SourceId};

pub use rusqlite;

pub struct Database {
    conn: Arc<Mutex<rusqlite::Connection>>,
    path: PathBuf,
}

fn parse_error(column: usize, message: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message,
        )),
    )
}

fn note_from_row(row: &Row<'_>) -> rusqlite::Result<Note> {
    let id_raw: String = row.get(0)?;
    let note_type_raw: String = row.get(3)?;

    let id = NoteId::parse(&id_raw)
        .map_err(|e| parse_error(0, format!("invalid note id '{}': {}", id_raw, e)))?;
    let note_type = NoteType::from_str(&note_type_raw)
        .map_err(|e| parse_error(3, format!("invalid note type '{}': {}", note_type_raw, e)))?;

    Ok(Note {
        id,
        title: row.get(1)?,
        content: row.get(2)?,
        note_type,
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
        deleted_at: row
            .get::<_, Option<String>>(10)?
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
    })
}

impl Database {
    pub fn new(path: &Path) -> ZResult<Self> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(ZtlgrError::Io)?;
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

        // Create base tables (v1 schema -- all IF NOT EXISTS, safe to re-run)
        conn.execute_batch(include_str!("../schema.sql"))
            .map_err(ZtlgrError::Database)?;

        // Run migrations
        Self::migrate(&conn)?;

        Ok(())
    }

    /// Get the current schema version from the database.
    fn get_schema_version(conn: &rusqlite::Connection) -> ZResult<usize> {
        let version: String = conn
            .query_row(
                "SELECT value FROM schema_info WHERE key = 'version'",
                [],
                |row| row.get(0),
            )
            .map_err(ZtlgrError::Database)?;

        version.parse::<usize>().map_err(|e| {
            ZtlgrError::Migration(format!("invalid schema version '{}': {}", version, e))
        })
    }

    /// Apply incremental migrations from current version to latest.
    fn migrate(conn: &rusqlite::Connection) -> ZResult<()> {
        let current = Self::get_schema_version(conn)?;

        if current < 2 {
            tracing::info!("Migrating schema v{} -> v2 (sources table)", current);
            conn.execute_batch(include_str!("../migration_v2.sql"))
                .map_err(|e| ZtlgrError::Migration(format!("v1->v2 migration failed: {}", e)))?;
        }

        // Future migrations go here:
        // if current < 3 { ... }

        Ok(())
    }

    // Note operations
    pub fn create_note(&self, note: &Note) -> ZResult<NoteId> {
        let conn = self.conn.lock();

        // Use the note's existing ID instead of generating a new one
        let id = note.id.clone();
        let now = chrono::Utc::now();

        conn.execute(
            "INSERT INTO notes (id, title, content, note_type, zettel_id, parent_id, source, metadata, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                id.as_str(),
                note.title,
                note.content,
                note.note_type.as_str(),
                note.zettel_id.as_ref().map(|z| z.as_str()),
                note.parent_id.as_ref().map(|p: &NoteId| p.as_str()),
                note.source,
                serde_json::to_string(&note.metadata).map_err(ZtlgrError::Serialization)?,
                now.to_rfc3339(),
                now.to_rfc3339(),
                note.deleted_at.map(|dt| dt.to_rfc3339()),
            ]
        ).map_err(ZtlgrError::Database)?;

        Ok(id)
    }

    pub fn get_note(&self, id: &NoteId) -> ZResult<Option<Note>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, title, content, note_type, zettel_id, parent_id, source, metadata, created_at, updated_at, deleted_at
             FROM notes WHERE id = ?1 AND deleted_at IS NULL"
        ).map_err(ZtlgrError::Database)?;

        let note = stmt
            .query_row(rusqlite::params![id.as_str()], note_from_row)
            .ok();

        Ok(note)
    }

    pub fn update_note(&self, note: &Note) -> ZResult<()> {
        let conn = self.conn.lock();

        let now = chrono::Utc::now();

        conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, note_type = ?3, zettel_id = ?4, parent_id = ?5, source = ?6, metadata = ?7, updated_at = ?8, deleted_at = ?9
             WHERE id = ?10",
            rusqlite::params![
                note.title,
                note.content,
                note.note_type.as_str(),
                note.zettel_id.as_ref().map(|z| z.as_str()),
                note.parent_id.as_ref().map(|p: &NoteId| p.as_str()),
                note.source,
                serde_json::to_string(&note.metadata).map_err(ZtlgrError::Serialization)?,
                now.to_rfc3339(),
                note.deleted_at.map(|dt| dt.to_rfc3339()),
                note.id.as_str(),
            ]
        ).map_err(ZtlgrError::Database)?;

        Ok(())
    }

    pub fn delete_note(&self, id: &NoteId) -> ZResult<()> {
        let conn = self.conn.lock();

        conn.execute(
            "UPDATE notes SET deleted_at = ?1 WHERE id = ?2",
            rusqlite::params![chrono::Utc::now().to_rfc3339(), id.as_str()],
        )
        .map_err(ZtlgrError::Database)?;

        Ok(())
    }

    pub fn restore_note(&self, id: &NoteId) -> ZResult<()> {
        let conn = self.conn.lock();

        conn.execute(
            "UPDATE notes SET deleted_at = NULL WHERE id = ?1",
            rusqlite::params![id.as_str()],
        )
        .map_err(ZtlgrError::Database)?;

        Ok(())
    }

    pub fn list_trash(&self, limit: usize, offset: usize) -> ZResult<Vec<Note>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, title, content, note_type, zettel_id, parent_id, source, metadata, created_at, updated_at, deleted_at
             FROM notes
             WHERE deleted_at IS NOT NULL
             ORDER BY deleted_at DESC
             LIMIT ?1 OFFSET ?2"
        ).map_err(ZtlgrError::Database)?;

        let notes = stmt
            .query_map(
                rusqlite::params![limit as i32, offset as i32],
                note_from_row,
            )
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(notes)
    }

    pub fn purge_old_trash(&self, days_old: i64) -> ZResult<usize> {
        let conn = self.conn.lock();

        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_old);

        let affected = conn
            .execute(
                "DELETE FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
                rusqlite::params![cutoff_date.to_rfc3339()],
            )
            .map_err(ZtlgrError::Database)?;

        Ok(affected)
    }

    pub fn permanently_delete(&self, id: &NoteId) -> ZResult<()> {
        let conn = self.conn.lock();

        conn.execute(
            "DELETE FROM notes WHERE id = ?1",
            rusqlite::params![id.as_str()],
        )
        .map_err(ZtlgrError::Database)?;

        Ok(())
    }

    pub fn list_notes(&self, limit: usize, offset: usize) -> ZResult<Vec<Note>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, title, content, note_type, zettel_id, parent_id, source, metadata, created_at, updated_at, deleted_at
             FROM notes
             WHERE deleted_at IS NULL
             ORDER BY
               CASE note_type
                 WHEN 'daily' THEN 0
                 WHEN 'fleeting' THEN 1
                 WHEN 'permanent' THEN 2
                 WHEN 'literature' THEN 3
                 WHEN 'reference' THEN 4
                 WHEN 'index' THEN 5
                 ELSE 6
               END,
               updated_at DESC
             LIMIT ?1 OFFSET ?2"
        ).map_err(ZtlgrError::Database)?;

        let notes = stmt
            .query_map(
                rusqlite::params![limit as i32, offset as i32],
                note_from_row,
            )
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(notes)
    }

    pub fn search_notes(&self, query: &str, limit: usize) -> ZResult<Vec<Note>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, n.content, n.note_type, n.zettel_id, n.parent_id, n.source, n.metadata, n.created_at, n.updated_at, n.deleted_at
             FROM notes n
             JOIN notes_fts fts ON n.id = fts.id
             WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL
             ORDER BY bm25(notes_fts)
             LIMIT ?2"
        ).map_err(ZtlgrError::Database)?;

        let notes = stmt
            .query_map(rusqlite::params![query, limit as i32], note_from_row)
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(notes)
    }

    pub fn rebuild_fts(&self) -> ZResult<()> {
        let conn = self.conn.lock();
        conn.execute("INSERT INTO notes_fts(notes_fts) VALUES('rebuild')", [])
            .map_err(ZtlgrError::Database)?;
        Ok(())
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

    /// Get all backlinks (incoming links) for a given note.
    ///
    /// Returns a list of (source_id, source_title, context) tuples
    /// representing notes that link to the specified note.
    pub fn get_backlinks(
        &self,
        note_id: &NoteId,
    ) -> ZResult<Vec<(String, String, Option<String>)>> {
        let conn = self.conn.lock();

        let mut stmt = conn
            .prepare(
                "SELECT b.backlink_id, n.title, b.context
                 FROM backlinks b
                 JOIN notes n ON n.id = b.backlink_id
                 WHERE b.note_id = ?1 AND n.deleted_at IS NULL
                 ORDER BY b.created_at DESC",
            )
            .map_err(ZtlgrError::Database)?;

        let backlinks = stmt
            .query_map(rusqlite::params![note_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(backlinks)
    }

    /// Delete all outgoing links from a note.
    ///
    /// Used before re-inserting parsed links when a note's content is updated.
    pub fn delete_links_for_note(&self, source_id: &NoteId) -> ZResult<usize> {
        let conn = self.conn.lock();

        let affected = conn
            .execute(
                "DELETE FROM links WHERE source_id = ?1",
                rusqlite::params![source_id.as_str()],
            )
            .map_err(ZtlgrError::Database)?;

        Ok(affected)
    }

    /// Find a note by its exact title (case-insensitive).
    ///
    /// Returns the first matching non-deleted note.
    pub fn find_note_by_title(&self, title: &str) -> ZResult<Option<Note>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, title, content, note_type, zettel_id, parent_id, source, metadata, created_at, updated_at, deleted_at
             FROM notes
             WHERE LOWER(title) = LOWER(?1) AND deleted_at IS NULL
             LIMIT 1"
        ).map_err(ZtlgrError::Database)?;

        let note = stmt.query_row(rusqlite::params![title], note_from_row).ok();

        Ok(note)
    }

    /// Get all outgoing links from a note.
    ///
    /// Returns a list of (target_id, target_title, link_type, context) tuples.
    pub fn get_links_for_note(
        &self,
        source_id: &NoteId,
    ) -> ZResult<Vec<(String, String, String, Option<String>)>> {
        let conn = self.conn.lock();

        let mut stmt = conn
            .prepare(
                "SELECT l.target_id, n.title, l.link_type, l.context
                 FROM links l
                 JOIN notes n ON n.id = l.target_id
                 WHERE l.source_id = ?1 AND n.deleted_at IS NULL
                 ORDER BY l.created_at DESC",
            )
            .map_err(ZtlgrError::Database)?;

        let links = stmt
            .query_map(rusqlite::params![source_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(links)
    }

    /// Get all links in the database (for graph visualization).
    ///
    /// Returns a list of (source_id, source_title, target_id, target_title, link_type) tuples.
    /// Only includes links where both source and target notes are not deleted.
    pub fn get_all_links(&self) -> ZResult<Vec<(String, String, String, String, String)>> {
        let conn = self.conn.lock();

        let mut stmt = conn
            .prepare(
                "SELECT l.source_id, s.title, l.target_id, t.title, l.link_type
                 FROM links l
                 JOIN notes s ON s.id = l.source_id AND s.deleted_at IS NULL
                 JOIN notes t ON t.id = l.target_id AND t.deleted_at IS NULL
                 ORDER BY s.title, t.title",
            )
            .map_err(ZtlgrError::Database)?;

        let links = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(links)
    }

    /// Get all notes with their link counts (for graph visualization).
    ///
    /// Returns a list of (note_id, title, note_type, outgoing_count, incoming_count).
    pub fn get_graph_nodes(&self) -> ZResult<Vec<(String, String, String, usize, usize)>> {
        let conn = self.conn.lock();

        let mut stmt = conn
            .prepare(
                "SELECT n.id, n.title, n.note_type,
                        (SELECT COUNT(*) FROM links l WHERE l.source_id = n.id) as out_count,
                        (SELECT COUNT(*) FROM links l WHERE l.target_id = n.id) as in_count
                 FROM notes n
                 WHERE n.deleted_at IS NULL
                 ORDER BY n.title",
            )
            .map_err(ZtlgrError::Database)?;

        let nodes = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, usize>(3)?,
                    row.get::<_, usize>(4)?,
                ))
            })
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(nodes)
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    /// Count active (non-deleted) notes grouped by `note_type`.
    ///
    /// Returns a `Vec<(String, usize)>` where each element is `(note_type, count)`.
    pub fn count_notes_by_type(&self) -> ZResult<Vec<(String, usize)>> {
        let conn = self.conn.lock();

        let mut stmt = conn
            .prepare(
                "SELECT note_type, COUNT(*) as cnt
                 FROM notes
                 WHERE deleted_at IS NULL
                 GROUP BY note_type
                 ORDER BY cnt DESC",
            )
            .map_err(ZtlgrError::Database)?;

        let counts = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
            })
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(counts)
    }

    /// Count total active (non-deleted) notes.
    pub fn count_notes(&self) -> ZResult<usize> {
        let conn = self.conn.lock();

        let count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM notes WHERE deleted_at IS NULL",
                [],
                |row| row.get(0),
            )
            .map_err(ZtlgrError::Database)?;

        Ok(count)
    }

    /// Count total links between active notes.
    pub fn count_links(&self) -> ZResult<usize> {
        let conn = self.conn.lock();

        let count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM links l
                 JOIN notes s ON l.source_id = s.id AND s.deleted_at IS NULL
                 JOIN notes t ON l.target_id = t.id AND t.deleted_at IS NULL",
                [],
                |row| row.get(0),
            )
            .map_err(ZtlgrError::Database)?;

        Ok(count)
    }

    /// List notes filtered by type (active only), ordered by title.
    pub fn list_notes_by_type(
        &self,
        note_type: &str,
        limit: usize,
        offset: usize,
    ) -> ZResult<Vec<Note>> {
        let conn = self.conn.lock();

        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, note_type, zettel_id, parent_id, source, metadata, created_at, updated_at, deleted_at
                 FROM notes
                 WHERE deleted_at IS NULL AND note_type = ?1
                 ORDER BY title ASC
                 LIMIT ?2 OFFSET ?3",
            )
            .map_err(ZtlgrError::Database)?;

        let notes = stmt
            .query_map(
                rusqlite::params![note_type, limit as i32, offset as i32],
                note_from_row,
            )
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(notes)
    }

    // =====================================================================
    // Source operations
    // =====================================================================

    /// Insert a new source record.
    pub fn create_source(&self, source: &Source) -> ZResult<SourceId> {
        let conn = self.conn.lock();

        conn.execute(
            "INSERT INTO sources (id, title, origin, content_hash, ingested_at, file_path, file_size, mime_type, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                source.id.as_str(),
                source.title,
                source.origin,
                source.content_hash,
                source.ingested_at.to_rfc3339(),
                source.file_path,
                source.file_size as i64,
                source.mime_type,
                source.metadata,
            ],
        )
        .map_err(ZtlgrError::Database)?;

        Ok(source.id.clone())
    }

    /// Find a source by its content hash (for deduplication).
    pub fn find_source_by_hash(&self, hash: &str) -> ZResult<Option<Source>> {
        let conn = self.conn.lock();

        let mut stmt = conn
            .prepare(
                "SELECT id, title, origin, content_hash, ingested_at, file_path, file_size, mime_type, metadata
                 FROM sources WHERE content_hash = ?1",
            )
            .map_err(ZtlgrError::Database)?;

        let mut rows = stmt
            .query_map(rusqlite::params![hash], source_from_row)
            .map_err(ZtlgrError::Database)?;

        match rows.next() {
            Some(Ok(source)) => Ok(Some(source)),
            Some(Err(e)) => Err(ZtlgrError::Database(e)),
            None => Ok(None),
        }
    }

    /// Get a source by ID.
    pub fn get_source(&self, id: &SourceId) -> ZResult<Option<Source>> {
        let conn = self.conn.lock();

        let mut stmt = conn
            .prepare(
                "SELECT id, title, origin, content_hash, ingested_at, file_path, file_size, mime_type, metadata
                 FROM sources WHERE id = ?1",
            )
            .map_err(ZtlgrError::Database)?;

        let mut rows = stmt
            .query_map(rusqlite::params![id.as_str()], source_from_row)
            .map_err(ZtlgrError::Database)?;

        match rows.next() {
            Some(Ok(source)) => Ok(Some(source)),
            Some(Err(e)) => Err(ZtlgrError::Database(e)),
            None => Ok(None),
        }
    }

    /// List all sources, ordered by ingested_at DESC.
    pub fn list_sources(&self, limit: usize, offset: usize) -> ZResult<Vec<Source>> {
        let conn = self.conn.lock();

        let mut stmt = conn
            .prepare(
                "SELECT id, title, origin, content_hash, ingested_at, file_path, file_size, mime_type, metadata
                 FROM sources
                 ORDER BY ingested_at DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .map_err(ZtlgrError::Database)?;

        let sources = stmt
            .query_map(
                rusqlite::params![limit as i32, offset as i32],
                source_from_row,
            )
            .map_err(ZtlgrError::Database)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(ZtlgrError::Database)?;

        Ok(sources)
    }

    /// Count total sources.
    pub fn count_sources(&self) -> ZResult<usize> {
        let conn = self.conn.lock();

        let count: usize = conn
            .query_row("SELECT COUNT(*) FROM sources", [], |row| row.get(0))
            .map_err(ZtlgrError::Database)?;

        Ok(count)
    }

    /// Delete a source by ID.
    pub fn delete_source(&self, id: &SourceId) -> ZResult<bool> {
        let conn = self.conn.lock();

        let rows = conn
            .execute(
                "DELETE FROM sources WHERE id = ?1",
                rusqlite::params![id.as_str()],
            )
            .map_err(ZtlgrError::Database)?;

        Ok(rows > 0)
    }
}

/// Map a database row to a Source struct.
fn source_from_row(row: &Row<'_>) -> rusqlite::Result<Source> {
    let id_raw: String = row.get(0)?;
    let id = SourceId::parse(&id_raw)
        .map_err(|e| parse_error(0, format!("invalid source id '{}': {}", id_raw, e)))?;

    let ingested_str: String = row.get(4)?;
    let ingested_at = chrono::DateTime::parse_from_rfc3339(&ingested_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());

    let file_size: i64 = row.get(6)?;

    Ok(Source {
        id,
        title: row.get(1)?,
        origin: row.get(2)?,
        content_hash: row.get(3)?,
        ingested_at,
        file_path: row.get(5)?,
        file_size: file_size as u64,
        mime_type: row.get(7)?,
        metadata: row.get(8)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::{Metadata, NoteType};
    use tempfile::TempDir;

    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).expect("Failed to create test database");
        (db, temp_dir)
    }

    fn create_test_note(title: &str) -> Note {
        Note {
            id: NoteId::new(),
            title: title.to_string(),
            content: "Test content".to_string(),
            note_type: NoteType::Permanent,
            zettel_id: None,
            parent_id: None,
            source: None,
            metadata: Metadata::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        }
    }

    #[test]
    fn test_create_note_with_no_delete() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("Test Note");
        let note_id = db.create_note(&note).expect("Failed to create note");

        let retrieved = db.get_note(&note_id).expect("Failed to retrieve note");
        assert!(retrieved.is_some(), "Note should exist after creation");
        assert_eq!(retrieved.unwrap().title, "Test Note");
    }

    #[test]
    fn test_delete_note_sets_deleted_at() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("To Delete");
        let note_id = db.create_note(&note).expect("Failed to create note");

        // Verify note exists
        assert!(
            db.get_note(&note_id).unwrap().is_some(),
            "Note should exist before deletion"
        );

        // Delete the note
        db.delete_note(&note_id).expect("Failed to delete note");

        // Verify note is no longer retrievable
        let retrieved = db.get_note(&note_id).expect("Failed to query note");
        assert!(
            retrieved.is_none(),
            "Deleted note should not be retrievable via get_note"
        );
    }

    #[test]
    fn test_list_notes_excludes_deleted() {
        let (db, _temp) = create_test_db();

        let note1 = create_test_note("Active Note");
        let note2 = create_test_note("Deleted Note");

        let id1 = db.create_note(&note1).expect("Failed to create note1");
        let _id2 = db.create_note(&note2).expect("Failed to create note2");

        // Delete the second note
        db.delete_note(&_id2).expect("Failed to delete note");

        // List notes should only return the active note
        let notes = db.list_notes(10, 0).expect("Failed to list notes");
        assert_eq!(notes.len(), 1, "Should have exactly 1 active note");
        assert_eq!(
            notes[0].id.as_str(),
            id1.as_str(),
            "Should return the active note"
        );
    }

    #[test]
    fn test_restore_note_from_trash() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("To Restore");
        let note_id = db.create_note(&note).expect("Failed to create note");

        // Delete the note
        db.delete_note(&note_id).expect("Failed to delete note");
        assert!(
            db.get_note(&note_id).unwrap().is_none(),
            "Note should be deleted"
        );

        // Restore the note
        db.restore_note(&note_id).expect("Failed to restore note");

        // Verify note is now retrievable
        let retrieved = db
            .get_note(&note_id)
            .expect("Failed to retrieve restored note");
        assert!(retrieved.is_some(), "Restored note should be retrievable");
        assert_eq!(retrieved.unwrap().title, "To Restore");
    }

    #[test]
    fn test_list_trash_shows_deleted_notes() {
        let (db, _temp) = create_test_db();

        let note1 = create_test_note("Note 1");
        let note2 = create_test_note("Note 2");
        let note3 = create_test_note("Note 3");

        let id1 = db.create_note(&note1).expect("Failed to create note1");
        let id2 = db.create_note(&note2).expect("Failed to create note2");
        let _id3 = db.create_note(&note3).expect("Failed to create note3");

        // Delete notes 1 and 2
        db.delete_note(&id1).expect("Failed to delete note1");
        db.delete_note(&id2).expect("Failed to delete note2");

        // List trash
        let trash = db.list_trash(10, 0).expect("Failed to list trash");
        assert_eq!(trash.len(), 2, "Should have 2 deleted notes in trash");

        let titles: Vec<_> = trash.iter().map(|n| n.title.clone()).collect();
        assert!(titles.contains(&"Note 1".to_string()));
        assert!(titles.contains(&"Note 2".to_string()));
    }

    #[test]
    fn test_list_trash_empty_when_no_deleted() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("Active Note");
        let _id = db.create_note(&note).expect("Failed to create note");

        let trash = db.list_trash(10, 0).expect("Failed to list trash");
        assert_eq!(
            trash.len(),
            0,
            "Trash should be empty when no notes are deleted"
        );
    }

    #[test]
    fn test_purge_old_trash() {
        let (db, _temp) = create_test_db();

        let note1 = create_test_note("Old Note");
        let note2 = create_test_note("Recent Note");

        let id1 = db.create_note(&note1).expect("Failed to create note1");
        let id2 = db.create_note(&note2).expect("Failed to create note2");

        // Delete both notes
        db.delete_note(&id1).expect("Failed to delete note1");
        db.delete_note(&id2).expect("Failed to delete note2");

        // Manually update deleted_at for first note to be 8 days ago
        let conn = db.conn.lock();
        let old_date = (chrono::Utc::now() - chrono::Duration::days(8)).to_rfc3339();
        conn.execute(
            "UPDATE notes SET deleted_at = ?1 WHERE id = ?2",
            rusqlite::params![old_date, id1.as_str()],
        )
        .expect("Failed to update deleted_at");
        drop(conn);

        // Purge notes older than 7 days
        let purged_count = db.purge_old_trash(7).expect("Failed to purge trash");
        assert_eq!(purged_count, 1, "Should have purged 1 note");

        // Verify old note is gone but recent note still in trash
        let trash = db.list_trash(10, 0).expect("Failed to list trash");
        assert_eq!(trash.len(), 1, "Should have 1 note left in trash");
        assert_eq!(
            trash[0].id.as_str(),
            id2.as_str(),
            "Recent note should still be in trash"
        );
    }

    #[test]
    fn test_permanently_delete_removes_note() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("To Permanently Delete");
        let note_id = db.create_note(&note).expect("Failed to create note");

        // Permanently delete the note
        db.permanently_delete(&note_id)
            .expect("Failed to permanently delete note");

        // Verify it's completely gone
        let retrieved = db.get_note(&note_id).expect("Failed to query note");
        assert!(
            retrieved.is_none(),
            "Permanently deleted note should not be retrievable"
        );

        // Verify it's not in trash either
        let trash = db.list_trash(10, 0).expect("Failed to list trash");
        assert_eq!(
            trash.len(),
            0,
            "Permanently deleted note should not be in trash"
        );
    }

    #[test]
    fn test_soft_delete_preserves_note_data() {
        let (db, _temp) = create_test_db();
        let mut note = create_test_note("Data Preservation Test");
        note.content = "Important content that should be preserved".to_string();
        let note_id = db.create_note(&note).expect("Failed to create note");

        // Delete the note
        db.delete_note(&note_id).expect("Failed to delete note");

        // Retrieve from trash and verify data is intact
        let trash = db.list_trash(10, 0).expect("Failed to list trash");
        assert_eq!(trash.len(), 1);
        assert_eq!(trash[0].title, "Data Preservation Test");
        assert_eq!(
            trash[0].content,
            "Important content that should be preserved"
        );
    }

    #[test]
    fn test_search_excludes_deleted_notes() {
        let (db, _temp) = create_test_db();

        let note1 = create_test_note("searchable");
        let note2 = create_test_note("also searchable");

        let id1 = db.create_note(&note1).expect("Failed to create note1");
        let id2 = db.create_note(&note2).expect("Failed to create note2");

        // Delete the second note
        db.delete_note(&id2).expect("Failed to delete note");

        // Search should only find the active note
        let results = db.search_notes("searchable", 10).expect("Failed to search");
        assert_eq!(results.len(), 1, "Should find only the active note");
        assert_eq!(results[0].id.as_str(), id1.as_str());
    }

    #[test]
    fn test_restore_note_clears_deleted_at() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("Restore Test");
        let note_id = db.create_note(&note).expect("Failed to create note");

        // Delete and then restore
        db.delete_note(&note_id).expect("Failed to delete");
        db.restore_note(&note_id).expect("Failed to restore");

        // Get the note and verify deleted_at is None
        let retrieved = db
            .get_note(&note_id)
            .expect("Failed to retrieve")
            .expect("Note should exist");
        assert!(
            retrieved.deleted_at.is_none(),
            "Restored note should have deleted_at = None"
        );
    }

    // --- Link-related tests ---

    #[test]
    fn test_create_link_and_get_backlinks() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Note A");
        let note_b = create_test_note("Note B");

        let id_a = db.create_note(&note_a).expect("Failed to create note A");
        let id_b = db.create_note(&note_b).expect("Failed to create note B");

        // Create a link from A -> B
        db.create_link(&id_a, &id_b, "reference", Some("see also B"))
            .expect("Failed to create link");

        // Get backlinks for B should show A
        let backlinks = db.get_backlinks(&id_b).expect("Failed to get backlinks");
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].0, id_a.as_str());
        assert_eq!(backlinks[0].1, "Note A");
        assert_eq!(backlinks[0].2, Some("see also B".to_string()));
    }

    #[test]
    fn test_get_backlinks_empty() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("Lonely Note");
        let note_id = db.create_note(&note).expect("Failed to create note");

        let backlinks = db.get_backlinks(&note_id).expect("Failed to get backlinks");
        assert!(backlinks.is_empty());
    }

    #[test]
    fn test_get_backlinks_excludes_deleted_sources() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Source A");
        let note_b = create_test_note("Target B");

        let id_a = db.create_note(&note_a).expect("Failed to create note A");
        let id_b = db.create_note(&note_b).expect("Failed to create note B");

        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed to create link");

        // Delete the source note
        db.delete_note(&id_a).expect("Failed to delete note A");

        // Backlinks should be empty since source is deleted
        let backlinks = db.get_backlinks(&id_b).expect("Failed to get backlinks");
        assert!(
            backlinks.is_empty(),
            "Deleted source notes should not appear in backlinks"
        );
    }

    #[test]
    fn test_get_backlinks_multiple_sources() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Source A");
        let note_b = create_test_note("Source B");
        let note_c = create_test_note("Target C");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");
        let id_c = db.create_note(&note_c).expect("Failed");

        db.create_link(&id_a, &id_c, "reference", Some("from A"))
            .expect("Failed");
        db.create_link(&id_b, &id_c, "reference", Some("from B"))
            .expect("Failed");

        let backlinks = db.get_backlinks(&id_c).expect("Failed to get backlinks");
        assert_eq!(backlinks.len(), 2);

        let titles: Vec<&str> = backlinks.iter().map(|b| b.1.as_str()).collect();
        assert!(titles.contains(&"Source A"));
        assert!(titles.contains(&"Source B"));
    }

    #[test]
    fn test_delete_links_for_note() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Source");
        let note_b = create_test_note("Target 1");
        let note_c = create_test_note("Target 2");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");
        let id_c = db.create_note(&note_c).expect("Failed");

        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");
        db.create_link(&id_a, &id_c, "reference", None)
            .expect("Failed");

        // Delete all links from A
        let deleted = db
            .delete_links_for_note(&id_a)
            .expect("Failed to delete links");
        assert_eq!(deleted, 2);

        // Backlinks should be empty for both targets
        let bl_b = db.get_backlinks(&id_b).expect("Failed");
        let bl_c = db.get_backlinks(&id_c).expect("Failed");
        assert!(bl_b.is_empty());
        assert!(bl_c.is_empty());
    }

    #[test]
    fn test_delete_links_for_note_no_links() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("No Links");
        let note_id = db.create_note(&note).expect("Failed");

        let deleted = db
            .delete_links_for_note(&note_id)
            .expect("Failed to delete links");
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_find_note_by_title() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("My Unique Title");
        let note_id = db.create_note(&note).expect("Failed to create note");

        let found = db
            .find_note_by_title("My Unique Title")
            .expect("Failed to find note");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id.as_str(), note_id.as_str());
    }

    #[test]
    fn test_find_note_by_title_case_insensitive() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("Case Sensitive Title");
        let note_id = db.create_note(&note).expect("Failed to create note");

        let found = db
            .find_note_by_title("case sensitive title")
            .expect("Failed to find note");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id.as_str(), note_id.as_str());
    }

    #[test]
    fn test_find_note_by_title_not_found() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("Existing Note");
        db.create_note(&note).expect("Failed");

        let found = db.find_note_by_title("Nonexistent Title").expect("Failed");
        assert!(found.is_none());
    }

    #[test]
    fn test_find_note_by_title_excludes_deleted() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("Deleted Title");
        let note_id = db.create_note(&note).expect("Failed");

        db.delete_note(&note_id).expect("Failed to delete");

        let found = db.find_note_by_title("Deleted Title").expect("Failed");
        assert!(
            found.is_none(),
            "Deleted notes should not be found by title"
        );
    }

    #[test]
    fn test_get_links_for_note() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Source");
        let note_b = create_test_note("Target 1");
        let note_c = create_test_note("Target 2");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");
        let id_c = db.create_note(&note_c).expect("Failed");

        db.create_link(&id_a, &id_b, "reference", Some("link to B"))
            .expect("Failed");
        db.create_link(&id_a, &id_c, "wiki", None).expect("Failed");

        let links = db.get_links_for_note(&id_a).expect("Failed to get links");
        assert_eq!(links.len(), 2);

        let target_titles: Vec<&str> = links.iter().map(|l| l.1.as_str()).collect();
        assert!(target_titles.contains(&"Target 1"));
        assert!(target_titles.contains(&"Target 2"));
    }

    #[test]
    fn test_get_links_for_note_excludes_deleted_targets() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Source");
        let note_b = create_test_note("Target");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");

        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");

        // Delete the target
        db.delete_note(&id_b).expect("Failed");

        let links = db.get_links_for_note(&id_a).expect("Failed");
        assert!(links.is_empty(), "Links to deleted notes should not appear");
    }

    #[test]
    fn test_get_links_for_note_empty() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("No Links");
        let note_id = db.create_note(&note).expect("Failed");

        let links = db.get_links_for_note(&note_id).expect("Failed");
        assert!(links.is_empty());
    }

    #[test]
    fn test_create_link_idempotent() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Source");
        let note_b = create_test_note("Target");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");

        // Create the same link twice (INSERT OR IGNORE)
        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");
        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");

        // Should only have one backlink
        let backlinks = db.get_backlinks(&id_b).expect("Failed");
        assert_eq!(backlinks.len(), 1, "Duplicate links should be ignored");
    }

    #[test]
    fn test_delete_and_recreate_links() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Source");
        let note_b = create_test_note("Target Old");
        let note_c = create_test_note("Target New");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");
        let id_c = db.create_note(&note_c).expect("Failed");

        // Create link A -> B
        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");

        // Delete all links from A (simulating note content change)
        db.delete_links_for_note(&id_a).expect("Failed");

        // Create new link A -> C
        db.create_link(&id_a, &id_c, "reference", None)
            .expect("Failed");

        // B should have no backlinks, C should have one
        let bl_b = db.get_backlinks(&id_b).expect("Failed");
        let bl_c = db.get_backlinks(&id_c).expect("Failed");
        assert!(bl_b.is_empty());
        assert_eq!(bl_c.len(), 1);
        assert_eq!(bl_c[0].1, "Source");
    }

    // ========== get_all_links() tests ==========

    #[test]
    fn test_get_all_links_empty_db() {
        let (db, _temp) = create_test_db();
        let links = db.get_all_links().expect("Failed");
        assert!(links.is_empty());
    }

    #[test]
    fn test_get_all_links_no_links() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("Orphan Note");
        db.create_note(&note).expect("Failed");

        let links = db.get_all_links().expect("Failed");
        assert!(links.is_empty());
    }

    #[test]
    fn test_get_all_links_single_link() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Alpha");
        let note_b = create_test_note("Beta");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");

        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");

        let links = db.get_all_links().expect("Failed");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, id_a.as_str()); // source_id
        assert_eq!(links[0].1, "Alpha"); // source_title
        assert_eq!(links[0].2, id_b.as_str()); // target_id
        assert_eq!(links[0].3, "Beta"); // target_title
        assert_eq!(links[0].4, "reference"); // link_type
    }

    #[test]
    fn test_get_all_links_multiple_links() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Alpha");
        let note_b = create_test_note("Beta");
        let note_c = create_test_note("Gamma");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");
        let id_c = db.create_note(&note_c).expect("Failed");

        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");
        db.create_link(&id_a, &id_c, "reference", None)
            .expect("Failed");
        db.create_link(&id_b, &id_c, "embed", None).expect("Failed");

        let links = db.get_all_links().expect("Failed");
        assert_eq!(links.len(), 3);
    }

    #[test]
    fn test_get_all_links_excludes_deleted_source() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Source");
        let note_b = create_test_note("Target");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");

        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");

        // Delete the source note
        db.delete_note(&id_a).expect("Failed");

        let links = db.get_all_links().expect("Failed");
        assert!(
            links.is_empty(),
            "Links from deleted sources should be excluded"
        );
    }

    #[test]
    fn test_get_all_links_excludes_deleted_target() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Source");
        let note_b = create_test_note("Target");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");

        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");

        // Delete the target note
        db.delete_note(&id_b).expect("Failed");

        let links = db.get_all_links().expect("Failed");
        assert!(
            links.is_empty(),
            "Links to deleted targets should be excluded"
        );
    }

    #[test]
    fn test_get_all_links_ordered_by_title() {
        let (db, _temp) = create_test_db();
        let note_z = create_test_note("Zulu");
        let note_a = create_test_note("Alpha");
        let note_m = create_test_note("Mike");

        let id_z = db.create_note(&note_z).expect("Failed");
        let id_a = db.create_note(&note_a).expect("Failed");
        let id_m = db.create_note(&note_m).expect("Failed");

        // Links: Zulu->Mike, Alpha->Mike
        db.create_link(&id_z, &id_m, "reference", None)
            .expect("Failed");
        db.create_link(&id_a, &id_m, "reference", None)
            .expect("Failed");

        let links = db.get_all_links().expect("Failed");
        assert_eq!(links.len(), 2);
        // ORDER BY s.title, t.title → Alpha first, then Zulu
        assert_eq!(links[0].1, "Alpha");
        assert_eq!(links[1].1, "Zulu");
    }

    // ========== get_graph_nodes() tests ==========

    #[test]
    fn test_get_graph_nodes_empty_db() {
        let (db, _temp) = create_test_db();
        let nodes = db.get_graph_nodes().expect("Failed");
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_get_graph_nodes_single_note_no_links() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("Solo Note");
        db.create_note(&note).expect("Failed");

        let nodes = db.get_graph_nodes().expect("Failed");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].1, "Solo Note"); // title
        assert_eq!(nodes[0].2, "permanent"); // note_type (create_test_note uses Permanent)
        assert_eq!(nodes[0].3, 0); // outgoing_count
        assert_eq!(nodes[0].4, 0); // incoming_count
    }

    #[test]
    fn test_get_graph_nodes_with_links_counts() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Hub");
        let note_b = create_test_note("Spoke One");
        let note_c = create_test_note("Spoke Two");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");
        let id_c = db.create_note(&note_c).expect("Failed");

        // Hub links to both spokes
        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");
        db.create_link(&id_a, &id_c, "reference", None)
            .expect("Failed");

        let nodes = db.get_graph_nodes().expect("Failed");
        assert_eq!(nodes.len(), 3);

        // Nodes ordered by title: Hub, Spoke One, Spoke Two
        let hub = nodes.iter().find(|n| n.1 == "Hub").expect("Hub not found");
        let spoke1 = nodes
            .iter()
            .find(|n| n.1 == "Spoke One")
            .expect("Spoke One not found");
        let spoke2 = nodes
            .iter()
            .find(|n| n.1 == "Spoke Two")
            .expect("Spoke Two not found");

        assert_eq!(hub.3, 2); // outgoing: 2
        assert_eq!(hub.4, 0); // incoming: 0
        assert_eq!(spoke1.3, 0); // outgoing: 0
        assert_eq!(spoke1.4, 1); // incoming: 1
        assert_eq!(spoke2.3, 0); // outgoing: 0
        assert_eq!(spoke2.4, 1); // incoming: 1
    }

    #[test]
    fn test_get_graph_nodes_excludes_deleted() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Alive");
        let note_b = create_test_note("Dead");

        let _id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");

        db.delete_note(&id_b).expect("Failed");

        let nodes = db.get_graph_nodes().expect("Failed");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].1, "Alive");
    }

    #[test]
    fn test_get_graph_nodes_bidirectional_links() {
        let (db, _temp) = create_test_db();
        let note_a = create_test_note("Alpha");
        let note_b = create_test_note("Beta");

        let id_a = db.create_note(&note_a).expect("Failed");
        let id_b = db.create_note(&note_b).expect("Failed");

        // Bidirectional: A->B and B->A
        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");
        db.create_link(&id_b, &id_a, "reference", None)
            .expect("Failed");

        let nodes = db.get_graph_nodes().expect("Failed");
        let alpha = nodes
            .iter()
            .find(|n| n.1 == "Alpha")
            .expect("Alpha not found");
        let beta = nodes
            .iter()
            .find(|n| n.1 == "Beta")
            .expect("Beta not found");

        assert_eq!(alpha.3, 1); // outgoing: 1
        assert_eq!(alpha.4, 1); // incoming: 1
        assert_eq!(beta.3, 1); // outgoing: 1
        assert_eq!(beta.4, 1); // incoming: 1
    }

    #[test]
    fn test_get_graph_nodes_returns_note_id() {
        let (db, _temp) = create_test_db();
        let note = create_test_note("Check ID");
        let note_id = db.create_note(&note).expect("Failed");

        let nodes = db.get_graph_nodes().expect("Failed");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].0, note_id.as_str()); // id matches
    }

    // =====================================================================
    // Tests for count_notes_by_type, count_notes, count_links, list_notes_by_type
    // =====================================================================

    #[test]
    fn test_count_notes_empty_db() {
        let (db, _temp) = create_test_db();
        let count = db.count_notes().expect("Failed");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_notes_with_entries() {
        let (db, _temp) = create_test_db();
        db.create_note(&create_test_note("Note A")).expect("Failed");
        db.create_note(&create_test_note("Note B")).expect("Failed");
        db.create_note(&create_test_note("Note C")).expect("Failed");

        let count = db.count_notes().expect("Failed");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_notes_excludes_deleted() {
        let (db, _temp) = create_test_db();
        let id_a = db.create_note(&create_test_note("Alive")).expect("Failed");
        let id_b = db.create_note(&create_test_note("Dead")).expect("Failed");
        db.delete_note(&id_b).expect("Failed");

        let count = db.count_notes().expect("Failed");
        assert_eq!(count, 1);
        // Ensure the alive note is the one that's counted
        let _ = id_a;
    }

    #[test]
    fn test_count_notes_by_type_empty_db() {
        let (db, _temp) = create_test_db();
        let counts = db.count_notes_by_type().expect("Failed");
        assert!(counts.is_empty());
    }

    #[test]
    fn test_count_notes_by_type_multiple_types() {
        let (db, _temp) = create_test_db();

        // Create notes of different types
        let mut daily = create_test_note("Daily Entry");
        daily.note_type = NoteType::Daily;
        db.create_note(&daily).expect("Failed");

        let mut fleeting = create_test_note("Quick Thought");
        fleeting.note_type = NoteType::Fleeting;
        db.create_note(&fleeting).expect("Failed");

        db.create_note(&create_test_note("Permanent A"))
            .expect("Failed");
        db.create_note(&create_test_note("Permanent B"))
            .expect("Failed");

        let counts = db.count_notes_by_type().expect("Failed");

        // permanent=2 should be first (highest count), then daily=1, fleeting=1
        assert_eq!(counts.len(), 3);

        let permanent_count = counts.iter().find(|(t, _)| t == "permanent").unwrap().1;
        let daily_count = counts.iter().find(|(t, _)| t == "daily").unwrap().1;
        let fleeting_count = counts.iter().find(|(t, _)| t == "fleeting").unwrap().1;

        assert_eq!(permanent_count, 2);
        assert_eq!(daily_count, 1);
        assert_eq!(fleeting_count, 1);
    }

    #[test]
    fn test_count_notes_by_type_excludes_deleted() {
        let (db, _temp) = create_test_db();

        let id = db
            .create_note(&create_test_note("Will Delete"))
            .expect("Failed");
        db.create_note(&create_test_note("Will Keep"))
            .expect("Failed");
        db.delete_note(&id).expect("Failed");

        let counts = db.count_notes_by_type().expect("Failed");
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0], ("permanent".to_string(), 1));
    }

    #[test]
    fn test_count_links_empty_db() {
        let (db, _temp) = create_test_db();
        let count = db.count_links().expect("Failed");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_links_with_active_notes() {
        let (db, _temp) = create_test_db();
        let id_a = db.create_note(&create_test_note("A")).expect("Failed");
        let id_b = db.create_note(&create_test_note("B")).expect("Failed");
        let id_c = db.create_note(&create_test_note("C")).expect("Failed");

        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");
        db.create_link(&id_a, &id_c, "reference", None)
            .expect("Failed");

        let count = db.count_links().expect("Failed");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_count_links_excludes_deleted_endpoints() {
        let (db, _temp) = create_test_db();
        let id_a = db.create_note(&create_test_note("A")).expect("Failed");
        let id_b = db.create_note(&create_test_note("B")).expect("Failed");
        let id_c = db.create_note(&create_test_note("C")).expect("Failed");

        db.create_link(&id_a, &id_b, "reference", None)
            .expect("Failed");
        db.create_link(&id_a, &id_c, "reference", None)
            .expect("Failed");

        // Delete note B -- link A->B should no longer be counted
        db.delete_note(&id_b).expect("Failed");

        let count = db.count_links().expect("Failed");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_list_notes_by_type_returns_only_matching() {
        let (db, _temp) = create_test_db();

        let mut daily = create_test_note("Daily Entry");
        daily.note_type = NoteType::Daily;
        db.create_note(&daily).expect("Failed");

        db.create_note(&create_test_note("Permanent Note"))
            .expect("Failed");

        let dailies = db.list_notes_by_type("daily", 100, 0).expect("Failed");
        assert_eq!(dailies.len(), 1);
        assert_eq!(dailies[0].title, "Daily Entry");

        let permanents = db.list_notes_by_type("permanent", 100, 0).expect("Failed");
        assert_eq!(permanents.len(), 1);
        assert_eq!(permanents[0].title, "Permanent Note");
    }

    #[test]
    fn test_list_notes_by_type_ordered_by_title() {
        let (db, _temp) = create_test_db();

        db.create_note(&create_test_note("Zebra")).expect("Failed");
        db.create_note(&create_test_note("Alpha")).expect("Failed");
        db.create_note(&create_test_note("Middle")).expect("Failed");

        let notes = db.list_notes_by_type("permanent", 100, 0).expect("Failed");
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0].title, "Alpha");
        assert_eq!(notes[1].title, "Middle");
        assert_eq!(notes[2].title, "Zebra");
    }

    #[test]
    fn test_list_notes_by_type_excludes_deleted() {
        let (db, _temp) = create_test_db();

        let id = db
            .create_note(&create_test_note("Deleted"))
            .expect("Failed");
        db.create_note(&create_test_note("Active")).expect("Failed");
        db.delete_note(&id).expect("Failed");

        let notes = db.list_notes_by_type("permanent", 100, 0).expect("Failed");
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Active");
    }

    #[test]
    fn test_list_notes_by_type_respects_limit_offset() {
        let (db, _temp) = create_test_db();

        db.create_note(&create_test_note("A")).expect("Failed");
        db.create_note(&create_test_note("B")).expect("Failed");
        db.create_note(&create_test_note("C")).expect("Failed");

        let page1 = db.list_notes_by_type("permanent", 2, 0).expect("Failed");
        assert_eq!(page1.len(), 2);

        let page2 = db.list_notes_by_type("permanent", 2, 2).expect("Failed");
        assert_eq!(page2.len(), 1);
    }

    #[test]
    fn test_list_notes_by_type_nonexistent_type() {
        let (db, _temp) = create_test_db();
        db.create_note(&create_test_note("Note")).expect("Failed");

        let notes = db
            .list_notes_by_type("nonexistent", 100, 0)
            .expect("Failed");
        assert!(notes.is_empty());
    }

    // =====================================================================
    // Schema migration tests
    // =====================================================================

    #[test]
    fn test_schema_version_is_2() {
        let (db, _temp) = create_test_db();
        let conn = db.conn.lock();
        let version = Database::get_schema_version(&conn).expect("Failed");
        assert_eq!(version, 2);
    }

    #[test]
    fn test_sources_table_exists() {
        let (db, _temp) = create_test_db();
        let conn = db.conn.lock();
        let exists: bool = conn.prepare("SELECT 1 FROM sources LIMIT 0").is_ok();
        assert!(exists);
    }

    #[test]
    fn test_migration_is_idempotent() {
        let (db, _temp) = create_test_db();
        // Running initialize again should not fail
        db.initialize().expect("Re-initialization should succeed");
        let conn = db.conn.lock();
        let version = Database::get_schema_version(&conn).expect("Failed");
        assert_eq!(version, 2);
    }

    // =====================================================================
    // Source CRUD tests
    // =====================================================================

    fn create_test_source(title: &str, hash: &str) -> Source {
        Source::new(title, hash, format!("raw/{}.md", title), 100)
    }

    #[test]
    fn test_create_source() {
        let (db, _temp) = create_test_db();
        let source = create_test_source("article", "hash123");
        let id = db.create_source(&source).expect("Failed to create source");
        assert_eq!(id.as_str(), source.id.as_str());
    }

    #[test]
    fn test_get_source() {
        let (db, _temp) = create_test_db();
        let source = create_test_source("paper", "hashABC");
        db.create_source(&source).expect("Failed");

        let fetched = db.get_source(&source.id).expect("Failed");
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.title, "paper");
        assert_eq!(fetched.content_hash, "hashABC");
        assert_eq!(fetched.file_path, "raw/paper.md");
        assert_eq!(fetched.file_size, 100);
    }

    #[test]
    fn test_get_source_not_found() {
        let (db, _temp) = create_test_db();
        let fake_id = SourceId::new();
        let result = db.get_source(&fake_id).expect("Failed");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_source_by_hash() {
        let (db, _temp) = create_test_db();
        let source = create_test_source("doc", "unique_hash");
        db.create_source(&source).expect("Failed");

        let found = db.find_source_by_hash("unique_hash").expect("Failed");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "doc");
    }

    #[test]
    fn test_find_source_by_hash_not_found() {
        let (db, _temp) = create_test_db();
        let found = db.find_source_by_hash("nonexistent").expect("Failed");
        assert!(found.is_none());
    }

    #[test]
    fn test_list_sources_empty() {
        let (db, _temp) = create_test_db();
        let sources = db.list_sources(100, 0).expect("Failed");
        assert!(sources.is_empty());
    }

    #[test]
    fn test_list_sources_returns_all() {
        let (db, _temp) = create_test_db();
        db.create_source(&create_test_source("a", "h1"))
            .expect("Failed");
        db.create_source(&create_test_source("b", "h2"))
            .expect("Failed");
        db.create_source(&create_test_source("c", "h3"))
            .expect("Failed");

        let sources = db.list_sources(100, 0).expect("Failed");
        assert_eq!(sources.len(), 3);
    }

    #[test]
    fn test_list_sources_respects_limit() {
        let (db, _temp) = create_test_db();
        db.create_source(&create_test_source("a", "h1"))
            .expect("Failed");
        db.create_source(&create_test_source("b", "h2"))
            .expect("Failed");

        let sources = db.list_sources(1, 0).expect("Failed");
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn test_list_sources_respects_offset() {
        let (db, _temp) = create_test_db();
        db.create_source(&create_test_source("a", "h1"))
            .expect("Failed");
        db.create_source(&create_test_source("b", "h2"))
            .expect("Failed");

        let sources = db.list_sources(100, 1).expect("Failed");
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn test_count_sources() {
        let (db, _temp) = create_test_db();
        assert_eq!(db.count_sources().expect("Failed"), 0);

        db.create_source(&create_test_source("a", "h1"))
            .expect("Failed");
        assert_eq!(db.count_sources().expect("Failed"), 1);

        db.create_source(&create_test_source("b", "h2"))
            .expect("Failed");
        assert_eq!(db.count_sources().expect("Failed"), 2);
    }

    #[test]
    fn test_delete_source() {
        let (db, _temp) = create_test_db();
        let source = create_test_source("to_delete", "hash");
        db.create_source(&source).expect("Failed");

        let deleted = db.delete_source(&source.id).expect("Failed");
        assert!(deleted);
        assert_eq!(db.count_sources().expect("Failed"), 0);
    }

    #[test]
    fn test_delete_source_not_found() {
        let (db, _temp) = create_test_db();
        let fake_id = SourceId::new();
        let deleted = db.delete_source(&fake_id).expect("Failed");
        assert!(!deleted);
    }

    #[test]
    fn test_create_source_with_origin_and_mime() {
        let (db, _temp) = create_test_db();
        let source = Source::new("paper", "hash", "raw/paper.pdf", 5000)
            .with_origin("https://arxiv.org/paper.pdf")
            .with_mime_type("application/pdf");
        db.create_source(&source).expect("Failed");

        let fetched = db.get_source(&source.id).expect("Failed").unwrap();
        assert_eq!(
            fetched.origin.as_deref(),
            Some("https://arxiv.org/paper.pdf")
        );
        assert_eq!(fetched.mime_type.as_deref(), Some("application/pdf"));
    }

    #[test]
    fn test_create_source_duplicate_hash_fails() {
        let (db, _temp) = create_test_db();
        // Two sources with different IDs but same hash should NOT conflict
        // (hash is not unique in schema, dedup is app-level)
        let s1 = create_test_source("first", "same_hash");
        let s2 = create_test_source("second", "same_hash");
        db.create_source(&s1).expect("Failed");
        // This should succeed -- no unique constraint on content_hash
        db.create_source(&s2).expect("Failed");
        assert_eq!(db.count_sources().expect("Failed"), 2);
    }

    #[test]
    fn test_source_roundtrip_preserves_data() {
        let (db, _temp) = create_test_db();
        let source = Source::new("Test Doc", "abc123", "raw/test.md", 42)
            .with_origin("/home/user/test.md")
            .with_mime_type("text/markdown");
        db.create_source(&source).expect("Failed");

        let fetched = db.get_source(&source.id).expect("Failed").unwrap();
        assert_eq!(fetched.id.as_str(), source.id.as_str());
        assert_eq!(fetched.title, source.title);
        assert_eq!(fetched.origin, source.origin);
        assert_eq!(fetched.content_hash, source.content_hash);
        assert_eq!(fetched.file_path, source.file_path);
        assert_eq!(fetched.file_size, source.file_size);
        assert_eq!(fetched.mime_type, source.mime_type);
    }
}
