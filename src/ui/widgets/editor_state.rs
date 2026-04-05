// Editor state module - some utility methods reserved for future features
#![allow(dead_code)]

use super::editor_history::{EditAction, EditHistory};
use std::fmt;
use std::ops::Range;

/// Estrutura tipo "Rope" para gerenciar texto grande eficientemente
/// Usamos Vec<String> (uma para cada linha) para simplicidade e performance
#[derive(Clone, Debug)]
pub struct TextRope {
    /// Linhas de texto
    lines: Vec<String>,
}

impl fmt::Display for TextRope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}

impl TextRope {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    #[allow(dead_code)]
    pub fn from_string(content: &str) -> Self {
        let lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.split('\n').map(|s| s.to_string()).collect()
        };

        Self { lines }
    }

    pub fn len(&self) -> usize {
        self.lines.iter().map(|l| l.len() + 1).sum::<usize>()
            - if self.lines.is_empty() { 0 } else { 1 }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() || (self.lines.len() == 1 && self.lines[0].is_empty())
    }

    /// Insere caractere na posição linear (byte offset)
    pub fn insert_char(&mut self, pos: usize, ch: char) {
        let (line_idx, col) = self.byte_to_line_col(pos);

        if line_idx < self.lines.len() {
            self.lines[line_idx].insert(col, ch);
        }
    }

    /// Insere string na posição linear
    pub fn insert_str(&mut self, pos: usize, text: &str) {
        for ch in text.chars().rev() {
            self.insert_char(pos, ch);
        }
    }

    /// Deleta range de bytes
    pub fn delete_range(&mut self, range: Range<usize>) {
        if range.start >= range.end || range.start >= self.len() {
            return;
        }

        let end = range.end.min(self.len());
        let (start_line, start_col) = self.byte_to_line_col(range.start);
        let (end_line, end_col) = self.byte_to_line_col(end);

        if start_line == end_line {
            if start_line < self.lines.len() {
                let max_col = self.lines[start_line].len();
                if start_col < max_col && end_col <= max_col {
                    self.lines[start_line].drain(start_col..end_col);
                }
            }
        } else if start_line < self.lines.len() && end_line < self.lines.len() {
            let mut merged = String::new();

            if start_col <= self.lines[start_line].len() {
                let start_part = self.lines[start_line][..start_col].to_string();
                merged.push_str(&start_part);
            }

            if end_col <= self.lines[end_line].len() {
                let end_part = self.lines[end_line][end_col..].to_string();
                merged.push_str(&end_part);
            }

            for _ in start_line..=end_line {
                if start_line < self.lines.len() {
                    self.lines.remove(start_line);
                }
            }

            if start_line <= self.lines.len() {
                self.lines.insert(start_line, merged);
            }
        }
    }

    fn byte_to_line_col(&self, mut byte_pos: usize) -> (usize, usize) {
        for (line_idx, line) in self.lines.iter().enumerate() {
            let line_len = line.len() + 1;
            if byte_pos < line_len {
                return (line_idx, byte_pos);
            }
            byte_pos -= line_len;
        }
        (self.lines.len().saturating_sub(1), 0)
    }

    pub fn line_col_to_byte(&self, line: usize, col: usize) -> usize {
        let mut offset = 0;
        for (idx, l) in self.lines.iter().enumerate() {
            if idx == line {
                return offset + col;
            }
            offset += l.len() + 1;
        }
        offset
    }

    pub fn get_line(&self, idx: usize) -> Option<&str> {
        self.lines.get(idx).map(|s| s.as_str())
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(|s| s.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        let (s, e) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        Self { start: s, end: e }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn contains(&self, pos: usize) -> bool {
        self.start <= pos && pos < self.end
    }
}

pub struct EditorState {
    pub text: TextRope,
    pub cursor: usize,
    pub selection: Option<Selection>,
    pub history: EditHistory,
    pub clipboard: String,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            text: TextRope::new(),
            cursor: 0,
            selection: None,
            history: EditHistory::new(100),
            clipboard: String::new(),
        }
    }

    pub fn from_string(content: &str) -> Self {
        let rope = TextRope::from_string(content);
        let cursor = rope.len();
        Self {
            text: rope,
            cursor,
            selection: None,
            history: EditHistory::new(100),
            clipboard: String::new(),
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let ch_str = ch.to_string();
        self.text.insert_str(self.cursor, &ch_str);
        self.history.push(EditAction::Insert {
            pos: self.cursor,
            text: ch_str,
        });
        self.cursor += ch.len_utf8();
        self.selection = None;
    }

    pub fn insert_str(&mut self, text: &str) {
        self.text.insert_str(self.cursor, text);
        self.history.push(EditAction::Insert {
            pos: self.cursor,
            text: text.to_string(),
        });
        self.cursor += text.len();
        self.selection = None;
    }

    pub fn delete_prev_char(&mut self) {
        if let Some(sel) = self.selection {
            if sel.start < sel.end {
                self.delete_selection();
                return;
            }
        }
        if self.cursor > 0 {
            let start = self.prev_char_boundary(self.cursor);
            let content = self.text.to_string();
            let deleted = content.get(start..self.cursor).unwrap_or("").to_string();
            self.text.delete_range(start..self.cursor);
            self.history.push(EditAction::Delete {
                pos: start,
                text: deleted,
                len: self.cursor - start,
            });
            self.cursor = start;
            self.selection = None;
        }
    }

    pub fn delete_next_char(&mut self) {
        if let Some(sel) = self.selection {
            if sel.start < sel.end {
                self.delete_selection();
                return;
            }
        }
        if self.cursor < self.text.len() {
            let end = self.next_char_boundary(self.cursor);
            let content = self.text.to_string();
            let deleted = content.get(self.cursor..end).unwrap_or("").to_string();
            self.text.delete_range(self.cursor..end);
            self.history.push(EditAction::Delete {
                pos: self.cursor,
                text: deleted,
                len: end - self.cursor,
            });
            self.selection = None;
        }
    }

    pub fn delete_selection(&mut self) {
        if let Some(sel) = self.selection {
            let content = self.text.to_string();
            let text = content
                .get(sel.start..sel.end.min(content.len()))
                .unwrap_or("")
                .to_string();
            self.text.delete_range(sel.start..sel.end);
            self.history.push(EditAction::Delete {
                pos: sel.start,
                text,
                len: sel.len(),
            });
            self.cursor = sel.start;
            self.selection = None;
        }
    }

    pub fn copy_selection(&mut self) {
        if let Some(sel) = self.selection {
            if sel.start < sel.end {
                let content = self.text.to_string();
                self.clipboard = content
                    .get(sel.start..sel.end.min(content.len()))
                    .unwrap_or("")
                    .to_string();
            }
        }
    }

    pub fn paste(&mut self) {
        let clipboard_copy = self.clipboard.clone();
        if !clipboard_copy.is_empty() {
            self.insert_str(&clipboard_copy);
        }
    }

    pub fn cut_selection(&mut self) {
        if let Some(_sel) = self.selection {
            self.copy_selection();
            self.delete_selection();
        }
    }

    pub fn extend_selection(&mut self, new_cursor: usize) {
        if let Some(mut sel) = self.selection {
            sel.end = new_cursor;
            self.selection = Some(sel);
        } else {
            self.selection = Some(Selection::new(self.cursor, new_cursor));
        }
        self.cursor = new_cursor;
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.history.undo() {
            self.apply_action(&action.inverse());
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.history.redo() {
            self.apply_action(&action);
        }
    }

    fn apply_action(&mut self, action: &EditAction) {
        match action {
            EditAction::Insert { pos, text } => {
                self.text.insert_str(*pos, text);
                self.cursor = pos + text.len();
            }
            EditAction::Delete { pos, text: _, len } => {
                self.text.delete_range(*pos..pos + len);
                self.cursor = *pos;
            }
            EditAction::SelectionChange { start, end } => {
                self.selection = Some(Selection::new(*start, *end));
                self.cursor = *end;
            }
        }
        self.selection = None;
    }

    pub fn cursor_left(&mut self) {
        self.cursor = self.prev_char_boundary(self.cursor);
        self.selection = None;
    }

    pub fn cursor_right(&mut self) {
        self.cursor = self.next_char_boundary(self.cursor);
        self.selection = None;
    }

    pub fn cursor_home(&mut self) {
        let content = self.text.to_string();
        let line_start = content[..self.cursor]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        self.cursor = line_start;
        self.selection = None;
    }

    pub fn cursor_end(&mut self) {
        let content = self.text.to_string();
        let line_end = content[self.cursor..]
            .find('\n')
            .map(|i| self.cursor + i)
            .unwrap_or(content.len());
        self.cursor = line_end;
        self.selection = None;
    }

    fn prev_char_boundary(&self, pos: usize) -> usize {
        let content = self.text.to_string();
        if pos == 0 {
            return 0;
        }
        let mut i = pos - 1;
        while i > 0 && !content.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    fn next_char_boundary(&self, pos: usize) -> usize {
        let content = self.text.to_string();
        if pos >= content.len() {
            return content.len();
        }
        let mut i = pos + 1;
        while i < content.len() && !content.is_char_boundary(i) {
            i += 1;
        }
        i
    }

    pub fn get_content(&self) -> String {
        self.text.to_string()
    }

    pub fn set_content(&mut self, content: &str) {
        self.text = TextRope::from_string(content);
        self.cursor = 0;
        self.selection = None;
        self.history.clear();
    }

    pub fn clear(&mut self) {
        self.text = TextRope::new();
        self.cursor = 0;
        self.selection = None;
        self.history.clear();
    }

    pub fn is_dirty(&self) -> bool {
        self.history.dirty
    }

    pub fn mark_saved(&mut self) {
        self.history.mark_saved();
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    #[allow(dead_code)]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        let content = self.text.to_string();
        let content_before_cursor = content.get(0..self.cursor.min(content.len())).unwrap_or("");
        let line = content_before_cursor.chars().filter(|&c| c == '\n').count();
        let col = content_before_cursor
            .rsplit('\n')
            .next()
            .map(|s: &str| s.chars().count())
            .unwrap_or(0);
        (line, col)
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_rope() {
        let mut rope = TextRope::new();
        rope.insert_char(0, 'h');
        rope.insert_char(1, 'i');
        assert_eq!(rope.to_string(), "hi");
    }

    #[test]
    fn test_editor_state_basic() {
        let mut state = EditorState::new();
        state.insert_char('a');
        state.insert_char('b');
        assert_eq!(state.get_content(), "ab");
    }

    #[test]
    fn test_undo_redo() {
        let mut state = EditorState::new();
        state.insert_char('a');
        state.insert_char('b');
        assert_eq!(state.get_content(), "ab");
        state.undo();
        assert_eq!(state.get_content(), "a");
        state.redo();
        assert_eq!(state.get_content(), "ab");
    }

    #[test]
    fn test_selection_and_copy() {
        let mut state = EditorState::from_string("hello world");
        state.cursor = 0;
        state.selection = Some(Selection::new(0, 5));
        state.copy_selection();
        assert_eq!(state.clipboard, "hello");
    }

    #[test]
    fn test_delete_prev_char() {
        let mut state = EditorState::from_string("hello");
        state.cursor = 2;
        state.delete_prev_char();
        assert_eq!(state.get_content(), "hllo");
    }

    #[test]
    fn test_delete_selection() {
        let mut state = EditorState::from_string("hello world");
        state.cursor = 0;
        state.selection = Some(Selection::new(0, 5));
        state.delete_selection();
        assert_eq!(state.get_content(), " world");
    }
}
