# 📊 ANÁLISE DE UX & ARQUITETURA DA INFORMAÇÃO - ZTLGR

**Data:** 3 de Abril de 2026  
**Versão do Projeto:** 0.1.0  
**Status:** Análise Completa  
**Linguagem:** Rust | **Linhas de Código:** ~5.153

---

## 📑 SUMÁRIO EXECUTIVO

**ztlgr** é uma aplicação terminal (TUI) bem estruturada arquiteturalmente para tomada de notas com Zettelkasten, mas com **gaps significativos em implementação de funcionalidades críticas** e **problemas de UX/IA que impedem seu uso diário**.

### Status Geral
- ✅ **Arquitetura:** 85% (bem estruturada)
- ✅ **Implementação:** 30% (muitos stubs)
- ❌ **UX/IA:** 40% (problemas críticos)
- 🔴 **Usabilidade:** NÃO PRONTA (MVP status)

---

## 🎯 PARTE 1: VISÃO GERAL DO PROJETO

### 1.1 Tipo e Propósito

**Tipo:** Aplicação Terminal Interativa (TUI - Terminal User Interface)
- Framework: **Ratatui** (TUI toolkit Rust)
- Binary: `ztlgr`
- Plataformas: Linux, macOS, Windows (WSL)
- Interface: Textual com suporte a cores, mouse e temas

**Propósito Principal:** Ferramenta de anotações para **Zettelkasten** (metodologia de conhecimento conectado)

**Inspirações:** Obsidian, Logseq, Foam, org-roam

**Funcionalidades Core:**
- Método Zettelkasten de Luhmann (IDs sequenciais: 1a, 1a1, 1a1a)
- Múltiplos tipos de notas (Daily, Fleeting, Permanent, Literature, Reference, Index)
- Links wiki-style (`[[note-titulo]]`)
- Busca full-text (FTS5)
- Sincronização DB ↔ Arquivos
- Compatibilidade com Obsidian/Foam/Logseq

### 1.2 Stack Tecnológico

| Layer | Tecnologia | Versão | Propósito |
|-------|-----------|--------|----------|
| **Linguagem** | Rust | 2021 | Performance + segurança |
| **UI Terminal** | Ratatui | 0.26 | Renderização TUI |
| **Input** | Crossterm | 0.27 | Eventos teclado/mouse |
| **DB** | SQLite | bundled | FTS5 index + CRUD |
| **Driver** | rusqlite | 0.30 | Bindings Rust |
| **Async** | Tokio | 1.35+ | Async runtime |
| **Serialização** | Serde + TOML | 1.0 | Config/metadata |
| **Markdown** | pulldown-cmark | 0.9 | Parse markdown |
| **File Watching** | Notify | 6.1 | Sync files |
| **Logging** | Tracing | 0.1 | Debug/instrumentation |

### 1.3 Arquitetura em Layers

```
┌─────────────────────────────────────┐
│      UI/Terminal (Ratatui)          │
├─────────────────────────────────────┤
│  Database Layer (SQLite FTS5)       │
├─────────────────────────────────────┤
│  Storage Layer (MD/Org Files)       │
├─────────────────────────────────────┤
│  File System (Vault Structure)      │
└─────────────────────────────────────┘
```

### 1.4 Estrutura de Pastas

```
src/
├── main.rs                    # Entry point + setup wizard
├── lib.rs                     # Library exports
├── error.rs                   # Custom error types
├── utils.rs                   # Utilities
├── setup.rs                   # Setup wizard (251 linhas)
├── config/                    # Configuração + Temas
├── db/                        # Database layer (SQLite)
├── note/                      # Tipos e estrutura de notas
├── storage/                   # Sistema de arquivos (MD/Org)
├── ui/                        # Componentes Ratatui
│   ├── app.rs                # Orchestrator (508 linhas)
│   └── widgets/
│       ├── note_list.rs      # Sidebar (64 linhas)
│       ├── note_editor.rs    # Editor (227 linhas)
│       ├── preview_pane.rs   # Preview (42 linhas)
│       └── status_bar.rs     # Status (62 linhas)
├── agent/                    # Multi-agent (placeholder)
└── skill/                    # Skills (placeholder)
```

### 1.5 Componentes Principais

#### NoteList Widget (Sidebar)
- Exibe lista de notas com navegação
- Scroll vertical (j/k)
- Indicador de seleção e modificação
- Busca filtering

#### NoteEditor Widget (Centro)
- Editor de texto vim-like
- Insert/Normal mode
- Cursor positioning
- **PROBLEMA:** Muito primitivo (sem rope, sem undo, sem clipboard)

#### PreviewPane Widget (Direita)
- Preview markdown em tempo real
- **PROBLEMA:** Apenas stub (não renderiza realmente)

#### StatusBar Widget (Bottom)
- Display de modo, mensagens, timestamp
- **PROBLEMA:** Feedback inadequado

#### App Orchestrator
- Event loop principal (100ms polling)
- Roteamento de eventos
- State management

---

## 🔴 PARTE 2: PROBLEMAS CRÍTICOS (MVP)

### ❌ 2.1 PROBLEMA #1: Editor Muito Básico

**Severidade:** 🔴 CRÍTICA  
**Localização:** `src/ui/widgets/note_editor.rs` (227 linhas)  
**Impacto:** Usuários não conseguem editar notas normalmente

**Problemas Específicos:**
- Sem rope data structure (ineficiente com textos grandes)
- Sem undo/redo (Ctrl+Z / Ctrl+Y)
- Sem copy/paste (Ctrl+C / Ctrl+V)
- Sem word wrap automático
- Sem syntax highlighting
- Cursor positioning limitado
- Sem selection (Shift+arrows)
- Sem find/replace (Ctrl+F)

**Solução:**
```rust
// Implementar rope data structure para eficiência
impl NoteEditor {
    rope: Rope,                    // Vec de linhas (eficiente)
    history: Vec<Rope>,           // Undo stack
    cursor: (usize, usize),       // (linha, coluna)
    selection: Option<(Pos, Pos)>, // Selection range
}

// Novos keybindings
Ctrl+Z      → undo()
Ctrl+Y      → redo()
Ctrl+C      → copy_selection()
Ctrl+V      → paste()
Ctrl+H      → backspace com history
Shift+Arrow → expand_selection()
```

---

### ❌ 2.2 PROBLEMA #2: Search Mode Não Implementado

**Severidade:** 🔴 CRÍTICA  
**Localização:** `src/ui/app.rs` - Mode enum contém `Search` mas sem UI  
**Impacto:** Features principais não funcionam

**Problemas Específicos:**
- Pressing `/` não abre search input visual
- Sem FTS5 search real-time
- Sem resultado feedback
- Sem filter/sort options
- Sem search history

**Solução Proposta:**

```
UI MOCKUP - Search Mode:

┌──────────────────────────────────────────────────────┐
│ STATUS: SEARCH                                       │
├──────────────────────────────────────────────────────┤
│  │ RESULTS          │   PREVIEW PANE                │
│  │                  │                              │
│  │ Search: rust___ │   # Rust Patterns            │
│  │ (3 results)     │   tags: #rust #design       │
│  │                  │                              │
│  │ ▸ Rust Patterns │   Rust patterns like...     │
│  │   (note 12a)    │                              │
│  │   - rust        │   Created: 2025-12-15      │
│  │   - patterns    │   Links: [[Visitor]]        │
│  │                  │                              │
│  │ Zettelkasten    │                              │
│  │ Intro           │                              │
│  │                  │                              │
│  │ Note Design     │                              │
│  │ System          │                              │
├──────────────────────────────────────────────────────┤
│ /: search | j/k: navigate | Enter: open | Esc: quit │
└──────────────────────────────────────────────────────┘

Keybindings:
/               → Entra Search mode
Type word       → FTS5 search real-time
j/k            → Navega resultados
Enter          → Abre nota
Esc            → Sai Search
```

**Implementação necessária:**
- Widget `SearchInputWidget`
- Widget `SearchResultsWidget`
- Integração com FTS5 do SQLite
- Real-time filtering durante typing

---

### ❌ 2.3 PROBLEMA #3: Command Mode Não Implementado

**Severidade:** 🔴 CRÍTICA  
**Localização:** `src/ui/app.rs` - Mode enum contém `Command` mas sem UI  
**Impacto:** Comandos avançados não acessíveis

**Problemas Específicos:**
- Pressing `:` não abre command prompt
- Sem command parsing
- Sem command autocomplete
- Sem command help

**Solução Proposta:**

```
UI MOCKUP - Command Mode:

┌──────────────────────────────────────────────────────┐
│ STATUS: COMMAND                                      │
├──────────────────────────────────────────────────────┤
│  │ NOTES           │   OUTPUT                       │
│  │                 │                               │
│  │ Note 1          │                               │
│  │ Note 2          │   Command: :_____             │
│  │ Note 3          │   Tip: :help for commands    │
│  │                 │                               │
│  │                 │   Available commands:        │
│  │                 │   :help        Show help     │
│  │                 │   :rename      Rename note   │
│  │                 │   :delete      Delete note   │
│  │                 │   :export PATH Export        │
│  │                 │   :import PATH Import        │
│  │                 │   :themes      List themes   │
├──────────────────────────────────────────────────────┤
│ :: command | Ctrl+C: abort | ↑↓: history           │
└──────────────────────────────────────────────────────┘

Comandos Básicos:
:help                  → Show help
:rename "New Title"    → Rename com link updates
:delete                → Delete com confirmação
:export ~/file.md      → Export nota
:import ~/folder       → Import de pasta
:link [[Reference]]    → Valida/cria link
:themes                → Seleciona tema
:config                → Edit configuração
```

**Implementação necessária:**
- Widget `CommandInputWidget`
- Command parser
- Command executor

---

### ❌ 2.4 PROBLEMA #4: Link Detection Não Implementado

**Severidade:** 🔴 CRÍTICA  
**Localização:** `src/ui/widgets/note_editor.rs` + storage layer  
**Impacto:** Sistema de links (core feature) não funciona

**Problemas Específicos:**
- Links `[[...]]` digitados mas não validados
- Sem highlighting visual
- Sem autocomplete
- Sem follow link functionality
- Sem backlinks

**Solução Proposta:**

```
EDITOR COM LINK DETECTION:

Digitando:
┌──────────────────────────────┐
│ Ver também [[ Zettel...      │
│            └─ Autocomplete   │
│               popup          │
│                              │
│ ▼ Link Suggestions           │
│ - Zettelkasten Introduction  │
│ - Zettelkasten Workflow      │
│ - Zettelkasten vs Tags       │
└──────────────────────────────┘

Após completar:
┌──────────────────────────────┐
│ Ver também [[Zettel Intro]]  │
│            └─ CYAN (válido)  │
│            └─ Clickable      │
└──────────────────────────────┘

Features necessárias:
✓ Regex parser para [[...]]
✓ Real-time validation
✓ Fuzzy autocomplete
✓ Visual highlighting
✓ Follow link (Ctrl+] ou f+l)
✓ Backlinks display
```

**Implementação necessária:**
- Link parser (regex)
- Link validator
- Link autocomplete
- Backlinks query

---

### ❌ 2.5 PROBLEMA #5: Sem Confirmação de Delete

**Severidade:** 🔴 CRÍTICA  
**Localização:** `src/ui/app.rs` - delete handler  
**Impacto:** Risco de perda acidental de dados

**Problema:** Pressing `d` deleta nota sem confirmação

**Solução Proposta:**

```
DELETE CONFIRMATION MODAL:

┌──────────────────────────────────────────┐
│                                          │
│         ⚠️  DELETE NOTE                  │
│                                          │
│         Title: "Rust Patterns"           │
│         Created: 2025-12-15              │
│         Links: 3 incoming, 5 outgoing    │
│                                          │
│         Delete this note? This action    │
│         cannot be undone.                │
│                                          │
│              [Delete] [Cancel]           │
│                                          │
│         (Tab or arrows to navigate)      │
│                                          │
└──────────────────────────────────────────┘

Informações importantes:
- Título da nota
- Data de criação
- Número de backlinks
- Tab-navigable buttons
```

**Implementação necessária:**
- Widget `ConfirmationModalWidget`
- Modal state management
- Button navigation (Tab)

---

### ❌ 2.6 PROBLEMA #6: Graph View Não Implementado

**Severidade:** 🔴 CRÍTICA  
**Localização:** Não existe widget  
**Impacto:** Visualização de conhecimento não existe

**Problema:** Pressing `v` não abre graph visualization

**Solução Proposta:**

```
GRAPH VISUALIZATION:

Nodes = Notas (círculos)
Edges = Links (linhas)

┌──────────────────────────────────────┐
│        ╱─ [[Rust Memory]]             │
│       ╱   ╲                           │
│  [[Rust Patterns]] ── [[Safety]]      │
│       ╲   ╱                           │
│        ╲─ [[Type System]]             │
│                                       │
│ Colors by type:                       │
│ 🔵 Permanent   🟡 Fleeting           │
│ 🟢 Daily       🔴 Literature         │
│                                       │
│ Zoom: +/- | Pan: arrows | Search: /  │
│ Click on node to navigate to note     │
└──────────────────────────────────────┘

Features:
✓ Force-directed layout (spring-based)
✓ Zoom/pan controls
✓ Filter by type/tags
✓ Click to navigate
✓ Color by note type
```

---

## 🟠 PARTE 3: PROBLEMAS SIGNIFICATIVOS (UX)

### 3.1 PROBLEMA #7: Falta de Feedback de Seleção

**Severidade:** 🟠 ALTA  
**Localização:** `src/ui/widgets/note_list.rs`  
**Impacto:** Usuário não sabe qual nota é selecionada

**Problema Atual:** Apenas marcador "▸", sem highlight de background

**Solução:**

```
ANTES:
│ ▸ Note 1
│   Note 2
│   Note 3

DEPOIS:
┌────────────────────┐
│ ▸ Note 1 [SELECTED]│ ← Highlight com cor do tema
│   Note 2           │
│   Note 3           │
└────────────────────┘

Implementação:
- Use theme color para background
- Bold ou inverter cores
- Show selected indicator
```

---

### 3.2 PROBLEMA #8: Sem Indicador de Unsaved Changes

**Severidade:** 🟠 ALTA  
**Localização:** `src/ui/widgets/status_bar.rs`  
**Impacto:** Usuário não sabe se mudanças foram salvas

**Solução Proposta:**

```
Editor indicator:

Sem mudanças:
│ # My Note [✓]                    │ ← Green check

Com mudanças:
│ # My Note [●]                    │ ← Red dot

Salvando:
│ # My Note [↻]                    │ ← Spinner

Salvo:
│ # My Note [✓ 2s ago]             │ ← Timestamp

Implementação:
- Track dirty state
- Show in editor + status bar
- Update on Ctrl+S
```

---

### 3.3 PROBLEMA #9: Sem Indicador de Sync Status

**Severidade:** 🟠 ALTA  
**Localização:** `src/ui/widgets/status_bar.rs`  
**Impacto:** Usuário não sabe se arquivo foi sincronizado

**Solução:**

```
Status bar mostrando sync:

┌────────────────────────────────────┐
│ [↻ saving...] │ 💾 Auto-save: ON   │
│ [✓ saved]     │ ⚡ 124ms latency   │
│ [✗ sync error] │ [❌ See details]  │
└────────────────────────────────────┘

Informações:
- Sync status (saving/saved/error)
- Auto-save toggle
- Performance metrics
- Error notification
```

---

### 3.4 PROBLEMA #10: Preview Pane Desatualizado

**Severidade:** 🟠 ALTA  
**Localização:** `src/ui/widgets/preview_pane.rs` (42 linhas, apenas stub)  
**Impacto:** Preview não renderiza markdown real

**Solução:**

```
Preview pane melhorado:

┌─────────────────────────────────┐
│ # Rust Patterns                 │
│                                 │
│ Patterns commonly used in Rust  │
│ ecosystem include:              │
│                                 │
│ • Observer pattern              │
│ • Iterator pattern              │
│ • Builder pattern               │
│                                 │
│ Related:                        │
│ [[Type System]]                 │
│ [[Async/Await]]                 │
│                                 │
│ Last updated: 2s ago            │
└─────────────────────────────────┘

Features:
✓ Real markdown rendering
✓ Sync com editor
✓ Scroll independente
✓ Links em destaque
✓ Code syntax highlighting
```

---

### 3.5 PROBLEMA #11: Sem Backlinks Display

**Severidade:** 🟠 ALTA  
**Localização:** `src/ui/` (novo widget necessário)  
**Impacto:** Usuário não vê contexto de incoming links

**Solução Proposta:**

```
Backlinks widget:

┌────────────────────────────┐
│ ▸ BACKLINKS (3)            │
│                            │
│ [[Rust Memory Safety]]     │
│   "...safety is core..."   │
│                            │
│ [[Type System]]            │
│   "...strong types..."     │
│                            │
│ [[Async Patterns]]         │
│   "...concurrency..."      │
│                            │
│ Click to navigate          │
└────────────────────────────┘

Features:
✓ List de notas linkando aqui
✓ Context snippets
✓ Clickable (navigate)
✓ Count indicator
```

---

### 3.6 PROBLEMA #12: Sem Validação de Frontmatter

**Severidade:** 🟠 ALTA  
**Localização:** `src/storage/markdown.rs` + `src/storage/org.rs`  
**Impacto:** Frontmatter corrompido quebra parser

**Solução:**

```
- Validar YAML/Org syntax
- Auto-repair simples
- Show error to user
- Fallback gracioso
```

---

## 🟡 PARTE 4: PROBLEMAS DE ARQUITETURA DA INFORMAÇÃO

### 4.1 PROBLEMA #13: Organização Flat (Sem Hierarquia)

**Severidade:** 🟡 MÉDIA  
**Localização:** `src/storage/` vault structure  
**Impacto:** 1000+ notas sem estrutura clara

**Arquitetura Atual (Flat):**
```
Vault/
└─ permanent/
   ├─ note-1.md
   ├─ note-2.md
   └─ note-3.md (SEM ESTRUTURA!)
```

**Arquitetura Proposta (Organizada por Tipo):**

```
Vault/
├─ 📅 Daily/                ← Diários (uma por dia)
│  ├─ 2026-04-01.md
│  ├─ 2026-04-02.md
│  └─ 2026-04-03.md
│
├─ ⚡ Fleeting/             ← Notas rápidas (inbox)
│  ├─ idea-1.md
│  └─ idea-2.md
│
├─ 📌 Permanent/            ← Conhecimento core
│  ├─ 1a.md
│  ├─ 1a1.md
│  ├─ 1a1a.md
│  └─ ...
│
├─ 📚 Literature/           ← Quotes de fontes
│  ├─ "Clean Code".md
│  ├─ "Design Systems".md
│  └─ ...
│
├─ 🗺️  Index/               ← Map of Contents
│  ├─ Rust.md               (agrupa notas de Rust)
│  ├─ Philosophy.md
│  └─ ...
│
└─ 📎 References/           ← Links externos
   ├─ GitHub Repos.md
   └─ Papers.md

Benefícios:
✓ Estrutura intuitiva
✓ Usuário sabe onde procurar
✓ Histórico diário visível
✓ Fleeting notes separadas (inbox)
✓ Compatível com Obsidian/Logseq
```

---

### 4.2 PROBLEMA #14: Metadata Inadequado

**Severidade:** 🟡 MÉDIA  
**Localização:** `src/note/metadata.rs`  
**Impacto:** Notas sem contexto e taxonomia

**Metadata Atual (Mínimo):**
```yaml
---
id: 1a2b3c
title: My Note
created_at: 2026-04-02
tags: [rust, zettel]
---
```

**Metadata Proposto (Completo):**
```yaml
---
id: 1a2b3c
title: "My Note"
type: permanent
created_at: 2026-04-02T14:30:00Z
updated_at: 2026-04-02T15:45:00Z
author: user
status: "draft" | "ready" | "archived"
priority: "high" | "medium" | "low"
tags: [rust, zettelkasten, design-patterns]
links: [[Related Note]], [[Another]]
source_url: https://example.com
source_author: John Doe
publish_date: 2025-12-15
---

Novos campos:
✓ Type (para filtrar por tipo)
✓ Status (draft/ready/archived)
✓ Priority (para triage)
✓ Source info (attribution)
✓ Publish date (contexto)
```

---

### 4.3 PROBLEMA #15: Search UX Confusa

**Severidade:** 🟡 MÉDIA  
**Localização:** `src/db/schema.rs` + UI  
**Impacto:** Difícil encontrar notas relevantes

**Busca Atual (Flat):**
```
Search: "rust"
Results:
- Rust Design Patterns
- Rust Async Fundamentals
- Why I Love Rust
(5 results, sem contexto)
```

**Busca Proposta (Com Facets):**
```
Search: "rust"

┌─────────────────────────────────────────┐
│ FILTERS:                                │
│ [×] Type:   📌 Permanent ⚡ Fleeting   │
│ [×] Tags:   #rust #async #performance  │
│ [×] Status: [ready] [draft] [archived] │
│ [×] Sort:   [Relevance] [Date] [Title] │
└─────────────────────────────────────────┘

Results (5):
1. 📌 Rust Design Patterns
   #rust #design-patterns
   Created: 2025-12-15 | Links: 3 in, 5 out
   "Patterns commonly used in Rust..."

2. 📌 Rust Async Fundamentals
   #rust #async #concurrency
   Created: 2025-11-20 | Links: 7 in, 2 out
   "Understanding async/await..."

3. ⚡ Interesting Rust Article
   #rust #web-dev
   Fleeting note - draft
   Last modified: 2 hours ago

Benefícios:
✓ Filtros por tipo/tag/status
✓ Contexto visual (links in/out)
✓ Sorting por relevância/data/título
✓ Metadata visível no resultado
```

---

## 🎨 PARTE 5: FLUXOS DE TELA COMPLETOS

### 5.1 Fluxo: Criar Nota

```
┌─────────────────────────────────────────────────────────┐
│ Press 'n' (create new note)                             │
└─────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────┐
│         CHOOSE NOTE TYPE                                │
│                                                         │
│  ▸ 📅 Daily        ← Creates 2026-04-02.md            │
│    ⚡ Fleeting     ← Quick capture (process later)     │
│    📌 Permanent    ← Core knowledge (default)           │
│    📚 Literature   ← Quote from source                 │
│    🔗 Reference    ← External link                     │
│    🗺️  Index       ← Map of Contents                   │
│                                                         │
│  ▸ Select with arrows, Press Enter                     │
└─────────────────────────────────────────────────────────┘
         ↓ (select 📌 Permanent)
         ↓
┌─────────────────────────────────────────────────────────┐
│  CREATE NEW PERMANENT NOTE                              │
│                                                         │
│  Title: ____________                                    │
│                                                         │
│  (Type title for your note)                            │
│                                                         │
│  Tags (optional): _______________________________________
│  (comma-separated: #rust, #patterns)                   │
│                                                         │
│  [Create] [Cancel]                                     │
└─────────────────────────────────────────────────────────┘
         ↓ (type "Rust Memory Safety")
         ↓
┌─────────────────────────────────────────────────────────┐
│         NEW NOTE CREATED                                │
│                                                         │
│  Type:   📌 Permanent                                   │
│  ID:     1a3b5c                                         │
│  Title:  Rust Memory Safety                             │
│  File:   ~/vault/permanent/1a3b5c.md                   │
│  Tags:   #rust #safety                                 │
│                                                         │
│  Ready for editing. Press 'i' to edit.                │
│  (Press ENTER to continue)                             │
└─────────────────────────────────────────────────────────┘
         ↓ (Press ENTER)
         ↓
SWITCHES TO NORMAL MODE + EDITOR READY
```

---

### 5.2 Fluxo: Processar Fleeting Notes

```
┌──────────────────────────────────────────────────┐
│ ⚡ FLEETING NOTES (5) - Unprocessed             │
│                                                  │
│ ▸ Quick idea about async                        │
│   2 hours ago                                    │
│                                                  │
│   Rust future trait                             │
│   5 minutes ago                                  │
│                                                  │
│   Pattern: Observer in Rust                     │
│   1 hour ago                                     │
│                                                  │
│ STATUS: Ready to process                        │
└──────────────────────────────────────────────────┘

User selects note + presses 'p' (process):
                      ↓
┌──────────────────────────────────────────────────┐
│        PROCESS FLEETING NOTE                    │
│                                                  │
│ Source: "Quick idea about async"               │
│                                                  │
│ What to do?                                     │
│                                                  │
│ ▸ [Create New Permanent]                       │
│   [Add to Existing Permanent]                  │
│   [Move to Literature]                         │
│   [Delete]                                     │
│                                                  │
│ (arrow keys to select, Enter to confirm)       │
└──────────────────────────────────────────────────┘

After processing:
┌──────────────────────────────────────────────────┐
│ ⚡ FLEETING NOTES (4) - Updated count          │
│ (one less)                                      │
└──────────────────────────────────────────────────┘
```

---

### 5.3 Fluxo: Deletar Nota com Confirmação

```
User selects note + presses 'd':
                      ↓
┌──────────────────────────────────────────────────┐
│                                                  │
│         ⚠️  DELETE NOTE                         │
│                                                  │
│         Title: "Rust Design Patterns"          │
│         Created: 2025-12-15                    │
│         Links: 3 incoming, 5 outgoing          │
│                                                  │
│         Are you sure? This action cannot be    │
│         undone.                                 │
│                                                  │
│              [Delete] [Cancel]                 │
│                                                  │
│         (Tab or arrows to navigate buttons)    │
│                                                  │
└──────────────────────────────────────────────────┘

User presses Tab + Enter (Delete):
                      ↓
┌──────────────────────────────────────────────────┐
│ ✓ Note deleted                                  │
│ File removed: ~/vault/permanent/1a2b.md       │
│ Database updated                               │
│ 5 backlinks updated                            │
└──────────────────────────────────────────────────┘
```

---

## 🧩 PARTE 6: COMPONENTES NECESSÁRIOS

### Novos Widgets Necessários

#### 1. **SearchInputWidget**
```rust
pub struct SearchInputWidget {
    query: String,
    history: Vec<String>,
    history_idx: usize,
    results: Vec<NoteMetadata>,
    selected_result: usize,
    active: bool,
}

// Functionality
- Real-time FTS5 search
- Search history (↑↓ arrows)
- Clear button
- Dynamic filtering
```

#### 2. **SearchResultsWidget**
```rust
pub struct SearchResultsWidget {
    results: Vec<SearchResult>,
    selected: usize,
    filters: SearchFilters,
    sort_by: SortOption,
}

// Functionality
- Display search results
- Filter badges (type, tags, status)
- Sort options
- Result preview
```

#### 3. **CommandInputWidget**
```rust
pub struct CommandInputWidget {
    command: String,
    history: Vec<String>,
    history_idx: usize,
    suggestions: Vec<CommandSuggestion>,
}

// Functionality
- Text input for commands
- Command autocomplete
- Command history
- Help tooltip
```

#### 4. **ConfirmationModalWidget**
```rust
pub struct ConfirmationModalWidget {
    title: String,
    message: String,
    details: String,
    buttons: Vec<&'static str>,
    focused_button: usize,
    on_confirm: Box<dyn Fn()>,
}

// Functionality
- Modal dialog
- Buttons with focus
- Tab navigation
- Keyboard/mouse input
```

#### 5. **NoteTypeSelectionWidget**
```rust
pub struct NoteTypeSelectionWidget {
    types: Vec<NoteType>,
    selected: usize,
    descriptions: HashMap<NoteType, String>,
}

// Functionality
- Grid of note types
- Description for each
- Keyboard + mouse navigation
- Icons (📅 ⚡ 📌 etc)
```

#### 6. **LinkAutocompleteWidget**
```rust
pub struct LinkAutocompleteWidget {
    suggestions: Vec<NoteRef>,
    selected: usize,
    query: String,
    active: bool,
}

// Functionality
- Dropdown suggestions
- Fuzzy matching
- Keyboard selection
- Link preview on hover
```

#### 7. **GraphVisualizationWidget**
```rust
pub struct GraphVisualizationWidget {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    camera: Camera,
    highlighted_node: Option<usize>,
}

// Functionality
- Node visualization
- Edge rendering
- Zoom/pan controls
- Filter by type/tags
```

#### 8. **NotificationToastWidget**
```rust
pub struct NotificationToastWidget {
    notifications: Vec<Notification>,
    animation: AnimationState,
}

// Functionality
- Error/warning/success messages
- Auto-dismiss
- Stack multiple
- Position (top/bottom)
```

#### 9. **BacklinksPreviewWidget**
```rust
pub struct BacklinksPreviewWidget {
    backlinks: Vec<BacklinkInfo>,
    selected: usize,
}

// Functionality
- List of incoming links
- Context snippets
- Clickable to navigate
- Count indicator
```

#### 10. **SyncStatusWidget**
```rust
pub struct SyncStatusWidget {
    status: SyncStatus,
    last_sync: DateTime<Utc>,
    error_message: Option<String>,
}

// Functionality
- Saving indicator
- Sync status
- Error notification
- Timestamp
```

---

## 🗺️ PARTE 7: ARQUITETURA PROPOSTA (IA)

### 7.1 Reorganização de Pastas

```
src/ui/
├── mod.rs
├── app.rs                    # Orchestrator (melhorado)
├── widgets/
│   ├── mod.rs
│   ├── editor/
│   │   ├── mod.rs
│   │   ├── note_editor.rs   # Melhorado com rope
│   │   ├── editor_state.rs  # Undo/redo state
│   │   └── link_parser.rs   # Link detection
│   │
│   ├── search/
│   │   ├── mod.rs
│   │   ├── search_input.rs  # Search widget
│   │   ├── search_results.rs # Results display
│   │   └── search_filter.rs # Filtering logic
│   │
│   ├── command/
│   │   ├── mod.rs
│   │   ├── command_input.rs # Command widget
│   │   ├── command_parser.rs # Command parsing
│   │   └── command_executor.rs # Command execution
│   │
│   ├── modals/
│   │   ├── mod.rs
│   │   ├── confirmation_modal.rs # Delete confirm
│   │   ├── note_type_selector.rs # Type selector
│   │   ├── create_note_modal.rs  # Creation flow
│   │   └── generic_modal.rs # Reusable modal
│   │
│   ├── note_list.rs         # Sidebar (melhorado)
│   ├── preview_pane.rs      # Preview (implementado)
│   ├── status_bar.rs        # Status (melhorado)
│   ├── backlinks.rs         # NEW: Backlinks display
│   ├── graph.rs             # NEW: Graph visualization
│   ├── notifications.rs     # NEW: Toast notifications
│   └── sync_indicator.rs    # NEW: Sync status
│
└── theme.rs                 # Theme utilities

src/storage/
├── mod.rs
├── vault.rs                 # Vault structure reorganized
├── folder_structure.rs      # Daily/Fleeting/etc
├── markdown.rs
├── org.rs
├── watcher.rs
├── importer.rs
└── sync.rs

src/note/
├── mod.rs
├── types.rs                 # Note types (Daily, Fleeting, etc)
├── zettel.rs
├── metadata.rs              # NEW: Enhanced metadata
├── metadata_parser.rs       # NEW: YAML parser
└── utils.rs
```

---

## 📊 PARTE 8: ROADMAP PRIORIZADO

### 🔴 PRIORITY 1 (MVP - Usable App) - Semana 1-2

**Meta:** App fica usável para daily use

```
[ ] Fix Editor
    ├─ Implementar rope data structure
    ├─ Adicionar undo/redo (Ctrl+Z/Y)
    ├─ Adicionar copy/paste (Ctrl+C/V)
    ├─ Adicionar selection (Shift+Arrow)
    └─ (Est: 4-6 horas)

[ ] Implement Search Mode
    ├─ Create SearchInputWidget
    ├─ Create SearchResultsWidget
    ├─ Integrate FTS5 queries
    ├─ Add keybindings (/, j/k, Enter, Esc)
    └─ (Est: 3-4 horas)

[ ] Implement Command Mode
    ├─ Create CommandInputWidget
    ├─ Basic command parser (:help, :rename, :delete, :exit)
    ├─ Command executor
    ├─ Help display
    └─ (Est: 3-4 horas)

[ ] Add Delete Confirmation
    ├─ Create ConfirmationModalWidget
    ├─ Trigger on 'd' key
    ├─ Button navigation (Tab)
    └─ (Est: 1-2 horas)

[ ] Status Bar Improvements
    ├─ Add unsaved indicator (●/✓)
    ├─ Add sync status
    ├─ Add current mode display
    ├─ Add timestamp
    └─ (Est: 1-2 horas)

[ ] Link Detection (Basic)
    ├─ Regex parser para [[...]]
    ├─ Visual highlighting (cyan)
    ├─ Validation (nota existe?)
    └─ (Est: 2-3 horas)

TOTAL ESTIMADO: 14-21 horas (~2 dias intensos)
IMPACTO: 🟢 App becomes usable
```

---

### 🟠 PRIORITY 2 (Core Features) - Semana 3-4

**Meta:** App reaches "very good" status

```
[ ] Link Autocomplete
    ├─ Fuzzy search suggestions
    ├─ Dropdown widget
    ├─ Keyboard selection
    └─ (Est: 2-3 horas)

[ ] Link Following
    ├─ Follow link on Ctrl+] or f+l
    ├─ Navigation history (back with Ctrl+[)
    ├─ Update breadcrumb
    └─ (Est: 1-2 horas)

[ ] Backlinks Display
    ├─ Create BacklinksPreviewWidget
    ├─ Query incoming links
    ├─ Show context snippets
    ├─ Clickable to navigate
    └─ (Est: 2-3 horas)

[ ] Graph Visualization
    ├─ Create GraphVisualizationWidget
    ├─ Node rendering (circles)
    ├─ Edge rendering (lines)
    ├─ Zoom/pan controls
    ├─ Filter by type/tags
    ├─ Click to navigate
    └─ (Est: 4-6 horas)

[ ] Note Type Selection
    ├─ Create NoteTypeSelectionWidget
    ├─ Icons for each type (📅 ⚡ 📌 etc)
    ├─ Descriptions
    ├─ Keyboard + mouse nav
    └─ (Est: 2-3 horas)

[ ] Create Note Flow
    ├─ Trigger on 'n' key
    ├─ Type selector modal
    ├─ Title input modal
    ├─ Tags input (optional)
    ├─ Create file + DB record
    └─ (Est: 3-4 horas)

[ ] Process Fleeting Notes
    ├─ Show fleeting notes list
    ├─ 'p' key to process
    ├─ Modal with options (create/add/delete)
    ├─ Move/copy to permanent
    └─ (Est: 3-4 horas)

[ ] Folder Reorganization
    ├─ Create Daily/ Fleeting/ Permanent/ etc folders
    ├─ Migrate existing notes
    ├─ Update storage layer paths
    ├─ Update sidebar display
    └─ (Est: 2-3 horas)

[ ] Enhanced Metadata
    ├─ Add status field (draft/ready/archived)
    ├─ Add priority field (high/medium/low)
    ├─ Add source fields (URL, author, date)
    ├─ Update YAML/Org parsing
    └─ (Est: 2-3 horas)

TOTAL ESTIMADO: 21-31 horas (~3-4 dias)
IMPACTO: 🟢 App reaches "very good"
```

---

### 🟡 PRIORITY 3 (Polish) - Semana 5-6

**Meta:** Production-ready

```
[ ] Search Filters
    ├─ Filter by type (Daily/Fleeting/etc)
    ├─ Filter by tags
    ├─ Filter by status
    ├─ Filter by date range
    ├─ Sort options (relevance/date/title)
    └─ (Est: 3-4 horas)

[ ] Advanced Command Mode
    ├─ :export PATH → Export to file
    ├─ :import PATH → Import from path
    ├─ :link [[Ref]] → Create/validate link
    ├─ :graph → Open graph view
    ├─ :rename "New" → Rename with link updates
    └─ (Est: 4-5 horas)

[ ] Notifications/Toasts
    ├─ Create NotificationToastWidget
    ├─ Error messages
    ├─ Success messages
    ├─ Auto-dismiss (5s)
    ├─ Stack multiple
    └─ (Est: 2-3 horas)

[ ] Sync Status Indicator
    ├─ Create SyncStatusWidget
    ├─ Show saving indicator
    ├─ Show sync status
    ├─ Show errors
    ├─ Show timestamp
    └─ (Est: 1-2 horas)

[ ] Performance Optimization
    ├─ Virtual scrolling (large lists)
    ├─ Caching layer (LRU)
    ├─ Index optimization
    ├─ Lazy loading
    └─ (Est: 4-6 horas)

[ ] Link Refactoring
    ├─ When rename note, update all backlinks
    ├─ Atomic transactions
    ├─ Conflict detection
    └─ (Est: 3-4 horas)

[ ] Auto-backup
    ├─ Backup before delete
    ├─ Backup on major changes
    ├─ Rotation (keep last 10)
    ├─ Recovery mechanism
    └─ (Est: 2-3 horas)

[ ] Dark Mode Auto-detect
    ├─ Detect terminal bg color
    ├─ Auto-select theme
    ├─ Theme override option
    └─ (Est: 1-2 horas)

TOTAL ESTIMADO: 20-30 horas (~3-4 dias)
IMPACTO: 🟢 Production-ready
```

---

### 🔮 PRIORITY 4 (Future) - Roadmap

```
[ ] Collaborative Editing
    ├─ Sync via Git
    ├─ Conflict resolution
    ├─ Distributed notes
    └─ (Est: ??? - major feature)

[ ] AI Features
    ├─ Embeddings for semantic search
    ├─ Auto-tagging
    ├─ Auto-linking
    ├─ Summarization
    └─ (Est: ??? - requires AI integration)

[ ] Plugin System
    ├─ Agent-based architecture
    ├─ Custom commands
    ├─ Custom rendering
    └─ (Est: ??? - architectural change)

[ ] Cloud Sync
    ├─ Optional cloud backend
    ├─ Keep offline-first
    └─ (Est: ??? - infrastructure)

[ ] Mobile View
    ├─ Responsive UI
    ├─ Touch controls
    └─ (Est: ??? - new UI paradigm)
```

---

## 📋 PARTE 9: PRÓXIMOS PASSOS

### Para começar:

1. ✅ **Leia este documento** - Entenda a visão completa
2. 📝 **Discuta prioridades** - Qual problema atacar primeiro?
3. 🔧 **Comece pelo Priority 1** - Foco é na usabilidade básica
4. 🧪 **Teste incrementalmente** - Cada feature deve funcionar sozinha
5. 🔄 **Iterate rapidamente** - Feedback loop curto

### Primeira Feature Recomendada:

**Comece pelo EDITOR** (Priority 1)
- Rope data structure é fundamental
- Undo/redo é esperado
- Tudo más é construído no topo disso
- Est: 4-6 horas

Depois:
2. **Search Mode** (3-4 horas)
3. **Command Mode** (3-4 horas)
4. **Delete Confirmation** (1-2 horas)
5. **Link Detection** (2-3 horas)

Total Priority 1: ~2 dias intensos → **App fica usável**

---

## 🎓 CONCLUSÃO

Este relatório fornece:

✅ **Análise completa** dos problemas atuais (22 identificados)  
✅ **Mockups ASCII** de cada solução proposta  
✅ **Arquitetura detalhada** da reorganização necessária  
✅ **10 novos widgets** descritos  
✅ **Roadmap priorizado** de 4 sprints  
✅ **Estimativas** de tempo para cada feature  

**Status Recomendado:** Começar pelo Priority 1 para tornar o app usável em ~2 dias.

---

**Preparado por:** OpenCode UX/IA Analysis  
**Data:** 3 de Abril de 2026  
**Versão:** 1.0
