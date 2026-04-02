use chrono::{DateTime, Utc};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

use super::{Link, LinkType, Storage};
use crate::error::{Result, ZtlgrError};
use crate::note::{Metadata, Note, NoteId, NoteType, ZettelId};

pub struct OrgStorage {
    link_regex: Regex,
    tag_regex: Regex,
    property_regex: Regex,
}

impl OrgStorage {
    pub fn new() -> Self {
        Self {
            link_regex: Regex::new(r"\[\[(?:id:)?([^\]]+)\]\]").unwrap(),
            tag_regex: Regex::new(r":(\w+):").unwrap(),
            property_regex: Regex::new(r"^:([A-Z_]+):\s*(.+)$").unwrap(),
        }
    }

    fn extract_properties(&self, content: &str) -> (HashMap<String, String>, String) {
        let mut properties = HashMap::new();
        let lines = content.lines().collect::<Vec<_>>();

        // Find :PROPERTIES: block
        let start = lines.iter().position(|line| line.trim() == ":PROPERTIES:");

        if let Some(start_idx) = start {
            // Find :END:
            let end = lines[start_idx..]
                .iter()
                .position(|line| line.trim() == ":END:");

            if let Some(end_idx) = end {
                // Extract properties
                for line in lines[start_idx + 1..start_idx + end_idx].iter() {
                    if let Some(cap) = self.property_regex.captures(line) {
                        let key = cap[1].to_string();
                        let value = cap[2].to_string();
                        properties.insert(key, value);
                    }
                }

                // Return remaining content
                let remaining = lines[start_idx + end_idx + 1..].join("\n");
                return (properties, remaining);
            }
        }

        (properties, content.to_string())
    }

    fn parse_properties(
        &self,
        properties: &HashMap<String, String>,
    ) -> Result<(
        Option<NoteId>,
        String,
        NoteType,
        Option<ZettelId>,
        Option<NoteId>,
        Option<String>,
        Metadata,
    )> {
        let id = properties.get("ID").and_then(|s| NoteId::parse(s).ok());

        let title = properties
            .get("TITLE")
            .map(|s| s.clone())
            .unwrap_or_else(|| "Untitled".to_string());

        let note_type = properties
            .get("TYPE")
            .and_then(|s| NoteType::from_str(s).ok())
            .unwrap_or_default();

        let zettel_id = properties
            .get("ZETTEL_ID")
            .and_then(|s| ZettelId::parse(s).ok());

        let parent_id = properties
            .get("PARENT_ID")
            .and_then(|s| NoteId::parse(s).ok());

        let source = properties.get("SOURCE").cloned();

        let mut metadata = Metadata::default();

        if let Some(tags) = properties.get("TAGS") {
            metadata.tags = Some(tags.split_whitespace().map(|s| s.to_string()).collect());
        }

        if let Some(aliases) = properties.get("ALIASES") {
            metadata.aliases = Some(aliases.split(',').map(|s| s.trim().to_string()).collect());
        }

        // Custom properties
        for (key, value) in properties {
            if ![
                "ID",
                "TITLE",
                "TYPE",
                "ZETTEL_ID",
                "PARENT_ID",
                "SOURCE",
                "CREATED",
                "UPDATED",
                "TAGS",
                "ALIASES",
                "DATE",
            ]
            .contains(&key.as_str())
            {
                metadata
                    .custom
                    .insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }

        Ok((id, title, note_type, zettel_id, parent_id, source, metadata))
    }
}

impl Storage for OrgStorage {
    fn extension(&self) -> &str {
        "org"
    }

    fn read_note(&self, path: &Path) -> Result<Note> {
        let content = std::fs::read_to_string(path).map_err(|e| ZtlgrError::Io(e))?;

        let (properties, remaining) = self.extract_properties(&content);

        let (id, title, note_type, zettel_id, parent_id, source, metadata) =
            self.parse_properties(&properties)?;

        let created = properties
            .get("CREATED")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let updated = properties
            .get("UPDATED")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Ok(Note {
            id: id.unwrap_or_else(NoteId::new),
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
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ZtlgrError::Io(e))?;
        }

        let content = self.render(note)?;

        std::fs::write(path, content).map_err(|e| ZtlgrError::Io(e))?;

        Ok(())
    }

    fn parse_metadata(&self, content: &str) -> Result<Metadata> {
        let (properties, _) = self.extract_properties(content);
        let (_, _, _, _, _, _, metadata) = self.parse_properties(&properties)?;
        Ok(metadata)
    }

    fn render(&self, note: &Note) -> Result<String> {
        let mut content = String::new();

        // :PROPERTIES: block
        content.push_str(":PROPERTIES:\n");
        content.push_str(&format!(":ID: {}\n", note.id));
        content.push_str(&format!(":TITLE: {}\n", note.title));
        content.push_str(&format!(":TYPE: {}\n", note.note_type.as_str()));

        if let Some(ref zettel_id) = note.zettel_id {
            content.push_str(&format!(":ZETTEL_ID: {}\n", zettel_id));
        }

        if let Some(ref parent_id) = note.parent_id {
            content.push_str(&format!(":PARENT_ID: {}\n", parent_id));
        }

        if let Some(ref source) = note.source {
            content.push_str(&format!(":SOURCE: {}\n", source));
        }

        content.push_str(&format!(":CREATED: {}\n", note.created_at.to_rfc3339()));
        content.push_str(&format!(":UPDATED: {}\n", note.updated_at.to_rfc3339()));

        if let Some(ref tags) = note.metadata.tags {
            content.push_str(&format!(":TAGS: {}\n", tags.join(" ")));
        }

        if let Some(ref aliases) = note.metadata.aliases {
            content.push_str(&format!(":ALIASES: {}\n", aliases.join(",")));
        }

        // Custom properties
        for (key, value) in &note.metadata.custom {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            content.push_str(&format!(":{}: {}\n", key.to_uppercase(), value_str));
        }

        content.push_str(":END:\n\n");

        // Title (Org-mode heading)
        content.push_str(&format!("* {}\n\n", note.title));

        // Content
        content.push_str(&note.content);
        content.push('\n');

        Ok(content)
    }

    fn extract_links(&self, content: &str) -> Result<Vec<Link>> {
        let mut links = Vec::new();

        for cap in self.link_regex.captures_iter(content) {
            let target = cap[1].to_string();

            let link_type = if target.starts_with("id:") {
                LinkType::Org
            } else {
                LinkType::Wiki
            };

            links.push(Link {
                source_note_id: String::new(),
                target,
                link_type,
                display_text: None,
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

impl Default for OrgStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_org_write_and_read() {
        let storage = OrgStorage::new();
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.org");

        let note = Note::new(
            "Test Note".to_string(),
            "Content with [[link]] :tag:".to_string(),
        );

        storage.write_note(&note, &path).unwrap();
        assert!(path.exists());

        let read_note = storage.read_note(&path).unwrap();
        assert_eq!(note.title, read_note.title);
    }
}
