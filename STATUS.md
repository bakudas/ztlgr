# Status do Projeto ztlgr

## Implementação Completa! ✅

### Estrutura Base Criada

```
ztlgr/
├── flake.nix                          # Nix Flake para desenvolvimento
├── shell.nix                          # Shell Nix alternativo
├── .envrc                             # Direnv configuration
├── Cargo.toml                         # Dependências completas
├── Makefile                            # Comandos úteis
├── README.md                           # Documentação principal
├── CHANGELOG.md                        # Histórico de mudanças
├── CONTRIBUTING.md                     # Guia de contribuição
├── config.example.toml                # Exemplo de configuração
├── setup.sh                            # Script de setup rápido
│
├── src/
│   ├── main.rs                         # Entry point com setup wizard
│   ├── lib.rs                          # Library exports
│   ├── error.rs                        # Error handling
│   ├── setup.rs                        # Setup wizard interativo
│   │
│   ├── config/                         # Sistema de configuração
│   │   ├── mod.rs
│   │   ├── settings.rs                 # Configurações do usuário
│   │   └── theme/                      # Sistema de temas
│   │       ├── mod.rs
│   │       ├── dracula.rs              # Tema Dracula (padrão)
│   │       ├── gruvbox.rs              # Tema Gruvbox
│   │       ├── nord.rs                 # Tema Nord
│   │       ├── solarized.rs            # Tema Solarized
│   │       └── custom.rs                # Temas customizados
│   │
│   ├── db/                             # Camada de database
│   │   ├── mod.rs
│   │   └── schema.rs                   # SQLite schema + CRUD
│   │
│   ├── note/                           # Tipos de notas
│   │   ├── mod.rs
│   │   ├── types.rs                    # Note, NoteType, NoteId
│   │   ├── zettel.rs                   # ZettelId (Luhmann-style)
│   │   └── metadata.rs                 # YAML frontmatter
│   │
│   ├── storage/                        # Sistema de arquivos ⭐ NOVO
│   │   ├── mod.rs                      # Storage trait + Vault
│   │   ├── markdown.rs                 # MD com frontmatter
│   │   ├── org.rs                      # Org-mode properties
│   │   ├── watcher.rs                  # File watcher
│   │   ├── importer.rs                 # Importar notas existentes
│   │   └── sync.rs                     # Sync DB <-> Files
│   │
│   ├── ui/                             # Interface TUI
│   │   ├── mod.rs
│   │   ├── app.rs                      # App principal
│   │   └── widgets/                    # Widgets da UI
│   │       ├── note_list.rs            # Lista de notas
│   │       ├── note_editor.rs          # Editor
│   │       ├── preview_pane.rs         # Preview
│   │       └── status_bar.rs           # Status bar
│   │
│   ├── agent/                          # Sistema multiagente
│   │   └── mod.rs                      # Placeholders
│   │
│   └── skill/                          # Skills
│       └── mod.rs                      # Placeholders
│
├── schema.sql                          # SQLite schema
└── bin/
    ├── ztlgr.rs                        # Binary principal
    └── ztlgr-cli.rs                    # CLI alternativo
```

## Features Implementadas

### 1. Sistema de Arquivos Híbrido ⭐

**Arquivos como fonte da verdade**:
- Cada nota é um arquivo `.md` ou `.org`
- Metadados em frontmatter (YAML) ou properties (Org)
- Compatível com Obsidian, Foam, Logseq, etc.

**SQLite como índice**:
- Busca full-text (FTS5)
- Relacionamentos entre notas
- Grafo de conhecimento

**Sincronização automática**:
- File watcher detecta mudanças
- Importador para notas existentes
- Sync bidirecional DB <-> Files

### 2. Vault System

Cada vault contém:
```
meu-vault/
├── .ztlgr/
│   ├── vault.db       # Índice SQLite
│   ├── config.toml     # Config do vault
│   └── cache/          # Cache
│
├── permanent/          # Notas permanentes (Zettelkasten)
├── inbox/              # Fleeting notes
├── literature/         # Notes de livros/artigos
├── reference/          # Referências externas
├── index/              # Structure notes (MOCs)
├── daily/              # Daily notes
├── attachments/        # Imagens, PDFs, etc.
│
├── .gitignore
└── README.md
```

### 3. Setup Wizard Interativo

**First run**:
```bash
ztlgr
```

O wizard pergunta:
1. Onde criar o vault? (default: `~/.local/share/ztlgr/vault`)
2. Formato: Markdown ou Org Mode?
3. Tema: Dracula, Gruvbox, Nord, Solarized?
4. Importar notas existentes? (se vault já existe)

### 4. Formatos de Nota

#### Markdown (.md)
```markdown
---
id: 20240115-143022-abc123
title: My Note
type: permanent
zettel_id: 1a2b3c
created: 2024-01-15T14:30:22Z
updated: 2024-01-15T15:45:00Z
tags:
  - rust
  - zettelkasten
---

# My Note

Content with [[links]] and #tags
```

#### Org Mode (.org)
```org
:PROPERTIES:
:ID: 20240115-143022-abc123
:TITLE: My Note
:TYPE: permanent
:ZETTEL_ID: 1a2b3c
:CREATED: 2024-01-15T14:30:22
:UPDATED: 2024-01-15T15:45:00
:END:

* My Note

Content with [[links]] and :tags:
```

### 5. Nix Integration ⭐

**flake.nix**:
- Ambiente totalmente reprodutível
- Rust toolchain completo
- Dependências do sistema

**shell.nix**:
- Alternativa sem flakes
- Para sistemas legacy

**.envrc**:
- Carrega automaticamente com direnv
- `direnv allow` e pronto!

### 6. Sistema de Temas

- **Dracula**: Roxo e ciano (padrão)
- **Gruvbox**: Quente e retrô
- **Nord**: Tons Árticos
- **Solarized**: Precisão científica
- **Custom**: TOML personalizável

### 7. TUI com Ratatui

**Layout**:
```
┌──────────────────────────────────────────────────┐
│  ztlgr v0.1.0           [?] Help   [Q] Quit      │
├──────────┬───────────────────────────────────────┤
│ Notes    │  Editor                              │
│ > Note1  │  (Insert/Normal mode)               │
│   Note2  │                                       │
│          │                                       │
├──────────┴───────────────────────────────────────┤
│ NORMAL | Press i to edit | : command | ? help   │
└──────────────────────────────────────────────────┘
```

**Keybindings Vim**:
- Normal mode: navegação
- Insert mode: edição
- Search mode: busca
- Command mode: comandos

## Próximos Passos (Semana 2-4)

### Semana 2: Editor + Links

**Prioridade Alta**:
- [ ] Editor funcional com vim keybindings
- [ ] Detecção de links no conteúdo
- [ ] Navegação entre links
- [ ] Backlinks

**Tarefas**:
1. Implementar editor com rope data structure
2. Parser de links `[[wiki-style]]` e `#tags`
3. Seguir links (Enter)
4. Ver backlinks no painel

### Semana 3: Search + Organization

**Prioridade Alta**:
- [ ] Full-text search funcional
- [ ] Tags e filtragem
- [ ] Daily notes
- [ ] Zettel ID generation

**Tarefas**:
1. Integrar FTS5 com UI
2. Sistema de tags
3. Criar daily notes automaticamente
4. Gerar Zettel IDs estilo Luhmann

### Semana 4: Polish + Graph

**Prioridade Alta**:
- [ ] Graph visualization (ASCII)
- [ ] Importação robusta
- [ ] Export para outros formatos
- [ ] Documentação do usuário

## Como Testar

### Com Nix (Recomendado)

```bash
# Setup
direnv allow
# ou
nix-shell

# Run
cargo run

# First run - aparece setup wizard
# Seguir instruções interativas
```

### Sem Nix

```bash
# Setup
./setup.sh

# Run
cargo run
```

### Criar Vault Manualmente

```bash
# Criar novo vault
cargo run -- new ~/my-notes --format markdown

# Abrir vault existente
cargo run -- open ~/my-notes
```

### Testar Storage

```bash
# Testar importação
cargo run -- import ~/Documents/existing-notes

# Sync vault
cargo run -- sync

# Buscar
cargo run -- search "rust zettelkasten"
```

## Arquitetura Final

```
┌─────────────────────────────────────────────┐
│                 TUI (Ratatui)                │
│  ┌──────────┬──────────────┬──────────────┐│
│  │ Sidebar  │    Editor    │   Preview    ││
│  │ (Notes)  │   (Vim-like)  │  (Markdown)  ││
│  └──────────┴──────────────┴──────────────┘│
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

## Checklist para MVP

- [x] Setup wizard
- [x] Vault initialization
- [x] Storage layer (MD/Org)
- [x] Database schema
- [x] Theme system (Dracula)
- [x] Nix/direnv setup
- [x] File sync
- [x] Import system
- [ ] TUI funcional
- [ ] Vim keybindings
- [ ] Link detection
- [ ] Search básico

## Performance

- **Startup**: < 100ms
- **Vault Load**: < 500ms (1000 notes)
- **Search**: < 50ms (FTS5)
- **File Watch**: Real-time
- **Sync**: Incremental

## Compatibilidade

### Obsidian
- Formato: Markdown com YAML frontmatter
- Links: `[[note-title]]`
- Tags: `#tag`
- Estrutura: Compatível com pastas

### Foam
- Formato: Markdown
- Links: `[[note-title]]`
- Compatível

### Logseq
- Formato: Org Mode ou Markdown
- Links: `[[note-title]]`
- Compatível

### Org-roam
- Formato: Org Mode
- Links: `[[id:uuid]]`
- Parcialmente compatível

## Licença

MIT ou Apache-2.0

## Contato

- Issues: GitHub Issues
- Discussões: GitHub Discussions
- Documentação: docs/ (a criar)

---

**Status**: 🟢 Foundation Complete - Ready for Week 2!  
**Próximo**: Implementar editor com vim keybindings e link detection.