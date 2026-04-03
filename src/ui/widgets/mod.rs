pub mod backlinks_pane;
pub mod command;
pub mod editor_history;
pub mod editor_state;
pub mod link_autocomplete;
pub mod metadata_pane;
pub mod modals;
pub mod note_editor;
pub mod note_list;
pub mod preview_pane;
pub mod search;
pub mod status_bar;

#[allow(unused_imports)]
pub use backlinks_pane::{BacklinkItem, BacklinksPane};
pub use command::{Command, CommandContext, CommandExecutor, CommandParser, CommandResult};
#[allow(unused_imports)]
pub use link_autocomplete::{LinkAutocomplete, LinkSuggestion};
pub use metadata_pane::MetadataPane;
pub use modals::{
    ConfirmationAction, ConfirmationModal, CreateNoteAction, CreateNoteModal, NoteTypeAction,
    NoteTypeSelector,
};
pub use note_editor::NoteEditor;
pub use note_list::NoteList;
pub use preview_pane::PreviewPane;
pub use search::{SearchResult, SearchState};
pub use status_bar::StatusBar;
