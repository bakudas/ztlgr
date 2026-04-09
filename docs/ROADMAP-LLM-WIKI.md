# Roadmap: LLM Wiki Integration

**Branch:** `feat/llm-wiki-integration`
**Created:** April 9, 2026
**Status:** Planning

---

## Context

This roadmap describes the evolution of ztlgr from a standalone Zettelkasten TUI into an
**LLM-maintained personal knowledge base** -- inspired by the "LLM Wiki" pattern where an
LLM incrementally builds, cross-references, and maintains a persistent wiki rather than
re-deriving knowledge from scratch on every query (as in traditional RAG).

The core insight: humans curate sources, direct analysis, and ask questions.
The LLM does the grunt work -- summarizing, cross-referencing, filing, and bookkeeping.

### What we're NOT doing

- **Not replacing Zettelkasten** -- the LLM Wiki pattern is complementary. Raw sources map
  to Literature notes, LLM summaries become Permanent notes, Index notes are already MOCs.
- **Not building RAG** -- FTS5 with BM25 is sufficient at moderate scale (~1000 notes).
  We use the index + search approach, not embedding-based retrieval.
- **Not removing the TUI** -- the TUI remains the primary interface for humans. LLM
  integration adds a parallel workflow where agents can operate on the vault.

### What we're cutting

The old "multi-agent architecture" references (NoteAgent, LinkAgent, SqliteSkill, etc.)
were aspirational documentation with zero implementation. That naming was misleading --
the actual code uses standard Rust modules. We're removing those references and replacing
them with the concrete LLM Wiki integration described here.

---

## Architecture Overview

```
                    Human                        LLM Agent
                      |                              |
                      v                              v
               ┌─────────────┐              ┌──────────────┐
               │   TUI App   │              │  .skills/    │
               │  (ratatui)  │              │  (schema +   │
               │             │              │   workflows) │
               └──────┬──────┘              └──────┬───────┘
                      |                            |
                      v                            v
               ┌────────────────────────────────────────┐
               │           CLI Layer (clap)              │
               │  new | open | search | import | sync   │
               │  ingest | ask | lint | index | mcp     │  <-- NEW
               └───────────────────┬────────────────────┘
                                   |
                                   v
               ┌────────────────────────────────────────┐
               │            Core Library                 │
               │  ┌────────┐ ┌────────┐ ┌────────────┐ │
               │  │  note/  │ │  link/ │ │   graph/   │ │
               │  │  db/    │ │storage/│ │   config/  │ │
               │  └────────┘ └────────┘ └────────────┘ │
               └───────────────────┬────────────────────┘
                                   |
                                   v
               ┌────────────────────────────────────────┐
               │         Vault (filesystem)              │
               │                                         │
               │  raw/          <- immutable sources     │
               │  permanent/    <- synthesized knowledge  │
               │  literature/   <- source summaries       │
               │  index/        <- MOCs, index.md         │
               │  inbox/        <- fleeting captures       │
               │  daily/        <- daily notes             │
               │  .ztlgr/       <- DB, config, log.md     │
               │  .skills/      <- LLM schema & workflows │
               └────────────────────────────────────────┘
```

### Three Layers (from the LLM Wiki pattern)

1. **Raw Sources** (`raw/`) -- Immutable input material. Articles, papers, transcripts,
   web clips. The LLM reads from these but never modifies them.

2. **The Wiki** (existing vault dirs) -- LLM-maintained synthesis. Summaries, entity pages,
   concept notes, comparisons. Cross-referenced and kept current.

3. **The Schema** (`.skills/`) -- Instructions that tell the LLM how the wiki is structured,
   what conventions to follow, and what workflows to execute. Co-evolved by human and LLM.

---

## .skills/ Directory Design

The `.skills/` directory lives inside each vault and serves as the "schema" layer --
it tells any LLM agent how to operate on this specific knowledge base. It is designed
to be **agent-agnostic**: works with OpenCode, Claude Code, Codex, or any tool that
reads instruction files.

```
.skills/
├── README.md              # Overview for the LLM: what this vault is about
├── conventions.md         # Wiki conventions (naming, formatting, frontmatter)
├── workflows/
│   ├── ingest.md          # How to process a new source
│   ├── query.md           # How to answer questions against the wiki
│   ├── lint.md            # How to health-check the wiki
│   └── maintain.md        # How to update cross-references and summaries
├── templates/
│   ├── source-summary.md  # Template for source summary pages
│   ├── entity-page.md     # Template for entity/concept pages
│   ├── comparison.md      # Template for comparison pages
│   └── index-entry.md     # Template for index.md entries
└── context/
    ├── domain.md          # Domain-specific knowledge and terminology
    └── priorities.md      # Current research questions and focus areas
```

**Key design decisions:**

- **Plain markdown** -- no proprietary format. Any LLM can read these files.
- **Vault-local** -- each vault has its own `.skills/` because each vault has its own
  domain, conventions, and focus.
- **Human-editable** -- you can customize workflows, add domain context, change templates.
- **LLM-evolvable** -- the LLM can suggest improvements to `.skills/` as it learns
  what works for your vault.
- **Git-tracked** -- `.skills/` is part of the vault repo, versioned alongside notes.

---

## Implementation Phases

### Phase 0: Cleanup & Foundation (this PR)
**Effort:** Small | **Impact:** Clears the path

- [ ] Remove aspirational "multi-agent" references from README, CONTRIBUTING, CHANGELOG
- [ ] Update docs/STATUS.md with new direction
- [ ] Create this roadmap document
- [ ] Update AGENTS.md with LLM Wiki context

### Phase 1: Index & Log System
**Effort:** Small | **Impact:** Medium
**Why first:** Zero external dependencies, pure Rust, immediately useful even without LLM.
Gives both humans and LLMs a navigable map of the vault.

- [ ] `index.md` auto-generation from DB (grouped by type, one-line summaries)
- [ ] `log.md` append-only activity log (ingests, syncs, queries)
- [ ] `ztlgr index` CLI command to generate/update index
- [ ] Hook into `FileSync` to regenerate on sync
- [ ] New module: `src/storage/index_generator.rs`
- [ ] New module: `src/storage/activity_log.rs`

**index.md format:**
```markdown
# Vault Index

> Auto-generated by ztlgr. Last updated: 2026-04-09T14:30:00Z
> Notes: 47 | Links: 123 | Sources: 12

## Permanent Notes
- [[Zettelkasten Method]] -- Core methodology for knowledge management (#methodology)
- [[Spaced Repetition]] -- Memory technique using increasing intervals (#learning)

## Literature Notes
- [[Ahrens2017]] -- Summary of "How to Take Smart Notes" (source: raw/ahrens-2017.md)

## Index Notes
- [[Knowledge Management MOC]] -- Map of content for KM topics

## Daily Notes
- [[2026-04-09]] -- Today's journal entry

## Sources (raw/)
- raw/ahrens-2017.md -- "How to Take Smart Notes" by Sonke Ahrens (ingested: 2026-04-01)
```

**log.md format:**
```markdown
# Activity Log

## [2026-04-09] sync | Full vault sync
- Files synced: 47
- New notes: 2
- Updated: 5

## [2026-04-09] ingest | "How to Take Smart Notes"
- Source: raw/ahrens-2017.md
- Pages created: 3 (summary, Zettelkasten Method, Sonke Ahrens)
- Pages updated: 2 (Knowledge Management MOC, index.md)
```

### Phase 2: Raw Sources Layer
**Effort:** Medium | **Impact:** High
**Why second:** Establishes the foundation the LLM Wiki pattern requires -- a separation
between immutable input and mutable synthesis.

- [ ] Add `raw/` to `VAULT_DIRS` and `Vault::initialize()`
- [ ] New DB table: `sources` (id, title, origin, content_hash, ingested_at, file_path)
- [ ] Schema migration system (v1 -> v2)
- [ ] `ztlgr ingest <file>` CLI command (copies to `raw/`, registers in DB)
- [ ] `ztlgr ingest --url <url>` (downloads article as markdown, stores in `raw/`)
- [ ] Source metadata in frontmatter (origin URL, author, date, hash)
- [ ] Sources appear in index.md but are read-only in editor
- [ ] New module: `src/source/mod.rs` (Source struct, SourceId)
- [ ] New module: `src/source/ingest.rs` (ingestion pipeline)

**Dependencies:** Phase 1 (for index/log updates)

### Phase 3: .skills/ Infrastructure
**Effort:** Medium | **Impact:** High
**Why third:** This is what makes the vault LLM-operable. Without `.skills/`, the LLM
has no instructions for how to maintain the wiki.

- [ ] `ztlgr init-skills` CLI command (generates `.skills/` with defaults)
- [ ] Default templates for all workflow files
- [ ] Vault-aware defaults (detect note types, count stats, populate domain.md)
- [ ] `.skills/conventions.md` generated from current vault config
- [ ] Integration with `ztlgr new` (offer to create `.skills/` during vault creation)
- [ ] New module: `src/skills/mod.rs` (Skills struct, loader)
- [ ] New module: `src/skills/generator.rs` (default content generation)

**Dependencies:** Phase 2 (sources referenced in workflows)

### Phase 4: LLM Provider Abstraction
**Effort:** Large | **Impact:** Very High
**Why fourth:** Now that the vault has sources, index, log, and skills, we can add
the LLM as the engine that operates on all of it.

- [ ] LLM provider trait: `fn complete(prompt, context) -> Result<String>`
- [ ] OpenAI provider (GPT-4o, o3)
- [ ] Anthropic provider (Claude)
- [ ] Ollama provider (local models)
- [ ] Config section in `.ztlgr/config.toml`: `[llm]` with provider, model, api_key_env
- [ ] API key management (env vars, not stored in vault files)
- [ ] Token/cost tracking in log.md
- [ ] New module: `src/llm/mod.rs`
- [ ] New module: `src/llm/provider.rs` (trait + impls)
- [ ] New module: `src/llm/context.rs` (context building from wiki pages)
- [ ] New dependency: `reqwest` for HTTP

**Dependencies:** Phase 3 (skills provide the system prompt)

### Phase 5: LLM Workflows (Ingest, Query, Lint)
**Effort:** Large | **Impact:** Very High
**Why fifth:** The core value -- automated wiki maintenance.

#### 5A: Ingest Workflow
- [ ] Read source from `raw/`, build prompt from `.skills/workflows/ingest.md`
- [ ] LLM generates summary page (filed in `literature/`)
- [ ] LLM identifies entities/concepts, creates or updates pages
- [ ] LLM updates `index.md` with new entries
- [ ] LLM appends to `log.md`
- [ ] `ztlgr ingest <file> --process` CLI (ingest + LLM processing)
- [ ] TUI command: `:ingest <file>`

#### 5B: Query Workflow
- [ ] Read index.md to find relevant pages
- [ ] Use FTS5 search as fallback/supplement
- [ ] Build context from relevant wiki pages
- [ ] LLM synthesizes answer with `[[wiki-link]]` citations
- [ ] Option to file answer as new note
- [ ] `ztlgr ask "<question>"` CLI
- [ ] TUI command: `:ask <question>`

#### 5C: Lint Workflow
- [ ] Detect orphan notes (no inbound links)
- [ ] Find notes referencing deleted/missing targets
- [ ] Identify stale content (old `last_reviewed` dates)
- [ ] Suggest missing cross-references
- [ ] Flag potential contradictions
- [ ] `ztlgr lint` CLI
- [ ] TUI command: `:lint`

**Dependencies:** Phase 4 (LLM provider)

### Phase 6: MCP Server
**Effort:** Medium | **Impact:** High
**Why last:** Complements internal LLM integration with external access. Lets any
LLM agent use ztlgr as a knowledge tool without the TUI.

- [ ] MCP server implementation (JSON-RPC over stdio)
- [ ] Tools: `search`, `get_note`, `list_notes`, `create_note`, `update_note`
- [ ] Tools: `get_backlinks`, `get_graph`, `ingest_source`
- [ ] Tools: `read_index`, `read_log`, `read_skills`
- [ ] `ztlgr mcp` CLI command to start server
- [ ] Config for MCP in `.ztlgr/config.toml`
- [ ] New module: `src/mcp/mod.rs`
- [ ] New module: `src/mcp/server.rs`
- [ ] New module: `src/mcp/tools.rs`

**Dependencies:** Phase 5 (full wiki operations available)

---

## Cost x Benefit Matrix

| Phase | Effort | Benefit | Dependencies | New Crates |
|-------|--------|---------|--------------|------------|
| 0: Cleanup | 1-2h | Clarity | None | None |
| 1: Index & Log | 1-2 days | Medium | None | None |
| 2: Raw Sources | 2-3 days | High | Phase 1 | None |
| 3: .skills/ | 2-3 days | High | Phase 2 | None |
| 4: LLM Provider | 3-5 days | Very High | Phase 3 | `reqwest` |
| 5: LLM Workflows | 5-7 days | Very High | Phase 4 | None |
| 6: MCP Server | 3-5 days | High | Phase 5 | `serde_json` (already) |

**Total estimated effort:** ~3-4 weeks of focused development

---

## Enhanced Metadata Fields

To support LLM-maintained wikis, notes gain optional frontmatter fields:

```yaml
---
title: "Zettelkasten Method"
type: permanent
tags: [methodology, knowledge-management]
# LLM Wiki fields (Phase 5+)
source_refs:
  - raw/ahrens-2017.md
  - raw/luhmann-1981.md
last_reviewed: 2026-04-09
confidence: high        # high | medium | low
superseded_by: null     # [[newer-note]] if this is outdated
contradicts: []         # [[other-note]] for flagged conflicts
generated_by: claude    # which LLM generated/updated this
---
```

---

## Relation to Existing Zettelkasten Types

| LLM Wiki Concept | ztlgr Zettelkasten Type | Notes |
|---|---|---|
| Raw Source | `raw/` (new) | Immutable input, not a note type |
| Source Summary | `NoteType::Literature` | LLM-generated summary of a source |
| Entity/Concept Page | `NoteType::Permanent` | LLM-maintained knowledge pages |
| Index / MOC | `NoteType::Index` | Maps of content, auto-updated |
| Quick Capture | `NoteType::Fleeting` | Human-written, inbox for processing |
| Daily Journal | `NoteType::Daily` | Human-written, daily reflections |
| Reference | `NoteType::Reference` | External URLs, bookmarks |

---

## Open Questions

1. **Should `.skills/` be inside the vault or in `~/.config/ztlgr/skills/`?**
   Current decision: inside the vault, because each vault has different domain/conventions.
   Global defaults can live in `~/.config/ztlgr/default-skills/`.

2. **Should the LLM write files directly or go through the DB?**
   Current decision: go through the normal note creation/update pipeline (DB + file sync).
   This ensures all indexes, FTS, and link graphs stay consistent.

3. **How to handle LLM costs?**
   Log token usage in `log.md`. Add `--dry-run` flag to preview what the LLM would do
   without making changes. Support local models via Ollama for zero-cost operation.

4. **Should `index.md` be LLM-generated or code-generated?**
   Phase 1: code-generated (structured from DB). Phase 5+: LLM can enrich with
   better summaries and groupings, but the code-generated version is the baseline.
