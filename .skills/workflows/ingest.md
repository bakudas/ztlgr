# Workflow: Ingest a New Source

Follow these steps when processing a new source document into the wiki.

## Trigger

User runs `ztlgr ingest <file>` or tells the LLM to process a new source.

## Steps

### 1. Read the Source
- Read the file from `raw/` directory
- Identify: title, author, date, URL/origin, main topic

### 2. Discuss with the User (if interactive)
- Summarize the key takeaways (3-5 bullet points)
- Ask if there are specific aspects to emphasize
- Note any connections to existing vault content the user cares about

### 3. Create a Literature Note
- Use template: `.skills/templates/source-summary.md`
- File in `literature/` directory
- Include: bibliographic info, key takeaways, detailed summary
- Link to the raw source: `source: raw/filename.md`

### 4. Update Entity/Concept Pages
For each significant entity or concept mentioned in the source:
- **If a page exists**: update it with new information, add source to `source_refs`
- **If no page exists**: create a new permanent note using `.skills/templates/entity-page.md`
- Always add wiki links between the literature note and entity pages

### 5. Update Index Notes (MOCs)
- Check existing index notes for relevant topic areas
- Add the new literature note and any new permanent notes to appropriate MOCs
- If no relevant MOC exists and the topic is significant, create one

### 6. Update index.md
- Add entries for all new pages
- Update summaries for modified pages
- Keep grouping consistent

### 7. Append to log.md
Format:
```markdown
## [YYYY-MM-DD] ingest | "Source Title"
- Source: raw/filename.md
- Pages created: N (list them)
- Pages updated: N (list them)
- New links: N
```

## Quality Checklist
- [ ] Literature note has complete frontmatter
- [ ] All new pages have wiki links to/from related content
- [ ] No orphan pages created (every new page links to at least one existing page)
- [ ] Index notes updated
- [ ] index.md updated
- [ ] log.md updated
