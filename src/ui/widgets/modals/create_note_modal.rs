use super::GenericModal;
use crate::config::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;

/// Modal for creating a new note with title and tags
///
/// ASCII mockup:
/// ```
/// ┌─ Create New [NoteType] Note ──────────┐
/// │                                       │
/// │  Title: [________________]            │
/// │  Tags:  [________________]            │
/// │                                       │
/// │  Press Tab to switch fields           │
/// │                                       │
/// │   [Create]  [Cancel]                 │
/// └───────────────────────────────────────┘
/// ```
#[derive(Debug, Clone)]
pub struct CreateNoteModal {
    base: GenericModal,
    title: String,
    tags: String,
    active_field: InputField, // Which field is being edited
    max_title_length: usize,
    max_tags_length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputField {
    Title,
    Tags,
}

impl CreateNoteModal {
    pub fn new() -> Self {
        let base = GenericModal::new(
            "Create New Note",
            "Enter note details:\n\nTitle: [                    ]\nTags:  [                    ]\n\nTab to switch fields · Enter to create · Esc to cancel",
        )
        .with_buttons(vec!["Create".to_string(), "Cancel".to_string()])
        .with_dimensions(60, 45);

        Self {
            base,
            title: String::new(),
            tags: String::new(),
            active_field: InputField::Title,
            max_title_length: 100,
            max_tags_length: 100,
        }
    }

    /// Set the note type in the title
    pub fn with_note_type(mut self, note_type: &str) -> Self {
        self.base.title = format!("Create New {} Note", note_type);
        self
    }

    /// Get the entered title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the entered tags
    pub fn tags(&self) -> &str {
        &self.tags
    }

    /// Check if title is provided
    pub fn is_valid(&self) -> bool {
        !self.title.trim().is_empty()
    }

    /// Switch to the other input field
    pub fn toggle_field(&mut self) {
        self.active_field = match self.active_field {
            InputField::Title => InputField::Tags,
            InputField::Tags => InputField::Title,
        };
    }

    /// Handle character input
    fn input_char(&mut self, c: char) {
        match self.active_field {
            InputField::Title if self.title.len() < self.max_title_length => {
                self.title.push(c);
            }
            InputField::Tags if self.tags.len() < self.max_tags_length => {
                self.tags.push(c);
            }
            _ => {}
        }
    }

    /// Handle backspace
    fn handle_backspace(&mut self) {
        match self.active_field {
            InputField::Title => {
                self.title.pop();
            }
            InputField::Tags => {
                self.tags.pop();
            }
        }
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<CreateNoteAction> {
        match key.code {
            // Tab to switch fields
            KeyCode::Tab => {
                self.toggle_field();
                None
            }
            // Backspace to delete character
            KeyCode::Backspace => {
                self.handle_backspace();
                None
            }
            // Enter to create note (only if on Create button)
            KeyCode::Enter => {
                if self.base.selected_button() == "Create" {
                    if self.is_valid() {
                        Some(CreateNoteAction::Created {
                            title: self.title.trim().to_string(),
                            tags: self.tags.trim().to_string(),
                        })
                    } else {
                        Some(CreateNoteAction::Error("Title cannot be empty".to_string()))
                    }
                } else {
                    Some(CreateNoteAction::Cancelled)
                }
            }
            KeyCode::Esc => Some(CreateNoteAction::Cancelled),
            // Printable characters
            KeyCode::Char(c) => {
                self.input_char(c);
                None
            }
            _ => None,
        }
    }

    /// Draw the create note modal
    pub fn draw(&self, f: &mut Frame, theme: &dyn Theme) {
        // Create a formatted message that shows the input fields
        let title_display = format!(
            "Title: [{}{}]",
            self.title,
            " ".repeat(30_usize.saturating_sub(self.title.len()))
        );

        let tags_display = format!(
            "Tags:  [{}{}]",
            self.tags,
            " ".repeat(30_usize.saturating_sub(self.tags.len()))
        );

        let field_indicator = match self.active_field {
            InputField::Title => "● editing title",
            InputField::Tags => "● editing tags",
        };

        let message = format!(
            "{}\n{}\n\n{}\n\nTab to switch · Esc to cancel",
            title_display, tags_display, field_indicator
        );

        // Update the base modal message temporarily
        let mut modal_copy = self.base.clone();
        modal_copy.message = message;
        modal_copy.draw(f, theme);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateNoteAction {
    Created { title: String, tags: String },
    Cancelled,
    Error(String),
}

impl Default for CreateNoteModal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_note_modal_creation() {
        let modal = CreateNoteModal::new();
        assert_eq!(modal.title(), "");
        assert_eq!(modal.tags(), "");
        assert!(!modal.is_valid());
    }

    #[test]
    fn test_input_title() {
        let mut modal = CreateNoteModal::new();
        modal.input_char('T');
        modal.input_char('e');
        modal.input_char('s');
        modal.input_char('t');

        assert_eq!(modal.title(), "Test");
        assert!(modal.is_valid());
    }

    #[test]
    fn test_input_tags() {
        let mut modal = CreateNoteModal::new();
        modal.toggle_field();
        modal.input_char('r');
        modal.input_char('u');
        modal.input_char('s');
        modal.input_char('t');

        assert_eq!(modal.tags(), "rust");
    }

    #[test]
    fn test_backspace() {
        let mut modal = CreateNoteModal::new();
        modal.input_char('a');
        modal.input_char('b');
        modal.input_char('c');

        modal.handle_backspace();
        assert_eq!(modal.title(), "ab");
    }

    #[test]
    fn test_toggle_field() {
        let mut modal = CreateNoteModal::new();
        assert_eq!(modal.active_field, InputField::Title);

        modal.toggle_field();
        assert_eq!(modal.active_field, InputField::Tags);

        modal.toggle_field();
        assert_eq!(modal.active_field, InputField::Title);
    }

    #[test]
    fn test_title_length_limit() {
        let mut modal = CreateNoteModal::new();
        modal.max_title_length = 5;

        for c in "abcdefgh".chars() {
            modal.input_char(c);
        }

        assert_eq!(modal.title(), "abcde");
    }

    #[test]
    fn test_handle_key_tab() {
        let mut modal = CreateNoteModal::new();
        let key = KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::NONE);
        let action = modal.handle_key(key);

        assert_eq!(modal.active_field, InputField::Tags);
        assert_eq!(action, None);
    }

    #[test]
    fn test_handle_key_enter_valid() {
        let mut modal = CreateNoteModal::new();
        modal.input_char('T');
        modal.input_char('e');
        modal.input_char('s');
        modal.input_char('t');

        let key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
        let action = modal.handle_key(key);

        match action {
            Some(CreateNoteAction::Created { title, tags }) => {
                assert_eq!(title, "Test");
                assert_eq!(tags, "");
            }
            _ => panic!("Expected Created action"),
        }
    }

    #[test]
    fn test_handle_key_enter_invalid() {
        let mut modal = CreateNoteModal::new();

        let key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
        let action = modal.handle_key(key);

        match action {
            Some(CreateNoteAction::Error(msg)) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected Error action"),
        }
    }
}
