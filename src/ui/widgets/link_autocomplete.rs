// TODO: This module is complete but not yet integrated into app.rs
// Integration pending: add Up/Down arrow key handling for autocomplete selection
// Tracked in: Phase 5B Link Features
#![allow(dead_code)]

use crate::db::Database;
use crate::link::fuzzy_match;
use crate::note::Note;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Link autocomplete suggestion
#[derive(Debug, Clone)]
pub struct LinkSuggestion {
    pub note_title: String,
    pub note_id: String,
    pub score: u32,
}

impl LinkSuggestion {
    pub fn from_note(note: &Note, pattern: &str) -> Self {
        // Score based on title match
        let title_score = fuzzy_match(pattern, &note.title);
        let id_score = fuzzy_match(pattern, note.id.as_str());
        let score = title_score.max(id_score);

        Self {
            note_title: note.title.clone(),
            note_id: note.id.as_str().to_string(),
            score,
        }
    }
}

/// Link autocomplete suggestion menu
pub struct LinkAutocomplete {
    suggestions: Vec<LinkSuggestion>,
    selected: usize,
    visible: bool,
}

impl LinkAutocomplete {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            selected: 0,
            visible: false,
        }
    }

    /// Search for link suggestions matching the pattern
    pub fn search(&mut self, pattern: &str, db: &Database, max_results: usize) {
        if pattern.is_empty() || pattern.len() < 2 {
            self.suggestions.clear();
            self.visible = false;
            self.selected = 0;
            return;
        }

        // Search for matching notes
        match db.search_notes(pattern, max_results * 2) {
            Ok(notes) => {
                let mut suggestions: Vec<LinkSuggestion> = notes
                    .iter()
                    .map(|note| LinkSuggestion::from_note(note, pattern))
                    .filter(|s| s.score > 0)
                    .collect();

                // Sort by score (descending)
                suggestions.sort_by(|a, b| b.score.cmp(&a.score));
                suggestions.truncate(max_results);

                self.suggestions = suggestions;
                self.visible = !self.suggestions.is_empty();
                self.selected = 0;
            }
            Err(_) => {
                self.suggestions.clear();
                self.visible = false;
                self.selected = 0;
            }
        }
    }

    /// Clear suggestions and hide
    pub fn clear(&mut self) {
        self.suggestions.clear();
        self.visible = false;
        self.selected = 0;
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected = (self.selected + 1) % self.suggestions.len();
        }
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected = if self.selected == 0 {
                self.suggestions.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    /// Get currently selected suggestion
    pub fn selected(&self) -> Option<&LinkSuggestion> {
        if self.visible && self.selected < self.suggestions.len() {
            self.suggestions.get(self.selected)
        } else {
            None
        }
    }

    /// Check if autocomplete is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Draw the autocomplete menu
    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme) {
        if !self.visible || area.height < 2 {
            return;
        }

        let mut lines = Vec::new();

        for (idx, suggestion) in self.suggestions.iter().enumerate() {
            let is_selected = idx == self.selected;
            let style = if is_selected {
                Style::default()
                    .fg(theme.bg())
                    .bg(theme.link())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.link())
            };

            // Display: "note-title (note-id) - score"
            let display = format!("{} ({})", suggestion.note_title, suggestion.note_id);
            lines.push(Line::from(Span::styled(display, style)));

            if lines.len() >= area.height.saturating_sub(1) as usize {
                break;
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("Link Suggestions")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.link())),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()));

        f.render_widget(paragraph, area);
    }
}

impl Default for LinkAutocomplete {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_autocomplete_new() {
        let ac = LinkAutocomplete::new();
        assert!(!ac.is_visible());
        assert_eq!(ac.selected, 0);
        assert!(ac.suggestions.is_empty());
    }

    #[test]
    fn test_link_autocomplete_clear() {
        let mut ac = LinkAutocomplete::new();
        ac.suggestions.push(LinkSuggestion {
            note_title: "Test".to_string(),
            note_id: "test-123".to_string(),
            score: 50,
        });
        ac.visible = true;

        ac.clear();
        assert!(!ac.is_visible());
        assert!(ac.suggestions.is_empty());
    }

    #[test]
    fn test_link_autocomplete_select_navigation() {
        let mut ac = LinkAutocomplete::new();
        ac.suggestions.push(LinkSuggestion {
            note_title: "Note1".to_string(),
            note_id: "note-1".to_string(),
            score: 50,
        });
        ac.suggestions.push(LinkSuggestion {
            note_title: "Note2".to_string(),
            note_id: "note-2".to_string(),
            score: 40,
        });
        ac.visible = true;

        assert_eq!(ac.selected, 0);
        ac.select_next();
        assert_eq!(ac.selected, 1);
        ac.select_next();
        assert_eq!(ac.selected, 0); // Wraps around
    }

    #[test]
    fn test_link_autocomplete_select_prev() {
        let mut ac = LinkAutocomplete::new();
        ac.suggestions.push(LinkSuggestion {
            note_title: "Note1".to_string(),
            note_id: "note-1".to_string(),
            score: 50,
        });
        ac.suggestions.push(LinkSuggestion {
            note_title: "Note2".to_string(),
            note_id: "note-2".to_string(),
            score: 40,
        });
        ac.visible = true;
        ac.selected = 1;

        ac.select_prev();
        assert_eq!(ac.selected, 0);
        ac.select_prev();
        assert_eq!(ac.selected, 1); // Wraps around
    }

    #[test]
    fn test_link_autocomplete_selected() {
        let mut ac = LinkAutocomplete::new();
        ac.suggestions.push(LinkSuggestion {
            note_title: "Note1".to_string(),
            note_id: "note-1".to_string(),
            score: 50,
        });
        ac.visible = true;

        let selected = ac.selected();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().note_id, "note-1");
    }

    #[test]
    fn test_link_autocomplete_not_visible() {
        let ac = LinkAutocomplete::new();
        assert!(ac.selected().is_none()); // Not visible, so no selection
    }
}
