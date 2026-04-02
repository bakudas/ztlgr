# ztlgr - Quick Start Guide

## Building and Running

```bash
# Build the project
cargo build

# Run the TUI application
cargo run --bin ztlgr

# Run the CLI
cargo run --bin ztlgr-cli -- new ~/my-notes
```

## Your First Note

1. **Launch** the app: `cargo run --bin ztlgr`
2. **Create** a note: Press `n`
3. **Edit**: Press `i` to enter insert mode
4. **Type**: Your note content
5. **Save**: Press `Ctrl+S`
6. **Exit**: Press `Esc` then `q`

## Navigation

### Vim-like Keys
- `j` / `k` - Move down/up through notes
- `h` / `l` - Move left/right between panels
- `g` / `G` - Jump to first/last note

### Mouse Support
- **Click** on a note to select it
- **Scroll** to navigate through list
- **Right-click** (coming soon - context menu)

## Editing

### Insert Mode
Press `i` to enter insert mode:
- Type normally
- `Ctrl+S` to save
- `Esc` to exit to normal mode

### Quick Actions
- `n` - New note
- `d` - Delete note
- `Ctrl+R` - Rename note
- `Ctrl+S` - Save note
- `p` - Toggle preview pane

## Advanced Features

### Search
- Press `/` to search (coming soon)
- Type your query
- Press `Enter` to find

### Graph View
- Press `v` for visual link graph (coming soon)
- Navigate with arrow keys

### Auto-save
- Configured in `~/.config/ztlgr/config.toml`
- Set `auto_save_interval` to auto-save interval in seconds
- 0 = disabled

## Troubleshooting

### Terminal Looks Broken After Exit
- Fixed! Terminal is always restored cleanly now
- If issues persist, type `reset` and press Enter

### Can't Save Notes
- Make sure you're in Normal mode (press `Esc`)
- Press `Ctrl+S` to save
- Check status bar for confirmation message

### Mouse Not Working
- Enabled by default
- Try using keyboard shortcuts as fallback:
  - `j` and `k` to navigate
  - `i` to edit

## File Structure

```
~/.local/share/ztlgr/vault/          # Default vault location
├── permanent/                         # Long-term notes
├── inbox/                            # Fleeting notes
├── literature/                       # Source quotes
├── reference/                        # External links
├── index/                            # Structure notes (MOCs)
├── daily/                            # Daily journal
└── .ztlgr/
    └── vault.db                      # Index database
```

## Configuration

Edit `~/.config/ztlgr/config.toml`:

```toml
[editor]
auto_save_interval = 30              # Auto-save every 30 seconds (0 = disabled)
theme = "dracula"                    # Theme: dracula, gruvbox, nord, solarized

[display]
show_preview = true                  # Show preview pane
mouse_enabled = true                 # Enable mouse support
```

## Keyboard Reference

| Key | Action |
|-----|--------|
| `n` | New note |
| `i` | Edit mode |
| `d` | Delete |
| `Ctrl+S` | Save |
| `Ctrl+R` | Rename |
| `Esc` | Normal mode |
| `q` | Quit |
| `/` | Search (coming) |
| `v` | Graph view (coming) |
| `:` | Command mode |
| `p` | Toggle preview |

## Tips & Tricks

1. **Fast Navigation**: Use `g` to jump to first note, then `j`/`k` to find others
2. **Quick Save**: Always press `Ctrl+S` before switching notes
3. **Auto-save**: Set `auto_save_interval` if you forget to save
4. **Backup**: Keep backups of `.ztlgr/vault.db` and note files

## Getting Help

- Check logs: `~/.local/share/ztlgr/ztlgr.log`
- Report bugs: [GitHub Issues]
- Read docs: See `FIXES_REPORT.md` for technical details

---

**Happy note-taking!** 📝

For more information, see `BUGFIXES.md` and `FIXES_REPORT.md`.
