/// Surge TUI - Main entry
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use surge_tui::{App, Config, SurgeClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging (only warnings and errors unless RUST_LOG is set)
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "surge_tui=warn".to_string()))
        .init();

    // Load config
    let config = Config::load(None)?;

    // Validate API Key
    if config.surge.http_api_key.is_empty() {
        eprintln!("Error: HTTP API Key not configured");
        eprintln!("\nPlease set SURGE_HTTP_API_KEY environment variable or create config file");
        eprintln!("\nExample config file:\n");
        eprintln!("{}", Config::example());
        std::process::exit(1);
    }

    // Create Surge client
    let client = SurgeClient::new(config.clone());

    // Create app
    let mut app = App::new(client, config.ui.refresh_interval);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = app.run(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Return result
    result
}
