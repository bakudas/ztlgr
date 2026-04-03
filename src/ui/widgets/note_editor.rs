use super::editor_state::EditorState;
use crate::ui::app::Mode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct NoteEditor {
    state: EditorState,
    preferred_col: Option<usize>,
}

impl NoteEditor {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
            preferred_col: None,
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.state.set_content(content);
        self.preferred_col = None;
    }

    pub fn get_content(&self) -> String {
        self.state.get_content()
    }

    pub fn clear(&mut self) {
        self.state.clear();
        self.preferred_col = None;
    }

    pub fn is_dirty(&self) -> bool {
        self.state.is_dirty()
    }

    pub fn mark_saved(&mut self) {
        self.state.mark_saved();
    }

    fn prev_boundary(&self, index: usize) -> usize {
        if index == 0 {
            return 0;
        }

        let content = self.state.get_content();
        let mut i = index - 1;
        while i > 0 && !content.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    fn next_boundary(&self, index: usize) -> usize {
        let content = self.state.get_content();
        if index >= content.len() {
            return content.len();
        }

        let mut i = index + 1;
        while i < content.len() && !content.is_char_boundary(i) {
            i += 1;
        }
        i
    }

    fn line_col_at_cursor(&self) -> (usize, usize) {
        self.state.cursor_line_col()
    }

    fn line_start(&self, pos: usize) -> usize {
        let content = self.state.get_content();
        content[..pos].rfind('\n').map(|idx| idx + 1).unwrap_or(0)
    }

    fn line_end(&self, pos: usize) -> usize {
        let content = self.state.get_content();
        content[pos..]
            .find('\n')
            .map(|idx| pos + idx)
            .unwrap_or(content.len())
    }

    fn byte_index_for_column(line_text: &str, col: usize) -> usize {
        line_text
            .char_indices()
            .nth(col)
            .map(|(idx, _)| idx)
            .unwrap_or(line_text.len())
    }

    pub fn draw(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &dyn crate::config::Theme,
        mode: Mode,
        is_focused: bool,
    ) {
        let mode_text = match mode {
            Mode::Normal => "-- NORMAL --",
            Mode::Insert => "-- INSERT --",
            Mode::Search => "-- SEARCH --",
            Mode::Command => "-- COMMAND --",
            Mode::Graph => "-- GRAPH --",
        };

        let (line, col) = self.line_col_at_cursor();
        let content = self.state.get_content();

        let lines = if content.is_empty() {
            vec![Line::from(Span::styled(
                "Press 'i' to enter insert mode or 'n' to create a new note",
                Style::default().fg(theme.fg_dim()),
            ))]
        } else {
            // Renderizar linhas com highlighting para seleção
            let spans: Vec<Line> = content
                .split('\n')
                .map(|l| Line::from(l.to_string()))
                .collect();
            spans
        };

        // Indicador de unsaved
        let unsaved_indicator = if self.state.is_dirty() {
            " [●]"
        } else {
            " [✓]"
        };

        let title = format!(
            " Editor {} - Line {} Col {} {}",
            mode_text,
            line + 1,
            col + 1,
            unsaved_indicator
        );

        let visible_height = area.height.saturating_sub(2) as usize;
        let scroll_y = if visible_height > 0 && line >= visible_height {
            line - visible_height + 1
        } else {
            0
        };

        let border_color = if is_focused {
            theme.border_highlight()
        } else {
            theme.border()
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .scroll((scroll_y as u16, 0));

        f.render_widget(paragraph, area);

        // Renderizar cursor em insert mode
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
            // Undo: Ctrl+Z
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.undo();
                self.preferred_col = None;
            }
            // Redo: Ctrl+Y
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.redo();
                self.preferred_col = None;
            }
            // Copy: Ctrl+C
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.copy_selection();
            }
            // Paste: Ctrl+V
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.paste();
                self.preferred_col = None;
            }
            // Cut: Ctrl+X
            KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.cut_selection();
                self.preferred_col = None;
            }
            // Caracteres normais
            KeyCode::Char(c) => {
                self.state.insert_char(c);
                self.preferred_col = None;
            }
            // Enter
            KeyCode::Enter => {
                self.state.insert_char('\n');
                self.preferred_col = None;
            }
            // Backspace
            KeyCode::Backspace => {
                self.state.delete_prev_char();
                self.preferred_col = None;
            }
            // Delete
            KeyCode::Delete => {
                self.state.delete_next_char();
                self.preferred_col = None;
            }
            // Cursor left
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.state
                        .extend_selection(self.prev_boundary(self.state.cursor));
                } else {
                    self.state.cursor_left();
                }
                self.preferred_col = None;
            }
            // Cursor right
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.state
                        .extend_selection(self.next_boundary(self.state.cursor));
                } else {
                    self.state.cursor_right();
                }
                self.preferred_col = None;
            }
            // Home
            KeyCode::Home => {
                self.state.cursor_home();
                self.preferred_col = None;
            }
            // End
            KeyCode::End => {
                self.state.cursor_end();
                self.preferred_col = None;
            }
            // Up arrow
            KeyCode::Up => {
                let (line, col) = self.line_col_at_cursor();
                if line > 0 {
                    let target_col = self.preferred_col.unwrap_or(col);
                    let current_start = self.line_start(self.state.cursor);
                    let prev_end = current_start.saturating_sub(1);
                    let prev_start = self.line_start(prev_end);
                    let prev_line = &self.state.get_content()[prev_start..=prev_end];
                    let prev_line_text = prev_line.strip_suffix('\n').unwrap_or(prev_line);
                    let offset = Self::byte_index_for_column(prev_line_text, target_col);
                    self.state.cursor = prev_start + offset;
                    self.preferred_col = Some(target_col);
                }
            }
            // Down arrow
            KeyCode::Down => {
                let (_, col) = self.line_col_at_cursor();
                let target_col = self.preferred_col.unwrap_or(col);
                let current_end = self.line_end(self.state.cursor);
                let content = self.state.get_content();
                if current_end < content.len() {
                    let next_start = current_end + 1;
                    let next_end = self.line_end(next_start);
                    let next_line = &content[next_start..next_end];
                    let offset = Self::byte_index_for_column(next_line, target_col);
                    self.state.cursor = next_start + offset;
                    self.preferred_col = Some(target_col);
                }
            }
            _ => {}
        }
    }
}
