-- Migration: v1 -> v2
-- Adds the sources table for raw source material tracking.

CREATE TABLE IF NOT EXISTS sources (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    origin TEXT,
    content_hash TEXT NOT NULL,
    ingested_at TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER DEFAULT 0,
    mime_type TEXT,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_sources_hash ON sources(content_hash);
CREATE INDEX IF NOT EXISTS idx_sources_ingested ON sources(ingested_at DESC);

UPDATE schema_info SET value = '2' WHERE key = 'version';
