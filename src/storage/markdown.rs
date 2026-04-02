use chrono::{DateTime, Utc};
use regex::Regex;
use std::path::Path;

use super::{Link, LinkType, Storage};
use crate::error::{Result, ZtlgrError};
use crate::note::{Metadata, Note, NoteId, NoteType, ZettelId};

pub struct MarkdownStorage {
    link_regex: Regex,
    tag_regex: Regex,
}

impl MarkdownStorage {
    pub fn new() -> Self {
        Self {
            link_regex: Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap(),
            tag_regex: Regex::new(r"#(\w+)").unwrap(),
        }
    }

    fn extract_frontmatter(&self, content: &str) -> (Option<String>, String) {
        let lines = content.lines().collect::<Vec<_>>();

        if lines.len() < 2 || lines[0] != "---" {
            return (None, content.to_string());
        }

        // Find closing ---
        if let Some(end_index) = lines[1..].iter().position(|line| *line == "---") {
            let frontmatter = lines[1..=end_index].join("\n");
            let remaining = lines[end_index + 2..].join("\n");
            (Some(frontmatter), remaining)
        } else {
            (None, content.to_string())
        }
    }

    fn parse_frontmatter(&self, frontmatter: &str) -> Result<Metadata> {
        // Parse YAML frontmatter
        let yaml: serde_yaml::Value = serde_yaml::from_str(frontmatter)
            .map_err(|e| ZtlgrError::Parse(format!("Failed to parse frontmatter: {}", e)))?;

        let mut metadata = Metadata::default();

        if let Some(obj) = yaml.as_mapping() {
            // Standard fields
            if let Some(id) = obj.get(&serde_yaml::Value::String("id".to_string())) {
                // ID is stored separately, not in metadata
            }

            if let Some(title) = obj.get(&serde_yaml::Value::String("title".to_string())) {
                // Convert serde_yaml::Value to serde_json::Value
                if let Ok(json_value) = serde_json::to_value(title) {
                    metadata.custom.insert("title".to_string(), json_value);
                }
            }

            if let Some(tags) = obj.get(&serde_yaml::Value::String("tags".to_string())) {
                if let Some(tags_vec) = tags.as_sequence() {
                    metadata.tags = Some(
                        tags_vec
                            .iter()
                            .filter_map(|t| t.as_str().map(|s| s.to_string()))
                            .collect(),
                    );
                }
            }

            if let Some(aliases) = obj.get(&serde_yaml::Value::String("aliases".to_string())) {
                if let Some(aliases_vec) = aliases.as_sequence() {
                    metadata.aliases = Some(
                        aliases_vec
                            .iter()
                            .filter_map(|a| a.as_str().map(|s| s.to_string()))
                            .collect(),
                    );
                }
            }

            // Custom fields
            for (key, value) in obj {
                if let Some(key_str) = key.as_str() {
                    if ![
                        "id",
                        "title",
                        "type",
                        "zettel_id",
                        "created",
                        "updated",
                        "parent_id",
                        "source",
                        "tags",
                        "aliases",
                    ]
                    .contains(&key_str)
                    {
                        // Convert serde_yaml::Value to serde_json::Value
                        if let Ok(json_value) = serde_json::to_value(value) {
                            metadata.custom.insert(key_str.to_string(), json_value);
                        }
                    }
                }
            }
        }

        Ok(metadata)
    }
}

impl Storage for MarkdownStorage {
    fn extension(&self) -> &str {
        "md"
    }

    fn read_note(&self, path: &Path) -> Result<Note> {
        let content = std::fs::read_to_string(path).map_err(|e| ZtlgrError::Io(e))?;

        let (frontmatter, remaining) = self.extract_frontmatter(&content);

        let metadata = if let Some(fm) = frontmatter.as_ref() {
            self.parse_frontmatter(fm)?
        } else {
            Metadata::default()
        };

        // Extract YAML from frontmatter for note fields
        let yaml: serde_yaml::Value = if let Some(fm) = frontmatter.as_ref() {
            serde_yaml::from_str(fm)
                .map_err(|e| ZtlgrError::Parse(format!("Failed to parse frontmatter: {}", e)))?
        } else {
            serde_yaml::Value::Null
        };

        let obj = yaml
            .as_mapping()
            .ok_or_else(|| ZtlgrError::Parse("Invalid frontmatter".to_string()))?;

        let id = obj
            .get(&serde_yaml::Value::String("id".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| NoteId::parse(s).unwrap_or_else(|_| NoteId::new()))
            .unwrap_or_else(NoteId::new);

        let title = obj
            .get(&serde_yaml::Value::String("title".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // Try to extract title from first heading
                remaining
                    .lines()
                    .find(|line| line.starts_with("# "))
                    .map(|line| line.trim_start_matches("# ").to_string())
                    .unwrap_or_else(|| "Untitled".to_string())
            });

        let note_type = obj
            .get(&serde_yaml::Value::String("type".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| NoteType::from_str(s).unwrap_or_default())
            .unwrap_or_default();

        let zettel_id = obj
            .get(&serde_yaml::Value::String("zettel_id".to_string()))
            .and_then(|v| v.as_str())
            .and_then(|s| ZettelId::parse(s).ok());

        let parent_id = obj
            .get(&serde_yaml::Value::String("parent_id".to_string()))
            .and_then(|v| v.as_str())
            .and_then(|s| NoteId::parse(s).ok());

        let source = obj
            .get(&serde_yaml::Value::String("source".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let created = obj
            .get(&serde_yaml::Value::String("created".to_string()))
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let updated = obj
            .get(&serde_yaml::Value::String("updated".to_string()))
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Ok(Note {
            id,
            title,
            content: remaining.to_string(),
            note_type,
            zettel_id,
            parent_id,
            source,
            metadata,
            created_at: created,
            updated_at: updated,
        })
    }

    fn write_note(&self, note: &Note, path: &Path) -> Result<()> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ZtlgrError::Io(e))?;
        }

        let content = self.render(note)?;

        std::fs::write(path, content).map_err(|e| ZtlgrError::Io(e))?;

        Ok(())
    }

    fn parse_metadata(&self, content: &str) -> Result<Metadata> {
        let (frontmatter, _) = self.extract_frontmatter(content);

        if let Some(fm) = frontmatter {
            self.parse_frontmatter(&fm)
        } else {
            Ok(Metadata::default())
        }
    }

    fn render(&self, note: &Note) -> Result<String> {
        let mut frontmatter = String::new();

        // YAML frontmatter
        frontmatter.push_str("---\n");
        frontmatter.push_str(&format!("id: {}\n", note.id));
        frontmatter.push_str(&format!(
            "title: {}\n",
            serde_yaml::to_string(&note.title).unwrap()
        ));
        frontmatter.push_str(&format!("type: {}\n", note.note_type.as_str()));

        if let Some(ref zettel_id) = note.zettel_id {
            frontmatter.push_str(&format!("zettel_id: {}\n", zettel_id));
        }

        if let Some(ref parent_id) = note.parent_id {
            frontmatter.push_str(&format!("parent_id: {}\n", parent_id));
        }

        if let Some(ref source) = note.source {
            frontmatter.push_str(&format!(
                "source: {}\n",
                serde_yaml::to_string(source).unwrap()
            ));
        }

        frontmatter.push_str(&format!("created: {}\n", note.created_at.to_rfc3339()));
        frontmatter.push_str(&format!("updated: {}\n", note.updated_at.to_rfc3339()));

        if let Some(ref tags) = note.metadata.tags {
            frontmatter.push_str(&format!("tags:\n"));
            for tag in tags {
                frontmatter.push_str(&format!("  - {}\n", tag));
            }
        }

        if let Some(ref aliases) = note.metadata.aliases {
            frontmatter.push_str(&format!("aliases:\n"));
            for alias in aliases {
                frontmatter.push_str(&format!("  - {}\n", alias));
            }
        }

        // Custom metadata
        for (key, value) in &note.metadata.custom {
            frontmatter.push_str(&format!(
                "{}: {}\n",
                key,
                serde_yaml::to_string(value).unwrap()
            ));
        }

        frontmatter.push_str("---\n\n");

        // Content
        frontmatter.push_str(&note.content);

        Ok(frontmatter)
    }

    fn extract_links(&self, content: &str) -> Result<Vec<Link>> {
        let mut links = Vec::new();

        for cap in self.link_regex.captures_iter(content) {
            let target = cap[1].to_string();
            let display_text = cap.get(2).map(|m| m.as_str().to_string());

            links.push(Link {
                source_note_id: String::new(), // Will be filled later
                target,
                link_type: LinkType::Wiki,
                display_text,
            });
        }

        Ok(links)
    }

    fn extract_tags(&self, content: &str) -> Result<Vec<String>> {
        let mut tags = Vec::new();

        for cap in self.tag_regex.captures_iter(content) {
            tags.push(cap[1].to_string());
        }

        Ok(tags)
    }
}

impl Default for MarkdownStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_and_read_note() {
        let storage = MarkdownStorage::new();
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.md");

        let note = Note::new(
            "Test Note".to_string(),
            "Content with [[link]] and #tag".to_string(),
        );

        storage.write_note(&note, &path).unwrap();
        assert!(path.exists());

        let read_note = storage.read_note(&path).unwrap();
        assert_eq!(note.title, read_note.title);
    }

    #[test]
    fn test_extract_links() {
        let storage = MarkdownStorage::new();
        let content = "This has [[link1]] and [[link2|display text]]";

        let links = storage.extract_links(content).unwrap();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target, "link1");
        assert_eq!(links[1].display_text, Some("display text".to_string()));
    }

    #[test]
    fn test_extract_tags() {
        let storage = MarkdownStorage::new();
        let content = "This has #rust #test tags";

        let tags = storage.extract_tags(content).unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"rust".to_string()));
    }
}
