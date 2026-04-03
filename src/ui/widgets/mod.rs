pub mod editor_history;
pub mod editor_state;
pub mod modals;
pub mod note_editor;
pub mod note_list;
pub mod preview_pane;
pub mod status_bar;

pub use editor_history::{EditAction, EditHistory};
pub use editor_state::{EditorState, Selection, TextRope};
pub use modals::{
    ConfirmationAction, ConfirmationModal, CreateNoteAction, CreateNoteModal, NoteType,
    NoteTypeAction, NoteTypeSelector,
};
pub use note_editor::NoteEditor;
pub use note_list::NoteList;
pub use preview_pane::PreviewPane;
pub use status_bar::StatusBar;
