// TODO: Custom themes feature - not yet integrated into settings.rs
// Integration pending: add theme loading from TOML files
// Tracked in: Theme System Enhancement
#![allow(dead_code)]

use super::Theme;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTheme {
    pub name: String,

    // Background colors
    pub bg: ColorDef,
    pub bg_secondary: ColorDef,
    pub bg_highlight: ColorDef,

    // Foreground colors
    pub fg: ColorDef,
    pub fg_secondary: ColorDef,
    pub fg_dim: ColorDef,

    // Accent colors
    pub accent: ColorDef,
    pub accent_secondary: ColorDef,

    // Semantic colors
    pub success: ColorDef,
    pub warning: ColorDef,
    pub error: ColorDef,
    pub info: ColorDef,

    // Note type colors
    pub note_daily: ColorDef,
    pub note_fleeting: ColorDef,
    pub note_literature: ColorDef,
    pub note_permanent: ColorDef,
    pub note_reference: ColorDef,
    pub note_index: ColorDef,

    // Link colors
    pub link: ColorDef,
    pub tag: ColorDef,

    // UI colors
    pub border: ColorDef,
    pub border_highlight: ColorDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDef {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<ColorDef> for Color {
    fn from(def: ColorDef) -> Self {
        Color::Rgb(def.r, def.g, def.b)
    }
}

impl From<Color> for ColorDef {
    fn from(color: Color) -> Self {
        match color {
            Color::Rgb(r, g, b) => ColorDef { r, g, b },
            _ => ColorDef { r: 0, g: 0, b: 0 },
        }
    }
}

impl Default for CustomTheme {
    fn default() -> Self {
        Self {
            name: "custom".to_string(),
            bg: ColorDef {
                r: 40,
                g: 42,
                b: 54,
            },
            bg_secondary: ColorDef {
                r: 44,
                g: 46,
                b: 59,
            },
            bg_highlight: ColorDef {
                r: 68,
                g: 71,
                b: 90,
            },
            fg: ColorDef {
                r: 248,
                g: 248,
                b: 242,
            },
            fg_secondary: ColorDef {
                r: 98,
                g: 114,
                b: 164,
            },
            fg_dim: ColorDef {
                r: 98,
                g: 114,
                b: 164,
            },
            accent: ColorDef {
                r: 189,
                g: 147,
                b: 249,
            },
            accent_secondary: ColorDef {
                r: 255,
                g: 121,
                b: 198,
            },
            success: ColorDef {
                r: 80,
                g: 250,
                b: 123,
            },
            warning: ColorDef {
                r: 255,
                g: 184,
                b: 108,
            },
            error: ColorDef {
                r: 255,
                g: 85,
                b: 85,
            },
            info: ColorDef {
                r: 139,
                g: 233,
                b: 253,
            },
            note_daily: ColorDef {
                r: 255,
                g: 184,
                b: 108,
            },
            note_fleeting: ColorDef {
                r: 255,
                g: 121,
                b: 198,
            },
            note_literature: ColorDef {
                r: 241,
                g: 250,
                b: 140,
            },
            note_permanent: ColorDef {
                r: 189,
                g: 147,
                b: 249,
            },
            note_reference: ColorDef {
                r: 139,
                g: 233,
                b: 253,
            },
            note_index: ColorDef {
                r: 80,
                g: 250,
                b: 123,
            },
            link: ColorDef {
                r: 139,
                g: 233,
                b: 253,
            },
            tag: ColorDef {
                r: 255,
                g: 121,
                b: 198,
            },
            border: ColorDef {
                r: 68,
                g: 71,
                b: 90,
            },
            border_highlight: ColorDef {
                r: 189,
                g: 147,
                b: 249,
            },
        }
    }
}

impl Theme for CustomTheme {
    fn name(&self) -> &str {
        &self.name
    }

    fn bg(&self) -> Color {
        self.bg.clone().into()
    }
    fn bg_secondary(&self) -> Color {
        self.bg_secondary.clone().into()
    }
    fn bg_highlight(&self) -> Color {
        self.bg_highlight.clone().into()
    }
    fn fg(&self) -> Color {
        self.fg.clone().into()
    }
    fn fg_secondary(&self) -> Color {
        self.fg_secondary.clone().into()
    }
    fn fg_dim(&self) -> Color {
        self.fg_dim.clone().into()
    }
    fn accent(&self) -> Color {
        self.accent.clone().into()
    }
    fn accent_secondary(&self) -> Color {
        self.accent_secondary.clone().into()
    }
    fn success(&self) -> Color {
        self.success.clone().into()
    }
    fn warning(&self) -> Color {
        self.warning.clone().into()
    }
    fn error(&self) -> Color {
        self.error.clone().into()
    }
    fn info(&self) -> Color {
        self.info.clone().into()
    }
    fn note_daily(&self) -> Color {
        self.note_daily.clone().into()
    }
    fn note_fleeting(&self) -> Color {
        self.note_fleeting.clone().into()
    }
    fn note_literature(&self) -> Color {
        self.note_literature.clone().into()
    }
    fn note_permanent(&self) -> Color {
        self.note_permanent.clone().into()
    }
    fn note_reference(&self) -> Color {
        self.note_reference.clone().into()
    }
    fn note_index(&self) -> Color {
        self.note_index.clone().into()
    }
    fn link(&self) -> Color {
        self.link.clone().into()
    }
    fn tag(&self) -> Color {
        self.tag.clone().into()
    }
    fn border(&self) -> Color {
        self.border.clone().into()
    }
    fn border_highlight(&self) -> Color {
        self.border_highlight.clone().into()
    }
}

impl CustomTheme {
    pub fn load(path: &std::path::Path) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::error::ZtlgrError::Config(format!("Failed to read theme: {}", e))
        })?;

        toml::from_str(&content)
            .map_err(|e| crate::error::ZtlgrError::Config(format!("Failed to parse theme: {}", e)))
    }

    pub fn save(&self, path: &std::path::Path) -> crate::error::Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::error::ZtlgrError::Config(format!("Failed to serialize theme: {}", e))
        })?;

        std::fs::write(path, content).map_err(|e| {
            crate::error::ZtlgrError::Config(format!("Failed to write theme: {}", e))
        })?;

        Ok(())
    }
}
