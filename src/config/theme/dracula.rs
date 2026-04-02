use super::Theme;
use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct DraculaTheme;

impl Default for DraculaTheme {
    fn default() -> Self {
        Self
    }
}

impl Theme for DraculaTheme {
    fn name(&self) -> &str {
        "dracula"
    }

    // Background colors
    fn bg(&self) -> Color {
        Color::Rgb(40, 42, 54) // #282a36
    }

    fn bg_secondary(&self) -> Color {
        Color::Rgb(44, 46, 59) // #2c2e3b
    }

    fn bg_highlight(&self) -> Color {
        Color::Rgb(68, 71, 90) // #44475a
    }

    // Foreground colors
    fn fg(&self) -> Color {
        Color::Rgb(248, 248, 242) // #f8f8f2
    }

    fn fg_secondary(&self) -> Color {
        Color::Rgb(98, 114, 164) // #6272a4
    }

    fn fg_dim(&self) -> Color {
        Color::Rgb(98, 114, 164) // #6272a4
    }

    // Accent colors
    fn accent(&self) -> Color {
        Color::Rgb(189, 147, 249) // #bd93f9 (purple)
    }

    fn accent_secondary(&self) -> Color {
        Color::Rgb(255, 121, 198) // #ff79c6 (pink)
    }

    // Semantic colors
    fn success(&self) -> Color {
        Color::Rgb(80, 250, 123) // #50fa7b (green)
    }

    fn warning(&self) -> Color {
        Color::Rgb(255, 184, 108) // #ffb86c (orange)
    }

    fn error(&self) -> Color {
        Color::Rgb(255, 85, 85) // #ff5555 (red)
    }

    fn info(&self) -> Color {
        Color::Rgb(139, 233, 253) // #8be9fd (cyan)
    }

    // Note type colors
    fn note_daily(&self) -> Color {
        Color::Rgb(255, 184, 108) // #ffb86c (orange)
    }

    fn note_fleeting(&self) -> Color {
        Color::Rgb(255, 121, 198) // #ff79c6 (pink)
    }

    fn note_literature(&self) -> Color {
        Color::Rgb(241, 250, 140) // #f1fa8c (yellow)
    }

    fn note_permanent(&self) -> Color {
        Color::Rgb(189, 147, 249) // #bd93f9 (purple)
    }

    fn note_reference(&self) -> Color {
        Color::Rgb(139, 233, 253) // #8be9fd (cyan)
    }

    fn note_index(&self) -> Color {
        Color::Rgb(80, 250, 123) // #50fa7b (green)
    }

    // Link colors
    fn link(&self) -> Color {
        Color::Rgb(139, 233, 253) // #8be9fd (cyan)
    }

    fn tag(&self) -> Color {
        Color::Rgb(255, 121, 198) // #ff79c6 (pink)
    }

    // UI colors
    fn border(&self) -> Color {
        Color::Rgb(68, 71, 90) // #44475a
    }

    fn border_highlight(&self) -> Color {
        Color::Rgb(189, 147, 249) // #bd93f9 (purple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dracula_theme_colors() {
        let theme = DraculaTheme::default();
        assert_eq!(theme.name(), "dracula");

        // Verify colors are correct
        let bg = theme.bg();
        assert!(matches!(bg, Color::Rgb(40, 42, 54)));
    }
}
