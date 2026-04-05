use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use std::io::stdout;
use std::panic;
use ztlgr::cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_dir = directories::ProjectDirs::from("com", "ztlgr", "ztlgr")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::never(&log_dir, "ztlgr.log");

    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .init();

    let default_panic = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        default_panic(panic_info);
    }));

    tracing::info!("Starting ztlgr v{}", env!("CARGO_PKG_VERSION"));

    let cli = cli::parse_args();
    cli::execute(&cli).await?;

    tracing::info!("ztlgr finished successfully");
    Ok(())
}
