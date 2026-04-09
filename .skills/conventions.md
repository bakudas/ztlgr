# Wiki Conventions

Rules for naming, formatting, and organizing notes in this vault.
LLM agents MUST follow these conventions when creating or updating pages.

## File Naming

- Use kebab-case for filenames: `spaced-repetition.md`, not `Spaced Repetition.md`
- Zettel ID prefix when assigned: `1a2-spaced-repetition.md`
- Sources in `raw/` keep their original filename or use: `author-year-short-title.md`
- Daily notes use ISO date: `2026-04-09.md`

## Frontmatter (YAML)

Every note MUST have YAML frontmatter. Required fields depend on note type.

### Permanent Notes
```yaml
---
title: "Note Title"
type: permanent
tags: [tag1, tag2]
created: 2026-04-09
updated: 2026-04-09
source_refs: []          # which raw sources informed this
last_reviewed: null      # when an LLM last verified this content
confidence: medium       # high | medium | low
---
```

### Literature Notes (Source Summaries)
```yaml
---
title: "Author2026 - Short Title"
type: literature
tags: [tag1]
source: raw/author-2026-title.md
created: 2026-04-09
updated: 2026-04-09
---
```

### Index Notes (MOCs)
```yaml
---
title: "Topic Map of Content"
type: index
tags: [moc]
created: 2026-04-09
updated: 2026-04-09
---
```

## Link Format

Use wiki-style links: `[[Note Title]]` or `[[Note Title|display text]]`.
This is the canonical format. Markdown links `[text](target)` are also supported
but wiki links are preferred for internal references.

## Content Structure

### Permanent Notes
- Start with a single clear claim or idea (1-2 sentences)
- Elaborate with supporting evidence and reasoning
- End with "Related" section listing `[[links]]` to connected notes
- Keep each note focused on ONE idea (Zettelkasten principle)

### Literature Notes
- Start with bibliographic info (title, author, date, URL)
- "Key Takeaways" section (3-5 bullet points)
- "Detailed Summary" section
- "Connections" section linking to existing permanent notes
- "Questions" section for open threads

### Index Notes (MOCs)
- Start with a brief description of the topic area
- Organize linked notes into logical groups
- Use headers for grouping
- Include brief (one-line) descriptions for each linked note

## Tags

- Lowercase, hyphenated: `#knowledge-management`, not `#KnowledgeManagement`
- Use broad categories, not ultra-specific tags
- Common tags: `#methodology`, `#tool`, `#concept`, `#person`, `#book`, `#article`

## When Updating Existing Pages

1. Preserve the original author's voice and structure
2. Add new information in clearly marked sections if needed
3. Update the `updated` date in frontmatter
4. Update `source_refs` if new sources inform the content
5. If new information contradicts existing content, add a note rather than silently replacing
