# ztlgr

A simple and fast terminal-based note-taking application with Zettelkasten methodology, built with Rust.

**[Installation Guide](INSTALL.md)** | **[Documentation](#)** | **[Changelog](CHANGELOG.md)**

## Features

- **Zettelkasten Methodology**: Luhmann-style IDs with flexible workflows
- **Fast Search**: SQLite FTS5 full-text search
- **Multiple Note Types**: Daily, fleeting, literature, permanent, reference, and index notes
- **Link System**: Wiki-style links `[[note-title]]` with backlinks
- **Themes**: Dracula (default), Gruvbox, Nord, Solarized, and custom themes
- **Vim Keybindings**: Modal editing with familiar vim shortcuts
- **CLI Interface**: Create vaults, search notes, and sync from the command line
- **Future-Proof Architecture**: Local files, smart db indexes and multi-agent system ready for extensions

## Installation

```bash
cargo install ztlgr
```

## Quick Start

```bash
# Create a new vault
ztlgr new my-vault

# Open the vault in the TUI
ztlgr open my-vault

# Or run without arguments for the interactive setup wizard
ztlgr
```

## CLI Commands

### `ztlgr new <path>`

Create a new Zettelkasten vault with the full directory structure.

```bash
# Create a markdown vault (default)
ztlgr new ~/notes

# Create an org-mode vault
ztlgr new ~/notes --format org

# With short flags
ztlgr new ~/notes -f org
```

Creates the following structure:
```
my-vault/
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

Open an existing vault in the TUI.

```bash
# Open a specific vault
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

# Search within a specific vault
ztlgr search "rust" --vault ~/notes
```

### `ztlgr import <source>`

Import existing notes from a directory into a vault.

```bash
# Import notes into the current vault
ztlgr import ~/old-notes --vault ~/notes

# Recursive import
ztlgr import ~/old-notes --vault ~/notes --recursive
ztlgr import ~/old-notes --vault ~/notes -r
```

### `ztlgr sync`

Synchronize vault files with the database.

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
| `--vault <path>` | Default vault directory (env: `ZTLGR_VAULT`) |
| `-f, --format <fmt>` | Note format: `markdown` or `org` (default: `markdown`) |
| `-c, --config <path>` | Configuration file path (env: `ZTLGR_CONFIG`) |
| `-v, --verbose` | Verbosity level (repeat for more: `-vv`, `-vvv`) |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

```bash
# Set vault via environment variable
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
path = "~/.local/share/ztlgr/default.vault"
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

ztlgr is built with a multi-agent architecture:

- **NoteAgent**: Manages note CRUD operations
- **LinkAgent**: Manages links between notes
- **SearchAgent**: Full-text search
- **GraphAgent**: Knowledge graph operations
- **UIAgent**: Terminal UI management

Each agent uses atomic skills:
- **SqliteSkill**: Database operations
- **MarkdownSkill**: Parse and render markdown
- **TemplateSkill**: Note templates

## Database

Each vault is a SQLite database with:

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
