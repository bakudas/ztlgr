use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::db::Database;
use crate::error::{Result, ZtlgrError};
use crate::storage::{FileImporter, FileSync, Format, Vault};
use crate::ui::App;

#[derive(Parser)]
#[command(version, author, about, long_about = None)]
pub struct Cli {
    /// Vault directory path
    #[arg(long, env = "ZTLGR_VAULT")]
    pub vault: Option<PathBuf>,

    /// Note format (markdown or org)
    #[arg(short = 'f', long, default_value = "markdown")]
    pub format: String,

    /// Configuration file path
    #[arg(short = 'c', long, env = "ZTLGR_CONFIG")]
    pub config: Option<PathBuf>,

    /// Verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new vault
    New {
        /// Vault path
        path: PathBuf,

        /// Note format (markdown or org)
        #[arg(short, long, default_value = "markdown")]
        format: Option<String>,
    },

    /// Open an existing vault in the TUI
    Open {
        /// Vault path
        path: Option<PathBuf>,
    },

    /// Import notes from a directory
    Import {
        /// Source directory
        source: PathBuf,

        /// Vault path
        #[arg(long)]
        vault: Option<PathBuf>,

        /// Recursive import
        #[arg(short, long)]
        recursive: bool,
    },

    /// Sync vault files with database
    Sync {
        /// Vault path
        #[arg(long)]
        vault: Option<PathBuf>,

        /// Force full sync
        #[arg(short, long)]
        force: bool,
    },

    /// Search notes
    Search {
        /// Search query
        query: String,

        /// Vault path
        #[arg(long)]
        vault: Option<PathBuf>,

        /// Maximum results
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
}

pub fn parse_args() -> Cli {
    Cli::parse()
}

pub async fn execute(cli: &Cli) -> Result<()> {
    let format = parse_format(&cli.format);

    match &cli.command {
        Some(Commands::New {
            path,
            format: cmd_format,
        }) => {
            cmd_new(path, cmd_format.as_deref().unwrap_or(&cli.format))?;
        }
        Some(Commands::Open { path }) => {
            let vault_path = resolve_vault_path(path.as_ref(), cli.vault.as_ref())?;
            cmd_open(&vault_path, format, cli.config.as_ref()).await?;
        }
        Some(Commands::Import {
            source,
            vault: cmd_vault,
            recursive,
        }) => {
            let vault_path = resolve_vault_path(cmd_vault.as_ref(), cli.vault.as_ref())?;
            cmd_import(source, &vault_path, format, *recursive)?;
        }
        Some(Commands::Sync {
            vault: cmd_vault,
            force,
        }) => {
            let vault_path = resolve_vault_path(cmd_vault.as_ref(), cli.vault.as_ref())?;
            cmd_sync(&vault_path, format, *force)?;
        }
        Some(Commands::Search {
            query,
            vault: cmd_vault,
            limit,
        }) => {
            let vault_path = resolve_vault_path(cmd_vault.as_ref(), cli.vault.as_ref())?;
            cmd_search(&vault_path, query, *limit)?;
        }
        None => {
            run_default_tui(cli).await?;
        }
    }

    Ok(())
}

fn cmd_new(path: &Path, format_str: &str) -> Result<()> {
    let format = parse_format(format_str);
    let vault = Vault::new(path.to_path_buf(), format);

    if vault.exists() {
        return Err(ZtlgrError::VaultExists(path.display().to_string()));
    }

    vault.initialize()?;

    println!("Vault created at {}", path.display());
    println!("  Format: {}", format.extension());
    println!();
    println!("Structure:");
    println!(
        "  {}/permanent/   - Permanent knowledge notes",
        path.display()
    );
    println!("  {}/inbox/       - Fleeting notes", path.display());
    println!("  {}/literature/  - Notes from sources", path.display());
    println!("  {}/reference/   - External references", path.display());
    println!("  {}/index/       - Structure notes (MOCs)", path.display());
    println!("  {}/daily/       - Daily journal", path.display());
    println!("  {}/attachments/ - Images and files", path.display());
    println!();
    println!("Run 'ztlgr open {}' to open this vault", path.display());

    Ok(())
}

async fn cmd_open(vault_path: &Path, format: Format, config_path: Option<&PathBuf>) -> Result<()> {
    let vault = Vault::new(vault_path.to_path_buf(), format);

    if !vault.exists() {
        return Err(ZtlgrError::VaultNotFound(vault_path.display().to_string()));
    }

    let db_path = vault.database_path();
    let db = Database::new(&db_path)?;

    let config = if let Some(cfg_path) = config_path {
        Config::load(cfg_path).map_err(|e| ZtlgrError::Config(e.to_string()))?
    } else {
        let vault_config_path = vault.config_path();
        if vault_config_path.exists() {
            Config::load(&vault_config_path).map_err(|e| ZtlgrError::Config(e.to_string()))?
        } else {
            let mut cfg = Config::default();
            cfg.vault.path = Some(vault_path.to_path_buf());
            cfg
        }
    };

    let mut app = App::new(config, db)?;
    app.run().await?;

    Ok(())
}

fn cmd_import(source: &Path, vault_path: &Path, format: Format, _recursive: bool) -> Result<()> {
    let vault = Vault::new(vault_path.to_path_buf(), format);

    if !vault.exists() {
        return Err(ZtlgrError::VaultNotFound(vault_path.display().to_string()));
    }

    let db_path = vault.database_path();
    let db = Database::new(&db_path)?;

    let importer = FileImporter::new(source.to_path_buf(), format, db);
    let result = importer.import_all()?;

    println!("Import complete:");
    println!("  Imported: {}", result.imported);
    println!("  Failed: {}", result.failed);

    if !result.errors.is_empty() {
        println!();
        println!("Errors:");
        for error in &result.errors {
            println!("  - {}", error);
        }
    }

    Ok(())
}

fn cmd_sync(vault_path: &Path, format: Format, force: bool) -> Result<()> {
    let vault = Vault::new(vault_path.to_path_buf(), format);

    if !vault.exists() {
        return Err(ZtlgrError::VaultNotFound(vault_path.display().to_string()));
    }

    let db_path = vault.database_path();
    let db = Database::new(&db_path)?;

    let sync = FileSync::new(vault_path.to_path_buf(), format, db);

    if force {
        let result = sync.full_sync()?;
        println!("Full sync complete:");
        println!("  Files created: {}", result.created_files);
        println!("  Notes imported: {}", result.imported_notes);
        println!("  Notes synced: {}", result.synced);
    } else {
        println!("Sync complete (use --force for full sync)");
    }

    Ok(())
}

fn cmd_search(vault_path: &Path, query: &str, limit: usize) -> Result<()> {
    let vault = Vault::new(vault_path.to_path_buf(), Format::Markdown);

    if !vault.exists() {
        return Err(ZtlgrError::VaultNotFound(vault_path.display().to_string()));
    }

    let db_path = vault.database_path();
    let db = Database::new(&db_path)?;

    let results = db.search_notes(query, limit)?;

    if results.is_empty() {
        println!("No notes found for '{}'", query);
    } else {
        println!("Found {} note(s) for '{}':\n", results.len(), query);
        for (i, note) in results.iter().enumerate() {
            println!("{}. {} [{}]", i + 1, note.title, note.note_type.as_str());
            let preview = note
                .content
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(80)
                .collect::<String>();
            if !preview.is_empty() {
                println!("   {}", preview);
            }
            println!();
        }
    }

    Ok(())
}

async fn run_default_tui(cli: &Cli) -> Result<()> {
    if let Some(vault_path) = &cli.vault {
        let format = parse_format(&cli.format);
        cmd_open(vault_path, format, cli.config.as_ref()).await
    } else {
        let wizard = crate::setup::SetupWizard::new();
        let config = wizard.run()?;

        let vault_path: PathBuf =
            config
                .vault_path()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| {
                    directories::ProjectDirs::from("com", "ztlgr", "ztlgr")
                        .map(|dirs| dirs.data_dir().join("vault"))
                        .unwrap_or_else(|| PathBuf::from("./vault"))
                });

        let db_path = vault_path.join(".ztlgr").join("vault.db");

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = Database::new(&db_path)?;
        let mut app = App::new(config, db)?;
        app.run().await?;

        Ok(())
    }
}

fn resolve_vault_path(
    cmd_path: Option<&PathBuf>,
    global_path: Option<&PathBuf>,
) -> Result<PathBuf> {
    cmd_path.or(global_path).cloned().ok_or_else(|| {
        ZtlgrError::VaultNotFound(
            "no vault path specified. Use --vault or provide path to command".to_string(),
        )
    })
}

fn parse_format(s: &str) -> Format {
    match s.to_lowercase().as_str() {
        "org" | "orgmode" => Format::Org,
        _ => Format::Markdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_format_markdown() {
        assert_eq!(parse_format("markdown"), Format::Markdown);
        assert_eq!(parse_format("Markdown"), Format::Markdown);
        assert_eq!(parse_format("md"), Format::Markdown);
    }

    #[test]
    fn test_parse_format_org() {
        assert_eq!(parse_format("org"), Format::Org);
        assert_eq!(parse_format("orgmode"), Format::Org);
        assert_eq!(parse_format("Org"), Format::Org);
    }

    #[test]
    fn test_resolve_vault_path_from_cmd() {
        let cmd = Some(PathBuf::from("/cmd/vault"));
        let global = Some(PathBuf::from("/global/vault"));
        let result = resolve_vault_path(cmd.as_ref(), global.as_ref()).unwrap();
        assert_eq!(result, PathBuf::from("/cmd/vault"));
    }

    #[test]
    fn test_resolve_vault_path_from_global() {
        let cmd: Option<PathBuf> = None;
        let global = Some(PathBuf::from("/global/vault"));
        let result = resolve_vault_path(cmd.as_ref(), global.as_ref()).unwrap();
        assert_eq!(result, PathBuf::from("/global/vault"));
    }

    #[test]
    fn test_resolve_vault_path_none() {
        let cmd: Option<PathBuf> = None;
        let global: Option<PathBuf> = None;
        let result = resolve_vault_path(cmd.as_ref(), global.as_ref());
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_new_creates_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("my_vault");

        cmd_new(&vault_path, "markdown").unwrap();

        assert!(vault_path.exists());
        assert!(vault_path.join(".ztlgr").exists());
        assert!(vault_path.join("permanent").exists());
        assert!(vault_path.join("inbox").exists());
        assert!(vault_path.join("literature").exists());
        assert!(vault_path.join("reference").exists());
        assert!(vault_path.join("index").exists());
        assert!(vault_path.join("daily").exists());
        assert!(vault_path.join("attachments").exists());
        assert!(vault_path.join(".gitignore").exists());
        assert!(vault_path.join("README.md").exists());
    }

    #[test]
    fn test_cmd_new_creates_org_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("org_vault");

        cmd_new(&vault_path, "org").unwrap();

        assert!(vault_path.exists());
        assert!(vault_path.join(".ztlgr").exists());
    }

    #[test]
    fn test_cmd_new_fails_on_existing_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("existing_vault");

        cmd_new(&vault_path, "markdown").unwrap();
        let result = cmd_new(&vault_path, "markdown");

        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_search_no_results() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("search_vault");

        cmd_new(&vault_path, "markdown").unwrap();

        let result = cmd_search(&vault_path, "nonexistent query", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_sync_non_force() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("sync_vault");

        cmd_new(&vault_path, "markdown").unwrap();

        let result = cmd_sync(&vault_path, Format::Markdown, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_sync_force() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("sync_force_vault");

        cmd_new(&vault_path, "markdown").unwrap();

        let result = cmd_sync(&vault_path, Format::Markdown, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_import_from_empty_source() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("import_vault");
        let source_path = temp_dir.path().join("source");

        cmd_new(&vault_path, "markdown").unwrap();
        std::fs::create_dir_all(&source_path).unwrap();

        let result = cmd_import(&source_path, &vault_path, Format::Markdown, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_open_nonexistent_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("nonexistent");

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(cmd_open(&vault_path, Format::Markdown, None));

        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_search_nonexistent_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("nonexistent_search");

        let result = cmd_search(&vault_path, "query", 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_sync_nonexistent_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("nonexistent_sync");

        let result = cmd_sync(&vault_path, Format::Markdown, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_import_nonexistent_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("nonexistent_import");
        let source_path = temp_dir.path().join("source");

        std::fs::create_dir_all(&source_path).unwrap();

        let result = cmd_import(&source_path, &vault_path, Format::Markdown, true);
        assert!(result.is_err());
    }
}
