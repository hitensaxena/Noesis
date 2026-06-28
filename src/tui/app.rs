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
    Detail,  // Deep field observability (uses detail_index to select which field)
    Settings, // Settings and configuration
}

/// Names of the deep detail views available in the Detail screen.
pub const DETAIL_NAMES: &[&str] = &[
    "Identity",
    "Memory",
    "Agency",
    "Awareness",
    "Reasoning",
    "Simulation",
    "Knowledge Graph",
    "Core",
];

impl Screen {
    pub fn all() -> &'static [Screen] {
        &[
            Screen::Dashboard,
            Screen::Signals,
            Screen::Fields,
            Screen::Processors,
            Screen::Observability,
            Screen::Log,
            Screen::Detail,
            Screen::Settings,
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
            Screen::Detail => "Detail",
            Screen::Settings => "Settings",
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
            Screen::Detail => "\u{1F50D}",       // magnifying glass
            Screen::Settings => "\u{2699}",      // gear
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

    // Deep detail data (fetched once to avoid N+1 queries per refresh)
    pub identity_detail: Value,
    pub memory_detail: Value,
    pub agency_detail: Value,
    pub awareness_detail: Value,
    pub reasoning_detail: Value,
    pub simulation_detail: Value,
    pub graph_detail: Value,
    pub core_detail: Value,

    // Plugin data
    pub plugins: Value,

    // Detail navigation
    pub detail_index: usize,

    // Settings
    pub auto_refresh: bool,
    pub api_url: String,

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
            identity_detail: serde_json::json!({}),
            memory_detail: serde_json::json!({}),
            agency_detail: serde_json::json!({}),
            awareness_detail: serde_json::json!({}),
            reasoning_detail: serde_json::json!({}),
            simulation_detail: serde_json::json!({}),
            graph_detail: serde_json::json!({}),
            core_detail: serde_json::json!({}),
            plugins: serde_json::json!({}),
            detail_index: 0,
            auto_refresh: true,
            api_url: api_url.to_string(),
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

        // Fetch deep detail data
        if let Ok(d) = self.api.identity_detail().await { self.identity_detail = d; }
        if let Ok(d) = self.api.memory_detail().await { self.memory_detail = d; }
        if let Ok(d) = self.api.agency_detail().await { self.agency_detail = d; }
        if let Ok(d) = self.api.awareness_detail().await { self.awareness_detail = d; }
        if let Ok(d) = self.api.reasoning_detail().await { self.reasoning_detail = d; }
        if let Ok(d) = self.api.simulation_detail().await { self.simulation_detail = d; }
        if let Ok(d) = self.api.graph_detail().await { self.graph_detail = d; }
        if let Ok(d) = self.api.core_detail().await { self.core_detail = d; }
        if let Ok(d) = self.api.plugins().await { self.plugins = d; }

        let auto = if self.auto_refresh { "" } else { " (paused)" };
        self.status_message = format!("OK ({}s){}", self.refresh_interval.as_secs(), auto);
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

    /// Toggle auto-refresh on/off.
    pub fn toggle_auto_refresh(&mut self) {
        self.auto_refresh = !self.auto_refresh;
        self.add_log(format!("Auto-refresh: {}", if self.auto_refresh { "ON" } else { "OFF" }));
    }

    /// Set refresh interval in seconds (clamped to 1-30).
    pub fn set_refresh_interval(&mut self, secs: u64) {
        let clamped = secs.clamp(1, 30);
        self.refresh_interval = Duration::from_secs(clamped);
        self.add_log(format!("Refresh interval: {}s", clamped));
    }

    /// Get current refresh interval in seconds.
    pub fn refresh_interval_secs(&self) -> u64 {
        self.refresh_interval.as_secs()
    }

    /// Next field detail (when on Detail screen).
    pub fn next_detail(&mut self) {
        self.detail_index = (self.detail_index + 1) % DETAIL_NAMES.len();
        self.add_log(format!("Detail: {} ({}/{})", DETAIL_NAMES[self.detail_index], self.detail_index + 1, DETAIL_NAMES.len()));
    }

    /// Previous field detail (when on Detail screen).
    pub fn prev_detail(&mut self) {
        self.detail_index = if self.detail_index == 0 {
            DETAIL_NAMES.len() - 1
        } else {
            self.detail_index - 1
        };
        self.add_log(format!("Detail: {} ({}/{})", DETAIL_NAMES[self.detail_index], self.detail_index + 1, DETAIL_NAMES.len()));
    }
}
