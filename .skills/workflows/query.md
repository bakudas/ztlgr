# Workflow: Query the Wiki

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
