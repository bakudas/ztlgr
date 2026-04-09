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
        println!("║    Local-first personal knowledge base               ║");
        println!("╚══════════════════════════════════════════════════════╝");
        println!();

        // Check if config exists
        if self.config_path.exists() {
            println!("Configuration found at {:?}", self.config_path);
            let config = Config::load(&self.config_path)
                .map_err(|e| ZtlgrError::Config(format!("Failed to load config: {}", e)))?;
            println!("Grimoire: {:?}", config.vault_path());
            return Ok(config);
        }

        println!("Let's set up your first grimoire.");
        println!();

        // Get grimoire path
        let vault_path = self.prompt_vault_path()?;

        // Get format
        let format = self.prompt_format()?;

        // Create grimoire
        let vault = Vault::new(vault_path.clone(), format);

        if vault.exists() {
            println!("Grimoire found at {:?}", vault_path);
            let import = self.prompt_import_existing()?;

            if import {
                println!("Importing existing notes...");
                // Import logic will be handled by FileImporter
            }
        } else {
            println!("Creating new grimoire at {:?}", vault_path);
            vault
                .initialize()
                .map_err(|e| ZtlgrError::Config(format!("Failed to initialize grimoire: {}", e)))?;

            println!("  Created directory structure");
            println!("  Created .gitignore");
            println!("  Created README.md");

            // Prompt for git initialization
            let init_git = self.prompt_git_init()?;
            if init_git {
                match vault.git_init() {
                    Ok(true) => println!("  Git repository initialized"),
                    Ok(false) => println!("  git not found, skipping (install git to enable)"),
                    Err(e) => {
                        eprintln!("  Warning: git init failed: {}", e);
                    }
                }
            }
        }

        // Create config
        let mut config = Config::default();
        config.vault.name = vault_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("grimoire")
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
        println!("Where would you like to store your grimoire?");
        println!();

        let default_path = directories::ProjectDirs::from("com", "ztlgr", "ztlgr")
            .map(|dirs| dirs.data_dir().join("grimoire"))
            .unwrap_or_else(|| PathBuf::from("./grimoire"));

        println!("Default: {:?}", default_path);
        println!();
        println!("Examples:");
        println!("  ~/notes              - Home directory");
        println!("  ~/Documents/zettel   - Documents");
        println!("  .                    - Current directory");
        println!("  /custom/path         - Custom path");
        println!();

        print!("Enter grimoire path (press Enter for default): ");
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
        println!("Would you like to import existing notes from this grimoire?");
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

    fn prompt_git_init(&self) -> Result<bool, ZtlgrError> {
        println!();
        println!("Initialize git repository for version history?");
        print!("(Y/n): ");
        io::stdout()
            .flush()
            .map_err(|e| ZtlgrError::Config(format!("IO error: {}", e)))?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin
            .lock()
            .read_line(&mut input)
            .map_err(|e| ZtlgrError::Config(format!("IO error: {}", e)))?;

        let input = input.trim().to_lowercase();
        // Default is yes (Y/n) -- empty or 'y' means yes
        Ok(input.is_empty() || input.starts_with('y'))
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
        println!("Your grimoire is ready!");
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
        println!("Run 'ztlgr' to open your grimoire!");
        println!();
    }
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self::new()
    }
}
