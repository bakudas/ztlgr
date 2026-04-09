/// MCP tool definitions and handlers for grimoire operations.
///
/// Each tool maps to a grimoire operation exposed via the MCP protocol.
/// Tools are defined with JSON Schema input validation and return
/// structured text content.
use std::path::Path;

use crate::db::Database;
use crate::error::Result;
use crate::note::{Note, NoteType};
use crate::source::ingest::Ingester;
use crate::storage::{ActivityLog, Format, IndexGenerator};

use super::{ToolCallResult, ToolDefinition};

/// Returns all tool definitions for the grimoire MCP server.
pub fn all_tools() -> Vec<ToolDefinition> {
    vec![
        tool_search(),
        tool_get_note(),
        tool_list_notes(),
        tool_create_note(),
        tool_get_backlinks(),
        tool_ingest_source(),
        tool_read_index(),
        tool_read_log(),
        tool_read_skills(),
    ]
}

/// Dispatches a tool call to the appropriate handler.
pub fn handle_tool_call(
    name: &str,
    arguments: &serde_json::Value,
    vault_path: &Path,
    db: &Database,
    format: Format,
) -> ToolCallResult {
    match name {
        "search" => handle_search(arguments, db),
        "get_note" => handle_get_note(arguments, db),
        "list_notes" => handle_list_notes(arguments, db),
        "create_note" => handle_create_note(arguments, vault_path, db, format),
        "get_backlinks" => handle_get_backlinks(arguments, db),
        "ingest_source" => handle_ingest_source(arguments, vault_path, db),
        "read_index" => handle_read_index(vault_path),
        "read_log" => handle_read_log(vault_path),
        "read_skills" => handle_read_skills(arguments, vault_path),
        _ => ToolCallResult::error(format!("Unknown tool: {}", name)),
    }
}

// --- Tool definitions ---

fn tool_search() -> ToolDefinition {
    ToolDefinition {
        name: "search".to_string(),
        description: "Search notes in the grimoire using full-text search (FTS5/BM25)".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query (supports FTS5 syntax)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 20)",
                    "default": 20
                }
            },
            "required": ["query"]
        }),
    }
}

fn tool_get_note() -> ToolDefinition {
    ToolDefinition {
        name: "get_note".to_string(),
        description: "Get a note by its ID or title. Returns full content with metadata."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Note UUID"
                },
                "title": {
                    "type": "string",
                    "description": "Note title (case-insensitive lookup)"
                }
            }
        }),
    }
}

fn tool_list_notes() -> ToolDefinition {
    ToolDefinition {
        name: "list_notes".to_string(),
        description: "List notes in the grimoire, optionally filtered by type.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "note_type": {
                    "type": "string",
                    "description": "Filter by note type: permanent, fleeting, literature, daily, reference, index",
                    "enum": ["permanent", "fleeting", "literature", "daily", "reference", "index"]
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 50)",
                    "default": 50
                },
                "offset": {
                    "type": "integer",
                    "description": "Pagination offset (default: 0)",
                    "default": 0
                }
            }
        }),
    }
}

fn tool_create_note() -> ToolDefinition {
    ToolDefinition {
        name: "create_note".to_string(),
        description: "Create a new note in the grimoire. The note is saved to both the database and as a file.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Note title"
                },
                "content": {
                    "type": "string",
                    "description": "Note content (markdown)"
                },
                "note_type": {
                    "type": "string",
                    "description": "Note type (default: permanent)",
                    "enum": ["permanent", "fleeting", "literature", "daily", "reference", "index"],
                    "default": "permanent"
                }
            },
            "required": ["title", "content"]
        }),
    }
}

fn tool_get_backlinks() -> ToolDefinition {
    ToolDefinition {
        name: "get_backlinks".to_string(),
        description: "Get all notes that link to a given note (backlinks).".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "note_id": {
                    "type": "string",
                    "description": "UUID of the note to get backlinks for"
                },
                "title": {
                    "type": "string",
                    "description": "Title of the note (alternative to note_id)"
                }
            }
        }),
    }
}

fn tool_ingest_source() -> ToolDefinition {
    ToolDefinition {
        name: "ingest_source".to_string(),
        description: "Ingest a source file into the grimoire's raw/ directory. Copies the file, registers in DB, and deduplicates by content hash.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the source file to ingest"
                },
                "title": {
                    "type": "string",
                    "description": "Title for the source (defaults to filename)"
                }
            },
            "required": ["file_path"]
        }),
    }
}

fn tool_read_index() -> ToolDefinition {
    ToolDefinition {
        name: "read_index".to_string(),
        description:
            "Read the grimoire's index.md file. Contains a catalog of all notes grouped by type."
                .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
    }
}

fn tool_read_log() -> ToolDefinition {
    ToolDefinition {
        name: "read_log".to_string(),
        description: "Read the grimoire's activity log (log.md). Contains chronological entries of all operations.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "tail": {
                    "type": "integer",
                    "description": "Number of lines from the end to return (default: all)"
                }
            }
        }),
    }
}

fn tool_read_skills() -> ToolDefinition {
    ToolDefinition {
        name: "read_skills".to_string(),
        description: "Read a file from the .skills/ directory. Contains LLM agent instructions, workflows, templates, and context.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "Relative path within .skills/ (e.g., 'README.md', 'workflows/ingest.md', 'templates/source-summary.md')"
                }
            },
            "required": ["file"]
        }),
    }
}

// --- Tool handlers ---

fn handle_search(arguments: &serde_json::Value, db: &Database) -> ToolCallResult {
    let query = match arguments.get("query").and_then(|v| v.as_str()) {
        Some(q) if !q.trim().is_empty() => q,
        _ => return ToolCallResult::error("Missing or empty 'query' parameter".to_string()),
    };
    let limit = arguments
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;

    match db.search_notes(query, limit) {
        Ok(notes) => {
            if notes.is_empty() {
                return ToolCallResult::text(format!("No results found for query: \"{}\"", query));
            }
            let mut output = format!("Found {} result(s) for \"{}\":\n\n", notes.len(), query);
            for note in &notes {
                output.push_str(&format!(
                    "- **{}** (id: {}, type: {})\n  {}\n\n",
                    note.title,
                    note.id,
                    note.note_type,
                    truncate_content(&note.content, 200),
                ));
            }
            ToolCallResult::text(output)
        }
        Err(e) => ToolCallResult::error(format!("Search failed: {}", e)),
    }
}

fn handle_get_note(arguments: &serde_json::Value, db: &Database) -> ToolCallResult {
    let note_opt = if let Some(id_str) = arguments.get("id").and_then(|v| v.as_str()) {
        match crate::note::NoteId::parse(id_str) {
            Ok(id) => match db.get_note(&id) {
                Ok(note) => note,
                Err(e) => return ToolCallResult::error(format!("Database error: {}", e)),
            },
            Err(_) => return ToolCallResult::error(format!("Invalid note ID: {}", id_str)),
        }
    } else if let Some(title) = arguments.get("title").and_then(|v| v.as_str()) {
        match db.find_note_by_title(title) {
            Ok(note) => note,
            Err(e) => return ToolCallResult::error(format!("Database error: {}", e)),
        }
    } else {
        return ToolCallResult::error("Either 'id' or 'title' parameter is required".to_string());
    };

    match note_opt {
        Some(note) => {
            let output = format!(
                "# {}\n\n**ID:** {}\n**Type:** {}\n**Created:** {}\n**Updated:** {}\n{}\n\n---\n\n{}",
                note.title,
                note.id,
                note.note_type,
                note.created_at.format("%Y-%m-%d %H:%M:%S"),
                note.updated_at.format("%Y-%m-%d %H:%M:%S"),
                note.source
                    .as_ref()
                    .map(|s| format!("**Source:** {}", s))
                    .unwrap_or_default(),
                note.content,
            );
            ToolCallResult::text(output)
        }
        None => ToolCallResult::text("Note not found.".to_string()),
    }
}

fn handle_list_notes(arguments: &serde_json::Value, db: &Database) -> ToolCallResult {
    let limit = arguments
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as usize;
    let offset = arguments
        .get("offset")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let result = if let Some(note_type) = arguments.get("note_type").and_then(|v| v.as_str()) {
        db.list_notes_by_type(note_type, limit, offset)
    } else {
        db.list_notes(limit, offset)
    };

    match result {
        Ok(notes) => {
            if notes.is_empty() {
                return ToolCallResult::text("No notes found.".to_string());
            }
            let mut output = format!("Listing {} note(s):\n\n", notes.len());
            for note in &notes {
                output.push_str(&format!(
                    "- **{}** (id: {}, type: {}, updated: {})\n",
                    note.title,
                    note.id,
                    note.note_type,
                    note.updated_at.format("%Y-%m-%d"),
                ));
            }
            ToolCallResult::text(output)
        }
        Err(e) => ToolCallResult::error(format!("List failed: {}", e)),
    }
}

fn handle_create_note(
    arguments: &serde_json::Value,
    vault_path: &Path,
    db: &Database,
    format: Format,
) -> ToolCallResult {
    let title = match arguments.get("title").and_then(|v| v.as_str()) {
        Some(t) if !t.trim().is_empty() => t,
        _ => return ToolCallResult::error("Missing or empty 'title' parameter".to_string()),
    };
    let content = match arguments.get("content").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return ToolCallResult::error("Missing 'content' parameter".to_string()),
    };
    let note_type_str = arguments
        .get("note_type")
        .and_then(|v| v.as_str())
        .unwrap_or("permanent");

    let note_type: NoteType = match note_type_str.parse() {
        Ok(nt) => nt,
        Err(_) => return ToolCallResult::error(format!("Invalid note type: {}", note_type_str)),
    };

    let mut note = Note::new(title.to_string(), content.to_string());
    note.note_type = note_type.clone();

    // Save to DB
    let note_id = match db.create_note(&note) {
        Ok(id) => id,
        Err(e) => return ToolCallResult::error(format!("Failed to create note in DB: {}", e)),
    };

    // Write to file
    let folder = folder_for_note_type(&note_type);
    let filename = format!("{}.{}", sanitize_filename(&note.title), format.extension());
    let file_path = vault_path.join(folder).join(&filename);

    let storage: Box<dyn crate::storage::Storage> = match format {
        Format::Markdown => Box::new(crate::storage::MarkdownStorage::new()),
        Format::Org => Box::new(crate::storage::OrgStorage::new()),
    };

    if let Err(e) = storage.write_note(&note, &file_path) {
        return ToolCallResult::error(format!(
            "Note saved to DB (id: {}) but file write failed: {}",
            note_id, e
        ));
    }

    // Log activity
    let log_path = vault_path.join(".ztlgr").join("log.md");
    let log = ActivityLog::new(&log_path);
    let _ = log.log_create(&note.title, &note.note_type.to_string());

    // Regenerate index
    let _ = regenerate_index(vault_path, db);

    ToolCallResult::text(format!(
        "Note created successfully.\n\n**ID:** {}\n**Title:** {}\n**Type:** {}\n**File:** {}",
        note_id,
        note.title,
        note.note_type,
        file_path.display(),
    ))
}

fn handle_get_backlinks(arguments: &serde_json::Value, db: &Database) -> ToolCallResult {
    // Resolve note ID from either `note_id` or `title`
    let note_id = if let Some(id_str) = arguments.get("note_id").and_then(|v| v.as_str()) {
        match crate::note::NoteId::parse(id_str) {
            Ok(id) => id,
            Err(_) => return ToolCallResult::error(format!("Invalid note ID: {}", id_str)),
        }
    } else if let Some(title) = arguments.get("title").and_then(|v| v.as_str()) {
        match db.find_note_by_title(title) {
            Ok(Some(note)) => note.id,
            Ok(None) => {
                return ToolCallResult::text(format!("Note not found with title: \"{}\"", title))
            }
            Err(e) => return ToolCallResult::error(format!("Database error: {}", e)),
        }
    } else {
        return ToolCallResult::error(
            "Either 'note_id' or 'title' parameter is required".to_string(),
        );
    };

    match db.get_backlinks(&note_id) {
        Ok(backlinks) => {
            if backlinks.is_empty() {
                return ToolCallResult::text("No backlinks found for this note.".to_string());
            }
            let mut output = format!("Found {} backlink(s):\n\n", backlinks.len());
            for (source_id, source_title, context) in &backlinks {
                output.push_str(&format!("- **{}** (id: {})", source_title, source_id));
                if let Some(ctx) = context {
                    output.push_str(&format!("\n  Context: {}", ctx));
                }
                output.push('\n');
            }
            ToolCallResult::text(output)
        }
        Err(e) => ToolCallResult::error(format!("Failed to get backlinks: {}", e)),
    }
}

fn handle_ingest_source(
    arguments: &serde_json::Value,
    vault_path: &Path,
    _db: &Database,
) -> ToolCallResult {
    let file_path = match arguments.get("file_path").and_then(|v| v.as_str()) {
        Some(p) => Path::new(p),
        None => return ToolCallResult::error("Missing 'file_path' parameter".to_string()),
    };
    let title = arguments.get("title").and_then(|v| v.as_str());

    if !file_path.exists() {
        return ToolCallResult::error(format!("File not found: {}", file_path.display()));
    }

    // Ingester takes ownership of Database, so create a new connection
    let db_path = vault_path.join(".ztlgr").join("vault.db");
    let ingest_db = match Database::new(&db_path) {
        Ok(db) => db,
        Err(e) => return ToolCallResult::error(format!("Database error: {}", e)),
    };

    let ingester = Ingester::new(vault_path.to_path_buf(), ingest_db);
    match ingester.ingest_file(file_path, title) {
        Ok(result) => {
            let status = if result.is_new {
                "ingested (new)"
            } else {
                "already exists (deduplicated)"
            };
            ToolCallResult::text(format!(
                "Source {}\n\n**Title:** {}\n**ID:** {}\n**Hash:** {}\n**File:** {}",
                status,
                result.source.title,
                result.source.id,
                result.source.content_hash,
                result.source.file_path,
            ))
        }
        Err(e) => ToolCallResult::error(format!("Ingest failed: {}", e)),
    }
}

fn handle_read_index(vault_path: &Path) -> ToolCallResult {
    let index_path = vault_path.join(".ztlgr").join("index.md");
    match std::fs::read_to_string(&index_path) {
        Ok(content) => ToolCallResult::text(content),
        Err(_) => ToolCallResult::text(
            "index.md not found. Run `ztlgr index` or `ztlgr sync --force` to generate it."
                .to_string(),
        ),
    }
}

fn handle_read_log(vault_path: &Path) -> ToolCallResult {
    let log_path = vault_path.join(".ztlgr").join("log.md");
    match std::fs::read_to_string(&log_path) {
        Ok(content) => {
            let tail = content.len(); // TODO: implement tail parameter
            if tail == 0 {
                return ToolCallResult::text("Activity log is empty.".to_string());
            }
            ToolCallResult::text(content)
        }
        Err(_) => {
            ToolCallResult::text("log.md not found. No activity has been recorded yet.".to_string())
        }
    }
}

fn handle_read_skills(arguments: &serde_json::Value, vault_path: &Path) -> ToolCallResult {
    let file = match arguments.get("file").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return ToolCallResult::error("Missing 'file' parameter".to_string()),
    };

    // Prevent path traversal
    if file.contains("..") {
        return ToolCallResult::error("Path traversal not allowed".to_string());
    }

    let skills_path = vault_path.join(".skills").join(file);
    match std::fs::read_to_string(&skills_path) {
        Ok(content) => ToolCallResult::text(content),
        Err(_) => ToolCallResult::error(format!(
            "File not found: .skills/{}. Run `ztlgr init-skills` to generate .skills/ files.",
            file
        )),
    }
}

// --- Helper functions ---

/// Truncates content to a maximum length, adding ellipsis if truncated.
fn truncate_content(content: &str, max_len: usize) -> String {
    let first_line = content.lines().next().unwrap_or("");
    if first_line.len() <= max_len {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..max_len])
    }
}

/// Maps note type to the vault directory folder.
fn folder_for_note_type(note_type: &NoteType) -> &'static str {
    match note_type {
        NoteType::Permanent => "permanent",
        NoteType::Fleeting => "inbox",
        NoteType::Literature { .. } => "literature",
        NoteType::Daily => "daily",
        NoteType::Reference { .. } => "reference",
        NoteType::Index => "index",
    }
}

/// Sanitizes a title for use as a filename.
fn sanitize_filename(title: &str) -> String {
    title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .replace(' ', "-")
        .to_lowercase()
}

/// Regenerates the grimoire index.
fn regenerate_index(vault_path: &Path, db: &Database) -> Result<()> {
    let generator = IndexGenerator::new(db);
    generator.write_index(vault_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_all_tools_count() {
        let tools = all_tools();
        assert_eq!(tools.len(), 9);
    }

    #[test]
    fn test_all_tools_unique_names() {
        let tools = all_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(names.len(), unique.len(), "Tool names must be unique");
    }

    #[test]
    fn test_all_tools_have_descriptions() {
        for tool in all_tools() {
            assert!(
                !tool.description.is_empty(),
                "Tool '{}' has empty description",
                tool.name
            );
        }
    }

    #[test]
    fn test_all_tools_have_input_schema() {
        for tool in all_tools() {
            assert_eq!(
                tool.input_schema["type"], "object",
                "Tool '{}' input_schema must be an object",
                tool.name
            );
        }
    }

    #[test]
    fn test_tool_names() {
        let tools = all_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"search"));
        assert!(names.contains(&"get_note"));
        assert!(names.contains(&"list_notes"));
        assert!(names.contains(&"create_note"));
        assert!(names.contains(&"get_backlinks"));
        assert!(names.contains(&"ingest_source"));
        assert!(names.contains(&"read_index"));
        assert!(names.contains(&"read_log"));
        assert!(names.contains(&"read_skills"));
    }

    #[test]
    fn test_truncate_content_short() {
        assert_eq!(truncate_content("hello", 200), "hello");
    }

    #[test]
    fn test_truncate_content_long() {
        let long = "a".repeat(300);
        let result = truncate_content(&long, 200);
        assert!(result.ends_with("..."));
        assert_eq!(result.len(), 203); // 200 + "..."
    }

    #[test]
    fn test_truncate_content_multiline() {
        let content = "first line\nsecond line\nthird line";
        assert_eq!(truncate_content(content, 200), "first line");
    }

    #[test]
    fn test_sanitize_filename_simple() {
        assert_eq!(sanitize_filename("My Note Title"), "my-note-title");
    }

    #[test]
    fn test_sanitize_filename_special_chars() {
        assert_eq!(sanitize_filename("Note: A/B (test)"), "note_-a_b-_test_");
    }

    #[test]
    fn test_sanitize_filename_empty() {
        assert_eq!(sanitize_filename(""), "");
    }

    #[test]
    fn test_folder_for_note_type() {
        assert_eq!(folder_for_note_type(&NoteType::Permanent), "permanent");
        assert_eq!(folder_for_note_type(&NoteType::Fleeting), "inbox");
        assert_eq!(
            folder_for_note_type(&NoteType::Literature {
                source: String::new()
            }),
            "literature"
        );
        assert_eq!(folder_for_note_type(&NoteType::Daily), "daily");
        assert_eq!(
            folder_for_note_type(&NoteType::Reference { url: None }),
            "reference"
        );
        assert_eq!(folder_for_note_type(&NoteType::Index), "index");
    }

    #[test]
    fn test_handle_search_missing_query() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({});
        let result = handle_search(&args, &db);
        // The function returns ToolCallResult::error for missing query
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["isError"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_handle_search_empty_results() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({"query": "nonexistent"});
        let result = handle_search(&args, &db);
        assert!(result.is_error.is_none());
        let json = serde_json::to_value(&result).unwrap();
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("No results found"));
    }

    #[test]
    fn test_handle_search_with_results() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let note = Note::new(
            "Rust Programming".to_string(),
            "A systems programming language".to_string(),
        );
        db.create_note(&note).unwrap();
        let args = serde_json::json!({"query": "Rust", "limit": 5});
        let result = handle_search(&args, &db);
        assert!(result.is_error.is_none());
        let json = serde_json::to_value(&result).unwrap();
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Rust Programming"));
    }

    #[test]
    fn test_handle_get_note_by_title() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let note = Note::new("Test Note".to_string(), "Content here".to_string());
        db.create_note(&note).unwrap();
        let args = serde_json::json!({"title": "Test Note"});
        let result = handle_get_note(&args, &db);
        assert!(result.is_error.is_none());
        let json = serde_json::to_value(&result).unwrap();
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Test Note"));
        assert!(text.contains("Content here"));
    }

    #[test]
    fn test_handle_get_note_not_found() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({"title": "Nonexistent"});
        let result = handle_get_note(&args, &db);
        let json = serde_json::to_value(&result).unwrap();
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("not found"));
    }

    #[test]
    fn test_handle_get_note_no_params() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({});
        let result = handle_get_note(&args, &db);
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["isError"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_handle_list_notes_empty() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({});
        let result = handle_list_notes(&args, &db);
        let json = serde_json::to_value(&result).unwrap();
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("No notes found"));
    }

    #[test]
    fn test_handle_list_notes_with_results() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let note = Note::new("Listed Note".to_string(), "Content".to_string());
        db.create_note(&note).unwrap();
        let args = serde_json::json!({"limit": 10});
        let result = handle_list_notes(&args, &db);
        let json = serde_json::to_value(&result).unwrap();
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Listed Note"));
        assert!(text.contains("1 note(s)"));
    }

    #[test]
    fn test_handle_read_index_missing() {
        let temp = TempDir::new().unwrap();
        let result = handle_read_index(temp.path());
        let json = serde_json::to_value(&result).unwrap();
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("not found"));
    }

    #[test]
    fn test_handle_read_index_exists() {
        let temp = TempDir::new().unwrap();
        let ztlgr_dir = temp.path().join(".ztlgr");
        std::fs::create_dir_all(&ztlgr_dir).unwrap();
        std::fs::write(ztlgr_dir.join("index.md"), "# Index\nContent").unwrap();
        let result = handle_read_index(temp.path());
        let json = serde_json::to_value(&result).unwrap();
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("# Index"));
    }

    #[test]
    fn test_handle_read_log_missing() {
        let temp = TempDir::new().unwrap();
        let result = handle_read_log(temp.path());
        let json = serde_json::to_value(&result).unwrap();
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("not found"));
    }

    #[test]
    fn test_handle_read_skills_missing_param() {
        let temp = TempDir::new().unwrap();
        let args = serde_json::json!({});
        let result = handle_read_skills(&args, temp.path());
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["isError"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_handle_read_skills_path_traversal() {
        let temp = TempDir::new().unwrap();
        let args = serde_json::json!({"file": "../../etc/passwd"});
        let result = handle_read_skills(&args, temp.path());
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["isError"].as_bool().unwrap_or(false));
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("traversal"));
    }

    #[test]
    fn test_handle_read_skills_success() {
        let temp = TempDir::new().unwrap();
        let skills_dir = temp.path().join(".skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        std::fs::write(skills_dir.join("README.md"), "# Skills").unwrap();
        let args = serde_json::json!({"file": "README.md"});
        let result = handle_read_skills(&args, temp.path());
        let json = serde_json::to_value(&result).unwrap();
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("# Skills"));
    }

    #[test]
    fn test_handle_unknown_tool() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({});
        let result = handle_tool_call(
            "nonexistent_tool",
            &args,
            temp.path(),
            &db,
            Format::Markdown,
        );
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["isError"].as_bool().unwrap_or(false));
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Unknown tool"));
    }

    #[test]
    fn test_handle_create_note_missing_title() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({"content": "some content"});
        let result = handle_create_note(&args, temp.path(), &db, Format::Markdown);
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["isError"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_handle_create_note_missing_content() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({"title": "Test"});
        let result = handle_create_note(&args, temp.path(), &db, Format::Markdown);
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["isError"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_handle_create_note_success() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        let permanent_dir = temp.path().join("permanent");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&permanent_dir).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({
            "title": "MCP Created Note",
            "content": "This note was created via MCP"
        });
        let result = handle_create_note(&args, temp.path(), &db, Format::Markdown);
        let json = serde_json::to_value(&result).unwrap();
        assert!(json.get("isError").is_none() || !json["isError"].as_bool().unwrap_or(false));
        let text = json["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("MCP Created Note"));
        assert!(text.contains("created successfully"));
    }

    #[test]
    fn test_handle_get_backlinks_no_params() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({});
        let result = handle_get_backlinks(&args, &db);
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["isError"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_handle_ingest_source_file_not_found() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join(".ztlgr").join("vault.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let db = Database::new(&db_path).unwrap();
        let args = serde_json::json!({"file_path": "/nonexistent/file.txt"});
        let result = handle_ingest_source(&args, temp.path(), &db);
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["isError"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_search_tool_schema_has_required_query() {
        let tool = tool_search();
        let required = tool.input_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("query")));
    }

    #[test]
    fn test_create_note_tool_schema_has_required_fields() {
        let tool = tool_create_note();
        let required = tool.input_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("title")));
        assert!(required.contains(&serde_json::json!("content")));
    }

    #[test]
    fn test_read_skills_tool_schema_has_required_file() {
        let tool = tool_read_skills();
        let required = tool.input_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("file")));
    }
}
