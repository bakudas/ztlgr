use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
    Frame,
};

/// A reusable modal dialog widget
///
/// Used as the base for all modal dialogs (confirmation, selection, input, etc)
/// Provides common layout, styling, and key handling patterns
#[derive(Debug, Clone)]
pub struct GenericModal {
    pub title: String,
    pub message: String,
    pub buttons: Vec<String>,   // e.g. ["Yes", "No"] or ["OK", "Cancel"]
    pub selected_button: usize, // Index of selected button (0-based)
    pub width_percent: u16,
    pub height_percent: u16,
}

impl GenericModal {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            buttons: vec!["OK".to_string(), "Cancel".to_string()],
            selected_button: 0,
            width_percent: 60,
            height_percent: 40,
        }
    }

    pub fn with_buttons(mut self, buttons: Vec<String>) -> Self {
        self.buttons = buttons;
        self
    }

    pub fn with_dimensions(mut self, width_percent: u16, height_percent: u16) -> Self {
        self.width_percent = width_percent;
        self.height_percent = height_percent;
        self
    }

    /// Get the currently selected button
    pub fn selected_button(&self) -> &str {
        self.buttons
            .get(self.selected_button)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Move selection left
    pub fn select_prev(&mut self) {
        if self.selected_button > 0 {
            self.selected_button -= 1;
        } else {
            self.selected_button = self.buttons.len().saturating_sub(1);
        }
    }

    /// Move selection right
    pub fn select_next(&mut self) {
        self.selected_button = (self.selected_button + 1) % self.buttons.len();
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Tab | KeyCode::Right => {
                self.select_next();
                true
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.select_prev();
                true
            }
            _ => false,
        }
    }

    /// Draw the modal
    pub fn draw(&self, f: &mut Frame, theme: &dyn crate::config::Theme) {
        let area = f.size();

        // Calculate modal dimensions (centered)
        let modal_width = (area.width * self.width_percent) / 100;
        let modal_height = (area.height * self.height_percent) / 100;

        let modal_x = (area.width.saturating_sub(modal_width)) / 2;
        let modal_y = (area.height.saturating_sub(modal_height)) / 2;

        let modal_area = Rect {
            x: modal_x,
            y: modal_y,
            width: modal_width,
            height: modal_height,
        };

        // Clear the modal area
        Clear.render(modal_area, f.buffer_mut());

        // Modal box
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .style(
                ratatui::style::Style::default()
                    .fg(theme.fg())
                    .bg(theme.bg()),
            );

        // Layout: title/message + buttons
        let inner = block.inner(modal_area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Length(2)])
            .split(inner);

        // Message
        let message = Paragraph::new(self.message.as_str())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .style(
                ratatui::style::Style::default()
                    .fg(theme.fg())
                    .bg(theme.bg()),
            );

        let mut message_area = chunks[0];
        message_area.y += 1; // Add some padding

        f.render_widget(block, modal_area);
        f.render_widget(message, message_area);

        // Buttons
        self.draw_buttons(f, chunks[1], theme);
    }

    fn draw_buttons(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme) {
        // Render buttons horizontally
        let button_line = ratatui::text::Line::from(
            self.buttons
                .iter()
                .enumerate()
                .flat_map(|(idx, btn)| {
                    let is_selected = idx == self.selected_button;
                    let style = if is_selected {
                        ratatui::style::Style::default()
                            .fg(theme.bg())
                            .bg(theme.accent())
                    } else {
                        ratatui::style::Style::default()
                            .fg(theme.fg())
                            .bg(theme.bg())
                    };
                    vec![
                        ratatui::text::Span::styled(format!(" {} ", btn), style),
                        if idx < self.buttons.len() - 1 {
                            ratatui::text::Span::raw("  ")
                        } else {
                            ratatui::text::Span::raw("")
                        },
                    ]
                })
                .collect::<Vec<_>>(),
        );

        let buttons_para = Paragraph::new(button_line)
            .alignment(Alignment::Center)
            .style(
                ratatui::style::Style::default()
                    .fg(theme.fg())
                    .bg(theme.bg()),
            );
        f.render_widget(buttons_para, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_modal_creation() {
        let modal = GenericModal::new("Test", "Message");
        assert_eq!(modal.title, "Test");
        assert_eq!(modal.message, "Message");
        assert_eq!(modal.buttons.len(), 2);
        assert_eq!(modal.selected_button, 0);
    }

    #[test]
    fn test_select_next() {
        let mut modal = GenericModal::new("Test", "Message").with_buttons(vec![
            "Yes".to_string(),
            "No".to_string(),
            "Cancel".to_string(),
        ]);

        assert_eq!(modal.selected_button, 0);
        modal.select_next();
        assert_eq!(modal.selected_button, 1);
        modal.select_next();
        assert_eq!(modal.selected_button, 2);
        modal.select_next(); // Wrap around
        assert_eq!(modal.selected_button, 0);
    }

    #[test]
    fn test_select_prev() {
        let mut modal = GenericModal::new("Test", "Message").with_buttons(vec![
            "Yes".to_string(),
            "No".to_string(),
            "Cancel".to_string(),
        ]);

        modal.selected_button = 2;
        modal.select_prev();
        assert_eq!(modal.selected_button, 1);
        modal.select_prev();
        assert_eq!(modal.selected_button, 0);
        modal.select_prev(); // Wrap around
        assert_eq!(modal.selected_button, 2);
    }

    #[test]
    fn test_selected_button() {
        let modal = GenericModal::new("Test", "Message")
            .with_buttons(vec!["Yes".to_string(), "No".to_string()]);

        assert_eq!(modal.selected_button(), "Yes");
    }

    #[test]
    fn test_with_buttons() {
        let modal = GenericModal::new("Test", "Message").with_buttons(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ]);

        assert_eq!(modal.buttons.len(), 3);
    }
}
