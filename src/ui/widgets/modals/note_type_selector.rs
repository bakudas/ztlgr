use super::GenericModal;
use crate::config::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;

/// Note type selector modal for creating new notes
///
/// ASCII mockup:
/// ```
/// ┌─ Select Note Type ───────────────────┐
/// │                                       │
/// │  Choose note type for new note:       │
/// │                                       │
/// │  • [Daily]                            │
/// │    [Fleeting]                         │
/// │    [Permanent]                        │
/// │    [Literature]                       │
/// │                                       │
/// │   [Select]  [Cancel]                 │
/// └───────────────────────────────────────┘
/// ```
#[derive(Debug, Clone)]
pub struct NoteTypeSelector {
    base: GenericModal,
    note_types: Vec<NoteType>,
    selected_type_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteType {
    Daily,
    Fleeting,
    Permanent,
    Literature,
}

impl NoteType {
    pub fn label(&self) -> &'static str {
        match self {
            NoteType::Daily => "Daily",
            NoteType::Fleeting => "Fleeting",
            NoteType::Permanent => "Permanent",
            NoteType::Literature => "Literature",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            NoteType::Daily => "Daily notes for capturing thoughts and ideas",
            NoteType::Fleeting => "Temporary notes to be processed later",
            NoteType::Permanent => "Permanent notes with evergreen content",
            NoteType::Literature => "Literature notes from external sources",
        }
    }

    pub fn folder(&self) -> &'static str {
        match self {
            NoteType::Daily => "daily",
            NoteType::Fleeting => "fleeting",
            NoteType::Permanent => "permanent",
            NoteType::Literature => "literature",
        }
    }
}

impl NoteTypeSelector {
    pub fn new() -> Self {
        let note_types = vec![
            NoteType::Daily,
            NoteType::Fleeting,
            NoteType::Permanent,
            NoteType::Literature,
        ];

        let base = GenericModal::new(
            "Select Note Type",
            "Choose type for new note:\n\nDaily - Today's quick captures\nFleeting - To be processed\nPermanent - Evergreen ideas\nLiterature - From sources",
        )
        .with_buttons(vec!["Select".to_string(), "Cancel".to_string()])
        .with_dimensions(55, 45);

        Self {
            base,
            note_types,
            selected_type_idx: 0,
        }
    }

    /// Get the currently selected note type
    pub fn selected_type(&self) -> NoteType {
        self.note_types[self.selected_type_idx]
    }

    /// Move to the next note type
    pub fn select_next_type(&mut self) {
        self.selected_type_idx = (self.selected_type_idx + 1) % self.note_types.len();
    }

    /// Move to the previous note type
    pub fn select_prev_type(&mut self) {
        if self.selected_type_idx > 0 {
            self.selected_type_idx -= 1;
        } else {
            self.selected_type_idx = self.note_types.len() - 1;
        }
    }

    /// Check if "Select" button is pressed
    pub fn is_selected(&self) -> bool {
        self.base.selected_button() == "Select"
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<NoteTypeAction> {
        match key.code {
            // Up/Down arrow to select note type
            KeyCode::Up => {
                self.select_prev_type();
                None
            }
            KeyCode::Down => {
                self.select_next_type();
                None
            }
            // Tab to move between buttons
            KeyCode::Tab | KeyCode::Right => {
                self.base.select_next();
                None
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.base.select_prev();
                None
            }
            // Enter to confirm selection
            KeyCode::Enter => {
                if self.is_selected() {
                    Some(NoteTypeAction::Selected(self.selected_type()))
                } else {
                    Some(NoteTypeAction::Cancelled)
                }
            }
            // Quick shortcuts
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.selected_type_idx = 0; // Daily
                Some(NoteTypeAction::Selected(self.selected_type()))
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.selected_type_idx = 1; // Fleeting
                Some(NoteTypeAction::Selected(self.selected_type()))
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.selected_type_idx = 2; // Permanent
                Some(NoteTypeAction::Selected(self.selected_type()))
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.selected_type_idx = 3; // Literature
                Some(NoteTypeAction::Selected(self.selected_type()))
            }
            KeyCode::Esc => Some(NoteTypeAction::Cancelled),
            _ => None,
        }
    }

    /// Draw the note type selector
    pub fn draw(&self, f: &mut Frame, theme: &dyn Theme) {
        self.base.draw(f, theme);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteTypeAction {
    Selected(NoteType),
    Cancelled,
}

impl Default for NoteTypeSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_type_selector_creation() {
        let selector = NoteTypeSelector::new();
        assert_eq!(selector.selected_type(), NoteType::Daily);
    }

    #[test]
    fn test_select_next_type() {
        let mut selector = NoteTypeSelector::new();
        selector.select_next_type();
        assert_eq!(selector.selected_type(), NoteType::Fleeting);
        selector.select_next_type();
        assert_eq!(selector.selected_type(), NoteType::Permanent);
    }

    #[test]
    fn test_select_prev_type() {
        let mut selector = NoteTypeSelector::new();
        selector.selected_type_idx = 2;
        selector.select_prev_type();
        assert_eq!(selector.selected_type(), NoteType::Fleeting);
    }

    #[test]
    fn test_note_type_labels() {
        assert_eq!(NoteType::Daily.label(), "Daily");
        assert_eq!(NoteType::Fleeting.label(), "Fleeting");
        assert_eq!(NoteType::Permanent.label(), "Permanent");
        assert_eq!(NoteType::Literature.label(), "Literature");
    }

    #[test]
    fn test_note_type_folders() {
        assert_eq!(NoteType::Daily.folder(), "daily");
        assert_eq!(NoteType::Fleeting.folder(), "fleeting");
        assert_eq!(NoteType::Permanent.folder(), "permanent");
        assert_eq!(NoteType::Literature.folder(), "literature");
    }

    #[test]
    fn test_is_selected() {
        let selector = NoteTypeSelector::new();
        assert!(selector.is_selected()); // "Select" is first button
    }

    #[test]
    fn test_handle_key_down() {
        let mut selector = NoteTypeSelector::new();
        let key = KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE);
        selector.handle_key(key);
        assert_eq!(selector.selected_type(), NoteType::Fleeting);
    }

    #[test]
    fn test_handle_key_up() {
        let mut selector = NoteTypeSelector::new();
        selector.selected_type_idx = 2;
        let key = KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE);
        selector.handle_key(key);
        assert_eq!(selector.selected_type(), NoteType::Fleeting);
    }

    #[test]
    fn test_handle_key_quick_shortcut() {
        let mut selector = NoteTypeSelector::new();
        let key = KeyEvent::new(KeyCode::Char('p'), crossterm::event::KeyModifiers::NONE);
        let action = selector.handle_key(key);

        assert_eq!(action, Some(NoteTypeAction::Selected(NoteType::Permanent)));
    }
}
