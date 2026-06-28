//! Blocking HTTP client + typed models for the Noesis REST API.
//! Used by the background worker thread to fetch state from the daemon.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::time::Duration;

pub struct Client {
    base: String,
    http: reqwest::blocking::Client,
}

impl Client {
    pub fn new(base: impl Into<String>) -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("build http client");
        Self { base: base.into(), http }
    }

    fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        let resp = self.http.get(&url).send().with_context(|| format!("GET {url}"))?;
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("GET {url} -> {status}: {}", truncate(&body, 200));
        }
        serde_json::from_str(&body).with_context(|| format!("decode GET {url}"))
    }

    fn post<T: for<'de> Deserialize<'de>>(&self, path: &str, body: serde_json::Value) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        let resp = self.http.post(&url).json(&body).send().with_context(|| format!("POST {url}"))?;
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("POST {url} -> {status}: {}", truncate(&text, 200));
        }
        serde_json::from_str(&text).with_context(|| format!("decode POST {url}"))
    }

    // ---- reads -----------------------------------------------------------

    pub fn health(&self) -> Result<Health> {
        self.get("/api/health")
    }

    pub fn stats(&self) -> Result<Stats> {
        self.get("/api/stats")
    }

    pub fn signal_types(&self) -> Result<Vec<SignalTypeItem>> {
        self.get("/api/signals")
    }

    pub fn signal_history(&self, limit: usize, field: Option<&str>) -> Result<Vec<SignalHistoryEntry>> {
        let mut path = format!("/api/signals/history?limit={limit}");
        if let Some(f) = field {
            path.push_str(&format!("&field={f}"));
        }
        self.get(&path)
    }

    pub fn observability(&self) -> Result<Observability> {
        self.get("/api/observability/overview")
    }

    pub fn processor_metrics(&self) -> Result<Vec<ProcessorMetric>> {
        self.get("/api/observability/processors")
    }

    pub fn signal_metrics(&self) -> Result<SignalMetricsData> {
        self.get("/api/observability/signals")
    }

    pub fn cascade_trace(&self) -> Result<CascadeTraceData> {
        self.get("/api/observability/cascade")
    }

    pub fn capabilities(&self) -> Result<Vec<Capability>> {
        self.get("/api/capabilities")
    }

    pub fn plugins(&self) -> Result<Vec<PluginSummary>> {
        self.get("/api/plugins")
    }

    pub fn config(&self) -> Result<SystemConfig> {
        self.get("/api/config")
    }

    pub fn detail_for(&self, name: &str) -> Result<serde_json::Value> {
        match name {
            "identity" => self.get("/api/identity/detail"),
            "memory" => self.get("/api/memory/detail"),
            "agency" => self.get("/api/agency/detail"),
            "awareness" => self.get("/api/awareness/detail"),
            "reasoning" => self.get("/api/cognition/meta"),
            "simulation" => self.get("/api/simulation/detail"),
            "graph" => self.get("/api/graph"),
            "core" => self.get("/api/core/detail"),
            _ => anyhow::bail!("unknown detail: {name}"),
        }
    }

    // ---- writes ----------------------------------------------------------

    pub fn ingest(&self, text: &str) -> Result<()> {
        let _: serde_json::Value = self.post("/api/ingest", serde_json::json!({ "text": text }))?;
        Ok(())
    }

    pub fn inject_signal(&self, signal_type: &str, payload: serde_json::Value) -> Result<()> {
        let _: serde_json::Value = self.post("/api/signals/inject", serde_json::json!({
            "signal_type": signal_type, "payload": payload,
        }))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Models — deserialised from the Noesis API
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct Health {
    pub status: String,
    #[serde(default)]
    pub uptime_seconds: f64,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub postgres: Option<String>,
    #[serde(default)]
    pub redis: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
pub struct Stats {
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub processors: Vec<String>,
    #[serde(default)]
    pub signal_types: Vec<(String, SignalTypeItem)>,
    #[serde(default)]
    pub field_count: usize,
    #[serde(default)]
    pub processor_count: usize,
    #[serde(default)]
    pub signal_type_count: usize,
    #[serde(default)]
    pub signals_total: usize,
    #[serde(default)]
    pub cascade_cycles: usize,
}

#[derive(Deserialize, Clone)]
pub struct SignalTypeItem {
    #[serde(default)]
    pub signal_type: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Deserialize, Clone)]
pub struct SignalHistoryEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub signal_type: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub data: serde_json::Value,
}

#[derive(Deserialize)]
pub struct Observability {
    #[serde(default)]
    pub signal_types: Vec<(String, String)>,
    #[serde(default)]
    pub signals_processed: serde_json::Value,
    #[serde(default)]
    pub signals_total: Option<usize>,
    #[serde(default)]
    pub fields: Option<usize>,
    #[serde(default)]
    pub processors: Option<usize>,
    #[serde(default)]
    pub uptime_seconds: f64,
    #[serde(default)]
    pub cascade_cycles: Option<usize>,
    #[serde(default)]
    pub signal_rates: std::collections::BTreeMap<String, f64>,
}

#[derive(Deserialize, Clone)]
pub struct ProcessorMetric {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub count: u64,
    #[serde(default)]
    pub avg_latency_ms: u64,
}

#[derive(Deserialize)]
pub struct SignalMetricsData {
    #[serde(default)]
    pub signals: std::collections::BTreeMap<String, u64>,
    #[serde(default)]
    pub total: u64,
}

#[derive(Deserialize)]
pub struct CascadeTraceData {
    #[serde(default)]
    pub depth: u32,
    #[serde(default)]
    pub signals: Vec<String>,
    #[serde(default)]
    pub duration_ms: f64,
}

#[derive(Deserialize, Clone)]
pub struct Capability {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub confidence: f32,
}

#[derive(Deserialize, Clone)]
pub struct PluginSummary {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub processors: Vec<String>,
}

#[derive(Deserialize)]
pub struct SystemConfig {
    #[serde(default)]
    pub rest_api_enabled: bool,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub storage_backend: String,
    #[serde(default)]
    pub settings: std::collections::BTreeMap<String, serde_json::Value>,
}

/// Aggregate fetched on dashboard load.
pub struct DashboardData {
    pub stats: Stats,
    pub health: Health,
    pub observability: Observability,
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n { s.to_string() } else { format!("{}…", &s[..n]) }
}
