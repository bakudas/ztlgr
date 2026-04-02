# Storage Module

This module handles file-based storage for notes in Markdown and Org formats.

## Architecture

ztlgr uses a **hybrid storage architecture**:

1. **Files as Source of Truth**: Each note is stored as a `.md` or `.org` file
2. **SQLite as Index**: Fast search and relationships
3. **Automatic Sync**: File watcher keeps database in sync

## File Structure

Each note file contains:

### Markdown (.md)
```markdown
---
id: 20240115-143022-a1b2c3
title: My Note Title
type: permanent
zettel_id: 1a2b3c
created: 2024-01-15T14:30:22Z
updated: 2024-01-15T15:45:00Z
tags:
  - rust
  - zettelkasten
aliases:
  - My Note
---

# My Note Title

Content with [[links]] and #tags
```

### Org (.org)
```org
:PROPERTIES:
:ID: 20240115-143022-a1b2c3
:TITLE: My Note Title
:TYPE: permanent
:ZETTEL_ID: 1a2b3c
:CREATED: 2024-01-15T14:30:22
:UPDATED: 2024-01-15T15:45:00
:END:

* My Note Title

Content with [[links]] and #tags
```

## Database as Index

The SQLite database indexes:
- Full-text search (FTS5)
- Links between notes
- Tags
- Metadata queries

But the **actual content lives in files**, making it:
- Version-control friendly (git)
- Portable (just copy folder)
- Editable with any tool
- Compatible with Obsidian, Foam, etc.

## Sync Process

1. **On Startup**: Scan vault directory, sync with database
2. **On File Change**: Watcher updates database
3. **On Edit in ztlgr**: Update both file and database
4. **On Import**: Detect new files, add to database