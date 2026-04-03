use crate::db::Database;
use crate::note::NoteId;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Widget to display backlinks (incoming links to current note)
pub struct BacklinksPane {
    backlinks: Vec<BacklinkItem>,
    scroll: usize,
}

#[derive(Debug, Clone)]
pub struct BacklinkItem {
    pub source_id: String,
    pub source_title: String,
    pub context: Option<String>,
}

impl BacklinksPane {
    pub fn new() -> Self {
        Self {
            backlinks: Vec::new(),
            scroll: 0,
        }
    }

    /// Load backlinks for a specific note
    pub fn load_backlinks(
        &mut self,
        note_id: &NoteId,
        db: &Database,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Query the backlinks view from the database
        // SELECT source_id, context FROM backlinks WHERE note_id = ?

        // For now, we'll leave this as a placeholder
        // In the real implementation, this would query the database
        self.backlinks.clear();
        Ok(())
    }

    /// Clear backlinks
    pub fn clear(&mut self) {
        self.backlinks.clear();
        self.scroll = 0;
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        let max_scroll = self.backlinks.len().saturating_sub(1);
        if self.scroll < max_scroll {
            self.scroll += 1;
        }
    }

    /// Draw the backlinks pane
    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme) {
        if self.backlinks.is_empty() {
            let no_backlinks = vec![Line::from(Span::styled(
                "No backlinks found",
                Style::default().fg(theme.fg_dim()),
            ))];

            let paragraph = Paragraph::new(no_backlinks)
                .block(
                    Block::default()
                        .title("Backlinks")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border())),
                )
                .style(Style::default().fg(theme.fg()).bg(theme.bg()));

            f.render_widget(paragraph, area);
            return;
        }

        let mut lines = Vec::new();

        for (idx, backlink) in self
            .backlinks
            .iter()
            .skip(self.scroll)
            .take(area.height.saturating_sub(2) as usize)
            .enumerate()
        {
            // Title of the note linking to us
            lines.push(Line::from(Span::styled(
                format!("󰌷 {}", backlink.source_title),
                Style::default()
                    .fg(theme.link())
                    .add_modifier(Modifier::BOLD),
            )));

            // Context/snippet if available
            if let Some(context) = &backlink.context {
                let truncated = if context.len() > 60 {
                    format!("{}...", &context[..60])
                } else {
                    context.clone()
                };
                lines.push(Line::from(Span::styled(
                    format!("  {}", truncated),
                    Style::default().fg(theme.fg_dim()),
                )));
            }

            lines.push(Line::from("")); // Spacing
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!("Backlinks ({})", self.backlinks.len()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border())),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

impl Default for BacklinksPane {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backlinks_pane_new() {
        let pane = BacklinksPane::new();
        assert!(pane.backlinks.is_empty());
        assert_eq!(pane.scroll, 0);
    }

    #[test]
    fn test_backlinks_pane_clear() {
        let mut pane = BacklinksPane::new();
        pane.backlinks.push(BacklinkItem {
            source_id: "source-1".to_string(),
            source_title: "Source Note".to_string(),
            context: Some("This is context".to_string()),
        });
        pane.scroll = 1;

        pane.clear();
        assert!(pane.backlinks.is_empty());
        assert_eq!(pane.scroll, 0);
    }

    #[test]
    fn test_backlinks_pane_scroll_up() {
        let mut pane = BacklinksPane::new();
        for i in 1..=5 {
            pane.backlinks.push(BacklinkItem {
                source_id: format!("source-{}", i),
                source_title: format!("Source {}", i),
                context: None,
            });
        }
        pane.scroll = 3;

        pane.scroll_up();
        assert_eq!(pane.scroll, 2);

        pane.scroll_up();
        assert_eq!(pane.scroll, 1);
    }

    #[test]
    fn test_backlinks_pane_scroll_down() {
        let mut pane = BacklinksPane::new();
        for i in 1..=5 {
            pane.backlinks.push(BacklinkItem {
                source_id: format!("source-{}", i),
                source_title: format!("Source {}", i),
                context: None,
            });
        }
        pane.scroll = 0;

        pane.scroll_down();
        assert_eq!(pane.scroll, 1);

        pane.scroll_down();
        assert_eq!(pane.scroll, 2);
    }

    #[test]
    fn test_backlinks_pane_scroll_bounds() {
        let mut pane = BacklinksPane::new();
        for i in 1..=3 {
            pane.backlinks.push(BacklinkItem {
                source_id: format!("source-{}", i),
                source_title: format!("Source {}", i),
                context: None,
            });
        }

        // Scroll up at top should do nothing
        pane.scroll_up();
        assert_eq!(pane.scroll, 0);

        // Scroll down to max
        pane.scroll_down();
        pane.scroll_down();
        assert_eq!(pane.scroll, 2);

        // Try to scroll down beyond max should not go further
        pane.scroll_down();
        assert_eq!(pane.scroll, 2);
    }

    #[test]
    fn test_backlink_item_creation() {
        let item = BacklinkItem {
            source_id: "note-123".to_string(),
            source_title: "My Note".to_string(),
            context: Some("relevant context".to_string()),
        };

        assert_eq!(item.source_id, "note-123");
        assert_eq!(item.source_title, "My Note");
        assert!(item.context.is_some());
    }
}
