-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_info (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO schema_info (key, value) VALUES ('version', '1');

-- Core notes table
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    note_type TEXT DEFAULT 'permanent',
    
    -- Zettelkasten fields
    zettel_id TEXT UNIQUE,
    parent_id TEXT,
    source TEXT,
    
    -- Metadata
    metadata TEXT,
    
    -- Timestamps
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    
    -- Status
    status TEXT DEFAULT 'active',
    
    FOREIGN KEY (parent_id) REFERENCES notes(id) ON DELETE SET NULL
);

-- Links between notes (graph edges)
CREATE TABLE IF NOT EXISTS links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    link_type TEXT DEFAULT 'reference',
    context TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (source_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES notes(id) ON DELETE CASCADE,
    UNIQUE(source_id, target_id, link_type)
);

-- Full-text search virtual table
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    id,
    title,
    content,
    content='notes',
    content_rowid='rowid'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts(rowid, id, title, content)
    VALUES (new.rowid, new.id, new.title, new.content);
END;

CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, id, title, content)
    VALUES('delete', old.rowid, old.id, old.title, old.content);
END;

CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, id, title, content)
    VALUES('delete', old.rowid, old.id, old.title, old.content);
    INSERT INTO notes_fts(rowid, id, title, content)
    VALUES (new.rowid, new.id, new.title, new.content);
END;

-- Tags
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS note_tags (
    note_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (note_id, tag_id)
);

-- Zettelkasten ID sequence
CREATE TABLE IF NOT EXISTS zettel_sequence (
    prefix TEXT PRIMARY KEY,
    next_number INTEGER DEFAULT 1
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_notes_created ON notes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notes_type ON notes(note_type);
CREATE INDEX IF NOT EXISTS idx_notes_zettel ON notes(zettel_id);
CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_id);
CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_id);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);

-- View for backlinks
CREATE VIEW IF NOT EXISTS backlinks AS
SELECT 
    target_id as note_id,
    source_id as backlink_id,
    link_type,
    context,
    created_at
FROM links;