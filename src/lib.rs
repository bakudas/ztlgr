pub mod error;
pub mod config;
pub mod db;
pub mod note;
pub mod storage;
pub mod setup;
pub mod ui;

// Re-export commonly used types
pub use error::{ZtlgrError, Result};
pub use config::Config;
pub use db::Database;
pub use note::{Note, NoteType, NoteId, ZettelId};
pub use storage::{Vault, Format, Storage, MarkdownStorage, OrgStorage};

// Version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");