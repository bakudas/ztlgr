# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive installation guide (INSTALL.md)
- Release process documentation (docs/RELEASE.md)
- Continuous Integration workflow (CI)
- Automated release workflow with GitHub Actions
- Support for multiple platforms (Linux, macOS, Windows)
- Binary checksums for verification
- Automatic crates.io publishing

## [0.3.0] - 2026-04-05

### Added
- **Vim Modal Editing for Editor Panel**
  - Full Vim-style navigation: `h/j/k/l`, arrows, `w/b`, `0/$`, `g/G`
  - Insert mode commands: `i/I/a/A/o/O`
  - Delete operations: `x/X`, `d` (delete line), `D` (delete to end of line)
  - Yank/Paste: `y` (yank line), `p` (paste)
  - Undo/Redo: `u` (undo), `Ctrl+r` (redo)
  - Block cursor in Normal mode, line cursor in Insert mode

- **Help Modal**
  - Comprehensive keybindings reference (Normal, Insert, Global modes)
  - CLI commands documentation (`:new`, `:open`, `:search`, `:import`, `:sync`)
  - Credits section with author, license, repo link
  - Navigation with `↑↓/j/k`, close with `Esc/?/q`
  - Access via `?` key or `:help` command

- **Editor Improvements**
  - Word wrap with unicode-width support
  - Arrow key support in Normal mode
  - Fixed sidebar width (no more collapsing)

### Changed
- **Editor Refactor**: Replaced custom `TextRope` with `tui-textarea` library
  - Better performance and reliability
  - Native Vim-like keybindings support
  - Built-in undo/redo stack

### Fixed
- Preview pane rendering with style-preserving wrap and list markers
- Layout constraints to prevent panel collapsing
- Phase 3 robustness issues (M1, M2, M5, M7, M8)
- Extracted shared VAULT_DIRS constant
- Removed dead code and orphaned files

### Technical
- Added `tui-textarea` dependency (v0.5)
- Added `unicode-width` dependency (v0.1)
- Test count: 279 passing (from 264)

## [0.1.1] - 2025-04-03

### Added
- **Real-time Markdown Preview**
  - Preview pane now updates instantly as you type
  - See rendered markdown without saving or switching modes
  - Improved editing workflow with live feedback
  - Performance optimized: only renders when preview is visible

### Fixed
- **Critical: UTF-8 Crash on Backspace/Delete**
  - Fixed panic when backspacing multi-byte characters (accents, emojis)
  - Prevented crash when cursor at invalid UTF-8 boundaries
  - Added safe string slicing with `.get()` instead of direct indexing
  
- **Multi-line Deletion Bug**
  - Fixed entire lines being deleted when backspacing at line start
  - Corrected merge logic in `TextRope::delete_range()`
  - Lines now properly merge without losing content

### Improved
- **Markdown Rendering in Preview**
  - Proper heading styles with colors (H1-H6)
  - Bold, italic, and bold+italic text formatting
  - Inline code blocks with accent color
  - Code blocks with language identifiers
  - Ordered and unordered lists with proper markers
  - Horizontal rules styled with theme
  - Links with underlined style and URL display
  
- **Text Wrapping**
  - Word-based wrapping prevents overflow
  - Proper width calculation based on panel size
  - Text respects panel boundaries
  
- **Navigation**
  - `j/k` keys scroll preview when panel has focus
  - Better panel focus handling

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