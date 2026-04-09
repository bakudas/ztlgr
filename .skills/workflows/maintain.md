# Workflow: Maintain the Wiki

Ongoing maintenance tasks to keep the wiki consistent when changes happen.

## When to Run

- After ingesting a new source
- After a user manually creates or edits notes
- After a batch of queries that generated new pages
- Periodically (weekly is a good cadence)

## Tasks

### 1. Update Cross-References
When a new page is created or significantly updated:
- Scan existing notes for mentions of the new page's topic
- Add `[[wiki-links]]` where the topic is discussed but not linked
- Update the new page with links to related existing content

### 2. Keep Summaries Current
When source pages change:
- Check if any literature notes reference outdated information
- Update summaries to reflect the current state
- Update `updated` date in frontmatter

### 3. Maintain Index Notes (MOCs)
- Ensure every permanent/literature note appears in at least one MOC
- Remove entries for deleted notes
- Reorder entries if the topic structure has evolved
- Split large MOCs (>50 entries) into sub-topics

### 4. Refresh index.md
- Regenerate from current DB state
- Verify one-line summaries are still accurate
- Update note counts and link counts

### 5. Clean Up Tags
- Find tags used only once (consider removing or merging)
- Normalize tag casing (all lowercase, hyphenated)
- Remove tags from deleted notes

### 6. Frontmatter Consistency
- Ensure all notes have required frontmatter fields
- Fill in missing `source_refs` where known
- Update `last_reviewed` dates for verified content

## Guidelines
- Make small, incremental changes rather than large rewrites
- Preserve the original note's intent and voice
- When in doubt, add information rather than replacing
- Always update `updated` date when modifying a note
- Log significant maintenance sessions in log.md
