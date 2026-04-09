# .skills/ -- LLM Knowledge Base Schema

This directory tells LLM agents how to operate on this Zettelkasten vault.
It is the "schema layer" of the LLM Wiki pattern -- the instructions that turn
a generic chatbot into a disciplined wiki maintainer.

## For LLM Agents

If you are an LLM agent (Claude, GPT, Codex, OpenCode, etc.) working on this vault,
read these files in order:

1. **This file** -- understand the structure
2. **conventions.md** -- naming, formatting, and frontmatter rules
3. **context/domain.md** -- what this vault is about
4. **context/priorities.md** -- current focus areas and questions
5. **workflows/** -- step-by-step instructions for common operations

## Directory Structure

```
.skills/
├── README.md              # This file
├── conventions.md         # Wiki naming, formatting, frontmatter rules
├── workflows/
│   ├── ingest.md          # Process a new source into wiki pages
│   ├── query.md           # Answer questions using the wiki
│   ├── lint.md            # Health-check the wiki
│   └── maintain.md        # Update cross-references and summaries
├── templates/
│   ├── source-summary.md  # Template for literature note from a source
│   ├── entity-page.md     # Template for entity/concept permanent note
│   ├── comparison.md      # Template for comparison/analysis pages
│   └── index-entry.md     # Template for index.md entries
└── context/
    ├── domain.md          # Domain-specific knowledge and terminology
    └── priorities.md      # Current research questions and focus areas
```

## Design Principles

- **Agent-agnostic**: works with any LLM that can read markdown files
- **Vault-local**: each vault has its own skills because each has its own domain
- **Human-editable**: customize workflows, templates, and context as needed
- **Evolvable**: suggest improvements to these files as the wiki grows
- **Git-tracked**: versioned alongside your notes

## Vault Structure

This vault follows the Zettelkasten methodology with these directories:

| Directory | Purpose | Mutability |
|-----------|---------|------------|
| `raw/` | Immutable source material (articles, papers, clips) | Read-only |
| `permanent/` | Synthesized knowledge notes | LLM + Human |
| `literature/` | Source summaries and analysis | LLM + Human |
| `index/` | Maps of Content (MOCs), index.md | LLM + Human |
| `inbox/` | Fleeting notes, quick captures | Human |
| `daily/` | Daily journal entries | Human |
| `reference/` | External URLs and bookmarks | Human |
| `attachments/` | Images and other files | N/A |

## Key Files

| File | Purpose |
|------|---------|
| `index/index.md` | Auto-generated catalog of all wiki pages |
| `.ztlgr/log.md` | Chronological activity log (ingests, queries, lint) |
| `.ztlgr/vault.db` | SQLite database (search index, link graph) |
| `.ztlgr/config.toml` | Vault configuration |
