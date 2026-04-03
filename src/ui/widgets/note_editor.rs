use crate::ui::app::Mode;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct NoteEditor {
    content: String,
    cursor: usize, // byte offset
    preferred_col: Option<usize>,
}

impl NoteEditor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            preferred_col: None,
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
        self.cursor = self.content.len();
        self.preferred_col = None;
    }

    pub fn get_content(&self) -> String {
        self.content.clone()
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
        self.preferred_col = None;
    }

    fn prev_boundary(&self, index: usize) -> usize {
        if index == 0 {
            return 0;
        }

        let mut i = index - 1;
        while i > 0 && !self.content.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    fn next_boundary(&self, index: usize) -> usize {
        if index >= self.content.len() {
            return self.content.len();
        }

        let mut i = index + 1;
        while i < self.content.len() && !self.content.is_char_boundary(i) {
            i += 1;
        }
        i
    }

    fn line_col_at_cursor(&self) -> (usize, usize) {
        let prefix = &self.content[..self.cursor];
        let line = prefix.chars().filter(|&c| c == '\n').count();
        let col = prefix
            .rsplit('\n')
            .next()
            .map(|segment| segment.chars().count())
            .unwrap_or(0);

        (line, col)
    }

    fn line_start(&self, pos: usize) -> usize {
        self.content[..pos]
            .rfind('\n')
            .map(|idx| idx + 1)
            .unwrap_or(0)
    }

    fn line_end(&self, pos: usize) -> usize {
        self.content[pos..]
            .find('\n')
            .map(|idx| pos + idx)
            .unwrap_or(self.content.len())
    }

    fn byte_index_for_column(line_text: &str, col: usize) -> usize {
        line_text
            .char_indices()
            .nth(col)
            .map(|(idx, _)| idx)
            .unwrap_or(line_text.len())
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme, mode: Mode) {
        let mode_text = match mode {
            Mode::Normal => "-- NORMAL --",
            Mode::Insert => "-- INSERT --",
            Mode::Search => "-- SEARCH --",
            Mode::Command => "-- COMMAND --",
            Mode::Graph => "-- GRAPH --",
        };

        let (line, col) = self.line_col_at_cursor();

        let lines = if self.content.is_empty() {
            vec![Line::from(Span::styled(
                "Press 'i' to enter insert mode or 'n' to create a new note",
                Style::default().fg(theme.fg_dim()),
            ))]
        } else {
            self.content
                .split('\n')
                .map(|l| Line::from(l.to_string()))
                .collect()
        };

        let title = format!(" Editor {} - Line {} Col {} ", mode_text, line + 1, col + 1);
        let visible_height = area.height.saturating_sub(2) as usize;
        let scroll_y = if visible_height > 0 && line >= visible_height {
            line - visible_height + 1
        } else {
            0
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_highlight())),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .scroll((scroll_y as u16, 0));

        f.render_widget(paragraph, area);

        if mode == Mode::Insert && area.width > 2 && area.height > 2 && line >= scroll_y {
            let max_col = area.width.saturating_sub(3) as usize;
            let cursor_x = area.x + 1 + (col.min(max_col) as u16);
            let cursor_y = area.y + 1 + ((line - scroll_y) as u16);

            if cursor_y < area.y + area.height - 1 {
                f.set_cursor(cursor_x, cursor_y);
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.content.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                self.preferred_col = None;
            }
            KeyCode::Enter => {
                self.content.insert(self.cursor, '\n');
                self.cursor += 1;
                self.preferred_col = None;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let remove_at = self.prev_boundary(self.cursor);
                    self.content.replace_range(remove_at..self.cursor, "");
                    self.cursor = remove_at;
                    self.preferred_col = None;
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.content.len() {
                    let next = self.next_boundary(self.cursor);
                    self.content.replace_range(self.cursor..next, "");
                    self.preferred_col = None;
                }
            }
            KeyCode::Left => {
                self.cursor = self.prev_boundary(self.cursor);
                self.preferred_col = None;
            }
            KeyCode::Right => {
                self.cursor = self.next_boundary(self.cursor);
                self.preferred_col = None;
            }
            KeyCode::Home => {
                self.cursor = self.line_start(self.cursor);
                self.preferred_col = None;
            }
            KeyCode::End => {
                self.cursor = self.line_end(self.cursor);
                self.preferred_col = None;
            }
            KeyCode::Up => {
                let (line, col) = self.line_col_at_cursor();
                if line > 0 {
                    let target_col = self.preferred_col.unwrap_or(col);
                    let current_start = self.line_start(self.cursor);
                    let prev_end = current_start.saturating_sub(1);
                    let prev_start = self.line_start(prev_end);
                    let prev_line = &self.content[prev_start..=prev_end];
                    let prev_line_text = prev_line.strip_suffix('\n').unwrap_or(prev_line);
                    let offset = Self::byte_index_for_column(prev_line_text, target_col);
                    self.cursor = prev_start + offset;
                    self.preferred_col = Some(target_col);
                }
            }
            KeyCode::Down => {
                let (_, col) = self.line_col_at_cursor();
                let target_col = self.preferred_col.unwrap_or(col);
                let current_end = self.line_end(self.cursor);
                if current_end < self.content.len() {
                    let next_start = current_end + 1;
                    let next_end = self.line_end(next_start);
                    let next_line = &self.content[next_start..next_end];
                    let offset = Self::byte_index_for_column(next_line, target_col);
                    self.cursor = next_start + offset;
                    self.preferred_col = Some(target_col);
                }
            }
            _ => {}
        }
    }
}
