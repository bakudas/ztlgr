use crate::ui::app::Mode;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub struct StatusBar {
    command_buffer: String,
    message: String,
    message_timeout: usize,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            command_buffer: String::new(),
            message: String::new(),
            message_timeout: 0,
        }
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.message_timeout = 50; // Display for ~50 frames (~1 second)
    }

    pub fn tick(&mut self) {
        if self.message_timeout > 0 {
            self.message_timeout -= 1;
            if self.message_timeout == 0 {
                self.message.clear();
            }
        }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme, mode: Mode) {
        let (mode_str, mode_style) = match mode {
            Mode::Normal => ("NORMAL", Style::default().fg(theme.fg())),
            Mode::Insert => (
                "INSERT",
                Style::default()
                    .fg(theme.success())
                    .add_modifier(Modifier::BOLD),
            ),
            Mode::Search => (
                "SEARCH",
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Mode::Command => (
                "COMMAND",
                Style::default()
                    .fg(theme.info())
                    .add_modifier(Modifier::BOLD),
            ),
            Mode::Graph => (
                "GRAPH",
                Style::default()
                    .fg(theme.warning())
                    .add_modifier(Modifier::BOLD),
            ),
        };

        let help_style = Style::default().fg(theme.fg_dim()).bg(theme.bg());

        let status_text = if !self.message.is_empty() {
            // Message takes priority
            Line::from(vec![
                Span::styled(format!(" [{:>7}] ", mode_str), mode_style),
                Span::styled(&self.message, help_style),
            ])
        } else {
            // Default help text with mode indicator
            Line::from(vec![
                Span::styled(format!(" [{:>7}] ", mode_str), mode_style),
                Span::styled(
                    "Press 'i' to edit | '/' to search | ':' for commands | ':q' to quit",
                    help_style,
                ),
            ])
        };

        let paragraph = Paragraph::new(status_text).style(Style::default().bg(theme.bg()));

        f.render_widget(paragraph, area);
    }

    pub fn set_command(&mut self, command: &str) {
        self.command_buffer = command.to_string();
    }
}
