//! HTTP client for the Noesis REST API.
//!
//! Used by the TUI to fetch state from the running daemon.

use anyhow::Result;
use serde_json::Value;
use std::time::Duration;

pub struct NoesisClient {
    base_url: String,
    client: reqwest::Client,
}

impl NoesisClient {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    pub async fn health(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/health", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn stats(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/stats", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn signal_stats(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/stats/signals", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn signal_types(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/signals", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn observability(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/observability/overview", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn processor_metrics(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/observability/processors", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn signal_metrics(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/observability/signals", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn ingest(&self, text: &str, source: &str) -> Result<Value> {
        let client = self.client.clone();
        let body = serde_json::json!({"text": text, "source": source});
        Ok(client.post(format!("{}/api/ingest", self.base_url))
            .json(&body)
            .send().await?
            .json().await?)
    }

    /// Fetch all dashboard data in one call.
    pub async fn dashboard(&self) -> Result<(Value, Value, Value)> {
        let stats = self.stats().await?;
        let signals = self.signal_stats().await?;
        let obs = self.observability().await?;
        Ok((stats, signals, obs))
    }

    // ---------------------------------------------------------------------------
    // Deep observability detail endpoints
    // ---------------------------------------------------------------------------

    pub async fn identity_detail(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/identity/detail", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn memory_detail(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/memory/detail", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn agency_detail(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/agency/detail", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn awareness_detail(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/awareness/detail", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn simulation_detail(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/simulation/detail", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn core_detail(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/core/detail", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn reasoning_detail(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/cognition/meta", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn graph_detail(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/graph", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn plugins(&self) -> Result<Value> {
        Ok(self.client.get(format!("{}/api/plugins", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn detail_for(&self, name: &str) -> Result<Value> {
        match name {
            "identity" => self.identity_detail().await,
            "memory" => self.memory_detail().await,
            "agency" => self.agency_detail().await,
            "awareness" => self.awareness_detail().await,
            "reasoning" => self.reasoning_detail().await,
            "simulation" => self.simulation_detail().await,
            "graph" => self.graph_detail().await,
            "core" => self.core_detail().await,
            _ => Ok(serde_json::json!({"error": format!("unknown detail: {}", name)})),
        }
    }
}
