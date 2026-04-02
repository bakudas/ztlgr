use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(version = "0.1.0", author = "ztlgr team")]
#[clap(about = "Terminal-based note-taking with Zettelkasten methodology", long_about = None)]
struct Cli {
    /// Vault directory path
    #[clap(long, env = "ZTLGR_VAULT")]
    vault: Option<PathBuf>,

    /// Note format (markdown or org)
    #[clap(short = 'f', long, default_value = "markdown")]
    format: String,

    /// Configuration file path
    #[clap(short = 'c', long, env = "ZTLGR_CONFIG")]
    config: Option<PathBuf>,

    /// Verbosity level
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new vault
    New {
        /// Vault path
        path: PathBuf,

        /// Note format
        #[clap(short, long, default_value = "markdown")]
        format: String,
    },

    /// Open an existing vault
    Open {
        /// Vault path
        path: PathBuf,
    },

    /// Import notes from directory
    Import {
        /// Source directory
        source: PathBuf,

        /// Recursive import
        #[clap(short, long)]
        recursive: bool,
    },

    /// Sync vault with database
    Sync {
        /// Force full sync
        #[clap(short, long)]
        force: bool,
    },

    /// Search notes
    Search {
        /// Search query
        query: String,

        /// Maximum results
        #[clap(short, long, default_value = "50")]
        limit: usize,
    },

    /// Create a new note
    Note {
        /// Note title
        title: String,

        /// Note type
        #[clap(short, long, default_value = "permanent")]
        r#type: String,
    },

    /// Export notes to another format
    Export {
        /// Export format
        #[clap(short, long)]
        format: String,

        /// Output directory
        #[clap(short, long)]
        output: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(log_level))
        .init();

    match cli.command {
        Some(Commands::New { path, format }) => {
            new_vault(&path, &format)?;
        }
        Some(Commands::Open { path }) => {
            open_vault(&path)?;
        }
        Some(Commands::Import { source, recursive }) => {
            import_notes(&source, recursive)?;
        }
        Some(Commands::Sync { force }) => {
            sync_vault(force)?;
        }
        Some(Commands::Search { query, limit }) => {
            search_notes(&query, limit)?;
        }
        Some(Commands::Note { title, r#type }) => {
            create_note(&title, &r#type)?;
        }
        Some(Commands::Export { format, output }) => {
            export_notes(&format, &output)?;
        }
        None => {
            // Default: run interactive TUI
            run_tui()?;
        }
    }

    Ok(())
}

fn new_vault(path: &PathBuf, format: &str) -> anyhow::Result<()> {
    use ztlgr::storage::{Format, Vault};

    let format = match format.to_lowercase().as_str() {
        "org" | "orgmode" => Format::Org,
        _ => Format::Markdown,
    };

    let vault = Vault::new(path.clone(), format);

    if vault.exists() {
        anyhow::bail!("Vault already exists at {:?}", path);
    }

    vault.initialize()?;

    println!("✓ Created vault at {:?}", path);
    println!("  Format: {}", vault.format.extension());
    println!();
    println!("Structure:");
    println!("  {:?}/permanent/   - Permanent knowledge notes", path);
    println!("  {:?}/inbox/       - Fleeting notes", path);
    println!("  {:?}/literature/  - Notes from sources", path);
    println!("  {:?}/reference/   - External references", path);
    println!("  {:?}/index/       - Structure notes (MOCs)", path);
    println!("  {:?}/daily/       - Daily journal", path);
    println!("  {:?}/attachments/ - Images and files", path);
    println!();
    println!("Run 'ztlgr {:?}' to open this vault", path);

    Ok(())
}

fn open_vault(path: &PathBuf) -> anyhow::Result<()> {
    println!("Opening vault at {:?}...", path);
    // TUI will be launched from main.rs
    Ok(())
}

fn import_notes(source: &PathBuf, recursive: bool) -> anyhow::Result<()> {
    println!(
        "Importing notes from {:?} (recursive: {})...",
        source, recursive
    );
    // Import logic
    Ok(())
}

fn sync_vault(force: bool) -> anyhow::Result<()> {
    println!("Syncing vault (force: {})...", force);
    // Sync logic
    Ok(())
}

fn search_notes(query: &str, limit: usize) -> anyhow::Result<()> {
    println!("Searching for '{}' (limit: {})...", query, limit);
    // Search logic
    Ok(())
}

fn create_note(title: &str, r#type: &str) -> anyhow::Result<()> {
    println!("Creating note '{}' with type '{}'...", title, r#type);
    // Create note logic
    Ok(())
}

fn export_notes(format: &str, output: &PathBuf) -> anyhow::Result<()> {
    println!("Exporting to {} at {:?}...", format, output);
    // Export logic
    Ok(())
}

fn run_tui() -> anyhow::Result<()> {
    // This is handled by the main async runtime in src/main.rs
    // This function is just a placeholder for the CLI binary
    println!("Use 'cargo run' to run the interactive TUI");
    Ok(())
}
