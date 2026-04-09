# Status do Projeto ztlgr

**Data Atualização:** 9 de Abril de 2026  
**Versão:** 0.5.0 (Knowledge Graph Visualization)  
**Status Geral:** 🟢 ACTIVE DEVELOPMENT  
**Testes:** 727 passing (100% success rate)

---

## 📊 RESUMO EXECUTIVO

### Progresso Geral
- ✅ **Infrastructure**: 100% (setup, DB, storage, themes)
- ✅ **Core Features**: 100% (editor, search, command, modals)
- ✅ **Link System**: 100% (parsing + validation + highlighting + autocomplete + following + backlinks + DB integration)
- ✅ **CLI Interface**: 100% (new, open, search, import, sync)
- ✅ **Markdown Preview**: 100% (blockquotes, tables, task lists, footnotes, images, wiki-links)
- ✅ **Inter-note Links**: 100% (backlinks pane, link following, navigation history, autocomplete, extract & store)
- ✅ **Knowledge Graph**: 100% (force-directed layout, Canvas rendering, pan/zoom, node selection, navigation)

---

## 🚀 LATEST RELEASE: v0.5.0

**Release Date:** April 7, 2026  
**Release Tag:** v0.5.0

**What's New in v0.5.0:**

### ✨ Knowledge Graph Visualization

Interactive visual graph of note connections using ratatui Canvas with Braille markers:

- **Graph Module** (`src/graph/`):
  - `types.rs` — `GraphNode`, `GraphEdge`, `GraphData` with `from_db()` builder (11 tests)
  - `layout.rs` — Fruchterman-Reingold force-directed layout algorithm with centering and bounding box (11 tests)

- **Database Layer** (2 new methods + 13 tests):
  - `get_all_links()` — All edges where both endpoints are non-deleted
  - `get_graph_nodes()` — All non-deleted notes with outgoing/incoming link counts

- **GraphView Widget** (`src/ui/widgets/graph_view.rs`):
  - `GraphState` — Pan, zoom, node selection, fit-to-view
  - `draw_graph()` — Canvas-based rendering with edges (lines), nodes (circles), labels (text)
  - Node color by type (daily/fleeting/literature/permanent/reference/index), selected node highlighted
  - Node size scales with degree (number of connections)
  - Empty state message when no notes exist (16 tests)

- **Graph Mode** (`v` to enter, `q`/`Esc` to exit):
  - `h/j/k/l` or arrow keys: pan view
  - `+`/`=`: zoom in, `-`: zoom out
  - `Tab`/`Shift+Tab`: cycle through nodes (auto-centers)
  - `Enter`: navigate to selected note (exits graph mode)
  - `c`: center on selected node
  - `f`: fit entire graph in view
  - Sidebar remains visible for context

- **Help Modal Updated** with Graph Mode keybindings section

### 🔧 Technical Changes

- Bumped version from 0.4.0 to 0.5.0
- 423 tests passing (+51 new tests, up from 372)
- New `src/graph/` module: `types.rs`, `layout.rs`, `mod.rs`
- New `src/ui/widgets/graph_view.rs` widget
- `draw()` in `app.rs` now branches between normal layout and graph layout
- `handle_graph_mode()` expanded from 3 lines to full keybinding handler
- `enter_graph_mode()` loads data from DB, runs layout, creates `GraphState`
- Zero clippy warnings

---

## 🚀 PREVIOUS RELEASE: v0.4.0

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

- **Link Following** (`Enter` to follow, `Ctrl+O` to go back):
  - Detects wiki-style `[[target]]`/`[[target|label]]` and markdown-style `[label](target)` at cursor position
  - Resolves targets by title (case-insensitive) or note ID
  - Opens external URLs with status bar message
  - Navigation history with LIFO ordering (max 50 entries)

- **Backlinks Pane** (`B` to toggle):
  - Shows all notes linking to the current note
  - Displayed as footer in the preview panel (70/30 split)
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
- 372 tests passing (+35 new tests, up from 337)
- Declared orphaned modules (`link_following`, `navigation_history`) in `ui/mod.rs`
- Removed blanket `#![allow(dead_code)]` from `backlinks_pane.rs` and `link_autocomplete.rs`
- Cleaned up unused imports in `widgets/mod.rs`
- Added `get_current_line()` and `cursor_col()` to `NoteEditor` for cursor context
- Fixed autocomplete to insert note title instead of UUID, and delete `[[` prefix to avoid double brackets
- Replaced impossible `Ctrl+]`/`Ctrl+[` keybindings with `Enter`/`Ctrl+O` (terminal-compatible)
- Moved backlinks from separate panel mode to preview footer (70/30 split)

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

### ✅ Knowledge Graph Visualization (v0.5.0)
- ✅ **Graph Data Layer** - `get_all_links()`, `get_graph_nodes()` DB methods (13 tests)
- ✅ **Graph Types** - `GraphNode`, `GraphEdge`, `GraphData` with `from_db()` builder (11 tests)
- ✅ **Force-Directed Layout** - Fruchterman-Reingold algorithm with centering (11 tests)
- ✅ **GraphView Widget** - Canvas-based rendering with Braille markers (16 tests)
- ✅ **Graph Mode** - Full keybindings: pan, zoom, select, navigate, center, fit
- ✅ **Help Modal** - Graph Mode keybindings section added

### ✅ Inter-note Links Integration (v0.4.0)
- ✅ **DB Methods** - `get_backlinks()`, `delete_links_for_note()`, `find_note_by_title()`, `get_links_for_note()` (15 tests)
- ✅ **Link Following** - Detect link at cursor, resolve by title/ID, navigate (`Enter`/`Ctrl+O`)
- ✅ **Navigation History** - LIFO with max 50 entries, go back support
- ✅ **Backlinks Pane** - `B` toggle, preview footer (70/30 split), auto-refresh
- ✅ **Autocomplete Wiring** - Tab/Enter accept, Up/Down navigate, insert mode only
- ✅ **Autocomplete Fix** - Inserts note title (not UUID), no double brackets
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
| `ztlgr new <path>` | Cria grimoire com estrutura Zettelkasten completa |
| `ztlgr open [path]` | Abre grimoire existente na TUI |
| `ztlgr search <query>` | Busca notas via FTS5 |
| `ztlgr import <source>` | Importa notas de diretório |
| `ztlgr sync` | Sincroniza grimoire com database |
| `ztlgr index` | Gera/atualiza index.md do grimoire |
| `ztlgr ingest <file>` | Ingere arquivo fonte no `raw/` |
| `ztlgr init-skills` | Gera/valida `.skills/` no grimoire |
| `ztlgr --help` | Ajuda completa |
| `ztlgr --version` | Versão |

**Flags globais:**
- `--vault <path>` - Caminho padrão do vault (env: `ZTLGR_VAULT`)
- `-f, --format <fmt>` - Formato: `markdown` ou `org`
- `-c, --config <path>` - Arquivo de configuração (env: `ZTLGR_CONFIG`)
- `-v, --verbose` - Nível de verbosidade
- `--no-git` - Não inicializar repositório git (apenas `ztlgr new`)
- `--no-skills` - Não gerar `.skills/` (apenas `ztlgr new`)

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

### Nova Direção: LLM Wiki Integration

> **Branch:** `feat/llm-wiki-integration`
> **Roadmap completo:** `docs/ROADMAP-LLM-WIKI.md`

Evolução do ztlgr para suportar o padrão "LLM Wiki" -- onde agentes LLM
mantêm incrementalmente a base de conhecimento (cross-references, summaries,
entity pages) ao invés de re-derivar conhecimento a cada query.

### Phase 0: Cleanup & Foundation (em andamento)
- [x] Remover referências aspiracionais a "multi-agent" (README, CONTRIBUTING, CHANGELOG)
- [x] Criar `.skills/` -- schema e workflows para agentes LLM
- [x] Criar `docs/ROADMAP-LLM-WIKI.md` com plano de ação
- [x] Atualizar STATUS.md com nova direção

### ✅ Phase 1: Index & Log System (+53 tests, 486 total)
- [x] `index.md` auto-gerado a partir do DB (agrupado por tipo, one-line summaries)
- [x] `log.md` activity log append-only (sync, create, delete, import, index)
- [x] `ztlgr index` CLI command
- [x] `src/storage/index_generator.rs` (20 tests)
- [x] `src/storage/activity_log.rs` (16 tests)
- [x] DB helpers: `count_notes_by_type()`, `count_notes()`, `count_links()`, `list_notes_by_type()` (14 tests)
- [x] CLI: `ztlgr index --vault <path>` (3 tests)
- [x] Hooks: `ztlgr sync --force` regenerates index + writes activity log
- [x] Hooks: `ztlgr import` writes activity log

### ✅ Phase 2: Raw Sources Layer (+66 tests, 552 total)
- [x] Diretório `raw/` criado durante vault init para fontes imutáveis
- [x] Tabela `sources` no DB (id, title, origin, hash, file_path, file_size, mime_type, ingested_at)
- [x] Schema migration v1 → v2 (`migration_v2.sql`, auto-applied on DB open)
- [x] `src/source/mod.rs` — `Source` struct, `SourceId`, builder pattern (9 tests)
- [x] `src/source/ingest.rs` — `Ingester` with SHA-256 dedup, copy to `raw/`, DB + log integration (~15 tests)
- [x] `src/db/schema.rs` — `get_schema_version()`, `migrate()`, 6 source CRUD methods (~20 tests)
- [x] `src/storage/index_generator.rs` — Sources section in index, `format_file_size()` (7 new tests)
- [x] `src/storage/activity_log.rs` — `Ingest` activity kind, `log_ingest()` (1 new test)
- [x] `ztlgr ingest <file> [--title <name>]` CLI command (6 tests)
- [x] Error variants: `SourceNotFound`, `Ingest`, `Migration`
- [x] `sha2` crate added for content hashing

### ✅ Phase 3: .skills/ Infrastructure (+53 tests, 605 total)
- [x] `src/skills/mod.rs` — `Skills` struct, `ValidationReport`, loader, file readers, `list_files()` (17 tests)
- [x] `src/skills/generator.rs` — `SkillsGenerator`, 12 content generators, `GenerateResult` with created/skipped (28 tests)
- [x] `ztlgr init-skills --vault <path>` CLI command (validates, fills missing files)
- [x] `--no-skills` flag for `ztlgr new` (skip .skills/ generation)
- [x] Default .skills/ generation during `ztlgr new` and setup wizard
- [x] `prompt_init_skills()` in setup wizard (Y/n prompt)
- [x] Error variant: `Skills(String)`
- [x] CLI tests: skills by default, skip with flag, vault name in skills, init-skills success, nonexistent vault, idempotent, fills missing files (7 new tests)
- [x] Help modal updated with `init-skills` command

### ✅ Phase 4: LLM Provider Abstraction (+122 tests, 727 total)
- [x] `src/llm/provider.rs` — `LlmProvider` trait (`Pin<Box<dyn Future>>` for object safety), `Role`, `Message`, `LlmRequest` (builder), `LlmResponse`, `TokenUsage` (16 tests)
- [x] `src/llm/mod.rs` — `ProviderKind` enum (Ollama, OpenAi, Anthropic) with `FromStr`, `create_provider()` factory (17 tests)
- [x] `src/llm/ollama.rs` — `OllamaProvider` with Ollama Chat API format, local-first (11 tests)
- [x] `src/llm/openai.rs` — `OpenAiProvider` with OpenAI Chat Completions API (14 tests)
- [x] `src/llm/anthropic.rs` — `AnthropicProvider` with Anthropic Messages API (system as top-level field) (18 tests)
- [x] `src/llm/context.rs` — `ContextBuilder` loads `.skills/` files, builds system prompts, estimates tokens (18 tests)
- [x] `src/llm/usage.rs` — `UsageTracker` per-model cost estimation, activity log integration, hardcoded pricing (26 tests)
- [x] `src/config/settings.rs` — `LlmConfig` struct (enabled, provider, model, api_base, api_key_env, max_tokens, temperature) with `#[serde(default)]` (4 tests)
- [x] `config.example.toml` — `[llm]` section with full documentation
- [x] `src/storage/activity_log.rs` — `Llm` variant in `ActivityKind`
- [x] `Cargo.toml` — Added `reqwest` 0.12 with `rustls-tls`
- [x] Error variants: `Llm(String)`, `LlmProvider(String)`
- [x] API keys read from env vars, NEVER stored in config files
- [x] Ollama is default (local-first, no API key needed)

### Phase 5: LLM Workflows (Ingest, Query, Lint)
- [ ] `:ingest` / `ztlgr ingest --process`
- [ ] `:ask` / `ztlgr ask "<question>"`
- [ ] `:lint` / `ztlgr lint`

### Phase 6: MCP Server
- [ ] `ztlgr mcp` -- expõe vault como MCP tools
- [ ] Tools: search, get_note, create_note, get_backlinks, ingest

### Backlog (mantido do sprint anterior)
- [ ] Graph filtering by note type, tags, or link depth
- [ ] Search filters (by type/tags/status/date)
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
# Criar novo grimoire
ztlgr new ~/my-notes --format markdown

# Abrir grimoire
ztlgr open ~/my-notes

# Buscar notas
ztlgr search "rust zettelkasten" --vault ~/my-notes

# Importar notas existentes
ztlgr import ~/old-notes --vault ~/my-notes --recursive

# Sincronizar (regenera index + activity log)
ztlgr sync --vault ~/my-notes --force

# Gerar/atualizar index
ztlgr index --vault ~/my-notes

# Ingerir arquivo fonte (copia para raw/, registra no DB)
ztlgr ingest ~/papers/article.pdf --vault ~/my-notes
ztlgr ingest ~/papers/article.pdf --title "My Article" --vault ~/my-notes

# Gerar/validar .skills/ (preenche arquivos faltantes)
ztlgr init-skills --vault ~/my-notes
```

---

## Arquitetura

```
┌─────────────────────────────────────────────────┐
│                  TUI (Ratatui)                   │
│  ┌──────────┬──────────────┬──────────────────┐ │
│  │ Sidebar  │    Editor    │    Preview       │ │
│  │ (Notes)  │  (Vim-like)  │   (Markdown)     │ │
│  │          │              ├──────────────────┤ │
│  │          │              │   Backlinks (B)  │ │
│  └──────────┴──────────────┴──────────────────┘ │
│  ┌──────────┬─────────────────────────────────┐ │
│  │ Sidebar  │  Graph View (v) — Canvas/Braille│ │
│  │ (Notes)  │  Force-directed layout          │ │
│  └──────────┴─────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
                 ▲
                 │
┌─────────────────────────────────────────────┐
│              CLI (clap)                          │
│  new | open | search | import | sync | index     │
│  ingest | init-skills                             │
└─────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│         LLM Provider Layer (Phase 4)             │
│  ┌─────────┬──────────┬──────────────────┐  │
│  │ Ollama  │  OpenAI  │   Anthropic      │  │
│  │ (local) │  (cloud) │   (cloud)        │  │
│  └─────────┴──────────┴──────────────────┘  │
│  ContextBuilder (loads .skills/) │ UsageTracker │
└─────────────────────────────────────────────┘
                 │
                 ▼
        ┌─────────────────────────┐
        │   Database Layer (DB)    │
        │  ┌───────────────────┐  │
        │  │   SQLite Index    │  │
        │  │  (FTS5 + Graph)   │  │
        │  │  + Sources (v2)   │  │
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
     ┌────────────────────────────────────────┐
     │   File System                          │
     │   ~/vault/permanent/*.md               │
     │   ~/vault/inbox/*.md                   │
     │   ~/vault/raw/* (immutable sources)    │
     │   ~/vault/.skills/* (LLM agent schema) │
     │   ~/vault/.ztlgr/vault.db              │
     └────────────────────────────────────────┘
```

---

**Status**: 🟢 v0.5.0 Released - LLM Wiki Phase 4 Complete (LLM Provider Abstraction)  
**Próximo**: Phase 5 - LLM Workflows (`:ingest --process`, `:ask`, `:lint` commands).
