use super::Theme;
use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct NordTheme;

impl Default for NordTheme {
    fn default() -> Self {
        Self
    }
}

impl Theme for NordTheme {
    fn name(&self) -> &str {
        "nord"
    }

    fn bg(&self) -> Color {
        Color::Rgb(46, 52, 64) // #2e3440
    }

    fn bg_secondary(&self) -> Color {
        Color::Rgb(59, 66, 82) // #3b4252
    }

    fn bg_highlight(&self) -> Color {
        Color::Rgb(67, 76, 94) // #434c5e
    }

    fn fg(&self) -> Color {
        Color::Rgb(236, 239, 244) // #eceff4
    }

    fn fg_secondary(&self) -> Color {
        Color::Rgb(216, 222, 233) // #d8dee9
    }

    fn fg_dim(&self) -> Color {
        Color::Rgb(143, 188, 187) // #8fbcbb
    }

    fn accent(&self) -> Color {
        Color::Rgb(136, 192, 208) // #88c0d0 (frost)
    }

    fn accent_secondary(&self) -> Color {
        Color::Rgb(129, 161, 193) // #81a1c1 (frost)
    }

    fn success(&self) -> Color {
        Color::Rgb(163, 190, 140) // #a3be8c (aurora)
    }

    fn warning(&self) -> Color {
        Color::Rgb(235, 203, 139) // #ebcb8b (aurora)
    }

    fn error(&self) -> Color {
        Color::Rgb(191, 97, 106) // #bf616a (aurora)
    }

    fn info(&self) -> Color {
        Color::Rgb(136, 192, 208) // #88c0d0 (frost)
    }

    fn note_daily(&self) -> Color {
        Color::Rgb(235, 203, 139) // #ebcb8b (aurora)
    }

    fn note_fleeting(&self) -> Color {
        Color::Rgb(180, 142, 173) // #b48ead (aurora)
    }

    fn note_literature(&self) -> Color {
        Color::Rgb(208, 135, 112) // #d08770 (aurora)
    }

    fn note_permanent(&self) -> Color {
        Color::Rgb(136, 192, 208) // #88c0d0 (frost)
    }

    fn note_reference(&self) -> Color {
        Color::Rgb(143, 188, 187) // #8fbcbb (frost)
    }

    fn note_index(&self) -> Color {
        Color::Rgb(163, 190, 140) // #a3be8c (aurora)
    }

    fn link(&self) -> Color {
        Color::Rgb(136, 192, 208) // #88c0d0 (frost)
    }

    fn tag(&self) -> Color {
        Color::Rgb(180, 142, 173) // #b48ead (aurora)
    }

    fn border(&self) -> Color {
        Color::Rgb(59, 66, 82) // #3b4252
    }

    fn border_highlight(&self) -> Color {
        Color::Rgb(136, 192, 208) // #88c0d0 (frost)
    }
}
