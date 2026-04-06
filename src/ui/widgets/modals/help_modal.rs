use std::cell::Cell;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
    Frame,
};

/// Help modal showing all keybindings, commands, credits and version info
#[derive(Debug)]
pub struct HelpModal {
    scroll: u16,
    max_scroll: Cell<u16>,
}

impl HelpModal {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            max_scroll: Cell::new(0),
        }
    }

    fn build_content(theme: &dyn crate::config::Theme) -> Text<'static> {
        let version = env!("CARGO_PKG_VERSION");
        let mut lines: Vec<Line<'static>> = Vec::new();

        let accent = theme.accent();
        let heading_style = Style::default().fg(accent).add_modifier(Modifier::BOLD);
        let key_style = Style::default()
            .fg(theme.link())
            .add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(theme.fg_dim());
        let fg_style = Style::default().fg(theme.fg());

        // Title
        lines.push(Line::from(Span::styled("ztlgr", heading_style)));
        lines.push(Line::from(Span::styled(
            format!("Zettelkasten TUI v{}", version),
            dim_style,
        )));
        lines.push(Line::from(""));

        // Editor - Normal Mode
        lines.push(Line::from(Span::styled(
            "Editor — Normal Mode",
            heading_style,
        )));
        lines.push(Line::from(vec![
            Span::styled("h/j/k/l  ", key_style),
            Span::styled("or ", dim_style),
            Span::styled("←↓↑→    ", key_style),
            Span::styled("  Move cursor", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("w/b      ", key_style),
            Span::styled("  Word forward/back", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("0/$      ", key_style),
            Span::styled("  Line start/end", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("g/G      ", key_style),
            Span::styled("  Document top/bottom", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("i        ", key_style),
            Span::styled("  Enter insert mode", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("a/A      ", key_style),
            Span::styled("  Append (at cursor / end of line)", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("o/O      ", key_style),
            Span::styled("  Open line below/above", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("x/X      ", key_style),
            Span::styled("  Delete next/prev char", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("d        ", key_style),
            Span::styled("  Delete line (dd)", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("D        ", key_style),
            Span::styled("  Delete to end of line", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("u        ", key_style),
            Span::styled("  Undo", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Ctrl+r   ", key_style),
            Span::styled("  Redo", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("y        ", key_style),
            Span::styled("  Yank (copy)", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("p        ", key_style),
            Span::styled("  Paste", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("H/L      ", key_style),
            Span::styled("  Switch panels", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Esc      ", key_style),
            Span::styled("  Focus note list", fg_style),
        ]));
        lines.push(Line::from(""));

        // Editor - Insert Mode
        lines.push(Line::from(Span::styled(
            "Editor — Insert Mode",
            heading_style,
        )));
        lines.push(Line::from(vec![
            Span::styled("Esc      ", key_style),
            Span::styled("  Exit insert mode (auto-save)", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Ctrl+s   ", key_style),
            Span::styled("  Save note", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Ctrl+z/y ", key_style),
            Span::styled("  Undo/Redo", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Ctrl+c/v/x", key_style),
            Span::styled("  Copy/Paste/Cut", fg_style),
        ]));
        lines.push(Line::from(""));

        // Global
        lines.push(Line::from(Span::styled("Global", heading_style)));
        lines.push(Line::from(vec![
            Span::styled("j/k      ", key_style),
            Span::styled("  Next/prev note", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("h/l      ", key_style),
            Span::styled("  Prev/next panel", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("g/G      ", key_style),
            Span::styled("  Top/bottom of list", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("n        ", key_style),
            Span::styled("  New note", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("d        ", key_style),
            Span::styled("  Delete note", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("/        ", key_style),
            Span::styled("  Search", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled(":        ", key_style),
            Span::styled("  Command mode", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("p        ", key_style),
            Span::styled("  Toggle preview", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("m        ", key_style),
            Span::styled("  Toggle metadata", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("?        ", key_style),
            Span::styled("  This help screen", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled(":q       ", key_style),
            Span::styled("  Quit", fg_style),
        ]));
        lines.push(Line::from(""));

        // CLI Commands
        lines.push(Line::from(Span::styled("CLI Commands", heading_style)));
        lines.push(Line::from(vec![
            Span::styled("ztlgr new <path>       ", key_style),
            Span::styled("  Create vault", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ztlgr open [path]      ", key_style),
            Span::styled("  Open vault in TUI", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ztlgr search <query>   ", key_style),
            Span::styled("  Full-text search", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ztlgr import <source>  ", key_style),
            Span::styled("  Import notes", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ztlgr sync             ", key_style),
            Span::styled("  Sync vault with DB", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ztlgr --help           ", key_style),
            Span::styled("  Show CLI help", fg_style),
        ]));
        lines.push(Line::from(""));

        // Credits
        lines.push(Line::from(Span::styled("Credits", heading_style)));
        lines.push(Line::from(vec![
            Span::styled("Author:  ", fg_style),
            Span::styled(
                "@bakudas",
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("License: ", fg_style),
            Span::styled("MIT OR Apache-2.0", fg_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Repo:    ", fg_style),
            Span::styled(
                "github.com/bakudas/ztlgr",
                Style::default().fg(theme.link()),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Inspired by: ", fg_style),
            Span::styled("Obsidian, zk, Helix, vit", dim_style),
        ]));

        Text::from(lines)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => false, // signal to close
            KeyCode::Down | KeyCode::Char('j') => {
                if self.scroll < self.max_scroll.get() {
                    self.scroll += 1;
                }
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
                true
            }
            _ => true,
        }
    }

    pub fn draw(&self, f: &mut Frame, theme: &dyn crate::config::Theme) {
        let area = f.size();

        // Fixed dimensions for readability
        let modal_width = 70.min(area.width.saturating_sub(4));
        let modal_height = 40.min(area.height.saturating_sub(4));

        let modal_x = area.width.saturating_sub(modal_width) / 2;
        let modal_y = area.height.saturating_sub(modal_height) / 2;

        let modal_area = Rect {
            x: modal_x,
            y: modal_y,
            width: modal_width,
            height: modal_height,
        };

        Clear.render(modal_area, f.buffer_mut());

        let block = Block::default()
            .title(" Help (? to close) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(theme.fg()).bg(theme.bg()));

        let inner = block.inner(modal_area);
        f.render_widget(block, modal_area);

        let content = Self::build_content(theme);
        let total_lines = content.height() as u16;
        let inner_height = inner.height.saturating_sub(1); // 1 line for hint
        self.max_scroll
            .set(total_lines.saturating_sub(inner_height));

        let paragraph = Paragraph::new(content)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        let content_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height.saturating_sub(1),
        };

        f.render_widget(paragraph, content_area);

        // Bottom hint
        let hint_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };

        let hint = Paragraph::new(" ↑↓/jk scroll  Esc/?/q close ")
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.fg_dim()));

        f.render_widget(hint, hint_area);
    }
}

impl Default for HelpModal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::themes::DraculaTheme;

    #[test]
    fn test_help_modal_creation() {
        let modal = HelpModal::new();
        assert_eq!(modal.scroll, 0);
    }

    #[test]
    fn test_help_modal_handle_close() {
        let mut modal = HelpModal::new();
        // Esc should return false (signal to close)
        let esc_key = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::NONE);
        assert!(!modal.handle_key(esc_key));

        let q_key = KeyEvent::new(KeyCode::Char('q'), crossterm::event::KeyModifiers::NONE);
        assert!(!modal.handle_key(q_key));
    }

    #[test]
    fn test_help_modal_scroll_down() {
        let mut modal = HelpModal::new();
        modal.max_scroll.set(10);
        let down_key = KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE);
        assert!(modal.handle_key(down_key));
        assert_eq!(modal.scroll, 1);
    }

    #[test]
    fn test_help_modal_scroll_up() {
        let mut modal = HelpModal::new();
        modal.scroll = 5;
        modal.max_scroll.set(10);
        let up_key = KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE);
        assert!(modal.handle_key(up_key));
        assert_eq!(modal.scroll, 4);
    }

    #[test]
    fn test_help_modal_scroll_bounds() {
        let mut modal = HelpModal::new();
        modal.max_scroll.set(2);

        // Scroll up at 0 should stay at 0
        let up_key = KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE);
        modal.handle_key(up_key);
        assert_eq!(modal.scroll, 0);

        // Scroll down past max should stop at max
        let down_key = KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE);
        modal.handle_key(down_key);
        modal.handle_key(down_key);
        modal.handle_key(down_key); // should stay at 2
        assert_eq!(modal.scroll, 2);
    }

    #[test]
    fn test_build_content_has_sections() {
        let theme = DraculaTheme::default();
        let content = HelpModal::build_content(&theme);
        let text: String = content
            .lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(text.contains("ztlgr"));
        assert!(text.contains("Normal Mode"));
        assert!(text.contains("Insert Mode"));
        assert!(text.contains("Global"));
        assert!(text.contains("CLI Commands"));
        assert!(text.contains("Credits"));
        assert!(text.contains("@bakudas"));
        assert!(text.contains("MIT OR Apache-2.0"));
    }
}
