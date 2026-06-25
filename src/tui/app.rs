//! TUI application state — holds all data fetched from the Noesis API.

use std::time::{Duration, Instant};
use anyhow::Result;
use serde_json::Value;

use super::api::NoesisClient;

/// Available screens in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Signals,
    Fields,
    Processors,
    Observability,
    Log,
}

impl Screen {
    pub fn all() -> &'static [Screen] {
        &[
            Screen::Dashboard,
            Screen::Signals,
            Screen::Fields,
            Screen::Processors,
            Screen::Observability,
            Screen::Log,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Signals => "Signals",
            Screen::Fields => "Fields",
            Screen::Processors => "Processors",
            Screen::Observability => "Observability",
            Screen::Log => "Log",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Screen::Dashboard => "\u{2699}",    // gear
            Screen::Signals => "\u{26A1}",       // lightning
            Screen::Fields => "\u{1F4CA}",       // chart
            Screen::Processors => "\u{1F9E0}",   // brain
            Screen::Observability => "\u{1F4F0}", // newspaper
            Screen::Log => "\u{1F4DD}",          // memo
        }
    }
}

/// TUI application state.
pub struct TuiApp {
    pub api: NoesisClient,
    pub screen: Screen,
    pub should_quit: bool,

    // Data refreshed from API
    pub stats: Value,
    pub signals: Value,
    pub obs: Value,
    pub signal_types: Value,
    pub processor_metrics: Value,

    // Refresh tracking
    pub last_refresh: Instant,
    pub refresh_interval: Duration,
    pub status_message: String,

    // Log entries (local)
    pub log_entries: Vec<String>,
}

impl TuiApp {
    pub async fn new(api_url: &str) -> Result<Self> {
        let api = NoesisClient::new(api_url);

        // Initial data fetch
        let (stats, signals, obs) = api.dashboard().await?;
        let signal_types = api.signal_types().await?;
        let processor_metrics = api.processor_metrics().await?;

        let mut app = Self {
            api,
            screen: Screen::Dashboard,
            should_quit: false,
            stats,
            signals,
            obs,
            signal_types,
            processor_metrics,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            status_message: "Connected".to_string(),
            log_entries: Vec::new(),
        };

        app.add_log("Connected to Noesis API");
        Ok(app)
    }

    /// Refresh all data from the API.
    pub async fn refresh(&mut self) {
        match self.api.dashboard().await {
            Ok((stats, signals, obs)) => {
                self.stats = stats;
                self.signals = signals;
                self.obs = obs;
            }
            Err(e) => {
                self.status_message = format!("API error: {}", e);
                return;
            }
        }

        if let Ok(st) = self.api.signal_types().await {
            self.signal_types = st;
        }
        if let Ok(pm) = self.api.processor_metrics().await {
            self.processor_metrics = pm;
        }

        self.status_message = format!("OK ({}s)", self.refresh_interval.as_secs());
        self.last_refresh = Instant::now();
    }

    pub fn add_log(&mut self, msg: impl Into<String>) {
        let entry = format!("[{}] {}", chrono::Utc::now().format("%H:%M:%S"), msg.into());
        self.log_entries.push(entry);
        if self.log_entries.len() > 100 {
            self.log_entries.remove(0);
        }
    }

    pub fn next_screen(&mut self) {
        let all = Screen::all();
        let idx = all.iter().position(|s| *s == self.screen).unwrap_or(0);
        self.screen = all[(idx + 1) % all.len()];
    }

    pub fn prev_screen(&mut self) {
        let all = Screen::all();
        let idx = all.iter().position(|s| *s == self.screen).unwrap_or(0);
        self.screen = all[(idx + all.len() - 1) % all.len()];
    }
}
