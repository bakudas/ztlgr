use serde::{Deserialize, Serialize};
use std::collections::HashMap as StdMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    // Standard fields
    pub aliases: Option<Vec<String>>,
    pub cssclasses: Option<Vec<String>>,
    pub publish: Option<bool>,
    pub date: Option<String>,
    pub tags: Option<Vec<String>>,

    // Custom fields
    pub custom: StdMap<String, serde_json::Value>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_alias(mut self, alias: String) -> Self {
        self.aliases.get_or_insert_with(Vec::new).push(alias);
        self
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.get_or_insert_with(Vec::new).push(tag);
        self
    }

    pub fn with_custom(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom.insert(key, value);
        self
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        // Parse YAML frontmatter
        let lines: Vec<&str> = yaml.lines().collect();
        if lines.len() < 2 || lines[0] != "---" || lines[lines.len() - 1] != "---" {
            return Err("Invalid YAML frontmatter".to_string());
        }

        let yaml_content = lines[1..lines.len() - 1].join("\n");

        let yaml: serde_yaml::Value = serde_yaml::from_str(&yaml_content)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;

        let mut metadata = Metadata::default();

        if let Some(obj) = yaml.as_mapping() {
            // Standard fields
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

            // Custom fields - convert YAML values to JSON values
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
                        let json_value = yaml_to_json(value);
                        metadata.custom.insert(key_str.to_string(), json_value);
                    }
                }
            }
        }

        Ok(metadata)
    }

    pub fn to_yaml(&self) -> Result<String, String> {
        let mut yaml = String::new();
        yaml.push_str("---\n");

        if let Some(ref tags) = self.tags {
            yaml.push_str("tags:\n");
            for tag in tags {
                yaml.push_str(&format!("  - {}\n", tag));
            }
        }

        if let Some(ref aliases) = self.aliases {
            yaml.push_str("aliases:\n");
            for alias in aliases {
                yaml.push_str(&format!("  - {}\n", alias));
            }
        }

        if let Some(publish) = self.publish {
            yaml.push_str(&format!("publish: {}\n", publish));
        }

        if let Some(ref date) = self.date {
            yaml.push_str(&format!("date: {}\n", date));
        }

        // Custom fields
        for (key, value) in &self.custom {
            yaml.push_str(&format!("{}: {}\n", key, json_to_yaml_string(value)));
        }

        yaml.push_str("---\n");

        Ok(yaml)
    }
}

fn yaml_to_json(yaml: &serde_yaml::Value) -> serde_json::Value {
    match yaml {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Value::Number(
                    serde_json::Number::from_f64(f).unwrap_or_else(|| 0.into()),
                )
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => serde_json::Value::Object(
            map.iter()
                .map(|(k, v)| {
                    let key = k.as_str().unwrap_or("key").to_string();
                    (key, yaml_to_json(v))
                })
                .collect(),
        ),
        _ => serde_json::Value::Null, // Catch-all for Tagged and any future variants
    }
}

fn json_to_yaml_string(json: &serde_json::Value) -> String {
    match json {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_to_yaml_string).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::Object(obj) => {
            // For simplicity, just convert to string
            serde_json::to_string(obj).unwrap_or_else(|_| "{}".to_string())
        }
    }
}

pub fn extract_frontmatter(content: &str) -> (Option<Metadata>, &str) {
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 2 || lines[0] != "---" {
        return (None, content);
    }

    // Find closing ---
    if let Some(end_index) = lines[1..].iter().position(|line| *line == "---") {
        let end_index = end_index + 1; // Adjust for the slice
        let yaml_content = lines[0..=end_index].join("\n");
        let remaining_start = end_index + 1;
        let remaining_content = &content[content.find(&lines[remaining_start]).unwrap_or(0)..];

        match Metadata::from_yaml(&yaml_content) {
            Ok(metadata) => (Some(metadata), remaining_content),
            Err(_) => (None, content),
        }
    } else {
        (None, content)
    }
}

pub fn prepend_frontmatter(content: &str, metadata: &Metadata) -> Result<String, String> {
    let yaml = metadata.to_yaml()?;
    Ok(format!("{}{}", yaml, content))
}
