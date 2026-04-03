use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use std::io::stdout;
use std::panic;
use std::path::PathBuf;
use tracing_appender;
use ztlgr::db::Database;
use ztlgr::setup::SetupWizard;
use ztlgr::ui::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get log file path
    let log_dir = directories::ProjectDirs::from("com", "ztlgr", "ztlgr")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    std::fs::create_dir_all(&log_dir).ok();

    // Initialize logging to file only (not to stderr/stdout which interferes with TUI)
    let file_appender = tracing_appender::rolling::never(&log_dir, "ztlgr.log");

    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .init();

    // Setup panic hook to restore terminal state on crash
    let default_panic = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        default_panic(panic_info);
    }));

    tracing::info!("Starting ztlgr v{}", env!("CARGO_PKG_VERSION"));

    // Setup wizard for first-time users
    let wizard = SetupWizard::new();
    let config = wizard.run()?;

    // Initialize database
    let vault_path: PathBuf = config
        .vault_path()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            directories::ProjectDirs::from("com", "ztlgr", "ztlgr")
                .map(|dirs| dirs.data_dir().join("vault"))
                .unwrap_or_else(|| PathBuf::from("./vault"))
        });

    let db_path = vault_path.join(".ztlgr").join("vault.db");
    tracing::info!("Database: {:?}", db_path);

    // Create .ztlgr directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db = Database::new(&db_path)?;
    tracing::info!("Database initialized");

    // Initialize TUI
    let mut app = App::new(config, db)?;

    // Run application
    app.run().await?;

    tracing::info!("ztlgr finished successfully");
    Ok(())
}
