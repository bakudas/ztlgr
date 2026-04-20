# Status do Projeto ztlgr

**Data Atualização:** 20 de Abril de 2026  
**Versão:** 0.6.0 (LLM Wiki Integration Complete)
**Status Geral:** 🟢 ACTIVE DEVELOPMENT
**Testes:** 905 passing (100% success rate)

---

## 📊 RESUMO EXECUTIVO

### Progresso Geral
- ✅ **Infrastructure**: 100% (setup, DB, storage, themes)
- ✅ **Core Features**: 100% (editor, search, command, modals)
- ✅ **Link System**: 100% (parsing + validation + highlighting + autocomplete + following + backlinks + DB integration)
- ✅ **CLI Interface**: 100% (new, open, search, import, sync, index, ingest, ask, lint, init-skills)
- ✅ **Markdown Preview**: 100% (blockquotes, tables, task lists, footnotes, images, wiki-links)
- ✅ **Inter-note Links**: 100% (backlinks pane, link following, navigation history, autocomplete, extract & store)
- ✅ **Knowledge Graph**: 100% (force-directed layout, Canvas rendering, pan/zoom, node selection, navigation)
- ✅ **LLM Wiki Phase 0**: 100% (cleanup, roadmap, .skills/ schema)
- ✅ **LLM Wiki Phase 1**: 100% (index.md generation, activity log)
- ✅ **LLM Wiki Phase 2**: 100% (raw sources, ingest pipeline, schema migration)
- ✅ **LLM Wiki Phase 3**: 100% (.skills/ infrastructure, generator, init-skills CLI)
- ✅ **LLM Wiki Phase 4**: 100% (LLM provider trait, Ollama/OpenAI/Anthropic, context builder, usage tracker)
- ✅ **LLM Wiki Phase 5**: 100% (workflow engine, ingest/query/lint workflows, ask/lint CLI)
- ✅ **LLM Wiki Phase 6**: 100% (MCP server -- stdio transport, 9 tools, 67 tests)
- ✅ **Document Conversion**: 100% (PDF, DOCX, PPTX, XLSX, HTML, CSV, JSON, XML → Markdown)
- ✅ **Progress Indicators**: 100% (multi-stage progress, spinner animations)
- ✅ **LLM Post-Processor**: 100% (note validation, formatting fixes)
- ✅ **Extended LLM Providers**: 100% (Gemini, OpenRouter, NVIDIA)

---

## 🔄 RECENT IMPROVEMENTS (April 20, 2026)

### Search Flow Hardening (TUI + FTS)

Correções no fluxo completo de busca dentro da TUI (entrada -> recuperação -> abertura da nota):

**TUI Search UX** (`src/ui/app.rs`, `src/ui/widgets/search.rs`):
- Busca agora renderiza overlay dedicado com input + lista de resultados em `Mode::Search`
- `Enter` abre a nota selecionada (antes reexecutava busca e não navegava)
- Resultado aberto é sincronizado com `self.notes` para evitar inconsistência na navegação
- Truncamento de excerpt corrigido para UTF-8 (`chars().take(100)`) evitando panic por byte boundary
- Lista de resultados agora faz windowing por viewport (mantém item selecionado visível)

**Busca no DB / FTS5** (`src/db/schema.rs`):
- `search_notes()` tenta query FTS raw primeiro (mantém sintaxe avançada)
- Fallback robusto para prefix query sanitizada (`?mapr` -> `mapr*`) quando a entrada tem caracteres especiais
- Query vazia retorna vazio imediatamente
- Novo teste de regressão para `?mapr` + `mapreduce`

**Validação:**
- ✅ `cargo fmt --all`
- ✅ `cargo test --lib` (905 passing)
- ✅ `cargo clippy --all-features -- -D warnings`

---

## 🔄 RECENT IMPROVEMENTS (April 10, 2026)

### Progress Indicators for CLI

Multi-stage progress feedback during `--process` operations:

**New Module** (`src/progress.rs`):
- `ProcessProgress` — Multi-phase progress with spinner animations
- `ProcessingPhase` enum: ReadingSource, Converting, SendingToLLM, CreatingNote, UpdatingIndex
- `SimpleProgress` — Single-operation progress
- Visual feedback: `◐◓◑◒` spinners, success (`✓`) and error (`✗`) markers

**Integration** (`src/cli.rs`):
- Progress indicator shows current phase during LLM processing
- Clear status messages replace silent waiting
- Better UX for long-running operations

### Improved LLM Prompts

More concise literature notes with better structure:

**Changes** (`src/llm/workflows/ingest.rs`):
- Prompt now requests 200-400 words (down from unlimited)
- Explicit output STRUCTURE: Summary, Key Points, Notable Quotes, Connections
- Added CONSTRAINTS section: no introductions, no filler, be factual
- Default system prompt updated with quality guidelines

### LLM Post-Processor

Automatic formatting fixes for poor-quality model output:

**New Module** (`src/llm/post_processor.rs`):
- `LiteratureNoteProcessor::validate_and_fix()` — Ensures proper note structure
- Auto-adds frontmatter if missing (`type`, `source`)
- Normalizes wiki-links (fixes spacing, pipes)
- Removes excessive whitespace
- Caps length at ~1500 chars for verbose models

### New LLM Providers

Extended provider support for 6 backends total:

**Providers** (`src/llm/`):
| Provider | Models | Auth |
|-----------|--------|------|
| **Ollama** | llama3, mistral, codellama, etc. | None (local) |
| **OpenAI** | gpt-4o, gpt-4o-mini, o3, etc. | `OPENAI_API_KEY` |
| **Anthropic** | claude-sonnet-4, claude-haiku | `ANTHROPIC_API_KEY` |
| **Google Gemini** | gemini-2.0-flash, gemini-1.5-pro | `GOOGLE_API_KEY` |
| **OpenRouter** | 200+ models (aggregator) | `OPENROUTER_API_KEY` |
| **NVIDIA NIM** | meta/llama-3.1-8b, etc. | `NVIDIA_API_KEY` |

**Pricing Estimation** (`src/llm/usage.rs`):
- Added Gemini pricing (free tier for gemini-2, paid for 1.5-pro)
- OpenRouter/NVIDIA default to $0 (varies by model)

**Config** (`config.example.toml`):
- Updated with all provider examples
- Model recommendations for each provider
- Environment variable documentation

---

Multi-format document conversion for the LLM Wiki workflow:

**New Dependencies:**
- `anytomd v1.2` — Converts DOCX, PPTX, XLSX, HTML, CSV, JSON, XML, images to Markdown
- `pdf-extract v0.10` — Extracts text from PDF files
- `epub v2.1` — Parses EPUB ebooks for HTML extraction

**New Module** (`src/source/convert.rs`):
- `DocumentFormat` enum: PDF, EPUB, Generic (anytomd)
- `convert_to_markdown(path)` — Auto-detects format and converts to Markdown
- `convert_pdf()` — PDF text extraction
- `convert_epub()` — EPUB HTML extraction and conversion
- `convert_anytomd()` — Delegates to anytomd for all other formats
- `extract_text_from_html()` — HTML to plain text (for EPUB)
- `parse_entity()` — HTML entity decoder
- 16 unit tests covering format detection, conversion, and HTML parsing

**Integration:**
- Modified `read_source_content()` in `src/llm/workflow.rs` to auto-convert non-Markdown files
- Markdown and text files pass through unchanged
- All other formats are converted before LLM processing

**Supported Formats:**
| Format | Converter | Notes |
|--------|-----------|-------|
| PDF | pdf-extract | Text extraction |
| EPUB | epub + anytomd | HTML extraction |
| DOCX | anytomd | Full support |
| PPTX | anytomd | Full support |
| XLSX/XLS | anytomd | Full support |
| HTML/HTM | anytomd | Full support |
| CSV | anytomd | Converted to Markdown tables |
| JSON/XML | anytomd | Pretty-printed in code blocks |
| Images | anytomd | Optional LLM-based description |
| Code files | anytomd | Fenced code blocks with language ID |

---

## 🚀 LATEST RELEASE: v0.6.0

**Release Date:** April 10, 2026  
**Release Tag:** v0.6.0

**What's New in v0.6.0:**

### ✨ LLM Wiki Integration (Phases 0-6)

Complete implementation of the "LLM Wiki" pattern where LLM agents maintain the knowledge base:

- **MCP Server** (Phase 6)
  - JSON-RPC 2.0 over stdio
  - 9 tools: search, get_note, list_notes, create_note, get_backlinks, ingest_source, read_index, read_log, read_skills
  - Full lifecycle: initialize → operation → shutdown
  - CLI: `ztlgr mcp`

- **LLM Workflows** (Phase 5)
  - `IngestWorkflow::process()` - automates literature note creation from sources
  - `QueryWorkflow::ask()` - queries grimoire with wiki-link citations
  - `LintWorkflow` - local lint (orphan notes, short notes) + full lint (LLM-assisted contradictions)
  - CLI: `ztlgr ingest --process`, `ztlgr ask "<question>"`, `ztlgr lint [--full]`

- **LLM Providers** (Phase 4)
  - Abstract `LlmProvider` trait with async `complete()`
  - 6 backends: Ollama (local), OpenAI, Anthropic, Google Gemini, OpenRouter, NVIDIA NIM
  - `ContextBuilder` builds prompts from `.skills/`
  - `UsageTracker` estimates costs per model

- **Skills Infrastructure** (Phase 3)
  - `.skills/` directory with LLM agent schema and prompts
  - `ztlgr init-skills` - generate/validate skills filesystem
  - 12 content generators for agent instructions

- **Raw Sources Layer** (Phase 2)
  - `raw/` directory for immutable source material
  - `sources` table with SHA-256 deduplication
  - Schema migration v1 → v2
  - CLI: `ztlgr ingest <file>`

- **Index & Log System** (Phase 1)
  - Auto-generated `index.md` with grouped notes
  - Append-only `log.md` activity log
  - CLI: `ztlgr index`

- **Progress Indicators**
  - Multi-stage spinners for CLI operations
  - Visual feedback: ReadingSource → Converting → SendingToLLM → CreatingNote → UpdatingIndex

- **Document Conversion**
  - PDF/EPUB/DOCX/PPTX/XLSX/HTML/CSV/JSON/XML → Markdown
  - Dependencies: anytomd, pdf-extract, epub

- **Post-Processor**
  - Auto-fixes LLM output formatting
  - Frontmatter normalization, wiki-link fixes

### 🔧 Technical Changes

- Bumped version from 0.5.0 to 0.6.0
- 904 tests passing (+481 from v0.5.0)
- New modules: `src/llm/`, `src/skills/`, `src/source/`, `src/mcp/`, `src/progress/`
- New dependencies: reqwest, regex, anytomd, pdf-extract, epub, indicatif
- Database migration v1 → v2 (sources table)
- Zero clippy warnings

---

## 🚀 PREVIOUS RELEASE: v0.5.0

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
| `ztlgr ingest --process` | Ingere + processa com LLM (gera nota de literatura) |
| `ztlgr ask "<question>"` | Consulta o grimoire via LLM |
| `ztlgr lint [--full]` | Lint local (sem LLM) ou completo (com LLM) |
| `ztlgr mcp` | Inicia MCP server (JSON-RPC over stdio) |
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

### Post-Release (v0.6.0)
- [ ] Monitor GitHub Actions build
- [ ] Monitor crates.io publish
- [ ] Update README with LLM features
- [ ] Create GitHub release notes from CHANGELOG
- [ ] Announce release (Discord, Reddit, Twitter/Mastodon)

### Future Enhancements (baseado em LLM Wiki Community)
- [ ] Confidence-tagged claims no frontmatter
- [ ] WIP.md para continuidade de sessão
- [ ] Link resolution at write time
- [ ] Progressive disclosure no MCP (search_brief)
- [ ] Contradiction detection no lint
- [ ] Auto-pruning / decay system

### Backlog
- [ ] Graph filtering by note type, tags, or link depth
- [ ] Search filters (by type/tags/status/date)
- [ ] Paginação real da busca (DB: `limit+offset+count`, TUI: page state + keybindings + total)
- [ ] Note templates
- [ ] Daily notes auto-creation

### Sprint Foco (Busca TUI)
- [ ] Implementar paginação incremental na busca (sem quebrar API atual)
- [ ] Exibir score/snippet real do FTS5 (`snippet()` + `bm25`) na lista de resultados
- [ ] Adicionar testes de integração do fluxo `/` -> digitar -> selecionar -> abrir nota

---

**Status**: 🟢 v0.6.0 Released - LLM Wiki Integration Complete  
**Próximo Release**: v0.7.0 (Future Enhancements)

---

## Future Enhancements (baseado em LLM Wiki Community)

Conceitos identificados nos comentários do gist LLM Wiki que podem enriquecer o ztlgr:

### P1: Confidence-Tagged Claims (Alto Valor, Baixa Complexidade)

**Problema:** Contradições entre notas são difíceis de detectar automaticamente.

**Solução:** Adicionar campo `confidence` no YAML frontmatter:
```yaml
---
title: "Rails Performance Guide"
confidence: high    # high | medium | low
last_reviewed: 2026-04-09
---
```

**Implementação:**
- [ ] Adicionar `confidence` ao `Note` struct
- [ ] Modifier `lint --full` para detectar contradições
- [ ] Query MCP: `get_contradictions()` — WHERE confidence=high AND claims conflict

**Benefício:** Torna audit do wiki semi-automático em vez de fuzzy re-read.

### P2: WIP.md para Continuidade de Sessão (Alto Valor, Média Complexidade)

**Problema:** `log.md` registra ações completadas, mas pensamentos em progresso desaparecem entre sessões.

**Solução:** Criar `wip.md` (Work In Progress):
```markdown
# Work in Progress

## [2026-04-09] Explorando: async Rust patterns
Question: How does tokio::select! handle cancellation?
Status: investigating
Related: [[Rust Concurrency]], [[Tokio Internals]]
Sources: raw/tokio-docs.md

## [2026-04-08] Thesis: ML architectures for NLP
Claim: Transformers are limited by quadratic attention
Confidence: medium
Evidence: [[Attention Paper]], [[Efficient Transformers]]
Counter-evidence: [[Linear Attention Variants]]
```

**Implementação:**
- [ ] Criar `.ztlgr/wip.md` durante `Vault::initialize()`
- [ ] Comando `ztlgr wip` para gerenciar (add/list/complete)
- [ ] Integrar com Query: LLM lê WIP para contexto

**Benefício:** Sessões de pesquisa são retomáveis; perguntas em aberto não se perdem.

### P3: Link Resolution at Write Time (Alto Valor, Média Complexidade)

**Problema:** `[[Wiki Links]]` são texto. LLM pode criar links para notas inexistentes → 404.

**Solução:** Validar e resolver links no momento da escrita:
```rust
// Ao criar nota:
let content = NoteBuilder::new()
    .title("Summary")
    .wiki_link("Rust Async Patterns", &db)?  // Valida contra DB
    .wiki_link_or_create("New Concept", &db)? // Cria stub se não existe
    .build();
```

**Implementação:**
- [ ] `LinkValidator::validate_all_links()` retorna lista de broken links
- [ ] `NoteBuilder` API para construir notas com links validados
- [ ] MCP tool `validate_links` — retorna broken links

**Benefício:** Elimina hallucinated links; wiki integrity garantida.

### P4: Progressive Disclosure no MCP (Médio Valor, Baixa Complexidade)

**Problema:** `search` retorna conteúdo completo → contexto desperdiçado.

**Solução:** Nova tool MCP `search_brief`:
```json
{
  "name": "search_brief",
  "arguments": {
    "query": "async patterns",
    "limit": 10
  }
}
// Returns: [{title, type, date, confidence, snippet(50 chars)}]
```

**Implementação:**
- [ ] Adicionar tool `search_brief` ao MCP
- [ ] Usar `snippet` field do FTS5 para extração rápida

**Benefício:** Agent pode escolher quais notas expandir; <400 tokens vs 4000.

### P5: Contradiction Detection no Lint (Médio Valor, Média Complexidade)

**Problema:** Contradições são detectadas manualmente pelo LLM no `lint --full`.

**Solução:** Detecção determinística:
```rust
// No lint:
// 1. Extrair claims com confidence=high
// 2. Comparar com outros claims sobre mesmo tópico
// 3. Flag contradições
// Ex: "Raft is simpler than Paxos" vs "Raft and Paxos have similar complexity"
```

**Implementação:**
- [ ] Adicionar `claim_extractor` no lint
- [ ] Comparação semântica via embeddings ou heurística simples
- [ ] Output: `## Contradictions` no lint report

**Benefício:** Audit automático de contradições sem LLM chamar.

### P6: Auto-Pruning / Decay (Baixo Valor, Alta Complexidade)

**Problema:** Wiki cresce indefinidamente; notas antigas ficam desatualizadas.

**Solução:** Sistema de decay:
```yaml
---
last_reviewed: 2026-01-15
importance: high    # high | medium | low
decay_rate: 30      # days until stale
---
```

**Implementação:**
- [ ] Adicionar campos ao `Note` struct
- [ ] `ztlgr lint --stale` lista notas que precisam review
- [ ] Workflow sugere notas antigas importante para review

**Benefício:** Wiki permanece atual; conhecimento stale é sinalizado.

---

## Implementation Plan (Pós-Phase 6)

### Sprint 1: Confidence & WIP (Coleção de Feedback)

| Prioridade | Feature | Estimativa | Pré-requisitos |
|------------|---------|------------|----------------|
| P1 | Confidence no frontmatter | 1 dia | None |
| P1 | Contradições no lint | 2 dias | Confidence |
| P2 | WIP.md creation | 1 dia | None |
| P2 | `ztlgr wip` commands | 2 dias | WIP.md |
| P3 | Link validation at write | 2 dias | None |
| P4 | `search_brief` tool | 1 dia | MCP Server |

**Total:** ~9 dias

### Sprint 2: Integration & Polish

| Prioridade | Feature | Estimativa | Pré-requisitos |
|------------|---------|------------|----------------|
| P3 | `NoteBuilder` API | 2 dias | None |
| P3 | `validate_links` tool | 1 dia | NoteBuilder |
| P5 | Contradiction detection | 3 dias | Confidence |
| P5 | Embeddings support (opcional) | 2 dias | None |

**Total:** ~8 dias

### Sprint 3: Advanced Features

| Prioridade | Feature | Estimativa | Pré-requisitos |
|------------|---------|------------|----------------|
| P6 | Decay system | 2 dias | None |
| P6 | `ztlgr stale` command | 1 dia | Decay |
| - | Multi-vault support | 3 dias | None |
| - | Cloud sync (opcional) | 5 dias | None |

**Total:** ~11 dias

### Critérios de Priorização

1. **Valor para o padrão LLM Wiki:** P1/P2 são críticos para o ciclo ingest → query → lint.
2. **Feedback da comunidade:** WIP.md foi mencionado por múltiplos implementadores.
3. **Complexidade:** Confidence é trivial de adicionar; embeddings é complexo.
4. **Dependencies:** Link validation é independente; decay precisa de infra.

---

## Como Testar Versões Futuras

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

# Consultar o grimoire via LLM
ztlgr ask "What is the Zettelkasten method?" --vault ~/my-notes

# Lint local (sem LLM) ou completo (com LLM)
ztlgr lint --vault ~/my-notes
ztlgr lint --full --vault ~/my-notes

# Ingerir + processar com LLM (gera nota de literatura)
ztlgr ingest ~/papers/article.pdf --process --vault ~/my-notes
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
│  ingest | init-skills | ask | lint                │
└─────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│        LLM Workflow Layer (Phase 5)              │
│  ┌──────────┬──────────┬────────────────┐   │
│  │  Ingest  │  Query   │     Lint       │   │
│  │ Workflow │ Workflow  │   Workflow     │   │
│  └──────────┴──────────┴────────────────┘   │
│        WorkflowEngine (orchestrator)             │
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

**Status**: 🟢 v0.5.0 Released - LLM Wiki Phase 6 Complete (MCP Server)  
**Próximo**: Merge to main, release v0.6.0 with full LLM Wiki integration (Phases 0-6 complete).
