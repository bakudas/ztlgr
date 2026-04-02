use super::Theme;
use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct SolarizedTheme;

impl Default for SolarizedTheme {
    fn default() -> Self {
        Self
    }
}

impl Theme for SolarizedTheme {
    fn name(&self) -> &str {
        "solarized"
    }

    fn bg(&self) -> Color {
        Color::Rgb(0, 43, 54) // #002b36 (base03)
    }

    fn bg_secondary(&self) -> Color {
        Color::Rgb(7, 54, 66) // #073642 (base02)
    }

    fn bg_highlight(&self) -> Color {
        Color::Rgb(42, 92, 104) // #2a5c68
    }

    fn fg(&self) -> Color {
        Color::Rgb(253, 246, 227) // #fdf6e3 (base3)
    }

    fn fg_secondary(&self) -> Color {
        Color::Rgb(147, 161, 161) // #93a1a1 (base1)
    }

    fn fg_dim(&self) -> Color {
        Color::Rgb(131, 148, 150) // #839496 (base0)
    }

    fn accent(&self) -> Color {
        Color::Rgb(38, 139, 210) // #268bd2 (blue)
    }

    fn accent_secondary(&self) -> Color {
        Color::Rgb(108, 113, 196) // #6c71c4 (violet)
    }

    fn success(&self) -> Color {
        Color::Rgb(133, 153, 0) // #859900 (green)
    }

    fn warning(&self) -> Color {
        Color::Rgb(181, 137, 0) // #b58900 (yellow)
    }

    fn error(&self) -> Color {
        Color::Rgb(220, 50, 47) // #dc322f (red)
    }

    fn info(&self) -> Color {
        Color::Rgb(42, 161, 152) // #2aa198 (cyan)
    }

    fn note_daily(&self) -> Color {
        Color::Rgb(181, 137, 0) // #b58900 (yellow)
    }

    fn note_fleeting(&self) -> Color {
        Color::Rgb(211, 54, 130) // #d33682 (magenta)
    }

    fn note_literature(&self) -> Color {
        Color::Rgb(203, 75, 22) // #cb4b16 (orange)
    }

    fn note_permanent(&self) -> Color {
        Color::Rgb(38, 139, 210) // #268bd2 (blue)
    }

    fn note_reference(&self) -> Color {
        Color::Rgb(42, 161, 152) // #2aa198 (cyan)
    }

    fn note_index(&self) -> Color {
        Color::Rgb(133, 153, 0) // #859900 (green)
    }

    fn link(&self) -> Color {
        Color::Rgb(38, 139, 210) // #268bd2 (blue)
    }

    fn tag(&self) -> Color {
        Color::Rgb(211, 54, 130) // #d33682 (magenta)
    }

    fn border(&self) -> Color {
        Color::Rgb(7, 54, 66) // #073642 (base02)
    }

    fn border_highlight(&self) -> Color {
        Color::Rgb(38, 139, 210) // #268bd2 (blue)
    }
}
