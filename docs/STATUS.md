# Status do Projeto ztlgr

**Data Atualização:** 7 de Abril de 2026  
**Versão:** 0.4.0 (Inter-note Links Integration)  
**Status Geral:** 🟢 ACTIVE DEVELOPMENT  
**Testes:** 370 passing (100% success rate)

---

## 📊 RESUMO EXECUTIVO

### Progresso Geral
- ✅ **Infrastructure**: 100% (setup, DB, storage, themes)
- ✅ **Core Features**: 100% (editor, search, command, modals)
- ✅ **Link System**: 100% (parsing + validation + highlighting + autocomplete + following + backlinks + DB integration)
- ✅ **CLI Interface**: 100% (new, open, search, import, sync)
- ✅ **Markdown Preview**: 100% (blockquotes, tables, task lists, footnotes, images, wiki-links)
- ✅ **Inter-note Links**: 100% (backlinks pane, link following, navigation history, autocomplete, extract & store)

---

## 🚀 LATEST RELEASE: v0.4.0

**Release Date:** April 7, 2026  
**Release Tag:** v0.4.0

**What's New in v0.4.0:**

### ✨ Inter-note Links Integration

Full integration of the link system into the application:

- **Database Layer** (4 new methods + 15 tests):
  - `get_backlinks()` - Retrieve all notes linking to a given note
  - `delete_links_for_note()` - Clear all outgoing links for a note
  - `find_note_by_title()` - Case-insensitive note lookup by title
  - `get_links_for_note()` - Retrieve all outgoing links for a note

- **Link Following** (`Ctrl+]` forward, `Ctrl+[` back):
  - Detects wiki-style `[[target]]`/`[[target|label]]` and markdown-style `[label](target)` at cursor position
  - Resolves targets by title (case-insensitive) or note ID
  - Opens external URLs with status bar message
  - Navigation history with LIFO ordering (max 50 entries)

- **Backlinks Pane** (`B` to toggle):
  - Shows all notes linking to the current note
  - Displayed as right panel option (`RightPanel::Backlinks`)
  - Scroll support with `j/k` keys
  - Auto-refreshes when loading notes

- **Link Autocomplete** (Insert mode):
  - Tab/Enter to accept suggestions
  - Up/Down to navigate suggestion list
  - Only active when autocomplete popup is visible

- **Extract & Store Links** (automatic):
  - On every note save, parses content via `LinkValidator::extract_all_links()`
  - Resolves targets by ID or title
  - Stores in DB with link type and context
  - Deletes old links first to keep graph in sync

### 🔧 Technical Changes

- Bumped version from 0.3.0 to 0.4.0
- 370 tests passing (+33 new tests, up from 337)
- Declared orphaned modules (`link_following`, `navigation_history`) in `ui/mod.rs`
- Removed blanket `#![allow(dead_code)]` from `backlinks_pane.rs` and `link_autocomplete.rs`
- Cleaned up unused imports in `widgets/mod.rs`
- Added `get_current_line()` and `cursor_col()` to `NoteEditor` for cursor context

---

## 🚀 PREVIOUS RELEASE: v0.3.1

**Release Date:** April 7, 2026  
**Release Tag:** v0.3.1

**What's New in v0.3.1:**

### ✨ Full Markdown Preview

Complete rewrite of the preview pane with full markdown support:
- **Upgrade**: `pulldown-cmark` 0.9 → 0.13.3 (all GFM extensions)
- **Blockquotes**: Nested levels with colored `│` prefix (up to 5 levels)
- **Tables**: Unicode borders (`│├┼┤`), column alignment (left/center/right), bold headers
- **Task Lists**: `[x]` (green) / `[ ]` (gray) checkbox rendering
- **Strikethrough**: `~~text~~` with crossed-out styling
- **Footnotes**: `[^ref]` inline references + `[^ref]: text` definitions
- **Images**: Placeholder `[IMG: alt text]` with URL display
- **Wiki-links**: `[[target]]` and `[[target|label]]` rendered with `[[]]` brackets
- **Code Blocks**: Box-drawing borders `┌─ lang ─` / `│ code` / `└───`
- **Nested Lists**: Indentation + alternating bullets (`•` `◦` `▸`)
- **Word Wrap**: Word-boundary aware wrapping (replaces character-level)
- **Headings**: `#`/`##`/`###` prefix indicators, H1 with background highlight
- **Smart Punctuation**: `--` → `—`, `"quotes"` → `"quotes"`

### 🔧 Technical Changes

- Upgraded `pulldown-cmark` from 0.9.6 to 0.13.3
- Rewrote `preview_pane.rs`: 351 → 1581 lines (complete rewrite)
- 337 tests passing (+58 new preview tests, up from 279)
- Zero clippy warnings

---

## 🚀 PREVIOUS RELEASE: v0.3.0

**Release Date:** April 5, 2026  
**Release Tag:** v0.3.0

**What's New in v0.3.0:**

### ✨ Vim Modal Editing for Editor

Complete Vim-style editing experience in the editor panel:
- **Navigation**: `h/j/k/l` (arrows), `w/b` (word), `0/$` (line), `g/G` (document)
- **Insert Mode**: `i/I/a/A/o/O` (insert/append/open line)
- **Delete**: `x/X` (char), `d` (line), `D` (to end of line)
- **Yank/Paste**: `y` (yank), `p` (paste)
- **Undo/Redo**: `u` (undo), `Ctrl+r` (redo)
- **Visual**: Block cursor in Normal mode

### ✨ Help Modal

Comprehensive help system accessible via `?` or `:help`:
- All keybindings organized by mode (Normal, Insert, Global)
- CLI commands reference (`ztlgr new/open/search/import/sync`)
- Credits: Author, License (MIT OR Apache-2.0), Repo link
- Navigation with `↑↓/j/k`, close with `Esc/?/q`

### ✨ Editor Improvements

- **Word Wrap**: Proper text wrapping with unicode-width support
- **Fixed Sidebar**: No more collapsing panels
- **Arrow Keys**: Full navigation support in Normal mode

### 🔧 Technical Changes

- Replaced custom `TextRope` with `tui-textarea` library
- Added `unicode-width` dependency
- 279 tests passing (up from 264)
- Zero clippy warnings

---

## ✅ Completed Features

### ✅ Inter-note Links Integration (v0.4.0)
- ✅ **DB Methods** - `get_backlinks()`, `delete_links_for_note()`, `find_note_by_title()`, `get_links_for_note()` (15 tests)
- ✅ **Link Following** - Detect link at cursor, resolve by title/ID, navigate (`Ctrl+]`/`Ctrl+[`)
- ✅ **Navigation History** - LIFO with max 50 entries, go back support
- ✅ **Backlinks Pane** - `B` toggle, right panel, scroll with `j/k`, auto-refresh
- ✅ **Autocomplete Wiring** - Tab/Enter accept, Up/Down navigate, insert mode only
- ✅ **Extract & Store Links** - Auto-parse on save, sync link graph in DB
- ✅ **Code Cleanup** - Removed dead_code allows, declared orphaned modules, cleaned imports

### ✅ Full Markdown Preview (v0.3.1)
- ✅ **Blockquotes** - Nested levels (up to 5), colored `│` prefix
- ✅ **Tables** - Unicode borders, alignment, bold headers
- ✅ **Task Lists** - `[x]`/`[ ]` checkbox rendering
- ✅ **Strikethrough** - `~~text~~` crossed-out
- ✅ **Footnotes** - Inline refs + definitions
- ✅ **Images** - `[IMG: alt]` placeholder
- ✅ **Wiki-links** - `[[target]]` / `[[target|label]]`
- ✅ **Code Blocks** - Box-drawing borders + gutter
- ✅ **Nested Lists** - Indentation + alternating bullets
- ✅ **Word Wrap** - Word-boundary aware
- ✅ **Headings** - `#` prefix indicators, H1 bg highlight
- ✅ **Smart Punctuation** - Ligatures and smart quotes

### ✅ Vim Editor Layer (v0.3.0)
- ✅ **Navigation** - `h/j/k/l`, arrows, `w/b`, `0/$`, `g/G`
- ✅ **Insert Mode** - `i/I/a/A/o/O`
- ✅ **Delete Ops** - `x/X`, `d` (dd), `D`
- ✅ **Yank/Paste** - `y` (yy), `p`
- ✅ **Undo/Redo** - `u`, `Ctrl+r`
- ✅ **Block Cursor** - Visual mode indicator

### ✅ Help Modal (v0.3.0)
- ✅ **Keybindings** - Organized by mode
- ✅ **CLI Commands** - Reference documentation
- ✅ **Credits** - Author, license, repo
- ✅ **Navigation** - Scroll and close bindings

### ✅ CLI Interface (v0.2.0)

| Comando | Descrição |
|---------|-----------|
| `ztlgr new <path>` | Cria vault com estrutura Zettelkasten completa |
| `ztlgr open [path]` | Abre vault existente na TUI |
| `ztlgr search <query>` | Busca notas via FTS5 |
| `ztlgr import <source>` | Importa notas de diretório |
| `ztlgr sync` | Sincroniza vault com database |
| `ztlgr --help` | Ajuda completa |
| `ztlgr --version` | Versão |

**Flags globais:**
- `--vault <path>` - Caminho padrão do vault (env: `ZTLGR_VAULT`)
- `-f, --format <fmt>` - Formato: `markdown` ou `org`
- `-c, --config <path>` - Arquivo de configuração (env: `ZTLGR_CONFIG`)
- `-v, --verbose` - Nível de verbosidade

**Comportamento:**
- Sem argumentos → Setup Wizard interativo (compatibilidade retroativa)
- Com subcomando → Executa comando CLI diretamente
- `--vault` funciona globalmente com qualquer comando

### 🧹 Code Quality

- 279 testes passando (16 novos testes CLI + 6 help modal)
- Zero warnings clippy (corrigidos 65+ warnings pré-existentes)
- Removidos stubs `src/bin/ztlgr-cli.rs` e `src/bin/ztlgr.rs`
- CLI unificado no `src/main.rs` via `src/cli.rs`

### 🐛 Bug Fixes (v0.1.1)

- ✨ **Real-time Markdown Preview** - See rendered markdown as you type
- 🐛 **Fixed UTF-8 crash** - Backspace/delete now handles accents and emojis
- 🐛 **Fixed line deletion bug** - No more deleting entire lines accidentally
- 🎨 **Improved markdown rendering** - Better headings, code blocks, lists, links
- 📏 **Text wrapping** - Proper word wrapping prevents overflow

---

## ✅ Completed Features

### ✅ CLI Interface (v0.2.0)
- ✅ **Command Parser** (clap derive, 5 subcommands, 16 tests)
- ✅ **`new` Handler** - Cria vault com estrutura completa
- ✅ **`open` Handler** - Abre vault e lança TUI
- ✅ **`search` Handler** - Busca via FTS5 com preview
- ✅ **`import` Handler** - Importa notas existentes
- ✅ **`sync` Handler** - Sincroniza DB <-> Files
- ✅ **Global Flags** - `--vault`, `--format`, `--config`, `--verbose`
- ✅ **Environment Variables** - `ZTLGR_VAULT`, `ZTLGR_CONFIG`

### ✅ Link System (v0.1.x)
- ✅ **Link Parsing** - Wiki/markdown/org formats (33 tests)
- ✅ **Link Validation & Highlighting** - Cyan for valid, red for invalid
- ✅ **Link Autocomplete** - Fuzzy matching (14 tests)
- ✅ **Link Following** - Navigation history (14 tests)
- ✅ **Backlinks Display** - Widget com scrolling (6 tests)

### ✅ Core Features (v0.1.x)
- ✅ **Editor** - Rope + undo/redo + copy/paste
- ✅ **Search Mode** - FTS5 integration + results nav
- ✅ **Command Mode** - Parser + executor (:rename, :move, :tag, :delete)
- ✅ **Modal System** - Delete confirm, note type selector, create flow
- ✅ **Help Modal** - All keybindings + CLI commands + credits (6 tests)
- ✅ **Soft Delete** - 7-day trash retention + recovery
- ✅ **Metadata Panel** - View/edit note properties
- ✅ **Markdown Preview** - Full GFM rendering (tables, blockquotes, task lists, footnotes, wiki-links, 58 tests)
- ✅ **UI/UX Polish** - Focus indicators, mode colors, theme consistency

### ✅ Infrastructure (v0.1.x)
- ✅ **Setup Wizard** - Interactive first-run configuration
- ✅ **Storage Layer** - Markdown + Org Mode
- ✅ **Database** - SQLite with FTS5
- ✅ **Theme System** - Dracula, Gruvbox, Nord, Solarized, Custom
- ✅ **File Watcher** - Detect external changes
- ✅ **Import System** - Import existing notes
- ✅ **File Sync** - Bidirectional DB <-> Files

---

## 🟠 PRÓXIMOS PASSOS

### Sprint Atual: Knowledge Graph Visualization (Phase 3)

- [ ] Design graph layout algorithm (force-directed or hierarchical)
- [ ] Implement ASCII/Unicode graph rendering in TUI
- [ ] Wire `Mode::Graph` with navigation (zoom, pan, select node)
- [ ] Show note connections from `links` table
- [ ] Graph filtering by note type, tags, or link depth

### Futuro:

- [ ] Search filters (by type/tags/status/date)
- [ ] Advanced CLI commands (`ztlgr note create`, `ztlgr export`)
- [ ] Notifications/toasts in TUI
- [ ] Sync status indicator
- [ ] Auto-backup system
- [ ] Note templates
- [ ] Daily notes auto-creation

---

## Como Testar

### Com Nix (Recomendado)

```bash
# Setup
direnv allow

# Run
cargo run
```

### Sem Nix

```bash
# Build
cargo build

# Run
cargo run
```

### CLI Commands

```bash
# Criar novo vault
ztlgr new ~/my-notes --format markdown

# Abrir vault
ztlgr open ~/my-notes

# Buscar notas
ztlgr search "rust zettelkasten" --vault ~/my-notes

# Importar notas existentes
ztlgr import ~/old-notes --vault ~/my-notes --recursive

# Sincronizar
ztlgr sync --vault ~/my-notes --force
```

---

## Arquitetura

```
┌─────────────────────────────────────────────┐
│                 TUI (Ratatui)                │
│  ┌──────────┬──────────────┬──────────────┐│
│  │ Sidebar  │    Editor    │   Preview    ││
│  │ (Notes)  │   (Vim-like)  │  (Markdown)  ││
│  └──────────┴──────────────┴──────────────┘│
└─────────────────────────────────────────────┘
                 ▲
                 │
┌─────────────────────────────────────────────┐
│              CLI (clap)                      │
│  new | open | search | import | sync        │
└─────────────────────────────────────────────┘
                 │
                 ▼
        ┌─────────────────────────┐
        │   Database Layer (DB)    │
        │  ┌───────────────────┐  │
        │  │   SQLite Index    │  │
        │  │  (FTS5 + Graph)   │  │
        │  └───────────────────┘  │
        └─────────────────────────┘
                     │
                     ▼
     ┌────────────────────────────────┐
     │   Storage Layer (Hybrid)       │
     │  ┌──────────────┬────────────┐│
     │  │  Files (MD)  │ Files (Org)││
     │  │  (Truth)     │  (Truth)   ││
     │  └──────────────┴────────────┘│
     └────────────────────────────────┘
                     │
                     ▼
     ┌────────────────────────────────┐
     │   File System                   │
     │   ~/vault/permanent/*.md        │
     │   ~/vault/inbox/*.md            │
     │   ~/vault/.ztlgr/vault.db       │
     └────────────────────────────────┘
```

---

**Status**: 🟢 Inter-note Links Integration Complete - Starting Knowledge Graph  
**Próximo**: Graph visualization, graph navigation, graph filtering.
