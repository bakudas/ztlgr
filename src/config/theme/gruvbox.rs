use super::Theme;
use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct GruvboxTheme;

impl Default for GruvboxTheme {
    fn default() -> Self {
        Self
    }
}

impl Theme for GruvboxTheme {
    fn name(&self) -> &str {
        "gruvbox"
    }

    fn bg(&self) -> Color {
        Color::Rgb(40, 40, 40) // #282828
    }

    fn bg_secondary(&self) -> Color {
        Color::Rgb(60, 56, 54) // #3c3836
    }

    fn bg_highlight(&self) -> Color {
        Color::Rgb(80, 73, 69) // #504945
    }

    fn fg(&self) -> Color {
        Color::Rgb(235, 219, 178) // #ebdbb2
    }

    fn fg_secondary(&self) -> Color {
        Color::Rgb(146, 131, 116) // #928374
    }

    fn fg_dim(&self) -> Color {
        Color::Rgb(146, 131, 116) // #928374
    }

    fn accent(&self) -> Color {
        Color::Rgb(254, 128, 25) // #fe8019 (orange)
    }

    fn accent_secondary(&self) -> Color {
        Color::Rgb(250, 189, 34) // #fabd2f (yellow)
    }

    fn success(&self) -> Color {
        Color::Rgb(142, 192, 124) // #8ec07c (green)
    }

    fn warning(&self) -> Color {
        Color::Rgb(254, 128, 25) // #fe8019 (orange)
    }

    fn error(&self) -> Color {
        Color::Rgb(251, 73, 52) // #fb4934 (red)
    }

    fn info(&self) -> Color {
        Color::Rgb(131, 165, 152) // #83a598 (blue)
    }

    fn note_daily(&self) -> Color {
        Color::Rgb(254, 128, 25) // #fe8019 (orange)
    }

    fn note_fleeting(&self) -> Color {
        Color::Rgb(211, 134, 155) // #d3869b (purple)
    }

    fn note_literature(&self) -> Color {
        Color::Rgb(250, 189, 34) // #fabd2f (yellow)
    }

    fn note_permanent(&self) -> Color {
        Color::Rgb(131, 165, 152) // #83a598 (blue)
    }

    fn note_reference(&self) -> Color {
        Color::Rgb(142, 192, 124) // #8ec07c (green)
    }

    fn note_index(&self) -> Color {
        Color::Rgb(214, 93, 14) // #d65d0e (bright orange)
    }

    fn link(&self) -> Color {
        Color::Rgb(131, 165, 152) // #83a598 (blue)
    }

    fn tag(&self) -> Color {
        Color::Rgb(211, 134, 155) // #d3869b (purple)
    }

    fn border(&self) -> Color {
        Color::Rgb(60, 56, 54) // #3c3836
    }

    fn border_highlight(&self) -> Color {
        Color::Rgb(254, 128, 25) // #fe8019 (orange)
    }
}
