use crate::ui::app::Mode;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct NoteEditor {
    content: String,
    cursor: usize,
}

impl NoteEditor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
        self.cursor = content.len();
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme, mode: Mode) {
        let text = if self.content.is_empty() {
            Text::from("Press 'i' to enter insert mode or 'n' to create a new note")
        } else {
            Text::from(self.content.as_str())
        };

        let mode_text = match mode {
            Mode::Normal => "-- NORMAL --",
            Mode::Insert => "-- INSERT --",
            Mode::Search => "-- SEARCH --",
            Mode::Command => "-- COMMAND --",
            Mode::Graph => "-- GRAPH --",
        };

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(format!(" Editor {} ", mode_text))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_highlight())),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()));

        f.render_widget(paragraph, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Char(c) => {
                self.content.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            crossterm::event::KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.content.remove(self.cursor - 1);
                    self.cursor -= 1;
                }
            }
            crossterm::event::KeyCode::Delete => {
                if self.cursor < self.content.len() {
                    self.content.remove(self.cursor);
                }
            }
            crossterm::event::KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            crossterm::event::KeyCode::Right => {
                if self.cursor < self.content.len() {
                    self.cursor += 1;
                }
            }
            crossterm::event::KeyCode::Home => {
                self.cursor = 0;
            }
            crossterm::event::KeyCode::End => {
                self.cursor = self.content.len();
            }
            _ => {}
        }
    }
}
