use crate::config::Config;
use crate::error::ZtlgrError;
use crate::storage::{Format, Vault};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

pub struct SetupWizard {
    config_path: PathBuf,
}

impl SetupWizard {
    pub fn new() -> Self {
        let config_path = directories::ProjectDirs::from("com", "ztlgr", "ztlgr")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("./config.toml"));

        Self { config_path }
    }

    pub fn run(&self) -> Result<Config, ZtlgrError> {
        println!();
        println!("╔══════════════════════════════════════════════════════╗");
        println!("║            Welcome to ztlgr!                          ║");
        println!("║      Terminal Zettelkasten Note Taking               ║");
        println!("╚══════════════════════════════════════════════════════╝");
        println!();

        // Check if config exists
        if self.config_path.exists() {
            println!("Configuration found at {:?}", self.config_path);
            let config = Config::load(&self.config_path)
                .map_err(|e| ZtlgrError::Config(format!("Failed to load config: {}", e)))?;
            println!("Vault: {:?}", config.vault_path());
            return Ok(config);
        }

        println!("Let's set up your first vault.");
        println!();

        // Get vault path
        let vault_path = self.prompt_vault_path()?;

        // Get format
        let format = self.prompt_format()?;

        // Create vault
        let vault = Vault::new(vault_path.clone(), format);

        if vault.exists() {
            println!("Vault found at {:?}", vault_path);
            let import = self.prompt_import_existing()?;

            if import {
                println!("Importing existing notes...");
                // Import logic will be handled by FileImporter
            }
        } else {
            println!("Creating new vault at {:?}", vault_path);
            vault
                .initialize()
                .map_err(|e| ZtlgrError::Config(format!("Failed to initialize vault: {}", e)))?;

            println!("✓ Created directory structure");
            println!("✓ Created .gitignore");
            println!("✓ Created README.md");
        }

        // Create config
        let mut config = Config::default();
        config.vault.name = vault_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("vault")
            .to_string();

        config.vault.path = Some(vault_path.clone());

        config.ui.theme = self.choose_theme()?;

        // Save config
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ZtlgrError::Config(format!("Failed to create config directory: {}", e))
            })?;
        }

        config
            .save(&self.config_path)
            .map_err(|e| ZtlgrError::Config(format!("Failed to save config: {}", e)))?;

        println!();
        println!("✓ Configuration saved to {:?}", self.config_path);

        self.show_quickstart();

        Ok(config)
    }

    fn prompt_vault_path(&self) -> Result<PathBuf, ZtlgrError> {
        println!("Where would you like to store your notes?");
        println!();

        let default_path = directories::ProjectDirs::from("com", "ztlgr", "ztlgr")
            .map(|dirs| dirs.data_dir().join("vault"))
            .unwrap_or_else(|| PathBuf::from("./vault"));

        println!("Default: {:?}", default_path);
        println!();
        println!("Examples:");
        println!("  ~/notes              - Home directory");
        println!("  ~/Documents/zettel   - Documents");
        println!("  .                    - Current directory");
        println!("  /custom/path/vault   - Custom path");
        println!();

        print!("Enter vault path (press Enter for default): ");
        io::stdout()
            .flush()
            .map_err(|e| ZtlgrError::Config(format!("IO error: {}", e)))?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin
            .lock()
            .read_line(&mut input)
            .map_err(|e| ZtlgrError::Config(format!("IO error: {}", e)))?;

        let input = input.trim();

        if input.is_empty() {
            Ok(default_path)
        } else {
            let path = shellexpand::tilde(&input).into_owned();
            Ok(PathBuf::from(path))
        }
    }

    fn prompt_format(&self) -> Result<Format, ZtlgrError> {
        println!();
        println!("Choose your note format:");
        println!();
        println!("  1) Markdown (.md) - Compatible with Obsidian, Foam, etc.");
        println!("  2) Org Mode (.org) - Emacs org-mode format");
        println!();

        print!("Enter choice (1-2) [default: 1]: ");
        io::stdout()
            .flush()
            .map_err(|e| ZtlgrError::Config(format!("IO error: {}", e)))?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin
            .lock()
            .read_line(&mut input)
            .map_err(|e| ZtlgrError::Config(format!("IO error: {}", e)))?;

        let input = input.trim();

        match input {
            "2" | "org" => Ok(Format::Org),
            _ => Ok(Format::Markdown),
        }
    }

    fn prompt_import_existing(&self) -> Result<bool, ZtlgrError> {
        println!();
        println!("Would you like to import existing notes from this vault?");
        print!("(y/N): ");
        io::stdout()
            .flush()
            .map_err(|e| ZtlgrError::Config(format!("IO error: {}", e)))?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin
            .lock()
            .read_line(&mut input)
            .map_err(|e| ZtlgrError::Config(format!("IO error: {}", e)))?;

        Ok(input.to_lowercase().starts_with('y'))
    }

    fn choose_theme(&self) -> Result<String, ZtlgrError> {
        println!();
        println!("Choose a theme:");
        println!();
        println!("  1) Dracula     - Purple and cyan (default)");
        println!("  2) Gruvbox     - Warm, retro colors");
        println!("  3) Nord        - Arctic, bluish tones");
        println!("  4) Solarized   - Precision color scheme");
        println!();

        print!("Enter choice (1-4) [default: 1]: ");
        io::stdout()
            .flush()
            .map_err(|e| ZtlgrError::Config(format!("IO error: {}", e)))?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin
            .lock()
            .read_line(&mut input)
            .map_err(|e| ZtlgrError::Config(format!("IO error: {}", e)))?;

        let input = input.trim();

        Ok(match input {
            "2" => "gruvbox".to_string(),
            "3" => "nord".to_string(),
            "4" => "solarized".to_string(),
            _ => "dracula".to_string(),
        })
    }

    fn show_quickstart(&self) {
        println!();
        println!("╔══════════════════════════════════════════════════════╗");
        println!("║                    Quick Start                       ║");
        println!("╚══════════════════════════════════════════════════════╝");
        println!();
        println!("Your vault is ready!");
        println!();
        println!("Key commands:");
        println!("  i     - Enter insert mode to edit notes");
        println!("  n     - Create a new note");
        println!("  /     - Search notes");
        println!("  v     - View knowledge graph");
        println!("  q     - Quit");
        println!();
        println!("Note types:");
        println!("  (.permanent/)  Permanent notes - Zettelkasten cards");
        println!("  (.inbox/)     Fleeting notes - Quick capture");
        println!("  (.literature/) Literature notes - From books/articles");
        println!("  (.reference/)  Reference notes - External sources");
        println!("  (.index/)      Index notes - Maps of content");
        println!("  (.daily/)      Daily notes - Journal entries");
        println!();
        println!("Links work like: [[note-title]]");
        println!("Tags work like: #tag");
        println!();
        println!("Run 'ztlgr' to open your vault!");
        println!();
    }
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self::new()
    }
}
