# Howto: LLM Integration e MCP Server

Este guia explica como usar as features de LLM e MCP do ztlgr para transformar seu grimoire em uma base de conhecimento mantida por LLM.

## Sumário

1. [Conceitos](#conceitos)
2. [Configuração](#configuração)
3. [Workflows LLM](#workflows-llm)
4. [MCP Server](#mcp-server)
5. [Integração com Claude Code / Cursor](#integração)
6. [Exemplos Práticos](#exemplos)
7. [Troubleshooting](#troubleshooting)

---

## Conceitos

### Arquitetura em Camadas

```
┌─────────────────────────────────────────────────────────┐
│                    LLM Agent                             │
│  (Claude Code, Cursor, OpenCode, etc.)                  │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │         MCP Tools (9 disponíveis)                │   │
│  │  search, get_note, list_notes, create_note       │   │
│  │  get_backlinks, ingest_source                     │   │
│  │  read_index, read_log, read_skills              │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   CLI / TUI                              │
│  ztlgr ask, ztlgr lint, ztlgr ingest --process          │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                 ztlgr Core (Rust)                        │
│  ┌───────────┐  ┌───────────┐  ┌───────────────────┐   │
│  │ Workflow  │  │  LLM      │  │  .skills/         │   │
│  │ Engine     │  │  Provider │  │  (Schema)         │   │
│  │            │  │  Layer    │  │                   │   │
│  └───────────┘  └───────────┘  └───────────────────┘   │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                  Grimoire (Filesystem)                  │
│  raw/          ← Fontes imutáveis                      │
│  literature/    ← Resumos de fontes (LLM)              │
│  permanent/     ← Conhecimento sintetizado             │
│  index/         ← MOCs, páginas de índice              │
│  .skills/       ← Instruções para LLM agents           │
│  .ztlgr/        ← DB, config, log.md, index.md        │
└─────────────────────────────────────────────────────────┘
```

### Três Camadas do LLM Wiki Pattern

1. **Raw Sources** (`raw/`) — Fontes imutáveis. Artigos, papers, URLs, screenshots. O LLM lê mas nunca modifica.

2. **The Wiki** (diretórios existentes) — Síntese mantida pelo LLM. Summaries, entity pages, concept pages, comparisons. O LLM escreve; você cura.

3. **The Schema** (`.skills/`) — Instruções que dizem ao LLM como o wiki é estruturado, convenções, workflows.

---

## Configuração

### 1. Configurar Provider LLM

Crie `.ztlgr/config.toml` no seu grimoire:

```toml
# .ztlgr/config.toml

[llm]
# Habilita LLM features (default: false)
enabled = true

# Provider: "ollama" (local), "openai", "anthropic"
provider = "ollama"

# Modelo (varia por provider)
model = "llama3"

# URL base (apenas para Ollama com host customizado)
api_base = "http://localhost:11434"

# Variável de ambiente com a API key (NUNCA colocar a key direto)
# Para OpenAI: OPENAI_API_KEY
# Para Anthropic: ANTHROPIC_API_KEY
api_key_env = ""

# Parâmetros de geração
max_tokens = 4096
temperature = 0.7

[vcs]
# Git integration
enabled = true
auto_commit = false
commit_message = "chore: {{action}} {{item}}"
```

### 2. Providers Suportados

#### Ollama (Local, Recomendado)

```toml
[llm]
enabled = true
provider = "ollama"
model = "llama3"      # ou "llama3.2", "mistral", "codellama"
api_base = ""         # vazio = localhost:11434 (default)
```

Instalação:
```bash
# macOS/Linux
curl -fsSL https://ollama.com/install.sh | sh

# Baixar modelo
ollama pull llama3

# Verificar
ollama list
```

#### OpenAI

```toml
[llm]
enabled = true
provider = "openai"
model = "gpt-4o"      # ou "gpt-4-turbo", "gpt-3.5-turbo"
api_key_env = "OPENAI_API_KEY"
```

Configuração:
```bash
export OPENAI_API_KEY="sk-..."
```

#### Anthropic Claude

```toml
[llm]
enabled = true
provider = "anthropic"
model = "claude-3-5-sonnet-20241022"
api_key_env = "ANTHROPIC_API_KEY"
```

Configuração:
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

### 3. Inicializar .skills/

```bash
# Criar novo grimoire com .skills/
ztlgr new ~/my-grimoire --format markdown

# Ou inicializar em grimoire existente
ztlgr init-skills --vault ~/my-grimoire
```

O diretório `.skills/` contém:

```
.skills/
├── README.md              # Visão geral para o LLM
├── conventions.md         # Convenções do wiki
├── workflows/
│   ├── ingest.md          # Como processar fontes
│   ├── query.md           # Como responder perguntas
│   └── lint.md            # Como auditar o wiki
├── templates/
│   ├── source-summary.md  # Template para resumos
│   ├── entity-page.md     # Template para entidades
│   └── ...
└── context/
    ├── domain.md          # Conhecimento de domínio
    └── priorities.md      # Questões em aberto
```

---

## Workflows LLM

### Ingest Workflow: Processar Fontes

O workflow `ingest` lê uma fonte de `raw/`, gera um summary, e cria uma Literature Note.

```bash
# 1. Ingerir arquivo (sem LLM)
ztlgr ingest ~/Downloads/article.pdf --vault ~/my-grimoire

# Isso copia para raw/ e registra no DB

# 2. Processar com LLM (gera Literature Note)
ztlgr ingest ~/Downloads/article.pdf --process --vault ~/my-grimoire

# Equivalente a:
# 1. Ingerir arquivo para raw/
# 2. Chamar LLM com .skills/workflows/ingest.md
# 3. Criar Literature Note em literature/
# 4. Atualizar index.md
# 5. Logar em log.md
```

**O que acontece:**

```
raw/article.pdf
       │
       ▼
┌─────────────────────────────────────────────────┐
│  LLM Processa com .skills/workflows/ingest.md   │
│  - Extrai conceitos principais                  │
│  - Identifica entidades                         │
│  - Gera summary estruturado                     │
└─────────────────────────────────────────────────┘
       │
       ▼
literature/article-summary.md

Title: Article Summary
Type: literature
Source: raw/article.pdf
Confidence: medium

## Key Concepts
- Concept 1: explanation
- Concept 2: explanation

## Summary
...

## Open Questions
- Question 1?
- Question 2?

## Related Notes
- [[Concept 1]]
- [[Entity Name]]
```

### Query Workflow: Fazer Perguntas

```bash
ztlgr ask "What did I learn about async patterns in Rust?" --vault ~/my-grimoire

# O LLM:
# 1. Busca no index.md para orientação
# 2. Faz FTS5 search com a query
# 3. Lê as notas relevantes
# 4. Sintetiza resposta com [[wiki-links]]
```

**Exemplo de resposta:**

```
Based on your grimoire, here's what you've learned about async patterns in Rust:

## Task-Based Concurrency
The [[Rust Async Patterns]] note explains that tasks are the unit of concurrency
in Tokio. Unlike threads, tasks are lightweight (~64 bytes) and scheduled by
the runtime...

## Key Insights
1. `tokio::select!` cancels branches on first completion (see [[Select Patterns]])
2. Channels are preferred over shared state (see [[Rust Channels]])
3. Async functions compile to state machines (see [[Async Internals]])

## Related Notes
- [[Rust Concurrency Model]]
- [[Tokio Runtime Architecture]]
- [[Future trait explained]]
```

### Lint Workflow: Auditar o Wiki

```bash
# Lint local (sem LLM, rápido)
ztlgr lint --vault ~/my-grimoire

# Output:
# - 3 orphan notes (no inbound links)
# - 2 short notes (<100 chars)
# - 1 unprocessed source in raw/

# Lint completo (com LLM, análise profunda)
ztlgr lint --full --vault ~/my-grimoire

# Output:
# - Local findings (orphan, short, unprocessed)
# - LLM analysis: contradictions, missing cross-refs, stale claims
# - Suggestions: new notes to create, sources to process
```

---

## MCP Server

O MCP (Model Context Protocol) permite que qualquer agente LLM compatível use o seu grimoire como fonte de conhecimento.

### Iniciar o Servidor

```bash
# Iniciar MCP server (bloqueante, roda em foreground)
ztlgr mcp --vault ~/my-grimoire

# O servidor:
# - Lê JSON-RPC de stdin
# - escreve JSON-RPC para stdout
# - Loga para stderr
```

### Protocolo

O MCP usa JSON-RPC 2.0 sobre stdio:

```json
// Request: initialize
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
  "protocolVersion": "2025-03-26",
  "clientInfo": {"name": "Claude Code", "version": "1.0"},
  "capabilities": {}
}}

// Response:
{"jsonrpc": "2.0", "id": 1, "result": {
  "protocolVersion": "2025-03-26",
  "serverInfo": {"name": "ztlgr", "version": "0.5.0"},
  "capabilities": {"tools": {"listChanged": false}},
  "instructions": "ztlgr grimoire MCP server..."
}}

// Request: tools/list
{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}

// Response: lista de 9 tools

// Request: tools/call
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
  "name": "search",
  "arguments": {"query": "async patterns", "limit": 10}
}}

// Response: resultados da busca
```

### Tools Disponíveis

| Tool | Descrição | Parâmetros |
|------|-----------|------------|
| `search` | Busca FTS5 no grimoire | `query`, `limit` |
| `get_note` | Obtém nota por ID ou título | `id` ou `title` |
| `list_notes` | Lista notas (opcionalmente por tipo) | `note_type`, `limit`, `offset` |
| `create_note` | Cria nova nota | `title`, `content`, `note_type` |
| `get_backlinks` | Notas que linkam para uma nota | `note_id` ou `title` |
| `ingest_source` | Ingest arquivo para `raw/` | `file_path`, `title` |
| `read_index` | Lê `.ztlgr/index.md` | — |
| `read_log` | Lê `.ztlgr/log.md` | `tail` |
| `read_skills` | Lê arquivo de `.skills/` | `file` |

### Exemplo de Uso Programático

```python
import subprocess
import json

def call_mcp_tool(tool_name, arguments):
    """Chama uma tool MCP via ztlgr mcp"""
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    }
    
    proc = subprocess.Popen(
        ["ztlgr", "mcp", "--vault", "/path/to/grimoire"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL
    )
    
    # Enviar request
    proc.stdin.write((json.dumps(request) + "\n").encode())
    proc.stdin.close()
    
    # Ler response
    response = proc.stdout.read().decode()
    return json.loads(response)

# Buscar notas
result = call_mcp_tool("search", {"query": "machine learning", "limit": 5})
print(result["result"]["content"][0]["text"])
```

---

## Integração com Claude Code / Cursor

### Claude Code

1. **Configurar MCP em `~/.claude/mcp.json`:**

```json
{
  "mcpServers": {
    "ztlgr": {
      "command": "ztlgr",
      "args": ["mcp", "--vault", "/path/to/grimoire"]
    }
  }
}
```

2. **Claude Code detecta automaticamente as tools:**

```
Claude: Posso acessar seu grimoire ztlgr. O que você gostaria?

You: Search for notes about async patterns

Claude: [uses search tool]

I found 5 notes matching "async patterns":

1. [[Rust Async Patterns]] - Key concurrency concepts
2. [[Tokio Select]] - The select! macro explained
3. [[Rust Channels]] - Channel patterns for message passing
...
```

### Cursor

1. **Configurar em `.cursor/mcp.json`:**

```json
{
  "mcpServers": {
    "ztlgr": {
      "command": "ztlgr",
      "args": ["mcp", "--vault", "/path/to/grimoire"]
    }
  }
}
```

2. **Usar no chat:**

```
@ztlgr Search for notes about machine learning

[Custo usa a tool search automaticamente]
```

---

## Exemplos Práticos

### Exemplo 1: Processar uma coleção de papers

```bash
# 1. Ingerir todos os PDFs
for pdf in ~/Downloads/papers/*.pdf; do
  ztlgr ingest "$pdf" --process --vault ~/my-grimoire
done

# Cada paper gera:
# - Literature Note em literature/
# - Atualização no index.md
# - Entrada no log.md
```

### Exemplo 2: Manter um wiki de aprendizado

```bash
# Query: O que eu aprendi sobre X?
ztlgr ask "What did I learn about distributed systems?" --vault ~/my-grimoire

# Lint: Identificar gaps
ztlgr lint --full --vault ~/my-grimoire

# Resultado:
# - "[[Consensus]] has no backlinks" — orphan concept
# - "Unprocessed source: raw/raft-paper.pdf" — needs summary
# - "Potential contradiction: [[Raft Overview]] claims Raft is simpler than Paxos, 
#    but [[Paxos vs Raft]] says both have similar complexity"
```

### Exemplo 3: Session continuity

```bash
# No início de uma sessão de pesquisa:

# 1. Ver o que foi feito recentemente
ztlgr ask "What did I work on in the last week?" --vault ~/my-grimoire

# 2. Ver perguntas em aberto
ztlgr ask "What open questions am I tracking?" --vault ~/my-grimoire

# 3. Processar novo material
ztlgr ingest ~/Downloads/new-article.pdf --process --vault ~/my-grimoire

# 4. Nova pergunta derivada
ztlgr ask "How does the new article relate to [[Topic X]]?" --vault ~/my-grimoire
```

---

## Troubleshooting

### Erro: "LLM features disabled"

**Causa:** `[llm].enabled = false` ou seção `[llm]` ausente.

**Solução:**
```bash
# Adicionar ao .ztlgr/config.toml
[llm]
enabled = true
provider = "ollama"
model = "llama3"
```

### Erro: "Ollama connection refused"

**Causa:** Ollama não está rodando ou está em porta diferente.

**Solução:**
```bash
# Iniciar Ollama
ollama serve

# Verificar URL
curl http://localhost:11434/api/tags

# Se estiver em porta diferente, configurar:
[llm]
api_base = "http://192.168.1.100:11434"
```

### Erro: "API key not found"

**Causa:** Variável de ambiente não configurada.

**Solução:**
```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic  
export ANTHROPIC_API_KEY="sk-ant-..."

# Verificar
echo $OPENAI_API_KEY
```

### Erro: "No sources to process"

**Causa:** Arquivo raw/ vazio ou inexistente.

**Solução:**
```bash
# Verificar fontes ingeridas
ztlgr ask "List all processed sources" --vault ~/my-grimoire

# Ou via MCP
ztlgr mcp --vault ~/my-grimoire
# tools/call: {"name": "read_index", "arguments": {}}
```

### Performance: Query lenta

**Causa:** Muitas notas, contexto grande para o LLM.

**Soluções:**

1. **Reduzir max_tokens:**
```toml
[llm]
max_tokens = 2048  # default: 4096
```

2. **Usar modelo mais rápido:**
```toml
[llm]
model = "llama3.2"  # menor que llama3
```

3. **Limitar resultados:**
```bash
# Ao invés de buscar tudo
ztlgr ask "..." --limit 5 --vault ~/my-grimoire
```

### MCP não encontrado

**Causa:** `ztlgr` não está no PATH do agente.

**Solução:**
```bash
# Verificar instalação
which ztlgr

# Adicionar ao PATH se necessário
export PATH="$PATH:/path/to/ztlgr/target/release"

# Ou usar caminho absoluto no MCP config
{
  "command": "/full/path/to/ztlgr",
  "args": ["mcp", "--vault", "/path/to/grimoire"]
}
```

---

## Notas sobre Uso

### Hierarquia de Operações

```
1. Human curates sources → raw/
2. Human directs analysis → questions
3. LLM does grunt work → summarize, file, cross-ref
```

### O que o LLM faz vs. o que você faz

| Você (Human) | LLM (Assistant) |
|--------------|-----------------|
| Curate sources (add to raw/) | Summarize sources |
| Direct analysis (ask questions) | Cross-reference notes |
| Review updates (browse wiki) | Maintain consistency |
| Identify gaps (new topics) | Suggest connections |
| Decide what matters | File and organize |

### Custo Estimado

| Provider | Model | ~Cost/Query | Notes |
|----------|-------|--------------|-------|
| Ollama | llama3 | $0 | Local, faster |
| Ollama | llama3.2 | $0 | Smaller, faster |
| OpenAI | gpt-4o | ~$0.01-0.05 | High quality |
| OpenAI | gpt-3.5-turbo | ~$0.001 | Fast, lower quality |
| Anthropic | claude-3-5-sonnet | ~$0.003 | Excellent for analysis |

### Limitações Conhecidas

1. **Links wiki `[[target]]` são texto, não validados** — O LLM pode criar links para notas inexistentes. Use `ztlgr lint` para detectar orphans.

2. **Sem versionamento automático de conteúdo LLM** — Todas as edições são finais. Use git para rollback.

3. **Contexto limitado pelo modelo** — Notas muito grandes podem exceder o contexto. Divida em páginas menores.

4. **Sem imagens no contexto** — O LLM não lê imagens em `raw/`. Transcreva manualmente se necessário.

---

## Referências

- **[LLM Wiki Pattern (Karpathy)](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)** — Original pattern that inspired ztlgr's LLM integration. The core idea: instead of RAG on every query, the LLM maintains a persistent, compounding wiki.
- [Model Context Protocol Spec](https://spec.modelcontextprotocol.io/) — Protocol for LLM agent tool integration
- [Ollama Documentation](https://ollama.com/docs) — Local model serving
- [AGENTS.md](./AGENTS.md) — Instruções para agentes AI
- [ROADMAP-LLM-WIKI.md](./ROADMAP-LLM-WIKI.md) — Plano de implementação completo

### Acknowledgments

Much of the architecture and future direction was informed by the community discussion around Karpathy's gist, including insights from:

- **@manavgup** — Typed JSON contracts, confidence-tagged claims, wikilink resolution
- **@glaucobrito** — WIP.md for session continuity, auto-pruning, feedback loops
- **@swartzlib7** — Decay model for knowledge articles
- **@waydelyle** — Contradiction detection with typed edges
- **@roomi-fields** — Progressive disclosure in search results

The Future Enhancements section in STATUS.md incorporates several of these patterns.
- [ROADMAP-LLM-WIKI.md](./ROADMAP-LLM-WIKI.md) — Plano de implementação completo