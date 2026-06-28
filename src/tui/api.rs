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
        let v: serde_json::Value = self.get("/api/signals")?;
        Ok(v.get("signal_types").and_then(|a| serde_json::from_value(a.clone()).ok()).unwrap_or_default())
    }

    pub fn signal_history(&self, limit: usize, field: Option<&str>) -> Result<Vec<SignalHistoryEntry>> {
        let mut path = format!("/api/signals/history?limit={limit}");
        if let Some(f) = field { path.push_str(&format!("&field={f}")); }
        self.get(&path)
    }

    pub fn observability(&self) -> Result<Observability> {
        self.get("/api/observability/overview")
    }

    pub fn processor_metrics(&self) -> Result<Vec<ProcessorMetric>> {
        let v: serde_json::Value = self.get("/api/observability/processors")?;
        let map = match &v {
            serde_json::Value::Object(m) => m,
            _ => return Ok(vec![]),
        };
        Ok(map.iter().map(|(name, stats)| ProcessorMetric {
            name: name.clone(),
            count: stats.get("count").and_then(|c| c.as_u64()).unwrap_or(0),
            avg_latency_ms: stats.get("avg_latency_ms").and_then(|c| c.as_u64()).unwrap_or(0),
        }).collect())
    }

    pub fn signal_metrics(&self) -> Result<SignalMetricsData> {
        let v: serde_json::Value = self.get("/api/observability/signals")?;
        let signals = match &v {
            serde_json::Value::Object(m) => m.iter().map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0))).collect(),
            _ => std::collections::BTreeMap::new(),
        };
        let total: u64 = signals.values().sum();
        Ok(SignalMetricsData { signals, total })
    }

    pub fn cascade_trace(&self) -> Result<CascadeTraceData> {
        let v: serde_json::Value = self.get("/api/observability/cascade")?;
        let recent = v.get("recent_cascades").and_then(|a| a.as_array()).cloned().unwrap_or_default();
        Ok(CascadeTraceData {
            depth: 0,
            signals: recent.iter().filter_map(|c| c.as_str().map(|s| s.to_string())).collect(),
            duration_ms: 0.0,
        })
    }

    pub fn capabilities(&self) -> Result<Vec<Capability>> {
        let v: serde_json::Value = self.get("/api/capabilities")?;
        Ok(v.get("capabilities").and_then(|a| serde_json::from_value(a.clone()).ok()).unwrap_or_default())
    }

    pub fn plugins(&self) -> Result<Vec<PluginSummary>> {
        let v: serde_json::Value = self.get("/api/plugins")?;
        Ok(v.get("plugins").and_then(|a| serde_json::from_value(a.clone()).ok()).unwrap_or_default())
    }

    pub fn config(&self) -> Result<SystemConfig> {
        let v: serde_json::Value = self.get("/api/config")?;
        Ok(SystemConfig {
            rest_api_enabled: v.get("auth_enabled").and_then(|b| b.as_bool()).unwrap_or(false),
            port: 8647,
            storage_backend: v.get("service").and_then(|s| s.as_str()).unwrap_or("memory").to_string(),
            settings: match &v {
                serde_json::Value::Object(m) => m.iter().map(|(k, val)| (k.clone(), val.clone())).collect(),
                _ => std::collections::BTreeMap::new(),
            }.into_iter().filter(|(k, _)| !["service", "version"].contains(&k.as_str())).collect(),
        })
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

    // ---- additional data -----------------------------------------------

    pub fn memory_state(&self) -> Result<serde_json::Value> {
        self.get("/api/memories")
    }

    pub fn episodes(&self) -> Result<serde_json::Value> {
        self.get("/api/episodes")
    }

    pub fn identity(&self) -> Result<serde_json::Value> {
        self.get("/api/identity")
    }

    pub fn signal_stats(&self) -> Result<serde_json::Value> {
        self.get("/api/stats/signals")
    }

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
    pub field_names: Vec<String>,
    #[serde(default)]
    pub fields: usize,
    #[serde(default)]
    pub processor_names: Vec<String>,
    #[serde(default)]
    pub processors: usize,
    #[serde(default)]
    pub signal_names: Vec<NamedSignalType>,
    #[serde(default)]
    pub signal_types: usize,
    #[serde(skip)]
    pub signals_total: usize,
    #[serde(skip)]
    pub cascade_cycles: usize,
}

#[derive(Deserialize, Clone)]
pub struct NamedSignalType {
    #[serde(default)]
    #[serde(rename = "type")]
    pub signal_type: String,
    #[serde(default)]
    pub description: String,
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

#[derive(Deserialize, Clone)]
pub struct Observability {
    #[serde(default)]
    pub signal_types: usize,
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
    // NOTE: cascade_cycles and signal_rates not returned by daemon yet
    #[serde(skip)]
    pub cascade_cycles: usize,
    #[serde(skip)]
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
    if s.len() <= n { s.to_string() } else { format!("{}...", &s[..n]) }
}
