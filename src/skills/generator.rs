use std::path::Path;

use crate::error::{Result, ZtlgrError};

use super::{SKILLS_DIRS, SKILLS_FILES};

/// Generates the default .skills/ directory tree with all template files.
///
/// The generator can be customized with a vault name and description before
/// calling `generate()`.
#[derive(Debug, Clone)]
pub struct SkillsGenerator {
    /// Vault name (used in README and domain context)
    vault_name: String,
    /// Vault description (used in domain context)
    vault_description: String,
}

/// Result of generating .skills/ files.
#[derive(Debug, Clone)]
pub struct GenerateResult {
    /// Number of files created
    pub files_created: usize,
    /// Number of files skipped (already existed)
    pub files_skipped: usize,
}

impl SkillsGenerator {
    /// Create a new generator with the given vault name.
    pub fn new(vault_name: &str) -> Self {
        Self {
            vault_name: vault_name.to_string(),
            vault_description: format!(
                "A personal knowledge base managed with ztlgr, named \"{}\".",
                vault_name
            ),
        }
    }

    /// Override the default vault description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.vault_description = description.to_string();
        self
    }

    /// Generate the .skills/ directory and all default files in the given vault.
    ///
    /// Existing files are NOT overwritten (skipped). This makes the command safe
    /// to run multiple times — it fills in missing files without touching customized ones.
    pub fn generate(&self, vault_path: &Path) -> Result<GenerateResult> {
        let skills_root = vault_path.join(".skills");

        // Create directories
        for &dir in SKILLS_DIRS {
            std::fs::create_dir_all(skills_root.join(dir)).map_err(|e| {
                ZtlgrError::Skills(format!("failed to create directory {}: {}", dir, e))
            })?;
        }

        let mut files_created = 0;
        let mut files_skipped = 0;

        for &file in SKILLS_FILES {
            let path = skills_root.join(file);
            if path.exists() {
                files_skipped += 1;
                continue;
            }

            let content = self.content_for(file);
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ZtlgrError::Skills(format!("failed to create parent for {}: {}", file, e))
                })?;
            }
            std::fs::write(&path, content)
                .map_err(|e| ZtlgrError::Skills(format!("failed to write {}: {}", file, e)))?;
            files_created += 1;
        }

        Ok(GenerateResult {
            files_created,
            files_skipped,
        })
    }

    /// Get the default content for a given skills file.
    fn content_for(&self, file: &str) -> String {
        match file {
            "README.md" => self.readme_content(),
            "conventions.md" => self.conventions_content(),
            "workflows/ingest.md" => self.workflow_ingest_content(),
            "workflows/query.md" => self.workflow_query_content(),
            "workflows/lint.md" => self.workflow_lint_content(),
            "workflows/maintain.md" => self.workflow_maintain_content(),
            "templates/source-summary.md" => self.template_source_summary(),
            "templates/entity-page.md" => self.template_entity_page(),
            "templates/comparison.md" => self.template_comparison(),
            "templates/index-entry.md" => self.template_index_entry(),
            "context/domain.md" => self.context_domain(),
            "context/priorities.md" => self.context_priorities(),
            _ => format!("# {}\n\nTODO: add content\n", file),
        }
    }

    // =====================================================================
    // Content generators
    // =====================================================================

    fn readme_content(&self) -> String {
        format!(
            r#"# .skills/ -- LLM Knowledge Base Schema

This directory tells LLM agents how to operate on this Zettelkasten grimoire.
It is the "schema layer" of the LLM Wiki pattern -- the instructions that turn
a generic chatbot into a disciplined wiki maintainer.

> **Local-first, human-first.** This grimoire works fully without any LLM.
> Your files live on your machine, no cloud required. The LLM is an optional
> amplifier that helps organize and connect knowledge -- you stay in control.

## For LLM Agents

If you are an LLM agent (Claude, GPT, Codex, OpenCode, etc.) working on this grimoire,
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

- **Local-first**: the grimoire works 100% without any LLM or network connection
- **Human-first**: you (the owner) direct; the LLM assists. Privacy by default
- **Agent-agnostic**: works with any LLM that can read markdown files
- **Vault-local**: each grimoire has its own skills because each has its own domain
- **Human-editable**: customize workflows, templates, and context as needed
- **Evolvable**: suggest improvements to these files as the wiki grows
- **Git-tracked**: versioned alongside your notes

## Grimoire Structure

This grimoire ("{name}") follows the Zettelkasten methodology with these directories:

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
"#,
            name = self.vault_name
        )
    }

    fn conventions_content(&self) -> String {
        r#"# Wiki Conventions

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
"#
        .to_string()
    }

    fn workflow_ingest_content(&self) -> String {
        r#"# Workflow: Ingest a New Source

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
"#
        .to_string()
    }

    fn workflow_query_content(&self) -> String {
        r#"# Workflow: Query the Wiki

Follow these steps when answering a question using the wiki's accumulated knowledge.

## Trigger

User runs `ztlgr ask "<question>"` or asks a question in conversation.

## Steps

### 1. Read the Index
- Start by reading `index/index.md` to identify relevant pages
- Note which topic areas, entities, and sources might be relevant

### 2. Search for Relevant Pages
- Use FTS5 search (`ztlgr search`) for keywords from the question
- Follow wiki links from index pages to drill into specific content
- Read the most relevant 5-10 pages in full

### 3. Synthesize an Answer
- Combine information from multiple pages
- Use `[[wiki-links]]` as inline citations
- If pages contradict each other, note the contradiction and explain
- If the wiki doesn't have enough information, say so clearly

### 4. Format the Response
- For factual questions: direct answer with citations
- For analysis questions: structured markdown with headers
- For comparison questions: use a table format
- Always cite sources with `[[page-title]]` links

### 5. Optionally File the Answer
If the answer represents valuable synthesized knowledge:
- Ask the user if they want to save it as a permanent note
- Use a descriptive title
- Add `source_refs` pointing to the pages used
- Add wiki links to/from related pages
- Update index.md

### 6. Log the Query
```markdown
## [YYYY-MM-DD] query | "Question text"
- Pages consulted: N (list key ones)
- Answer filed as: [[Note Title]] (or "not filed")
```

## Guidelines
- Prefer wiki content over general knowledge -- the wiki is the source of truth
- If the wiki is incomplete, say "the wiki doesn't cover X" rather than filling in
  from general knowledge (unless the user asks for it)
- When citing, use the exact note title in `[[brackets]]`
"#
        .to_string()
    }

    fn workflow_lint_content(&self) -> String {
        r#"# Workflow: Lint the Wiki

Periodic health check to keep the wiki consistent, current, and well-connected.

## Trigger

User runs `ztlgr lint` or asks the LLM to review wiki health.

## Checks

### 1. Orphan Notes
- Find notes with zero inbound links (no other note links to them)
- Exclude daily notes and index notes (these are entry points)
- **Action**: suggest which existing notes should link to the orphan, or flag for deletion

### 2. Broken Links
- Find `[[wiki-links]]` that don't resolve to any existing note
- **Action**: create the missing page, fix the link target, or remove the link

### 3. Stale Content
- Find notes where `last_reviewed` is older than 90 days (configurable)
- Find notes where `confidence: low`
- **Action**: review and update, or mark for review

### 4. Missing Cross-References
- Find notes that discuss the same topics but don't link to each other
- Use tag overlap and content similarity as signals
- **Action**: suggest adding `[[links]]` between related notes

### 5. Contradictions
- Find notes that make conflicting claims about the same topic
- Look for phrases like "however", "in contrast", "unlike"
- **Action**: flag the contradiction, suggest resolution or a comparison page

### 6. Incomplete Pages
- Find notes with very short content (< 100 words) that aren't fleeting notes
- Find literature notes missing key sections (no "Key Takeaways", no source link)
- **Action**: suggest expanding or merging with another note

### 7. Index Freshness
- Compare `index.md` against actual notes in the DB
- Find notes not listed in any index
- Find index entries pointing to deleted notes
- **Action**: regenerate index

### 8. Source Coverage
- List sources in `raw/` that have no corresponding literature note
- **Action**: suggest processing these unprocessed sources

## Output Format

```markdown
# Wiki Lint Report -- YYYY-MM-DD

## Summary
- Orphan notes: N
- Broken links: N
- Stale notes: N
- Missing cross-refs: N (suggested)
- Unprocessed sources: N

## Orphan Notes
- [[Note Title]] -- created YYYY-MM-DD, tags: #foo
  Suggestion: link from [[Related Note]]

## Broken Links
- In [[Source Note]]: [[Missing Target]] (line 42)
  Suggestion: create page or fix link

## Stale Notes
- [[Old Note]] -- last reviewed YYYY-MM-DD (N days ago)
  confidence: low

...
```

## Log Entry
```markdown
## [YYYY-MM-DD] lint | Wiki health check
- Orphan notes: N
- Broken links: N
- Stale notes: N
- Issues resolved: N
- Issues remaining: N
```
"#
        .to_string()
    }

    fn workflow_maintain_content(&self) -> String {
        r#"# Workflow: Maintain the Wiki

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
"#
        .to_string()
    }

    fn template_source_summary(&self) -> String {
        r#"---
title: "${AUTHOR}${YEAR} - ${SHORT_TITLE}"
type: literature
tags: []
source: raw/${FILENAME}
created: ${DATE}
updated: ${DATE}
---

# ${FULL_TITLE}

**Author:** ${AUTHOR}
**Date:** ${YEAR}
**Source:** [Original](${URL})

## Key Takeaways

1. 
2. 
3. 

## Detailed Summary

<!-- Summarize the main arguments, findings, or narrative -->

## Notable Quotes

> "Quote here" (p. N)

## Connections

- Related to [[existing-note]] because...
- Contradicts/supports [[other-note]] regarding...

## Questions

- [ ] Open question raised by this source
- [ ] Follow-up to investigate
"#
        .to_string()
    }

    fn template_entity_page(&self) -> String {
        r#"---
title: "${ENTITY_NAME}"
type: permanent
tags: []
source_refs: []
created: ${DATE}
updated: ${DATE}
last_reviewed: null
confidence: medium
---

# ${ENTITY_NAME}

<!-- One clear sentence defining this entity or concept -->

## Overview

<!-- 2-3 paragraphs explaining the concept, its significance, and key aspects -->

## Key Points

- 
- 
- 

## Evidence & Sources

- From [[literature-note-1]]: ...
- From [[literature-note-2]]: ...

## Related

- [[Related Concept 1]] -- how it relates
- [[Related Concept 2]] -- how it relates

## Open Questions

- [ ] What remains unclear or worth investigating
"#
        .to_string()
    }

    fn template_comparison(&self) -> String {
        r#"---
title: "${TOPIC_A} vs ${TOPIC_B}"
type: permanent
tags: [comparison]
source_refs: []
created: ${DATE}
updated: ${DATE}
confidence: medium
---

# ${TOPIC_A} vs ${TOPIC_B}

## Summary

<!-- One paragraph stating the key differences and similarities -->

## Comparison

| Aspect | ${TOPIC_A} | ${TOPIC_B} |
|--------|-----------|-----------|
| Key feature | ... | ... |
| Strength | ... | ... |
| Weakness | ... | ... |
| Best for | ... | ... |

## Detailed Analysis

### ${TOPIC_A}

<!-- Key characteristics, drawn from wiki pages -->

### ${TOPIC_B}

<!-- Key characteristics, drawn from wiki pages -->

## When to Choose Which

- Choose ${TOPIC_A} when: ...
- Choose ${TOPIC_B} when: ...

## Sources

- [[literature-note-1]]
- [[literature-note-2]]
"#
        .to_string()
    }

    fn template_index_entry(&self) -> String {
        "- [[${NOTE_TITLE}]] -- ${ONE_LINE_SUMMARY} (${NOTE_TYPE}, ${TAGS})\n".to_string()
    }

    fn context_domain(&self) -> String {
        format!(
            r#"# Domain Context

<!-- This file describes what this vault is about. Update it as the vault's focus evolves. -->
<!-- LLM agents read this to understand the domain before operating on the wiki. -->

## About This Vault

{description}

## Domain

<!-- What topics does this vault cover? What is the primary focus area? -->

General knowledge management and personal notes.

## Key Terminology

<!-- Domain-specific terms and their definitions that the LLM should know -->

| Term | Definition |
|------|-----------|
| Zettelkasten | A method of knowledge management using atomic, interlinked notes |
| MOC | Map of Content -- an index note that organizes notes by topic |
| Fleeting note | A quick capture, not yet processed into permanent knowledge |
| Literature note | A summary of an external source (book, article, paper) |
| Permanent note | A single idea expressed in your own words, linked to evidence |

## Conventions

See `../conventions.md` for full formatting and naming rules.
"#,
            description = self.vault_description,
        )
    }

    fn context_priorities(&self) -> String {
        r#"# Current Priorities

<!-- What are you currently focused on? What questions are you trying to answer? -->
<!-- LLM agents use this to prioritize what to emphasize during ingests and queries. -->

## Active Research Questions

<!-- Questions you're currently investigating -->

1. _Add your current questions here_

## Focus Areas

<!-- Topics to emphasize when processing new sources -->

- _Add your focus areas here_

## Backburner

<!-- Topics that are interesting but not urgent -->

- _Add deferred topics here_

## Recently Resolved

<!-- Questions that have been answered -- move here from Active when done -->

_None yet_
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_vault(temp: &TempDir) -> PathBuf {
        let vault_path = temp.path().join("vault");
        std::fs::create_dir_all(vault_path.join(".ztlgr")).unwrap();
        vault_path
    }

    // =====================================================================
    // SkillsGenerator construction
    // =====================================================================

    #[test]
    fn test_generator_new() {
        let gen = SkillsGenerator::new("my-grimoire");
        assert_eq!(gen.vault_name, "my-grimoire");
        assert!(gen.vault_description.contains("my-grimoire"));
    }

    #[test]
    fn test_generator_with_description() {
        let gen = SkillsGenerator::new("test").with_description("Custom desc");
        assert_eq!(gen.vault_description, "Custom desc");
    }

    // =====================================================================
    // generate() -- full directory creation
    // =====================================================================

    #[test]
    fn test_generate_creates_all_files() {
        let temp = TempDir::new().unwrap();
        let vault_path = setup_vault(&temp);

        let gen = SkillsGenerator::new("test-vault");
        let result = gen.generate(&vault_path).unwrap();

        assert_eq!(result.files_created, 12);
        assert_eq!(result.files_skipped, 0);

        // Verify all files exist
        let skills_root = vault_path.join(".skills");
        for &file in crate::skills::SKILLS_FILES {
            let path = skills_root.join(file);
            assert!(path.exists(), "Missing file: {}", file);
            let content = std::fs::read_to_string(&path).unwrap();
            assert!(!content.is_empty(), "Empty file: {}", file);
        }
    }

    #[test]
    fn test_generate_creates_subdirectories() {
        let temp = TempDir::new().unwrap();
        let vault_path = setup_vault(&temp);

        let gen = SkillsGenerator::new("test");
        gen.generate(&vault_path).unwrap();

        let skills_root = vault_path.join(".skills");
        for &dir in crate::skills::SKILLS_DIRS {
            assert!(skills_root.join(dir).is_dir(), "Missing directory: {}", dir);
        }
    }

    #[test]
    fn test_generate_skips_existing_files() {
        let temp = TempDir::new().unwrap();
        let vault_path = setup_vault(&temp);
        let skills_root = vault_path.join(".skills");

        // Create one file manually
        std::fs::create_dir_all(skills_root.join("workflows")).unwrap();
        std::fs::write(skills_root.join("README.md"), "# Custom README").unwrap();

        let gen = SkillsGenerator::new("test");
        let result = gen.generate(&vault_path).unwrap();

        assert_eq!(result.files_created, 11);
        assert_eq!(result.files_skipped, 1);

        // Verify custom README was NOT overwritten
        let content = std::fs::read_to_string(skills_root.join("README.md")).unwrap();
        assert_eq!(content, "# Custom README");
    }

    #[test]
    fn test_generate_idempotent() {
        let temp = TempDir::new().unwrap();
        let vault_path = setup_vault(&temp);

        let gen = SkillsGenerator::new("test");
        let first = gen.generate(&vault_path).unwrap();
        let second = gen.generate(&vault_path).unwrap();

        assert_eq!(first.files_created, 12);
        assert_eq!(second.files_created, 0);
        assert_eq!(second.files_skipped, 12);
    }

    // =====================================================================
    // Content generation -- README
    // =====================================================================

    #[test]
    fn test_readme_contains_vault_name() {
        let gen = SkillsGenerator::new("my-grimoire");
        let content = gen.readme_content();
        assert!(content.contains("my-grimoire"));
    }

    #[test]
    fn test_readme_contains_structure_table() {
        let gen = SkillsGenerator::new("test");
        let content = gen.readme_content();
        assert!(content.contains("| Directory | Purpose | Mutability |"));
        assert!(content.contains("raw/"));
        assert!(content.contains("permanent/"));
    }

    #[test]
    fn test_readme_contains_local_first() {
        let gen = SkillsGenerator::new("test");
        let content = gen.readme_content();
        assert!(content.contains("Local-first"));
        assert!(content.contains("human-first"));
    }

    #[test]
    fn test_readme_contains_directory_tree() {
        let gen = SkillsGenerator::new("test");
        let content = gen.readme_content();
        assert!(content.contains(".skills/"));
        assert!(content.contains("workflows/"));
        assert!(content.contains("templates/"));
        assert!(content.contains("context/"));
    }

    // =====================================================================
    // Content generation -- conventions
    // =====================================================================

    #[test]
    fn test_conventions_contains_frontmatter_examples() {
        let gen = SkillsGenerator::new("test");
        let content = gen.conventions_content();
        assert!(content.contains("Frontmatter (YAML)"));
        assert!(content.contains("type: permanent"));
        assert!(content.contains("type: literature"));
        assert!(content.contains("type: index"));
    }

    #[test]
    fn test_conventions_contains_link_format() {
        let gen = SkillsGenerator::new("test");
        let content = gen.conventions_content();
        assert!(content.contains("[[Note Title]]"));
    }

    #[test]
    fn test_conventions_contains_tag_rules() {
        let gen = SkillsGenerator::new("test");
        let content = gen.conventions_content();
        assert!(content.contains("#knowledge-management"));
    }

    // =====================================================================
    // Content generation -- workflows
    // =====================================================================

    #[test]
    fn test_workflow_ingest_has_steps() {
        let gen = SkillsGenerator::new("test");
        let content = gen.workflow_ingest_content();
        assert!(content.contains("## Trigger"));
        assert!(content.contains("### 1. Read the Source"));
        assert!(content.contains("### 3. Create a Literature Note"));
        assert!(content.contains("## Quality Checklist"));
    }

    #[test]
    fn test_workflow_query_has_steps() {
        let gen = SkillsGenerator::new("test");
        let content = gen.workflow_query_content();
        assert!(content.contains("## Trigger"));
        assert!(content.contains("### 1. Read the Index"));
        assert!(content.contains("## Guidelines"));
    }

    #[test]
    fn test_workflow_lint_has_checks() {
        let gen = SkillsGenerator::new("test");
        let content = gen.workflow_lint_content();
        assert!(content.contains("### 1. Orphan Notes"));
        assert!(content.contains("### 8. Source Coverage"));
        assert!(content.contains("## Output Format"));
    }

    #[test]
    fn test_workflow_maintain_has_tasks() {
        let gen = SkillsGenerator::new("test");
        let content = gen.workflow_maintain_content();
        assert!(content.contains("### 1. Update Cross-References"));
        assert!(content.contains("### 6. Frontmatter Consistency"));
        assert!(content.contains("## Guidelines"));
    }

    // =====================================================================
    // Content generation -- templates
    // =====================================================================

    #[test]
    fn test_template_source_summary_has_placeholders() {
        let gen = SkillsGenerator::new("test");
        let content = gen.template_source_summary();
        assert!(content.contains("${AUTHOR}"));
        assert!(content.contains("${YEAR}"));
        assert!(content.contains("${FILENAME}"));
        assert!(content.contains("## Key Takeaways"));
    }

    #[test]
    fn test_template_entity_page_has_placeholders() {
        let gen = SkillsGenerator::new("test");
        let content = gen.template_entity_page();
        assert!(content.contains("${ENTITY_NAME}"));
        assert!(content.contains("confidence: medium"));
        assert!(content.contains("## Evidence & Sources"));
    }

    #[test]
    fn test_template_comparison_has_table() {
        let gen = SkillsGenerator::new("test");
        let content = gen.template_comparison();
        assert!(content.contains("${TOPIC_A}"));
        assert!(content.contains("${TOPIC_B}"));
        assert!(content.contains("| Aspect |"));
    }

    #[test]
    fn test_template_index_entry_format() {
        let gen = SkillsGenerator::new("test");
        let content = gen.template_index_entry();
        assert!(content.contains("${NOTE_TITLE}"));
        assert!(content.contains("${ONE_LINE_SUMMARY}"));
    }

    // =====================================================================
    // Content generation -- context
    // =====================================================================

    #[test]
    fn test_context_domain_contains_vault_description() {
        let gen = SkillsGenerator::new("research-vault");
        let content = gen.context_domain();
        assert!(content.contains("research-vault"));
    }

    #[test]
    fn test_context_domain_custom_description() {
        let gen = SkillsGenerator::new("test").with_description("A vault about quantum computing");
        let content = gen.context_domain();
        assert!(content.contains("A vault about quantum computing"));
    }

    #[test]
    fn test_context_domain_has_terminology_table() {
        let gen = SkillsGenerator::new("test");
        let content = gen.context_domain();
        assert!(content.contains("| Term | Definition |"));
        assert!(content.contains("Zettelkasten"));
        assert!(content.contains("MOC"));
    }

    #[test]
    fn test_context_priorities_has_sections() {
        let gen = SkillsGenerator::new("test");
        let content = gen.context_priorities();
        assert!(content.contains("## Active Research Questions"));
        assert!(content.contains("## Focus Areas"));
        assert!(content.contains("## Backburner"));
        assert!(content.contains("## Recently Resolved"));
    }

    // =====================================================================
    // content_for dispatch
    // =====================================================================

    #[test]
    fn test_content_for_all_known_files() {
        let gen = SkillsGenerator::new("test");
        for &file in crate::skills::SKILLS_FILES {
            let content = gen.content_for(file);
            assert!(!content.is_empty(), "Empty content for: {}", file);
        }
    }

    #[test]
    fn test_content_for_unknown_file_returns_placeholder() {
        let gen = SkillsGenerator::new("test");
        let content = gen.content_for("unknown/file.md");
        assert!(content.contains("TODO: add content"));
    }
}
