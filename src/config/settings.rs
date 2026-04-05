use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::theme::Theme;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Path to vault database file
    pub vault: VaultConfig,

    /// UI settings
    pub ui: UiConfig,

    /// Editor settings
    pub editor: EditorConfig,

    /// Note settings
    pub notes: NotesConfig,

    /// Search settings
    pub search: SearchConfig,

    /// Graph settings
    pub graph: GraphConfig,

    /// Zettelkasten settings
    pub zettelkasten: ZettelkastenConfig,

    #[serde(skip)]
    config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Path to vault database file
    pub path: Option<PathBuf>,

    /// Name of the vault
    pub name: String,

    /// Auto-backup interval in seconds (0 = disabled)
    pub auto_backup_interval: u64,

    /// Maximum backup count
    pub max_backups: usize,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            path: None,
            name: "default".to_string(),
            auto_backup_interval: 3600, // 1 hour
            max_backups: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme name
    pub theme: String,

    /// Sidebar width in percent
    pub sidebar_width: u16,

    /// Show preview panel
    pub show_preview: bool,

    /// Show line numbers
    pub show_line_numbers: bool,

    /// Show backlinks panel
    pub show_backlinks: bool,

    /// Show tags panel
    pub show_tags: bool,

    /// Animation frames per second (0 = disabled)
    pub fps: u16,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dracula".to_string(),
            sidebar_width: 25,
            show_preview: true,
            show_line_numbers: true,
            show_backlinks: true,
            show_tags: true,
            fps: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Keybindings style: "vim", "emacs", "default"
    pub keybindings: String,

    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u64,

    /// Tab width
    pub tab_width: usize,

    /// Use soft tabs (spaces)
    pub soft_tabs: bool,

    /// Word wrap
    pub word_wrap: bool,

    /// Show whitespace characters
    pub show_whitespace: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            keybindings: "vim".to_string(),
            auto_save_interval: 30,
            tab_width: 4,
            soft_tabs: true,
            word_wrap: true,
            show_whitespace: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesConfig {
    /// Default note type for new notes
    pub default_type: String,

    /// Auto-generate Zettel IDs
    pub auto_zettel_id: bool,

    /// Default parent for new notes
    pub default_parent: Option<String>,

    /// Note templates directory
    pub templates_dir: Option<PathBuf>,
}

impl Default for NotesConfig {
    fn default() -> Self {
        Self {
            default_type: "permanent".to_string(),
            auto_zettel_id: true,
            default_parent: None,
            templates_dir: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Use fuzzy search
    pub fuzzy: bool,

    /// Case sensitive search
    pub case_sensitive: bool,

    /// Maximum results to show
    pub max_results: usize,

    /// Search in content (not just titles)
    pub search_content: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            fuzzy: true,
            case_sensitive: false,
            max_results: 50,
            search_content: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Show labels on nodes
    pub show_labels: bool,

    /// Maximum nodes to display
    pub max_nodes: usize,

    /// Graph layout: "force-directed", "circular", "tree"
    pub layout: String,

    /// Graph depth to show
    pub depth: usize,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            show_labels: true,
            max_nodes: 100,
            layout: "force-directed".to_string(),
            depth: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZettelkastenConfig {
    /// ID style: "luhmann", "timestamp", "custom"
    pub id_style: String,

    /// Create daily notes automatically
    pub create_daily_notes: bool,

    /// Daily note time (HH:MM)
    pub daily_note_time: String,
}

impl Default for ZettelkastenConfig {
    fn default() -> Self {
        Self {
            id_style: "luhmann".to_string(),
            create_daily_notes: true,
            daily_note_time: "00:00".to_string(),
        }
    }
}

impl Config {
    pub fn load_or_create() -> Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            tracing::info!("Loading configuration from {:?}", config_path);
            Self::load(&config_path)
        } else {
            tracing::info!("Creating default configuration at {:?}", config_path);
            let config = Self::default();
            config.save(&config_path)?;
            Ok(config)
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))?;

        let mut config: Config = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))?;

        config.config_path = Some(path.to_path_buf());

        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("Failed to create config dir: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;

        fs::write(path, content).map_err(|e| anyhow::anyhow!("Failed to write config: {}", e))?;

        Ok(())
    }

    pub fn config_path() -> PathBuf {
        ProjectDirs::from("com", "ztlgr", "ztlgr")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("./config.toml"))
    }

    pub fn vault_path(&self) -> Option<&Path> {
        self.vault.path.as_deref()
    }

    pub fn config_path_ref(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    pub fn get_theme(&self) -> Box<dyn Theme> {
        use crate::config::themes::*;
        match self.ui.theme.as_str() {
            "dracula" => Box::new(DraculaTheme),
            "gruvbox" => Box::new(GruvboxTheme),
            "nord" => Box::new(NordTheme),
            "solarized" => Box::new(SolarizedTheme),
            name => {
                tracing::warn!("Unknown theme '{}', using dracula", name);
                Box::new(DraculaTheme)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.ui.theme, "dracula");
        assert_eq!(config.editor.keybindings, "vim");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml).unwrap();
        assert_eq!(config.ui.theme, deserialized.ui.theme);
    }
}
