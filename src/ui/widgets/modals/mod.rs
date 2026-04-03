pub mod confirmation_modal;
pub mod create_note_modal;
pub mod generic_modal;
pub mod note_type_selector;

pub use confirmation_modal::{ConfirmationAction, ConfirmationModal};
pub use create_note_modal::{CreateNoteAction, CreateNoteModal};
pub use generic_modal::GenericModal;
pub use note_type_selector::{NoteType, NoteTypeAction, NoteTypeSelector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModalType {
    Confirmation,
    NoteTypeSelector,
    CreateNote,
}

#[derive(Debug, Clone)]
pub enum ModalAction {
    None,
    Confirm,
    Cancel,
}
