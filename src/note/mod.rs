mod utils;
pub mod note_types;
pub mod zettel;
pub mod metadata;

pub use note_types::{Note, NoteType};
pub use zettel::ZettelId;
pub use metadata::Metadata;
pub use utils::Id;

pub type NoteId = Id<Note>;