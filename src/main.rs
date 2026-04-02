use ztlgr::setup::SetupWizard;
use ztlgr::db::Database;
use ztlgr::ui::App;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    tracing::info!("Starting ztlgr v{}", env!("CARGO_PKG_VERSION"));
    
    // Setup wizard for first-time users
    let wizard = SetupWizard::new();
    let config = wizard.run()?;
    
    // Initialize database
    let vault_path: PathBuf = config.vault_path()
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