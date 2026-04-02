use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZtlgrError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Note not found: {0}")]
    NotFound(String),

    #[error("Invalid note ID: {0}")]
    InvalidNoteId(String),

    #[error("Invalid Zettel ID: {0}")]
    InvalidZettelId(String),

    #[error("Link error: {0}")]
    Link(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("UI error: {0}")]
    Ui(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(String),

    #[error("File watcher error: {0}")]
    FileWatcher(String),
}

pub type Result<T> = std::result::Result<T, ZtlgrError>;

impl From<notify::Error> for ZtlgrError {
    fn from(err: notify::Error) -> Self {
        ZtlgrError::FileWatcher(err.to_string())
    }
}
