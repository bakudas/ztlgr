// Search widgets - some methods reserved for future search enhancements
#![allow(dead_code)]

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Search result item containing note info and relevance score
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub note_id: String,
    pub title: String,
    pub excerpt: String,
    pub score: f32, // Relevance score 0.0-1.0
}

/// Search input widget - handles user text input for search queries
#[derive(Debug, Clone)]
pub struct SearchInput {
    query: String,
    cursor_pos: usize, // Character position, not byte position
    max_length: usize,
}

impl SearchInput {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            cursor_pos: 0,
            max_length: 200,
        }
    }

    /// Get current search query
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get cursor position (in characters)
    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    /// Get character count
    fn char_count(&self) -> usize {
        self.query.chars().count()
    }

    /// Add character to query at end
    pub fn add_char(&mut self, c: char) {
        if self.char_count() < self.max_length {
            self.query.push(c);
            self.cursor_pos = self.char_count();
        }
    }

    /// Remove character before cursor
    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            let mut chars = self.query.chars().collect::<Vec<_>>();
            chars.remove(self.cursor_pos - 1);
            self.query = chars.iter().collect();
            self.cursor_pos -= 1;
        }
    }

    /// Clear entire query
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_pos = 0;
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.char_count() {
            self.cursor_pos += 1;
        }
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor_pos = self.char_count();
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => self.add_char(c),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Home => self.cursor_home(),
            KeyCode::End => self.cursor_end(),
            _ => {}
        }
    }

    /// Draw the search input widget
    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme, active: bool) {
        let style = if active {
            ratatui::style::Style::default()
                .fg(theme.accent())
                .bg(theme.bg())
        } else {
            ratatui::style::Style::default()
                .fg(theme.fg())
                .bg(theme.bg())
        };

        let block = Block::default()
            .title("🔍 Search")
            .borders(Borders::ALL)
            .style(style);

        let inner = block.inner(area);
        let cursor_char = if active { "│" } else { " " };

        // Build display with cursor at correct character position
        let chars: Vec<char> = self.query.chars().collect();
        let mut display_query = String::new();

        for (i, ch) in chars.iter().enumerate() {
            if i == self.cursor_pos {
                display_query.push_str(cursor_char);
            }
            display_query.push(*ch);
        }

        if self.cursor_pos >= chars.len() {
            display_query.push_str(cursor_char);
        }

        let text = Line::from(display_query);
        let para = Paragraph::new(text).style(style);

        f.render_widget(block, area);
        f.render_widget(para, inner);
    }
}

impl Default for SearchInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Search results widget - displays search results with selection
#[derive(Debug, Clone)]
pub struct SearchResults {
    results: Vec<SearchResult>,
    selected_index: usize,
}

impl SearchResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            selected_index: 0,
        }
    }

    /// Add a search result
    pub fn add_result(&mut self, result: SearchResult) {
        self.results.push(result);
        // Keep sorted by score descending
        self.results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Get number of results
    pub fn count(&self) -> usize {
        self.results.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get selected result index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Select next result
    pub fn select_next(&mut self) {
        if self.selected_index < self.results.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Select previous result
    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Get selected result
    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.results.get(self.selected_index)
    }

    /// Get all results sorted by score
    pub fn sorted_results(&self) -> Vec<SearchResult> {
        let mut sorted = self.results.clone();
        sorted.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// Clear all results
    pub fn clear(&mut self) {
        self.results.clear();
        self.selected_index = 0;
    }

    /// Draw the search results widget
    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme) {
        let block = Block::default()
            .title(format!("Results ({})", self.count()))
            .borders(Borders::ALL)
            .style(
                ratatui::style::Style::default()
                    .fg(theme.fg())
                    .bg(theme.bg()),
            );

        let inner = block.inner(area);

        if self.is_empty() {
            let message = Paragraph::new("No results. Type to search...").style(
                ratatui::style::Style::default()
                    .fg(theme.fg_dim())
                    .bg(theme.bg()),
            );
            f.render_widget(block, area);
            f.render_widget(message, inner);
        } else {
            // Render results list
            let mut lines = Vec::new();
            for (idx, result) in self.results.iter().enumerate() {
                let is_selected = idx == self.selected_index;
                let style = if is_selected {
                    ratatui::style::Style::default()
                        .fg(theme.bg())
                        .bg(theme.accent())
                } else {
                    ratatui::style::Style::default()
                        .fg(theme.fg())
                        .bg(theme.bg())
                };

                let line_text = format!("▸ {} ({}%)", result.title, (result.score * 100.0) as u32);
                lines.push(Line::from(ratatui::text::Span::styled(line_text, style)));
            }

            let para = Paragraph::new(lines);
            f.render_widget(block, area);
            f.render_widget(para, inner);
        }
    }
}

impl Default for SearchResults {
    fn default() -> Self {
        Self::new()
    }
}

/// Overall search state
#[derive(Debug, Clone)]
pub struct SearchState {
    pub input: SearchInput,
    pub results: SearchResults,
    pub is_searching: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            input: SearchInput::new(),
            results: SearchResults::new(),
            is_searching: false,
        }
    }

    /// Clear search state
    pub fn clear(&mut self) {
        self.input.clear();
        self.results.clear();
        self.is_searching = false;
    }

    /// Set searching flag
    pub fn set_searching(&mut self, searching: bool) {
        self.is_searching = searching;
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ====== SearchInput Widget Tests ======

    #[test]
    fn test_search_input_creation() {
        let input = SearchInput::new();
        assert_eq!(input.query(), "");
        assert_eq!(input.cursor_pos(), 0);
    }

    #[test]
    fn test_search_input_add_char() {
        let mut input = SearchInput::new();
        input.add_char('t');
        input.add_char('e');
        input.add_char('s');
        input.add_char('t');

        assert_eq!(input.query(), "test");
        assert_eq!(input.cursor_pos(), 4);
    }

    #[test]
    fn test_search_input_backspace() {
        let mut input = SearchInput::new();
        input.add_char('t');
        input.add_char('e');
        input.add_char('s');
        input.add_char('t');

        input.backspace();
        assert_eq!(input.query(), "tes");
        assert_eq!(input.cursor_pos(), 3);

        input.backspace();
        input.backspace();
        input.backspace();
        assert_eq!(input.query(), "");
        assert_eq!(input.cursor_pos(), 0);
    }

    #[test]
    fn test_search_input_backspace_empty() {
        let mut input = SearchInput::new();
        input.backspace();
        assert_eq!(input.query(), "");
        assert_eq!(input.cursor_pos(), 0);
    }

    #[test]
    fn test_search_input_clear() {
        let mut input = SearchInput::new();
        input.add_char('s');
        input.add_char('e');
        input.add_char('a');
        input.add_char('r');
        input.add_char('c');
        input.add_char('h');

        input.clear();
        assert_eq!(input.query(), "");
        assert_eq!(input.cursor_pos(), 0);
    }

    #[test]
    fn test_search_input_cursor_navigation() {
        let mut input = SearchInput::new();
        input.add_char('a');
        input.add_char('b');
        input.add_char('c');
        input.add_char('d');

        assert_eq!(input.cursor_pos(), 4);

        input.move_cursor_left();
        assert_eq!(input.cursor_pos(), 3);

        input.move_cursor_left();
        input.move_cursor_left();
        assert_eq!(input.cursor_pos(), 1);

        input.move_cursor_right();
        assert_eq!(input.cursor_pos(), 2);

        input.move_cursor_right();
        input.move_cursor_right();
        input.move_cursor_right();
        assert_eq!(input.cursor_pos(), 4); // Clamped to end (4 chars, so max pos is 4)
    }

    #[test]
    fn test_search_input_cursor_home_end() {
        let mut input = SearchInput::new();
        input.add_char('h');
        input.add_char('e');
        input.add_char('l');
        input.add_char('l');
        input.add_char('o');

        input.cursor_home();
        assert_eq!(input.cursor_pos(), 0);

        input.cursor_end();
        assert_eq!(input.cursor_pos(), 5);
    }

    #[test]
    fn test_search_input_max_length() {
        let mut input = SearchInput::new();
        for i in 0..250 {
            input.add_char('a');
            if i < 200 {
                assert_eq!(input.query().chars().count(), i + 1);
            } else {
                // Should be capped at 200
                assert_eq!(input.query().chars().count(), 200);
            }
        }
    }

    #[test]
    fn test_search_input_utf8_safe() {
        let mut input = SearchInput::new();
        input.add_char('é');
        input.add_char('à');
        input.add_char('ñ');

        assert_eq!(input.query(), "éàñ");
        assert_eq!(input.cursor_pos(), 3);
    }

    // ====== SearchResults Tests ======

    #[test]
    fn test_search_results_creation() {
        let results = SearchResults::new();
        assert!(results.is_empty());
        assert_eq!(results.selected_index(), 0);
    }

    #[test]
    fn test_search_results_add_result() {
        let mut results = SearchResults::new();

        let result = SearchResult {
            note_id: "note1".to_string(),
            title: "My Note".to_string(),
            excerpt: "This is a matching excerpt...".to_string(),
            score: 0.95,
        };

        results.add_result(result);
        assert_eq!(results.count(), 1);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_results_navigation() {
        let mut results = SearchResults::new();

        for i in 0..5 {
            results.add_result(SearchResult {
                note_id: format!("note{}", i),
                title: format!("Note {}", i),
                excerpt: format!("Excerpt {}", i),
                score: 0.9,
            });
        }

        assert_eq!(results.selected_index(), 0);

        results.select_next();
        assert_eq!(results.selected_index(), 1);

        results.select_next();
        results.select_next();
        assert_eq!(results.selected_index(), 3);

        results.select_next();
        assert_eq!(results.selected_index(), 4);

        results.select_next(); // Should not go beyond
        assert_eq!(results.selected_index(), 4);
    }

    #[test]
    fn test_search_results_select_prev() {
        let mut results = SearchResults::new();

        for i in 0..3 {
            results.add_result(SearchResult {
                note_id: format!("note{}", i),
                title: format!("Note {}", i),
                excerpt: format!("Excerpt {}", i),
                score: 0.9,
            });
        }

        results.select_next();
        results.select_next();
        assert_eq!(results.selected_index(), 2);

        results.select_prev();
        assert_eq!(results.selected_index(), 1);

        results.select_prev();
        assert_eq!(results.selected_index(), 0);

        results.select_prev(); // Should not go below
        assert_eq!(results.selected_index(), 0);
    }

    #[test]
    fn test_search_results_selected_result() {
        let mut results = SearchResults::new();

        let result1 = SearchResult {
            note_id: "note1".to_string(),
            title: "First".to_string(),
            excerpt: "First excerpt".to_string(),
            score: 0.9,
        };

        let result2 = SearchResult {
            note_id: "note2".to_string(),
            title: "Second".to_string(),
            excerpt: "Second excerpt".to_string(),
            score: 0.8,
        };

        results.add_result(result1);
        results.add_result(result2);

        let selected = results.selected_result();
        assert_eq!(selected.unwrap().note_id, "note1");

        results.select_next();
        let selected = results.selected_result();
        assert_eq!(selected.unwrap().note_id, "note2");
    }

    #[test]
    fn test_search_results_clear() {
        let mut results = SearchResults::new();

        for i in 0..3 {
            results.add_result(SearchResult {
                note_id: format!("note{}", i),
                title: format!("Note {}", i),
                excerpt: format!("Excerpt {}", i),
                score: 0.9,
            });
        }

        assert_eq!(results.count(), 3);

        results.clear();
        assert_eq!(results.count(), 0);
        assert!(results.is_empty());
        assert_eq!(results.selected_index(), 0);
    }

    #[test]
    fn test_search_results_sorted_by_score() {
        let mut results = SearchResults::new();

        results.add_result(SearchResult {
            note_id: "note1".to_string(),
            title: "First".to_string(),
            excerpt: "First".to_string(),
            score: 0.5,
        });

        results.add_result(SearchResult {
            note_id: "note2".to_string(),
            title: "Second".to_string(),
            excerpt: "Second".to_string(),
            score: 0.9,
        });

        results.add_result(SearchResult {
            note_id: "note3".to_string(),
            title: "Third".to_string(),
            excerpt: "Third".to_string(),
            score: 0.7,
        });

        // Results should be sorted by score descending
        let sorted = results.sorted_results();
        assert_eq!(sorted[0].note_id, "note2"); // 0.9
        assert_eq!(sorted[1].note_id, "note3"); // 0.7
        assert_eq!(sorted[2].note_id, "note1"); // 0.5
    }

    // ====== SearchState Tests ======

    #[test]
    fn test_search_state_creation() {
        let state = SearchState::new();
        assert_eq!(state.input.query(), "");
        assert!(state.results.is_empty());
        assert!(!state.is_searching);
    }

    #[test]
    fn test_search_state_input_typing() {
        let mut state = SearchState::new();

        state.input.add_char('r');
        state.input.add_char('u');
        state.input.add_char('s');
        state.input.add_char('t');

        assert_eq!(state.input.query(), "rust");
    }

    #[test]
    fn test_search_state_clear_results() {
        let mut state = SearchState::new();

        state.results.add_result(SearchResult {
            note_id: "note1".to_string(),
            title: "Rust".to_string(),
            excerpt: "Rust programming".to_string(),
            score: 0.9,
        });

        assert_eq!(state.results.count(), 1);

        state.clear();
        assert_eq!(state.results.count(), 0);
        assert_eq!(state.input.query(), "");
    }

    #[test]
    fn test_search_state_is_searching_flag() {
        let mut state = SearchState::new();
        assert!(!state.is_searching);

        state.set_searching(true);
        assert!(state.is_searching);

        state.set_searching(false);
        assert!(!state.is_searching);
    }
}
