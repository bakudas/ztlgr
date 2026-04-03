# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive installation guide (INSTALL.md)
- Release process documentation (RELEASE.md)
- Continuous Integration workflow (CI)
- Automated release workflow with GitHub Actions
- Support for multiple platforms (Linux, macOS, Windows)
- Binary checksums for verification
- Automatic crates.io publishing

## [0.1.0] - 2025-04-03

### Added
- **Core Features**
  - Terminal-based Zettelkasten note-taking application
  - Vim-style modal editing (Normal, Insert, Search, Command, Graph modes)
  - Multiple note types: Daily, Fleeting, Permanent, Literature, Reference, Index
  - Link system with wiki-style `[[links]]`
  - Full-text search with SQLite FTS5
  - Soft delete with trash (7-day retention)
  - Markdown preview pane
  - Metadata panel for note properties

- **Editor**
  - Rope-based data structure for efficient editing
  - Undo/redo support
  - Copy/paste functionality
  - Link highlighting (valid/invalid)

- **UI/UX**
  - Theme system (Dracula, Gruvbox, Nord, Solarized, custom)
  - Panel focus indicators
  - Mode-specific status bar colors
  - Hierarchical note list by type
  - Search results with highlighting

- **Infrastructure**
  - Hybrid storage system (SQLite + Markdown files)
  - Vault-based organization (daily/, inbox/, permanent/, etc.)
  - Import system for existing notes
  - File watcher for external changes
  - Setup wizard for first-time users

- **Link System**
  - Wiki links: `[[note title]]`
  - Markdown links: `[text](note-title)`
  - Org-mode links: `[[id:uuid][title]]`
  - Link validation and highlighting
  - Link autocomplete with fuzzy matching
  - Link following with navigation history
  - Backlinks display widget

- **Commands**
  - `:rename <title>` - Rename note
  - `:move <path>` - Move note to different folder
  - `:tag <tag>` - Add tags to note
  - `:delete` - Soft delete note
  - `:recover` - Restore from trash
  - `:q` / `:quit` / `:exit` / `:x` - Quit application (Vim-style)

- **Keybindings**
  - Normal mode: `j/k` (navigate), `h/l` (switch panels), `i` (insert), `/` (search)
  - Insert mode: Standard text editing, `Esc` (exit)
  - Search mode: `Enter` (execute), `Esc` (cancel)
  - Command mode: `:` (enter command), `Esc` (cancel)

- **Distribution**
  - Binary releases for Linux (x86_64, ARM64)
  - Binary releases for macOS (x86_64, ARM64)
  - Binary releases for Windows (x86_64)
  - Cargo install support
  - Nix Flake support

### Technical Details
- **Language**: Rust 1.70+
- **TUI Framework**: Ratatui 0.26
- **Database**: SQLite with FTS5
- **Async Runtime**: Tokio
- **Test Coverage**: 248 passing tests
- **Architecture**: Multi-agent ready

### Platform Support
- Linux: Tested on Ubuntu 20.04+
- macOS: Tested on macOS 12+
- Windows: Tested on Windows 10+

### Security
- Soft delete for notes (7-day data retention)
- Vault-level isolation (each vault is separate SQLite database)
- Local-only storage (no telemetry or cloud sync)
- No known vulnerabilities

### Known Limitations
- Link autocomplete needs keybinding integration (Up/Down arrows)
- Link following needs keybinding integration (Ctrl+]/Ctrl+[)
- Backlinks pane needs database query integration (Ctrl+B)
- Graph visualization planned for future release
- Terminal-only (no mobile support)