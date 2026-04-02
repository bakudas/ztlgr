use crate::ui::app::Mode;
use ratatui::{layout::Rect, style::Style, text::Text, widgets::Paragraph, Frame};

pub struct StatusBar {
    command_buffer: String,
    message: String,
    message_timeout: usize,
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

    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme, mode: Mode) {
        let mode_str = match mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Search => "SEARCH",
            Mode::Command => "COMMAND",
            Mode::Graph => "GRAPH",
        };

        let help_style = Style::default().fg(theme.fg_dim()).bg(theme.bg());

        let status_text = if !self.message.is_empty() {
            self.message.clone()
        } else {
            format!(
                " {} | Press 'i' to edit | '/' to search | ':' for command | 'q' to quit",
                mode_str
            )
        };

        let text = Text::from(status_text);
        let paragraph = Paragraph::new(text).style(help_style);

        f.render_widget(paragraph, area);
    }

    pub fn set_command(&mut self, command: &str) {
        self.command_buffer = command.to_string();
    }
}
