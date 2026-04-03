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
- **Future-Proof Architecture**: Local files, smart db indexes and multi-agent system ready for extensions

## Installation

```bash
cargo install ztlgr
```

## Quick Start

```bash
# Create a new vault
ztlgr new my-vault

# Open the vault
ztlgr open my-vault

# Or use the default vault location
ztlgr
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
```

## License

MIT OR Apache-2.0

## Credits

Inspired by:
- [Obsidian](https://obsidian.md/) - Knowledge base
- [zk](https://github.com/zk-org/zk) - Zettelkasten tool
- [Helix](https://helix-editor.com/) - Text editor
- [vit](https://github.com/yanarb/vit) - Terminal UI
