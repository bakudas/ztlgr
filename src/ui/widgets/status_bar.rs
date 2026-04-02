use ratatui::{
    layout::Rect,
    style::Style,
    text::Text,
    widgets::Paragraph,
    Frame,
};
use crate::ui::app::Mode;

pub struct StatusBar {
    command_buffer: String,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            command_buffer: String::new(),
        }
    }
    
    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme, mode: Mode) {
        let mode_str = match mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Search => "SEARCH",
            Mode::Command => "COMMAND",
            Mode::Graph => "GRAPH",
        };
        
        let help_style = Style::default()
            .fg(theme.fg_dim())
            .bg(theme.bg());
        
        let text = Text::from(format!(
            " {} | Press 'i' to edit | '/' to search | ':' for command | 'q' to quit",
            mode_str
        ));
        
        let paragraph = Paragraph::new(text).style(help_style);
        
        f.render_widget(paragraph, area);
    }
    
    pub fn set_command(&mut self, command: &str) {
        self.command_buffer = command.to_string();
    }
}