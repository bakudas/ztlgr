# ztlgr

A local-first, personal knowledge base for the terminal. Zettelkasten methodology, built with Rust.

Your notes live as plain files on your machine. No cloud, no telemetry, no lock-in.
An optional LLM layer amplifies your ability to organize and connect knowledge --
but the app works fully without it. You think, the tool organizes.

**[Installation Guide](INSTALL.md)** | **[Documentation](#)** | **[Changelog](CHANGELOG.md)**

## Principles

> **Your knowledge, your machine, your rules.**

ztlgr is built on three convictions:

1. **Local-first.** Your notes are plain files on your filesystem. No server, no sync service,
   no account. The SQLite database is just a search index -- delete it and it regenerates
   from your files. You can read, edit, and move your notes with any tool you want.

2. **Human-first.** You are the owner of your knowledge base, not a consumer of a platform.
   ztlgr is a tool that serves you -- it never phones home, never tracks usage, never
   makes decisions on your behalf. Every automated action (git commits, LLM suggestions)
   requires your explicit consent.

3. **LLM as amplifier, not dependency.** An optional LLM layer can help summarize, cross-reference,
   and organize -- but the app is 100% functional without it. When you do use an LLM,
   local models (Ollama) are the first-class option. Cloud providers are supported but never required.
   The hierarchy is clear: **Human** (owner) > **ztlgr** (local tool) > **LLM** (optional assistant).

## Features

- **Local & Personal First**: Your files, your machine, your control. No account required.
- **Zettelkasten Methodology**: Luhmann-style IDs with flexible workflows
- **Fast Search**: SQLite FTS5 full-text search
- **Multiple Note Types**: Daily, fleeting, literature, permanent, reference, and index notes
- **Link System**: Wiki-style links `[[note-title]]` with backlinks and knowledge graph
- **Git-native Versioning**: Optional `git init` on grimoire creation for built-in history
- **Themes**: Dracula (default), Gruvbox, Nord, Solarized, and custom themes
- **Vim Keybindings**: Modal editing with familiar vim shortcuts
- **CLI Interface**: Create grimoires, search notes, and sync from the command line
- **LLM-augmented** (optional): `.skills/` schema lets LLM agents help maintain your knowledge base

## Installation

```bash
cargo install ztlgr
```

## Quick Start

```bash
# Create a new grimoire
ztlgr new my-grimoire

# Open the grimoire in the TUI
ztlgr open my-grimoire

# Or run without arguments for the interactive setup wizard
ztlgr
```

## CLI Commands

### `ztlgr new <path>`

Create a new Zettelkasten grimoire with the full directory structure.

```bash
# Create a markdown grimoire (default)
ztlgr new ~/notes

# Create an org-mode grimoire
ztlgr new ~/notes --format org

# Skip git initialization
ztlgr new ~/notes --no-git

# With short flags
ztlgr new ~/notes -f org
```

Creates the following structure:
```
my-grimoire/
├── .ztlgr/
│   └── vault.db
├── permanent/      # Permanent knowledge notes
├── inbox/          # Fleeting notes
├── literature/     # Notes from books, articles
├── reference/      # External reference notes
├── index/          # Structure notes (MOCs)
├── daily/          # Daily journal
├── attachments/    # Images and files
├── .gitignore
└── README.md
```

### `ztlgr open [path]`

Open an existing grimoire in the TUI.

```bash
# Open a specific grimoire
ztlgr open ~/notes

# Open with global vault flag
ztlgr --vault ~/notes open

# Without path, uses --vault or falls back to setup wizard
ztlgr open
```

### `ztlgr search <query>`

Search notes using SQLite FTS5 full-text search.

```bash
# Search for a term
ztlgr search "rust programming"

# Limit results
ztlgr search "zettelkasten" --limit 10
ztlgr search "zettelkasten" -l 10

# Search within a specific grimoire
ztlgr search "rust" --vault ~/notes
```

### `ztlgr import <source>`

Import existing notes from a directory into a grimoire.

```bash
# Import notes into the current grimoire
ztlgr import ~/old-notes --vault ~/notes

# Recursive import
ztlgr import ~/old-notes --vault ~/notes --recursive
ztlgr import ~/old-notes --vault ~/notes -r
```

### `ztlgr sync`

Synchronize grimoire files with the database.

```bash
# Quick sync
ztlgr sync --vault ~/notes

# Force full sync (reconciles all files)
ztlgr sync --vault ~/notes --force
ztlgr sync --vault ~/notes -f
```

### Global Options

| Flag | Description |
|------|-------------|
| `--vault <path>` | Grimoire directory (env: `ZTLGR_VAULT`) |
| `-f, --format <fmt>` | Note format: `markdown` or `org` (default: `markdown`) |
| `-c, --config <path>` | Configuration file path (env: `ZTLGR_CONFIG`) |
| `-v, --verbose` | Verbosity level (repeat for more: `-vv`, `-vvv`) |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

```bash
# Set grimoire via environment variable
export ZTLGR_VAULT=~/notes
ztlgr search "rust"

# Use verbose mode
ztlgr -vv sync --vault ~/notes
```

## KeyBindings

### Normal Mode
- `j/k` - Move down/up in note list
- `h/l` - Switch between panels
- `g/G` - Go to top/bottom
- `i` - Enter insert mode
- `n` - Create new note
- `d` - Delete note
- `/` - Search
- `v` - Graph view
- `p` - Toggle preview
- `m` - Toggle metadata
- `:q` - Quit (Vim-style)

### Insert Mode
- `Esc` - Exit insert mode
- Standard text editing keys

### Search Mode
- `Esc` - Return to normal mode
- `Enter` - Execute search

## Configuration

Configuration is stored in `~/.config/ztlgr/config.toml`:

```toml
[vault]
path = "~/.local/share/ztlgr/default.grimoire"
name = "default"
auto_backup_interval = 3600

[ui]
theme = "dracula"  # dracula, gruvbox, nord, solarized, custom
sidebar_width = 25
show_preview = true

[editor]
keybindings = "vim"
auto_save_interval = 30

[notes]
default_type = "permanent"
auto_zettel_id = true

[zettelkasten]
id_style = "luhmann"
create_daily_notes = true

[vcs]
enabled = true
auto_commit = false
commit_message = "{action}: {details}"
```

## Themes

### Built-in Themes
- **dracula** (default) - Purple and cyan accents
- **gruvbox** - Warm, retro colors
- **nord** - Arctic, bluish tones
- **solarized** - Precision color scheme

### Custom Themes
Create a custom theme in `~/.config/ztlgr/themes/my-theme.toml`:

```toml
name = "my-theme"

bg = { r = 40, g = 42, b = 54 }
bg_secondary = { r = 44, g = 46, b = 59 }
bg_highlight = { r = 68, g = 71, b = 90 }

fg = { r = 248, g = 248, b = 242 }
fg_secondary = { r = 98, g = 114, b = 164 }

accent = { r = 189, g = 147, b = 249 }
accent_secondary = { r = 255, g = 121, b = 198 }

# ... define all colors
```

## Architecture

ztlgr uses a hybrid storage architecture:

- **Files as Source of Truth**: Notes stored as `.md`/`.org` files, compatible with Obsidian, Foam, Logseq
- **SQLite as Index**: Fast full-text search (FTS5), link graph, and metadata queries
- **File Sync**: Bidirectional sync between files and database
- **Git-native**: Optional `git init` on grimoire creation for built-in version history

### LLM Wiki Integration (planned)

ztlgr supports the "LLM Wiki" pattern -- where LLM agents incrementally build and
maintain the knowledge base rather than re-deriving knowledge from scratch on each query.
The LLM is an optional amplifier, not a requirement. The app works fully without it.

- **`.skills/`** directory: schema and workflows for LLM agents
- **`raw/`** directory: immutable source material for ingestion
- **`index.md`**: auto-generated catalog of all wiki pages
- **`log.md`**: chronological activity log

See `docs/ROADMAP-LLM-WIKI.md` for the full implementation plan.

## Database

Each grimoire is backed by a SQLite database with:

- **notes** table: Core note storage
- **links** table: Graph edges
- **tags** table: Tag management
- **notes_fts**: Full-text search index

Graph queries use recursive CTEs for traversal.

## Development

```bash
# Clone the repository
git clone https://github.com/bakudas/ztlgr
cd ztlgr

# Build
cargo build

# Run tests
cargo test

# Run
cargo run

# Format, lint, and test
make check
```

## License

MIT OR Apache-2.0

## Credits

Inspired by:
- [Obsidian](https://obsidian.md/) - Knowledge base
- [zk](https://github.com/zk-org/zk) - Zettelkasten tool
- [Helix](https://helix-editor.com/) - Text editor
- [vit](https://github.com/yanarb/vit) - Terminal UI
