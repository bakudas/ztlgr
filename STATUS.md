# Status do Projeto ztlgr

**Data Atualização:** 5 de Abril de 2026  
**Versão:** 0.2.0 (CLI + TUI unificada 🚀)  
**Status Geral:** 🟢 ACTIVE DEVELOPMENT  
**Testes:** 264 passing (100% success rate)

---

## 📊 RESUMO EXECUTIVO

### Progresso Geral
- ✅ **Infrastructure**: 100% (setup, DB, storage, themes)
- ✅ **Core Features**: 100% (editor, search, command, modals)
- ✅ **Link System**: 100% (parsing + validation + highlighting + autocomplete + following + backlinks)
- ✅ **CLI Interface**: 100% (new, open, search, import, sync)
- ✅ **Distribution**: 100% (CI/CD, release workflow, documentation)

---

## 🚀 LATEST RELEASE: v0.2.0

**Release Date:** April 5, 2026  
**Release Tag:** v0.2.0

**What's New in v0.2.0:**

### ✨ CLI Interface Completa

Subcomandos implementados com `clap` derive:

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

- 264 testes passando (16 novos testes CLI)
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
- ✅ **Soft Delete** - 7-day trash retention + recovery
- ✅ **Metadata Panel** - View/edit note properties
- ✅ **Markdown Preview** - Rendered preview pane
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

### Semana 3-4: Advanced Features

- [ ] Graph visualization (ASCII art knowledge graph)
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

**Status**: 🟢 CLI Complete - Ready for v0.2.0 release!  
**Próximo**: Graph visualization, search filters, advanced commands.
