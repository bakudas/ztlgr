pub mod metadata;
pub mod note_types;
pub mod utils;
pub mod zettel;

pub use metadata::Metadata;
pub use note_types::{Note, NoteType};
pub use utils::Id;
pub use zettel::ZettelId;

pub type NoteId = Id<Note>;
