use crate::db::Database;
use crate::link::LinkValidator;
use crate::ui::app::Mode;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::{CursorMove, TextArea};
use unicode_width::UnicodeWidthChar;

use super::link_autocomplete::LinkAutocomplete;

pub struct NoteEditor {
    textarea: TextArea<'static>,
    autocomplete: LinkAutocomplete,
    autocomplete_pattern: String,
}

impl Default for NoteEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl NoteEditor {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_placeholder_text("Press 'i' to enter insert mode or 'n' to create a new note");
        Self {
            textarea,
            autocomplete: LinkAutocomplete::new(),
            autocomplete_pattern: String::new(),
        }
    }

    pub fn set_content(&mut self, content: &str) {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        self.textarea = TextArea::new(lines);
        self.textarea
            .set_placeholder_text("Press 'i' to enter insert mode or 'n' to create a new note");
        self.autocomplete.clear();
        self.autocomplete_pattern.clear();
    }

    pub fn get_content(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn clear(&mut self) {
        self.textarea = TextArea::default();
        self.textarea
            .set_placeholder_text("Press 'i' to enter insert mode or 'n' to create a new note");
        self.autocomplete.clear();
        self.autocomplete_pattern.clear();
    }

    pub fn is_dirty(&self) -> bool {
        !self.textarea.lines().is_empty() && self.textarea.lines().iter().any(|l| !l.is_empty())
    }

    pub fn mark_saved(&mut self) {
        // App handles save via DB write
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        (self.textarea.cursor().0, self.textarea.cursor().1)
    }

    /// Wraps a line into segments that fit within `max_width` display columns.
    /// Returns a vector of (start_char_idx, end_char_idx) for each segment.
    pub fn wrap_line_chars(line: &str, max_width: usize) -> Vec<(usize, usize)> {
        if max_width == 0 {
            return vec![(0, line.chars().count())];
        }

        let mut segments = Vec::new();
        let mut current_width = 0;
        let mut segment_start = 0;

        let chars: Vec<char> = line.chars().collect();
        let char_count = chars.len();

        for (i, &ch) in chars.iter().enumerate() {
            let char_width = ch.width().unwrap_or(1);

            if current_width + char_width > max_width && current_width > 0 {
                segments.push((segment_start, i));
                segment_start = i;
                current_width = char_width;
            } else {
                current_width += char_width;
            }
        }

        if segment_start < char_count {
            segments.push((segment_start, char_count));
        }

        if segments.is_empty() {
            segments.push((0, char_count));
        }

        segments
    }

    fn extract_link_pattern_at_cursor(&self) -> Option<String> {
        let lines = self.textarea.lines();
        let (row, col) = self.textarea.cursor();
        let line = lines.get(row)?;
        let before_cursor = line.chars().take(col).collect::<String>();
        if let Some(link_start) = before_cursor.rfind("[[") {
            let pattern = before_cursor[link_start + 2..].to_string();
            if !pattern.contains("]]") && !pattern.is_empty() {
                return Some(pattern);
            }
        }
        None
    }

    pub fn update_autocomplete(&mut self, db: &Database) {
        if let Some(pattern) = self.extract_link_pattern_at_cursor() {
            self.autocomplete_pattern = pattern.clone();
            self.autocomplete.search(&pattern, db, 10);
        } else {
            self.autocomplete.clear();
            self.autocomplete_pattern.clear();
        }
    }

    pub fn get_selected_suggestion(&self) -> Option<(String, String)> {
        self.autocomplete
            .selected()
            .map(|s| (s.note_title.clone(), s.note_id.clone()))
    }

    pub fn insert_suggestion(&mut self, note_id: &str) {
        if let Some(pattern) = self.extract_link_pattern_at_cursor() {
            for _ in 0..pattern.len() {
                self.textarea.delete_char();
            }
            self.textarea.insert_str(format!("[[{}]]", note_id));
            self.autocomplete.clear();
        }
    }

    pub fn draw(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &dyn crate::config::Theme,
        mode: Mode,
        is_focused: bool,
        db: &Database,
    ) {
        let mode_text = match mode {
            Mode::Normal => "-- NORMAL --",
            Mode::Insert => "-- INSERT --",
            Mode::Search => "-- SEARCH --",
            Mode::Command => "-- COMMAND --",
            Mode::Graph => "-- GRAPH --",
        };

        let (cursor_line, cursor_col) = self.cursor_line_col();
        let lines = self.textarea.lines();

        let max_width = area.width.saturating_sub(2) as usize;

        let mut rendered_lines: Vec<Line> = Vec::new();
        let mut wrapped_cursor_row = 0;
        let mut wrapped_cursor_col_offset = 0;

        if lines.iter().all(|l| l.is_empty()) {
            rendered_lines.push(Line::from(Span::styled(
                "Press 'i' to enter insert mode or 'n' to create a new note",
                Style::default().fg(theme.fg_dim()),
            )));
        } else {
            for (line_idx, text) in lines.iter().enumerate() {
                let segments = Self::wrap_line_chars(text, max_width);

                for (start, end) in segments.iter() {
                    let segment_text: String =
                        text.chars().skip(*start).take(end - start).collect();
                    let validated_links = LinkValidator::extract_links(text, line_idx, db);

                    if validated_links.is_empty() {
                        rendered_lines.push(Line::from(segment_text));
                    } else {
                        let mut spans = Vec::new();
                        let mut last_end = *start;

                        for link in &validated_links {
                            let link_start = link.info.position.start_col;
                            let link_end = link.info.position.end_col;

                            if link_end <= *start || link_start >= *end {
                                continue;
                            }

                            let overlap_start = link_start.max(*start);
                            let overlap_end = link_end.min(*end);

                            if last_end < overlap_start {
                                let gap: String = text
                                    .chars()
                                    .skip(last_end)
                                    .take(overlap_start - last_end)
                                    .collect();
                                spans.push(Span::raw(gap));
                            }

                            let link_text: String = text
                                .chars()
                                .skip(overlap_start)
                                .take(overlap_end - overlap_start)
                                .collect();
                            let link_color = if link.is_valid {
                                theme.link()
                            } else {
                                theme.error()
                            };
                            spans.push(Span::styled(
                                link_text,
                                Style::default().fg(link_color).add_modifier(Modifier::BOLD),
                            ));
                            last_end = overlap_end;
                        }

                        if last_end < *end {
                            let remainder: String =
                                text.chars().skip(last_end).take(*end - last_end).collect();
                            spans.push(Span::raw(remainder));
                        }

                        rendered_lines.push(Line::from(spans));
                    }

                    if line_idx == cursor_line && cursor_col >= *start && cursor_col <= *end {
                        wrapped_cursor_row = rendered_lines.len() - 1;
                        wrapped_cursor_col_offset = cursor_col - start;
                    }
                }

                if segments.is_empty() && line_idx == cursor_line && cursor_col == 0 {
                    wrapped_cursor_row = rendered_lines.len();
                    wrapped_cursor_col_offset = 0;
                    rendered_lines.push(Line::from(""));
                }
            }
        }

        let unsaved_indicator = if self.is_dirty() { " [●]" } else { " [✓]" };
        let title = format!(
            " Editor {} - Line {} Col {} {}",
            mode_text,
            cursor_line + 1,
            cursor_col + 1,
            unsaved_indicator
        );

        let visible_height = area.height.saturating_sub(2) as usize;
        let scroll_y = if visible_height > 0 && wrapped_cursor_row >= visible_height {
            wrapped_cursor_row - visible_height + 1
        } else {
            0
        };

        let border_color = if is_focused {
            theme.border_highlight()
        } else {
            theme.border()
        };

        let paragraph = Paragraph::new(rendered_lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .scroll((scroll_y as u16, 0));

        f.render_widget(paragraph, area);

        if area.width > 2 && area.height > 2 {
            let display_row = wrapped_cursor_row.saturating_sub(scroll_y);
            if display_row < visible_height {
                let cursor_x =
                    area.x + 1 + (wrapped_cursor_col_offset as u16).min(max_width as u16);
                let cursor_y = area.y + 1 + (display_row as u16);

                if cursor_y < area.y + area.height - 1 {
                    if mode == Mode::Normal && is_focused {
                        let block = Block::default().style(
                            Style::default()
                                .fg(Color::White)
                                .bg(theme.accent())
                                .add_modifier(Modifier::BOLD),
                        );
                        let block_area = Rect {
                            x: cursor_x,
                            y: cursor_y,
                            width: 1,
                            height: 1,
                        };
                        f.render_widget(block, block_area);
                    } else if mode == Mode::Insert {
                        f.set_cursor(cursor_x, cursor_y);
                    }
                }
            }
        }

        if self.autocomplete.is_visible() && area.height > 5 {
            let autocomplete_area = Rect {
                x: area.x + 1,
                y: area.y + 3,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(5),
            };
            self.autocomplete.draw(f, autocomplete_area, theme);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.textarea.input(key);
    }

    pub fn undo(&mut self) {
        self.textarea.undo();
    }

    pub fn redo(&mut self) {
        self.textarea.redo();
    }

    pub fn copy(&mut self) {
        self.textarea.copy();
    }

    pub fn cut(&mut self) {
        self.textarea.cut();
    }

    pub fn paste(&mut self) {
        self.textarea.paste();
    }

    pub fn update_autocomplete_db(&mut self, db: &Database) {
        if let Some(pattern) = self.extract_link_pattern_at_cursor() {
            self.autocomplete_pattern = pattern.clone();
            self.autocomplete.search(&pattern, db, 10);
        } else {
            self.autocomplete.clear();
            self.autocomplete_pattern.clear();
        }
    }

    pub fn handle_autocomplete_key(&mut self, key: KeyEvent, db: &Database) -> bool {
        match key.code {
            KeyCode::Tab | KeyCode::Enter => {
                if self.autocomplete.is_visible() {
                    if let Some(suggestion) = self.autocomplete.selected() {
                        let note_id = suggestion.note_id.clone();
                        self.insert_suggestion(&note_id);
                        return true;
                    }
                }
                false
            }
            KeyCode::Esc => {
                self.autocomplete.clear();
                false
            }
            KeyCode::Down => {
                if self.autocomplete.is_visible() {
                    self.autocomplete.select_next();
                }
                false
            }
            KeyCode::Up => {
                if self.autocomplete.is_visible() {
                    self.autocomplete.select_prev();
                }
                false
            }
            _ => {
                self.update_autocomplete_db(db);
                false
            }
        }
    }

    pub fn is_autocomplete_visible(&self) -> bool {
        self.autocomplete.is_visible()
    }

    pub fn autocomplete_height(&self) -> u16 {
        if self.autocomplete.is_visible() {
            5
        } else {
            0
        }
    }

    pub fn move_cursor_up(&mut self) {
        self.textarea.move_cursor(CursorMove::Up);
    }

    pub fn move_cursor_down(&mut self) {
        self.textarea.move_cursor(CursorMove::Down);
    }

    pub fn move_cursor_left(&mut self) {
        self.textarea.move_cursor(CursorMove::Back);
    }

    pub fn move_cursor_right(&mut self) {
        self.textarea.move_cursor(CursorMove::Forward);
    }

    pub fn move_cursor_home(&mut self) {
        self.textarea.move_cursor(CursorMove::Head);
    }

    pub fn move_cursor_end(&mut self) {
        self.textarea.move_cursor(CursorMove::End);
    }

    pub fn move_cursor_top(&mut self) {
        self.textarea.move_cursor(CursorMove::Top);
    }

    pub fn move_cursor_bottom(&mut self) {
        self.textarea.move_cursor(CursorMove::Bottom);
    }

    pub fn move_cursor_word_forward(&mut self) {
        self.textarea.move_cursor(CursorMove::WordForward);
    }

    pub fn move_cursor_word_back(&mut self) {
        self.textarea.move_cursor(CursorMove::WordBack);
    }

    pub fn delete_next_char(&mut self) {
        self.textarea.delete_next_char();
    }

    pub fn delete_prev_char(&mut self) {
        self.textarea.delete_char();
    }

    pub fn delete_next_word(&mut self) {
        self.textarea.delete_word();
    }

    pub fn delete_prev_word(&mut self) {
        self.textarea.delete_word();
        self.textarea.move_cursor(CursorMove::WordBack);
    }

    pub fn delete_to_end_of_line(&mut self) {
        self.textarea.delete_line_by_end();
    }

    pub fn delete_line(&mut self) {
        self.textarea.delete_line_by_head();
    }

    pub fn insert_newline(&mut self) {
        self.textarea.insert_newline();
    }

    pub fn line_count(&self) -> usize {
        self.textarea.lines().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_set_and_get_content() {
        let mut editor = NoteEditor::new();
        editor.set_content("hello\nworld");
        assert_eq!(editor.get_content(), "hello\nworld");
    }

    #[test]
    fn test_editor_clear() {
        let mut editor = NoteEditor::new();
        editor.set_content("test content");
        editor.clear();
        assert_eq!(editor.get_content(), "");
    }

    #[test]
    fn test_editor_cursor_starts_at_origin() {
        let mut editor = NoteEditor::new();
        editor.set_content("hello\nworld");
        let (line, col) = editor.cursor_line_col();
        assert_eq!(line, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_editor_cursor_movement() {
        let mut editor = NoteEditor::new();
        editor.set_content("hello world");
        editor.move_cursor_home();
        let (_, col) = editor.cursor_line_col();
        assert_eq!(col, 0);
        editor.move_cursor_end();
        let (_, col) = editor.cursor_line_col();
        assert_eq!(col, 11);
    }

    #[test]
    fn test_editor_cursor_multiline() {
        let mut editor = NoteEditor::new();
        editor.set_content("hello\nworld");
        editor.move_cursor_bottom();
        editor.move_cursor_end();
        let (line, col) = editor.cursor_line_col();
        assert_eq!(line, 1);
        assert_eq!(col, 5);
    }

    #[test]
    fn test_editor_undo_redo() {
        let mut editor = NoteEditor::new();
        editor.set_content("hello");
        editor.undo();
        editor.redo();
    }

    #[test]
    fn test_editor_word_movement() {
        let mut editor = NoteEditor::new();
        editor.set_content("hello world foo");
        editor.move_cursor_home();
        editor.move_cursor_word_forward();
        let (_, col) = editor.cursor_line_col();
        assert_eq!(col, 6);
        editor.move_cursor_word_back();
        let (_, col) = editor.cursor_line_col();
        assert_eq!(col, 0);
    }

    #[test]
    fn test_editor_delete_operations() {
        let mut editor = NoteEditor::new();
        editor.set_content("hello");
        editor.move_cursor_end();
        editor.delete_prev_char();
        assert_eq!(editor.get_content(), "hell");
        editor.move_cursor_home();
        editor.delete_next_char();
        assert_eq!(editor.get_content(), "ell");
    }

    #[test]
    fn test_editor_insert_newline() {
        let mut editor = NoteEditor::new();
        editor.set_content("hello");
        editor.move_cursor_end();
        editor.insert_newline();
        assert_eq!(editor.get_content(), "hello\n");
    }

    #[test]
    fn test_editor_line_count() {
        let mut editor = NoteEditor::new();
        editor.set_content("line1\nline2\nline3");
        assert_eq!(editor.line_count(), 3);
    }

    #[test]
    fn test_editor_copy_paste() {
        let mut editor = NoteEditor::new();
        editor.set_content("hello");
        editor.textarea.select_all();
        editor.textarea.copy();
        let yanked = editor.textarea.yank_text();
        assert_eq!(yanked, "hello");
    }

    #[test]
    fn test_wrap_line_chars_empty_string() {
        let segments = NoteEditor::wrap_line_chars("", 10);
        assert_eq!(segments, vec![(0, 0)]);
    }

    #[test]
    fn test_wrap_line_chars_fits_in_width() {
        let segments = NoteEditor::wrap_line_chars("hello", 10);
        assert_eq!(segments, vec![(0, 5)]);
    }

    #[test]
    fn test_wrap_line_chars_exactly_fits() {
        let segments = NoteEditor::wrap_line_chars("hello", 5);
        assert_eq!(segments, vec![(0, 5)]);
    }

    #[test]
    fn test_wrap_line_chars_splits_long_line() {
        let segments = NoteEditor::wrap_line_chars("hello world", 5);
        assert_eq!(segments, vec![(0, 5), (5, 10), (10, 11)]);
    }

    #[test]
    fn test_wrap_line_chars_splits_into_three() {
        let segments = NoteEditor::wrap_line_chars("abcdefghijkl", 4);
        assert_eq!(segments, vec![(0, 4), (4, 8), (8, 12)]);
    }

    #[test]
    fn test_wrap_line_chars_zero_width() {
        let segments = NoteEditor::wrap_line_chars("hello", 0);
        assert_eq!(segments, vec![(0, 5)]);
    }

    #[test]
    fn test_wrap_line_chars_single_char_width() {
        let segments = NoteEditor::wrap_line_chars("abc", 1);
        assert_eq!(segments, vec![(0, 1), (1, 2), (2, 3)]);
    }

    #[test]
    fn test_wrap_line_chars_with_unicode_wide_chars() {
        let segments = NoteEditor::wrap_line_chars("中文测试", 4);
        assert_eq!(segments, vec![(0, 2), (2, 4)]);
    }

    #[test]
    fn test_wrap_line_chars_mixed_width() {
        let segments = NoteEditor::wrap_line_chars("ab中文cd", 4);
        assert_eq!(segments, vec![(0, 3), (3, 6)]);
    }

    #[test]
    fn test_wrap_line_chars_preserves_char_boundaries() {
        let segments = NoteEditor::wrap_line_chars("hello world foo bar", 10);
        assert_eq!(segments, vec![(0, 10), (10, 19)]);
    }

    #[test]
    fn test_wrap_line_chars_emoji_handling() {
        let segments = NoteEditor::wrap_line_chars("a🎉b🎉c", 3);
        assert_eq!(segments, vec![(0, 2), (2, 4), (4, 5)]);
    }

    #[test]
    fn test_wrap_line_chars_large_width_no_wrap() {
        let segments = NoteEditor::wrap_line_chars("short", 100);
        assert_eq!(segments, vec![(0, 5)]);
    }
}
