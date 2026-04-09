# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-04-07

### Added
- **Knowledge Graph Visualization**
  - New `src/graph/` module with `types.rs` (GraphNode, GraphEdge, GraphData) and `layout.rs` (Fruchterman-Reingold force-directed algorithm)
  - Database methods: `get_all_links()` and `get_graph_nodes()` for graph data retrieval (+13 tests)
  - `GraphView` widget using ratatui Canvas with Braille markers for high-resolution rendering
  - `GraphState` with pan, zoom, node selection, fit-to-view, and center-on-selected
  - Graph mode (`v` to enter): sidebar + full-area graph view replaces editor/preview
  - Node colors by note type (daily, fleeting, literature, permanent, reference, index)
  - Node size scales with degree (number of connections)
  - Edge rendering as lines between connected nodes
  - Labels rendered above nodes (configurable via `graph.show_labels`)

- **Graph Mode Keybindings**
  - `h/j/k/l` or arrow keys: pan view
  - `+`/`=`: zoom in, `-`: zoom out
  - `Tab`/`Shift+Tab`: cycle through nodes (auto-centers on selection)
  - `Enter`: navigate to selected note and exit graph mode
  - `c`: center view on selected node
  - `f`: fit entire graph in view
  - `q`/`Esc`: exit graph mode

- **Help Modal Updates**
  - New Graph Mode section documenting all graph keybindings

### Changed
- `draw()` method branches between normal layout and graph layout based on mode
- `handle_graph_mode()` expanded from 3-line stub to full keybinding handler
- Added `enter_graph_mode()` method that loads data from DB, runs layout algorithm, and creates `GraphState`

### Technical
- New module: `src/graph/` (mod.rs, types.rs, layout.rs) — 22 tests
- New widget: `src/ui/widgets/graph_view.rs` — 16 tests
- Database tests for graph methods: 13 tests
- Test count: 423 passing (from 372, +51 new tests)
- Zero clippy warnings

## [0.4.0] - 2026-04-07

### Added
- **Inter-note Links Integration**
  - Database methods: `get_backlinks()`, `delete_links_for_note()`, `find_note_by_title()`, `get_links_for_note()` (+15 tests)
  - Link following: `Enter` to follow link under cursor (wiki/markdown format), resolves by title or ID
  - Navigation history: `Ctrl+O` to go back, LIFO stack with max 50 entries
  - Backlinks pane: `B` to toggle, shown as footer in preview panel (70/30 split)
  - Link autocomplete: `[[` triggers suggestions, `Tab`/`Enter` accepts, `Up`/`Down` navigates
  - Auto extract & store links on every note save, keeping the link graph in sync
  - Editor cursor context: `get_current_line()` and `cursor_col()` for link detection

- **Help Modal Updates**
  - Link navigation keybindings documented (Enter, Ctrl+O)
  - Autocomplete instructions (`[[` to trigger, Tab/Enter to accept)
  - Backlinks toggle (B key)

### Fixed
- **Autocomplete inserted UUID instead of note title** - now correctly inserts `[[Note Title]]`
- **Autocomplete left duplicate `[[` brackets** - now deletes the opening `[[` before inserting the full link
- **Link following keybindings unreachable** - `Ctrl+]`/`Ctrl+[` are impossible in standard terminals (Ctrl+[ = ESC). Replaced with `Enter`/`Ctrl+O` (Vim convention)
- **Link following only worked in NoteList panel** - now works in Editor normal mode where the cursor sits on links
- **Backlinks replaced entire preview** - moved from separate panel mode to preview footer

### Changed
- Removed `RightPanel::Backlinks` variant; backlinks now overlay the preview as a footer
- Link keybindings: `Enter` (follow), `Ctrl+O` (back) replace impossible `Ctrl+]`/`Ctrl+[`

### Technical
- Declared orphaned modules (`link_following`, `navigation_history`) in `ui/mod.rs`
- Removed blanket `#![allow(dead_code)]` from `backlinks_pane.rs` and `link_autocomplete.rs`
- Cleaned up unused imports in `widgets/mod.rs`
- Test count: 372 passing (from 337)

## [0.3.1] - 2026-04-07

### Added
- **Full Markdown Preview** (complete rewrite of preview pane)
  - Blockquotes: Nested levels (up to 5) with colored `│` prefix
  - Tables: Unicode borders (`│├┼┤`), column alignment (left/center/right), bold headers
  - Task lists: `[x]` (green) / `[ ]` (gray) checkbox rendering
  - Strikethrough: `~~text~~` with crossed-out styling
  - Footnotes: `[^ref]` inline references + `[^ref]: text` definitions
  - Images: Placeholder `[IMG: alt text]` with URL display
  - Wiki-links: `[[target]]` and `[[target|label]]` rendered with brackets
  - Code blocks: Box-drawing borders `┌─ lang ─` / `│ code` / `└───`
  - Nested lists: Indentation + alternating bullets (`•` `◦` `▸`)
  - Word wrap: Word-boundary aware (replaces character-level)
  - Headings: `#`/`##`/`###` prefix indicators, H1 with background highlight
  - Smart punctuation: `--` to em-dash, smart quotes

### Changed
- Upgraded `pulldown-cmark` from 0.9.6 to 0.13.3 (all GFM extensions)
- Complete rewrite of `preview_pane.rs`: 351 to 1581 lines

### Technical
- Test count: 337 passing (+58 new preview tests, from 279)
- Zero clippy warnings

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
- **Architecture**: Hybrid storage (files + SQLite index)

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
- Graph filtering by note type/tags not yet implemented (planned for future release)
- Terminal-only (no mobile support)