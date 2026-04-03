use super::GenericModal;
use crate::config::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;

/// Confirmation modal for delete operations and other destructive actions
///
/// ASCII mockup:
/// ```
/// ┌─ Confirm Delete ─────────┐
/// │                           │
/// │  Delete note "My Note"?   │
/// │  This action cannot be    │
/// │  undone.                  │
/// │                           │
/// │   [Yes]  [No]            │
/// └───────────────────────────┘
/// ```
#[derive(Debug, Clone)]
pub struct ConfirmationModal {
    base: GenericModal,
    action_name: String, // "Delete", "Discard", etc
    target_name: String, // The thing being acted upon
}

impl ConfirmationModal {
    pub fn new(action: impl Into<String>, target: impl Into<String>) -> Self {
        let action = action.into();
        let target = target.into();

        let title = format!("Confirm {}", action);
        let message = format!(
            "{} \"{}\"?\n\nThis action cannot be undone.",
            action, target
        );

        let base = GenericModal::new(title, message)
            .with_buttons(vec!["Yes".to_string(), "No".to_string()])
            .with_dimensions(50, 35);

        Self {
            base,
            action_name: action,
            target_name: target,
        }
    }

    /// Check if the user confirmed
    pub fn is_confirmed(&self) -> bool {
        self.base.selected_button() == "Yes"
    }

    /// Handle key events for the confirmation modal
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ConfirmationAction> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Quick 'y' to confirm
                self.base.selected_button = 0;
                Some(ConfirmationAction::Confirm)
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                // Quick 'n' to cancel
                self.base.selected_button = 1;
                Some(ConfirmationAction::Cancel)
            }
            KeyCode::Enter => {
                if self.is_confirmed() {
                    Some(ConfirmationAction::Confirm)
                } else {
                    Some(ConfirmationAction::Cancel)
                }
            }
            KeyCode::Esc => Some(ConfirmationAction::Cancel),
            _ => {
                // Let base handle Tab/Arrow navigation
                self.base.handle_key(key);
                None
            }
        }
    }

    /// Draw the confirmation modal
    pub fn draw(&self, f: &mut Frame, theme: &dyn Theme) {
        self.base.draw(f, theme);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfirmationAction {
    Confirm,
    Cancel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirmation_modal_creation() {
        let modal = ConfirmationModal::new("Delete", "My Note");
        assert_eq!(modal.action_name, "Delete");
        assert_eq!(modal.target_name, "My Note");
        assert!(modal.base.title.contains("Confirm Delete"));
    }

    #[test]
    fn test_is_confirmed_yes() {
        let modal = ConfirmationModal::new("Delete", "Test");
        assert!(modal.is_confirmed()); // Yes is selected by default
    }

    #[test]
    fn test_is_confirmed_no() {
        let mut modal = ConfirmationModal::new("Delete", "Test");
        modal.base.select_next(); // Move to "No"
        assert!(!modal.is_confirmed());
    }

    #[test]
    fn test_handle_key_quick_confirm() {
        let mut modal = ConfirmationModal::new("Delete", "Test");
        modal.base.select_next(); // Move to "No"

        let key = KeyEvent::new(KeyCode::Char('y'), crossterm::event::KeyModifiers::NONE);
        let action = modal.handle_key(key);

        assert_eq!(action, Some(ConfirmationAction::Confirm));
        assert!(modal.is_confirmed());
    }

    #[test]
    fn test_handle_key_quick_cancel() {
        let mut modal = ConfirmationModal::new("Delete", "Test");

        let key = KeyEvent::new(KeyCode::Char('n'), crossterm::event::KeyModifiers::NONE);
        let action = modal.handle_key(key);

        assert_eq!(action, Some(ConfirmationAction::Cancel));
        assert!(!modal.is_confirmed());
    }

    #[test]
    fn test_handle_key_enter_confirm() {
        let mut modal = ConfirmationModal::new("Delete", "Test");

        let key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
        let action = modal.handle_key(key);

        assert_eq!(action, Some(ConfirmationAction::Confirm));
    }

    #[test]
    fn test_handle_key_escape() {
        let mut modal = ConfirmationModal::new("Delete", "Test");

        let key = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::NONE);
        let action = modal.handle_key(key);

        assert_eq!(action, Some(ConfirmationAction::Cancel));
    }
}
