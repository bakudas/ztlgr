use ratatui::style::{Color, Style};

pub trait Theme: Send + Sync {
    fn name(&self) -> &str;

    // Background colors
    fn bg(&self) -> Color;
    fn bg_secondary(&self) -> Color;
    fn bg_highlight(&self) -> Color;

    // Foreground colors
    fn fg(&self) -> Color;
    fn fg_secondary(&self) -> Color;
    fn fg_dim(&self) -> Color;

    // Accent colors
    fn accent(&self) -> Color;
    fn accent_secondary(&self) -> Color;

    // Semantic colors
    fn success(&self) -> Color;
    fn warning(&self) -> Color;
    fn error(&self) -> Color;
    fn info(&self) -> Color;

    // Note type colors
    fn note_daily(&self) -> Color;
    fn note_fleeting(&self) -> Color;
    fn note_literature(&self) -> Color;
    fn note_permanent(&self) -> Color;
    fn note_reference(&self) -> Color;
    fn note_index(&self) -> Color;

    // Link colors
    fn link(&self) -> Color;
    fn tag(&self) -> Color;

    // UI colors
    fn border(&self) -> Color;
    fn border_highlight(&self) -> Color;

    // Note color helper
    fn note_color(&self, note_type: &crate::note::NoteType) -> Color {
        match note_type {
            crate::note::NoteType::Daily => self.note_daily(),
            crate::note::NoteType::Fleeting => self.note_fleeting(),
            crate::note::NoteType::Literature { .. } => self.note_literature(),
            crate::note::NoteType::Permanent => self.note_permanent(),
            crate::note::NoteType::Reference { .. } => self.note_reference(),
            crate::note::NoteType::Index => self.note_index(),
        }
    }

    // Styles
    fn title_style(&self) -> Style {
        Style::default().fg(self.accent()).bg(self.bg())
    }

    fn selected_style(&self) -> Style {
        Style::default().fg(self.fg()).bg(self.bg_highlight())
    }

    fn error_style(&self) -> Style {
        Style::default().fg(self.error()).bg(self.bg())
    }
}

pub mod custom;
pub mod dracula;
pub mod gruvbox;
pub mod nord;
pub mod solarized;
