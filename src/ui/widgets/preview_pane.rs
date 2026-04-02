use ratatui::{
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct PreviewPane {
    content: String,
}

impl PreviewPane {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme) {
        let text = if self.content.is_empty() {
            Text::from("Select a note to preview")
        } else {
            Text::from(self.content.as_str())
        };

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Preview ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border())),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()));

        f.render_widget(paragraph, area);
    }
}
