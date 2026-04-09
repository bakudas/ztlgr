use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::db::Database;
use crate::error::{Result, ZtlgrError};
use crate::skills::generator::SkillsGenerator;
use crate::skills::Skills;
use crate::source::ingest::Ingester;
use crate::storage::{ActivityLog, FileImporter, FileSync, Format, IndexGenerator, Vault};
use crate::ui::App;

#[derive(Parser)]
#[command(version, author, about, long_about = None)]
pub struct Cli {
    /// Grimoire directory path
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
    /// Create a new grimoire
    New {
        /// Grimoire path
        path: PathBuf,

        /// Note format (markdown or org)
        #[arg(short, long, default_value = "markdown")]
        format: Option<String>,

        /// Skip git repository initialization
        #[arg(long)]
        no_git: bool,

        /// Skip .skills/ directory generation
        #[arg(long)]
        no_skills: bool,
    },

    /// Open an existing grimoire in the TUI
    Open {
        /// Grimoire path
        path: Option<PathBuf>,
    },

    /// Import notes from a directory
    Import {
        /// Source directory
        source: PathBuf,

        /// Grimoire path
        #[arg(long)]
        vault: Option<PathBuf>,

        /// Recursive import
        #[arg(short, long)]
        recursive: bool,
    },

    /// Sync grimoire files with database
    Sync {
        /// Grimoire path
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

        /// Grimoire path
        #[arg(long)]
        vault: Option<PathBuf>,

        /// Maximum results
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// Generate or update the grimoire index
    Index {
        /// Grimoire path
        #[arg(long)]
        vault: Option<PathBuf>,
    },

    /// Ingest a source file into the raw/ directory
    Ingest {
        /// Source file to ingest
        file: PathBuf,

        /// Title for the source (defaults to filename)
        #[arg(short, long)]
        title: Option<String>,

        /// Grimoire path
        #[arg(long)]
        vault: Option<PathBuf>,
    },

    /// Generate .skills/ directory for LLM agents
    InitSkills {
        /// Grimoire path
        #[arg(long)]
        vault: Option<PathBuf>,
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
            no_git,
            no_skills,
        }) => {
            cmd_new(
                path,
                cmd_format.as_deref().unwrap_or(&cli.format),
                *no_git,
                *no_skills,
            )?;
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
            cmd_search(&vault_path, query, *limit, format)?;
        }
        Some(Commands::Index { vault: cmd_vault }) => {
            let vault_path = resolve_vault_path(cmd_vault.as_ref(), cli.vault.as_ref())?;
            cmd_index(&vault_path, format)?;
        }
        Some(Commands::Ingest {
            file,
            title,
            vault: cmd_vault,
        }) => {
            let vault_path = resolve_vault_path(cmd_vault.as_ref(), cli.vault.as_ref())?;
            cmd_ingest(file, title.as_deref(), &vault_path)?;
        }
        Some(Commands::InitSkills { vault: cmd_vault }) => {
            let vault_path = resolve_vault_path(cmd_vault.as_ref(), cli.vault.as_ref())?;
            cmd_init_skills(&vault_path)?;
        }
        None => {
            run_default_tui(cli).await?;
        }
    }

    Ok(())
}

fn cmd_new(path: &Path, format_str: &str, no_git: bool, no_skills: bool) -> Result<()> {
    let format = parse_format(format_str);
    let vault = Vault::new(path.to_path_buf(), format);

    if vault.exists() {
        return Err(ZtlgrError::VaultExists(path.display().to_string()));
    }

    vault.initialize()?;

    // Generate .skills/ (unless --no-skills)
    if !no_skills {
        let vault_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("grimoire");
        let generator = SkillsGenerator::new(vault_name);
        match generator.generate(path) {
            Ok(result) => {
                println!("  .skills/ generated ({} files)", result.files_created);
            }
            Err(e) => {
                eprintln!("  Warning: .skills/ generation failed: {}", e);
            }
        }
    }

    // Git initialization (unless --no-git)
    if !no_git {
        match vault.git_init() {
            Ok(true) => println!("  Git repository initialized"),
            Ok(false) => {} // git not available, skip silently
            Err(e) => {
                eprintln!("  Warning: git init failed: {}", e);
            }
        }
    }

    println!("Grimoire created at {}", path.display());
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
    println!("  {}/raw/         - Source material", path.display());
    println!("  {}/.skills/     - LLM agent schema", path.display());
    println!();
    println!("Run 'ztlgr open {}' to open this grimoire", path.display());

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

    // Log the import activity
    let activity_log = ActivityLog::new(vault_path);
    let _ = activity_log.log_import(result.imported, result.failed);

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

    let activity_log = ActivityLog::new(vault_path);

    if force {
        let sync = FileSync::new(vault_path.to_path_buf(), format, db);
        let result = sync.full_sync()?;

        // Log the sync activity
        let _ = activity_log.log_sync(result.created_files, result.imported_notes, result.synced);

        // Regenerate index after sync
        let db2 = Database::new(&vault.database_path())?;
        let total_notes = db2.count_notes()?;
        let generator = IndexGenerator::new(&db2);
        generator.write_index(vault_path)?;
        let _ = activity_log.log_index(total_notes);

        println!("Full sync complete:");
        println!("  Files created: {}", result.created_files);
        println!("  Notes imported: {}", result.imported_notes);
        println!("  Notes synced: {}", result.synced);
        println!("  Index regenerated");
    } else {
        println!("Sync complete (use --force for full sync)");
    }

    Ok(())
}

fn cmd_search(vault_path: &Path, query: &str, limit: usize, format: Format) -> Result<()> {
    let vault = Vault::new(vault_path.to_path_buf(), format);

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

fn cmd_index(vault_path: &Path, format: Format) -> Result<()> {
    let vault = Vault::new(vault_path.to_path_buf(), format);

    if !vault.exists() {
        return Err(ZtlgrError::VaultNotFound(vault_path.display().to_string()));
    }

    let db_path = vault.database_path();
    let db = Database::new(&db_path)?;

    let total_notes = db.count_notes()?;
    let total_links = db.count_links()?;

    let generator = IndexGenerator::new(&db);
    generator.write_index(vault_path)?;

    // Log the activity
    let activity_log = ActivityLog::new(vault_path);
    activity_log.log_index(total_notes)?;

    println!("Index generated:");
    println!("  Notes indexed: {}", total_notes);
    println!("  Links: {}", total_links);
    println!(
        "  Written to: {}",
        vault_path.join(".ztlgr").join("index.md").display()
    );

    Ok(())
}

fn cmd_ingest(source_path: &Path, title: Option<&str>, vault_path: &Path) -> Result<()> {
    let vault = Vault::new(vault_path.to_path_buf(), Format::Markdown);

    if !vault.exists() {
        return Err(ZtlgrError::VaultNotFound(vault_path.display().to_string()));
    }

    let db_path = vault.database_path();
    let db = Database::new(&db_path)?;

    let ingester = Ingester::new(vault_path.to_path_buf(), db);
    let result = ingester.ingest_file(source_path, title)?;

    if result.is_new {
        println!("Ingested source:");
        println!("  Title: {}", result.source.title);
        println!("  File: {}", result.source.file_path);
        println!("  Hash: {}", &result.source.content_hash[..16]);
        if let Some(mime) = &result.source.mime_type {
            println!("  Type: {}", mime);
        }
        println!("  Size: {} bytes", result.source.file_size);
    } else {
        println!("Source already ingested (duplicate content):");
        println!("  Existing: {}", result.source.file_path);
        println!("  Hash: {}", &result.source.content_hash[..16]);
    }

    Ok(())
}

fn cmd_init_skills(vault_path: &Path) -> Result<()> {
    let vault = Vault::new(vault_path.to_path_buf(), Format::Markdown);

    if !vault.exists() {
        return Err(ZtlgrError::VaultNotFound(vault_path.display().to_string()));
    }

    let skills = Skills::new(vault_path);
    let vault_name = vault_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("grimoire");
    let generator = SkillsGenerator::new(vault_name);
    let result = generator.generate(vault_path)?;

    println!(".skills/ initialized:");
    println!("  Files created: {}", result.files_created);
    println!("  Files skipped: {}", result.files_skipped);

    let report = skills.validate();
    if report.is_complete() {
        println!("  Status: complete ({} files)", report.present.len());
    } else {
        println!(
            "  Status: {} present, {} missing",
            report.present.len(),
            report.missing.len()
        );
    }

    println!();
    println!("The .skills/ directory provides LLM agents with:");
    println!("  - Wiki conventions and formatting rules");
    println!("  - Workflows: ingest, query, lint, maintain");
    println!("  - Templates: source-summary, entity-page, comparison");
    println!("  - Context: domain description, research priorities");
    println!();
    println!("Customize these files to match your grimoire's domain.");

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
    let raw_path = cmd_path.or(global_path).cloned().ok_or_else(|| {
        ZtlgrError::VaultNotFound(
            "no grimoire path specified. Use --vault or provide path to command".to_string(),
        )
    })?;

    let path_str = raw_path.to_string_lossy();
    let expanded = shellexpand::full(&path_str)
        .map_err(|e| ZtlgrError::Config(format!("path expansion failed: {e}")))?;

    Ok(PathBuf::from(expanded.into_owned()))
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

        cmd_new(&vault_path, "markdown", true, true).unwrap();

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

        cmd_new(&vault_path, "org", true, true).unwrap();

        assert!(vault_path.exists());
        assert!(vault_path.join(".ztlgr").exists());
    }

    #[test]
    fn test_cmd_new_fails_on_existing_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("existing_vault");

        cmd_new(&vault_path, "markdown", true, true).unwrap();
        let result = cmd_new(&vault_path, "markdown", true, true);

        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_search_no_results() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("search_vault");

        cmd_new(&vault_path, "markdown", true, true).unwrap();

        let result = cmd_search(&vault_path, "nonexistent query", 10, Format::Markdown);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_sync_non_force() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("sync_vault");

        cmd_new(&vault_path, "markdown", true, true).unwrap();

        let result = cmd_sync(&vault_path, Format::Markdown, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_sync_force() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("sync_force_vault");

        cmd_new(&vault_path, "markdown", true, true).unwrap();

        let result = cmd_sync(&vault_path, Format::Markdown, true);
        assert!(result.is_ok());

        // Verify index.md was generated during force sync
        let index_path = vault_path.join(".ztlgr").join("index.md");
        assert!(index_path.exists());

        // Verify log.md was created with sync and index entries
        let log_path = vault_path.join(".ztlgr").join("log.md");
        assert!(log_path.exists());
        let log_content = std::fs::read_to_string(&log_path).unwrap();
        assert!(log_content.contains("sync |"));
        assert!(log_content.contains("index |"));
    }

    #[test]
    fn test_cmd_import_from_empty_source() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("import_vault");
        let source_path = temp_dir.path().join("source");

        cmd_new(&vault_path, "markdown", true, true).unwrap();
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

        let result = cmd_search(&vault_path, "query", 10, Format::Markdown);
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

    #[test]
    fn test_cmd_index_generates_index_file() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("index_vault");

        cmd_new(&vault_path, "markdown", true, true).unwrap();

        let result = cmd_index(&vault_path, Format::Markdown);
        assert!(result.is_ok());

        // Verify index.md was created
        let index_path = vault_path.join(".ztlgr").join("index.md");
        assert!(index_path.exists());

        let content = std::fs::read_to_string(&index_path).unwrap();
        assert!(content.contains("# Grimoire Index"));
    }

    #[test]
    fn test_cmd_index_nonexistent_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("nonexistent_index");

        let result = cmd_index(&vault_path, Format::Markdown);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_index_creates_activity_log() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("log_vault");

        cmd_new(&vault_path, "markdown", true, true).unwrap();
        cmd_index(&vault_path, Format::Markdown).unwrap();

        // Verify log.md was created
        let log_path = vault_path.join(".ztlgr").join("log.md");
        assert!(log_path.exists());

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("index | Index regenerated"));
    }

    #[test]
    fn test_cmd_new_with_git_init() {
        if !Vault::is_git_available() {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("git_vault");

        // no_git = false => git init should run
        cmd_new(&vault_path, "markdown", false, true).unwrap();

        assert!(vault_path.join(".git").exists());
    }

    #[test]
    fn test_cmd_new_with_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("nogit_vault");

        // no_git = true => no .git directory
        cmd_new(&vault_path, "markdown", true, true).unwrap();

        assert!(!vault_path.join(".git").exists());
    }

    // =====================================================================
    // Ingest command tests
    // =====================================================================

    #[test]
    fn test_cmd_ingest_success() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("ingest_vault");
        cmd_new(&vault_path, "markdown", true, true).unwrap();

        let source_file = temp_dir.path().join("article.md");
        std::fs::write(&source_file, "# Article\n\nContent here").unwrap();

        let result = cmd_ingest(&source_file, None, &vault_path);
        assert!(result.is_ok());

        // Verify file was copied to raw/
        assert!(vault_path.join("raw").exists());
        let raw_files: Vec<_> = std::fs::read_dir(vault_path.join("raw")).unwrap().collect();
        assert_eq!(raw_files.len(), 1);
    }

    #[test]
    fn test_cmd_ingest_with_title() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("ingest_title");
        cmd_new(&vault_path, "markdown", true, true).unwrap();

        let source_file = temp_dir.path().join("a.md");
        std::fs::write(&source_file, "content").unwrap();

        let result = cmd_ingest(&source_file, Some("Custom Title"), &vault_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_ingest_duplicate() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("ingest_dup");
        cmd_new(&vault_path, "markdown", true, true).unwrap();

        let file1 = temp_dir.path().join("first.md");
        std::fs::write(&file1, "same content").unwrap();

        let file2 = temp_dir.path().join("second.md");
        std::fs::write(&file2, "same content").unwrap();

        cmd_ingest(&file1, None, &vault_path).unwrap();
        let result = cmd_ingest(&file2, None, &vault_path);
        assert!(result.is_ok()); // Should succeed but report duplicate
    }

    #[test]
    fn test_cmd_ingest_nonexistent_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("nonexistent_ingest");
        let source_file = temp_dir.path().join("file.md");
        std::fs::write(&source_file, "content").unwrap();

        let result = cmd_ingest(&source_file, None, &vault_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_ingest_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("ingest_nofile");
        cmd_new(&vault_path, "markdown", true, true).unwrap();

        let result = cmd_ingest(Path::new("/nonexistent/file.md"), None, &vault_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_new_creates_raw_dir() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("raw_dir_vault");

        cmd_new(&vault_path, "markdown", true, true).unwrap();

        assert!(vault_path.join("raw").exists());
    }

    // =====================================================================
    // .skills/ integration tests
    // =====================================================================

    #[test]
    fn test_cmd_new_creates_skills_by_default() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("skills_vault");

        // no_skills = false => .skills/ should be created
        cmd_new(&vault_path, "markdown", true, false).unwrap();

        assert!(vault_path.join(".skills").exists());
        assert!(vault_path.join(".skills").join("README.md").exists());
        assert!(vault_path.join(".skills").join("conventions.md").exists());
        assert!(vault_path
            .join(".skills")
            .join("workflows")
            .join("ingest.md")
            .exists());
        assert!(vault_path
            .join(".skills")
            .join("templates")
            .join("source-summary.md")
            .exists());
        assert!(vault_path
            .join(".skills")
            .join("context")
            .join("domain.md")
            .exists());
    }

    #[test]
    fn test_cmd_new_skips_skills_with_flag() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("no_skills_vault");

        // no_skills = true => .skills/ should NOT be created
        cmd_new(&vault_path, "markdown", true, true).unwrap();

        assert!(!vault_path.join(".skills").exists());
    }

    #[test]
    fn test_cmd_new_skills_contain_vault_name() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("named-grimoire");

        cmd_new(&vault_path, "markdown", true, false).unwrap();

        let readme = std::fs::read_to_string(vault_path.join(".skills").join("README.md")).unwrap();
        assert!(readme.contains("named-grimoire"));
    }

    #[test]
    fn test_cmd_init_skills_success() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("init_skills_vault");

        // Create vault without skills
        cmd_new(&vault_path, "markdown", true, true).unwrap();
        assert!(!vault_path.join(".skills").exists());

        // Now run init-skills
        let result = cmd_init_skills(&vault_path);
        assert!(result.is_ok());
        assert!(vault_path.join(".skills").exists());
        assert!(vault_path.join(".skills").join("README.md").exists());
    }

    #[test]
    fn test_cmd_init_skills_nonexistent_vault() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("nonexistent_skills");

        let result = cmd_init_skills(&vault_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_init_skills_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("idempotent_skills");

        cmd_new(&vault_path, "markdown", true, false).unwrap();

        // Custom-modify one file
        std::fs::write(
            vault_path.join(".skills").join("README.md"),
            "# Custom README",
        )
        .unwrap();

        // Run init-skills again -- should not overwrite custom README
        cmd_init_skills(&vault_path).unwrap();

        let readme = std::fs::read_to_string(vault_path.join(".skills").join("README.md")).unwrap();
        assert_eq!(readme, "# Custom README");
    }

    #[test]
    fn test_cmd_init_skills_fills_missing_files() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("fill_skills");

        cmd_new(&vault_path, "markdown", true, false).unwrap();

        // Delete some files
        std::fs::remove_file(vault_path.join(".skills").join("workflows").join("lint.md")).unwrap();
        std::fs::remove_file(
            vault_path
                .join(".skills")
                .join("context")
                .join("priorities.md"),
        )
        .unwrap();

        // Run init-skills to fill them back
        cmd_init_skills(&vault_path).unwrap();

        assert!(vault_path
            .join(".skills")
            .join("workflows")
            .join("lint.md")
            .exists());
        assert!(vault_path
            .join(".skills")
            .join("context")
            .join("priorities.md")
            .exists());
    }
}
