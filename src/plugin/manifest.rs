use serde::{Deserialize, Serialize};

/// Metadata for a Noesis plugin.
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub required_signals: Vec<String>,
    pub provided_signals: Vec<String>,
    pub config_schema: Option<serde_json::Value>,
}
