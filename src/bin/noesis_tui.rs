//! Noesis TUI — terminal interface for the decentralized cognitive architecture.
//!
//! Connects to the Noesis REST API at NOESIS_API_URL (default http://127.0.0.1:8647)
//! and renders real-time views of the cognitive state.

use anyhow::Result;
use noesis::tui::app::TuiApp;
use noesis::tui::ui;

#[tokio::main]
async fn main() -> Result<()> {
    let api_url = std::env::var("NOESIS_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8647".to_string());
    
    let mut terminal = ratatui::init();
    let app_result = TuiApp::new(&api_url).await;
    
    match app_result {
        Ok(mut app) => {
            let result = ui::run(&mut terminal, &mut app).await;
            ratatui::restore();
            result
        }
        Err(e) => {
            ratatui::restore();
            eprintln!("Failed to connect to Noesis API at {}: {}", api_url, e);
            eprintln!("Make sure the Noesis daemon is running: cargo run -- start --rest");
            Err(e)
        }
    }
}
