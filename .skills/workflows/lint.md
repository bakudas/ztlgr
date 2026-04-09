# Workflow: Lint the Wiki

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
