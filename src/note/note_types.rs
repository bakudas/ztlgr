use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Metadata, NoteId, ZettelId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub title: String,
    pub content: String,
    pub note_type: NoteType,

    // Zettelkasten fields
    pub zettel_id: Option<ZettelId>,
    pub parent_id: Option<NoteId>,
    pub source: Option<String>,

    // Metadata
    pub metadata: Metadata,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NoteType {
    Daily,
    Fleeting,
    Literature { source: String },
    Permanent,
    Reference { url: Option<String> },
    Index,
}

impl NoteType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NoteType::Daily => "daily",
            NoteType::Fleeting => "fleeting",
            NoteType::Literature { .. } => "literature",
            NoteType::Permanent => "permanent",
            NoteType::Reference { .. } => "reference",
            NoteType::Index => "index",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "daily" => Ok(NoteType::Daily),
            "fleeting" => Ok(NoteType::Fleeting),
            "literature" => Ok(NoteType::Literature {
                source: String::new(),
            }),
            "permanent" => Ok(NoteType::Permanent),
            "reference" => Ok(NoteType::Reference { url: None }),
            "index" => Ok(NoteType::Index),
            _ => Err(format!("Invalid note type: {}", s)),
        }
    }
}

impl std::fmt::Display for NoteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for NoteType {
    fn default() -> Self {
        NoteType::Permanent
    }
}

impl Note {
    pub fn new(title: String, content: String) -> Self {
        Self {
            id: NoteId::new(),
            title,
            content,
            note_type: NoteType::default(),
            zettel_id: None,
            parent_id: None,
            source: None,
            metadata: Metadata::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    pub fn with_type(mut self, note_type: NoteType) -> Self {
        self.note_type = note_type;
        self
    }

    pub fn with_zettel_id(mut self, zettel_id: ZettelId) -> Self {
        self.zettel_id = Some(zettel_id);
        self
    }

    pub fn with_parent(mut self, parent_id: NoteId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}
