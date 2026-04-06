# Installation Guide

## Table of Contents

- [Quick Start](#quick-start)
- [Installation Methods](#installation-methods)
  - [Cargo (Recommended)](#cargo-recommended)
  - [Binary Releases](#binary-releases)
  - [Building from Source](#building-from-source)
  - [Nix Flakes](#nix-flakes)
- [Platform-Specific Notes](#platform-specific-notes)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)

---

## Quick Start

```bash
# Install via Cargo
cargo install ztlgr

# Run
ztlgr

# Or specify a vault
ztlgr open ~/my-notes
```

---

## Installation Methods

### Cargo (Recommended)

The easiest way to install ztlgr is via Cargo:

```bash
cargo install ztlgr
```

This downloads and compiles the latest release from [crates.io](https://crates.io/crates/ztlgr).

**Requirements:**
- Rust 1.70 or later
- Cargo (comes with Rust)

**Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

### Binary Releases

Download pre-compiled binaries from [GitHub Releases](https://github.com/bakudas/ztlgr/releases).

#### Linux

```bash
# Download
curl -LO https://github.com/bakudas/ztlgr/releases/download/v0.1.0/ztlgr-linux-x86_64.tar.gz

# Extract
tar -xzf ztlgr-linux-x86_64.tar.gz

# Move to PATH
sudo mv ztlgr /usr/local/bin/

# Verify
ztlgr --version
```

#### macOS

```bash
# Download (Intel)
curl -LO https://github.com/bakudas/ztlgr/releases/download/v0.1.0/ztlgr-macos-x86_64.tar.gz

# Download (Apple Silicon)
curl -LO https://github.com/bakudas/ztlgr/releases/download/v0.1.0/ztlgr-macos-aarch64.tar.gz

# Extract
tar -xzf ztlgr-macos-*.tar.gz

# Move to PATH
sudo mv ztlgr /usr/local/bin/

# Verify
ztlgr --version
```

#### Windows

```powershell
# Download
Invoke-WebRequest -Uri https://github.com/bakudas/ztlgr/releases/download/v0.1.0/ztlgr-windows-x86_64.exe.tar.gz -OutFile ztlgr-windows.tar.gz

# Extract (requires 7-Zip or similar)
tar -xzf ztlgr-windows.tar.gz

# Move to PATH
Move-Item ztlgr.exe C:\Windows\System32\

# Verify
ztlgr --version
```

---

### Building from Source

```bash
# Clone repository
git clone https://github.com/bakudas/ztlgr.git
cd ztlgr

# Build release
cargo build --release

# Binary is at
./target/release/ztlgr

# Install locally
cargo install --path .
```

**Build Requirements:**
- Rust 1.70+
- C compiler (for SQLite compilation)
- Make (optional, for shortcuts)

---

### Nix Flakes

For Nix users, ztlgr provides a flake.nix for reproducible builds.

#### With Flakes Enabled

```nix
# Run directly
nix run github:bakudas/ztlgr

# Install to profile
nix profile install github:bakudas/ztlgr
```

#### With Nix_Shell (Legacy)

```bash
# Enter development shell
nix-shell

# Or use direnv
direnv allow
```

---

## Platform-Specific Notes

### Linux

**Terminal Requirements:**
- Must support ANSI colors
- UTF-8 support recommended
- True color support for theme colors

**Font:**
- Nerd Font recommended for icons
- Install: `sudo apt install fonts-noto-color-emoji`

### macOS

**Homebrew (Coming Soon):**
```bash
brew tap bakudas/ztlgr
brew install ztlgr
```

**Permissions:**
If you get "cannot verify developer", run:
```bash
xattr -cr /usr/local/bin/ztlgr
```

### Windows

**Windows Terminal:**
- Recommended for best experience
- PowerShell 7+ or Windows Terminal
- WSL2 also supported

**Path Issues:**
If ztlgr is not found:
1. Add `%USERPROFILE%\.cargo\bin` to PATH
2. Or move binary to `C:\Windows\System32\`

---

## Configuration

### Default Locations

**Config:**
- Linux: `~/.config/ztlgr/config.toml`
- macOS: `~/.config/ztlgr/config.toml`
- Windows: `%APPDATA%\ztlgr\config.toml`

**Vault (default):**
- Linux: `~/.local/share/ztlgr/vault`
- macOS: `~/.local/share/ztlgr/vault`
- Windows: `%APPDATA%\ztlgr\vault`

### First Run

When you first run `ztlgr`, you'll see:

```
Welcome to ztlgr!
No vault found at ~/.local/share/ztlgr/vault

Would you like to:
  1. Create a new vault
  2. Open existing vault
  3. Import from directory

Choice [1-3]:
```

Follow the interactive setup to configure:
- Vault location
- Note format (Markdown or Org Mode)
- Theme (Dracula, Gruvbox, Nord, Solarized)
- Import existing notes (optional)

---

## Troubleshooting

### "Cannot find Cargo"

Install Rust and Cargo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### "Linker `cc` not found"

Install a C compiler:
- **Ubuntu/Debian**: `sudo apt install build-essential`
- **macOS**: `xcode-select --install`
- **Fedora/RHEL**: `sudo dnf install gcc`

### "Failed to create vault"

Check permissions:
```bash
# Ensure directory is writable
mkdir -p ~/.local/share/ztlgr
chmod 755 ~/.local/share/ztlgr
```

### "Database error"

Reset vault (WARNING: deletes all notes):
```bash
rm -rf ~/.local/share/ztlgr/vault/.ztlgr/vault.db
```

### "Nix build fails"

Update flake inputs:
```bash
nix flake update
```

Or rebuild with:
```bash
nix build github:bakudas/ztlgr --rebuild
```

---

## Next Steps

After installation:

1. **Create your first note:** Press `n` in Normal mode
2. **Learn keybindings:** Press `?` for help
3. **Configure your workflow:** Edit `~/.config/ztlgr/config.toml`
4. **Choose a theme:** Set `theme = "gruvbox"` in config

See the [User Guide](#) for detailed workflow instructions.

---

## Questions?

- **Issues:** [GitHub Issues](https://github.com/bakudas/ztlgr/issues)
- **Discussions:** [GitHub Discussions](https://github.com/bakudas/ztlgr/discussions)
- **Documentation:** [docs/](docs/)

---

## Appendix: Quick Reference

### Keyboard Shortcuts

#### Normal Mode
| Key | Action |
|-----|--------|
| `j` / `k` | Move down/up through notes |
| `h` / `l` | Move left/right between panels |
| `g` / `G` | Jump to first/last note |
| `n` | New note |
| `d` | Delete note |
| `i` | Enter insert mode |
| `/` | Search |
| `:` | Command mode |
| `p` | Toggle preview |
| `m` | Toggle metadata |
| `?` | Help |
| `q` | Quit |

#### Editor Normal Mode (Vim-style)
| Key | Action |
|-----|--------|
| `h/j/k/l` or arrows | Move cursor |
| `w` / `b` | Word forward/back |
| `0` / `$` | Line start/end |
| `g` / `G` | Document top/bottom |
| `i` | Insert mode |
| `a` / `A` | Append (at cursor / end of line) |
| `o` / `O` | Open line below/above |
| `x` / `X` | Delete next/prev char |
| `d` | Delete line (dd) |
| `D` | Delete to end of line |
| `u` | Undo |
| `Ctrl+r` | Redo |
| `y` | Yank (copy) |
| `p` | Paste |
| `Esc` | Focus note list |

#### Insert Mode
| Key | Action |
|-----|--------|
| `Esc` | Exit insert mode (auto-save) |
| `Ctrl+s` | Save note |
| `Ctrl+z/y` | Undo/Redo |
| `Ctrl+c/v/x` | Copy/Paste/Cut |

### File Structure

```
~/.local/share/ztlgr/vault/          # Default vault location
├── permanent/                        # Long-term notes
├── inbox/                            # Fleeting notes
├── literature/                       # Source quotes
├── reference/                        # External links
├── index/                            # Structure notes (MOCs)
├── daily/                            # Daily journal
└── .ztlgr/
    └── vault.db                      # Index database
```

### Configuration Example

Edit `~/.config/ztlgr/config.toml`:

```toml
[editor]
auto_save_interval = 30              # Auto-save every 30 seconds (0 = disabled)

[display]
show_preview = true                  # Show preview pane
mouse_enabled = true                 # Enable mouse support
theme = "dracula"                    # Theme: dracula, gruvbox, nord, solarized
```

### CLI Commands

```bash
# Create a new vault
ztlgr new ~/my-notes --format markdown

# Open vault in TUI
ztlgr open ~/my-notes

# Search notes via FTS5
ztlgr search "rust zettelkasten" --vault ~/my-notes

# Import existing notes
ztlgr import ~/existing-notes --vault ~/my-notes

# Sync vault with database
ztlgr sync --vault ~/my-notes

# Show help
ztlgr --help
```

### Tips & Tricks

1. **Fast Navigation**: Use `g` to jump to first note, then `j`/`k` to find others
2. **Quick Save**: Press `Ctrl+s` in any mode to save
3. **Vim Mode**: Use full Vim keybindings in the editor panel
4. **Help**: Press `?` for comprehensive help modal
5. **Auto-save**: Set `auto_save_interval` in config if you forget to save
6. **Backup**: Keep backups of `.ztlgr/vault.db` and note files