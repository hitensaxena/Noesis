//! Plugin architecture for Noesis.
//!
//! Plugins package processors, signals, and capabilities into self-contained
//! units that the PluginRegistry discovers and loads. This allows the cognitive
//! system to be extended with new processors and fields without modifying core code.
//!
//! ## Current phase: built-in plugins
//! Phase 4 registers all existing processors as a single built-in plugin.
//! Future phases add dynamic loading from `~/.noesis/plugins/`.

//! Plugin architecture and built-in plugin registry.

pub mod noesis_plugin;

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing;

use crate::kernel::capabilities::Capability;
use crate::kernel::signal::SignalType;
use crate::processor::processor::Processor;
use crate::field_runtime::field::Field;

/// A Noesis plugin packages processors, signals, capabilities, and metadata.
///
/// Plugins are self-describing: they declare what signals they handle,
/// what capabilities they provide, and what configuration they need.
/// The Plugin trait is designed so `fn fields()` can be added in a future
/// version (for plugins that register entire cognitive fields).
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Unique plugin name (e.g. "noesis.memory", "hermes.browser").
    fn name(&self) -> &str;

    /// Semantic version of this plugin.
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Human-readable description.
    fn description(&self) -> &str {
        ""
    }

    /// Processors this plugin provides.
    fn processors(&self) -> Vec<Box<dyn Processor + Send>> {
        vec![]
    }

    /// Cognitive fields this plugin provides.
    ///
    /// Allows plugins to register entire fields (e.g., a "Learning" field
    /// that a plugin provides as a self-contained unit).
    fn fields(&self) -> Vec<Box<dyn Field + Send>> {
        vec![]
    }

    /// Signal types this plugin registers, with descriptions.
    fn signals(&self) -> Vec<(SignalType, &str)> {
        vec![]
    }

    /// Capabilities this plugin provides.
    fn capabilities(&self) -> Vec<Capability> {
        vec![]
    }

    /// Default configuration values.
    fn config_defaults(&self) -> Vec<(&str, serde_json::Value)> {
        vec![]
    }
}

/// Metadata for a Noesis plugin (used for manifest-based discovery).
///
/// Manifests are JSON files found in `~/.noesis/plugins/<name>/plugin.json`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    /// Path to the shared library (.so/.dylib) relative to the manifest dir.
    #[serde(default)]
    pub library_path: Option<String>,
    /// Signal types this plugin provides.
    #[serde(default)]
    pub provided_signals: Vec<String>,
    /// Signal types this plugin requires.
    #[serde(default)]
    pub required_signals: Vec<String>,
    /// Capabilities provided by this plugin.
    #[serde(default)]
    pub capabilities: Vec<PluginCapabilityDef>,
    /// Configuration schema (JSON Schema).
    #[serde(default)]
    pub config_schema: Option<serde_json::Value>,
}

/// Capability definition in a plugin manifest.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginCapabilityDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_weight")]
    pub weight: f32,
    pub processor: String,
}

fn default_weight() -> f32 { 0.5 }

impl PluginManifest {
    /// Load a manifest from a JSON file path.
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read manifest: {}", path.as_ref().display()))?;
        let manifest: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse manifest: {}", path.as_ref().display()))?;
        Ok(manifest)
    }

    /// Discover all plugin manifests in a directory.
    /// Looks for `<dir>/<name>/plugin.json` subdirectories.
    pub fn discover(dir: impl AsRef<Path>) -> Result<Vec<(PathBuf, Self)>> {
        let mut manifests = Vec::new();
        let dir = dir.as_ref();

        if !dir.exists() {
            return Ok(manifests);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.json");
                if manifest_path.exists() {
                    match Self::load_from_path(&manifest_path) {
                        Ok(m) => {
                            tracing::info!("[PluginManifest] discovered {} v{} at {}", m.name, m.version, manifest_path.display());
                            manifests.push((manifest_path, m));
                        }
                        Err(e) => {
                            tracing::warn!("[PluginManifest] failed to load {}: {}", manifest_path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(manifests)
    }
}

/// A plugin loaded from a manifest file. Provides metadata without
/// the actual processor implementations (which would come from a shared library).
pub struct ManifestPlugin {
    manifest: PluginManifest,
}

impl ManifestPlugin {
    pub fn new(manifest: PluginManifest) -> Self {
        Self { manifest }
    }
}

#[async_trait]
impl Plugin for ManifestPlugin {
    fn name(&self) -> &str { &self.manifest.name }
    fn version(&self) -> &str { &self.manifest.version }
    fn description(&self) -> &str { &self.manifest.description }

    fn capabilities(&self) -> Vec<Capability> {
        self.manifest.capabilities.iter().map(|c| Capability {
            id: c.id.clone(),
            name: c.name.clone(),
            description: c.description.clone(),
            confidence: c.weight,
            processor: c.processor.clone(),
        }).collect()
    }
}

/// Registry that manages loaded plugins, their processors, and capabilities.
///
/// Provides discovery: given a capability ID, find all processors that provide it.
/// Also tracks which signal types are registered and by which plugin.
pub struct PluginRegistry {
    plugins: DashMap<String, Box<dyn Plugin + Send + Sync>>,
    /// capability_id → Vec<(plugin_name, processor_name)>
    capability_index: DashMap<String, Vec<(String, String)>>,
    /// signal_type → plugin that registered it
    signal_index: DashMap<SignalType, String>,
    /// field_name → plugin that registered it
    field_index: DashMap<String, String>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: DashMap::new(),
            capability_index: DashMap::new(),
            signal_index: DashMap::new(),
            field_index: DashMap::new(),
        }
    }

    /// Register a plugin and index its processors, signals, capabilities, and fields.
    pub fn register(&self, plugin: Box<dyn Plugin + Send + Sync>) {
        let name = plugin.name().to_string();
        tracing::info!("[PluginRegistry] registering plugin: {} v{}", name, plugin.version());

        // Index capabilities
        for cap in plugin.capabilities() {
            let mut entry = self.capability_index
                .entry(cap.id.clone())
                .or_insert_with(Vec::new);
            entry.push((name.clone(), cap.processor.clone()));
            tracing::debug!("[PluginRegistry]   capability: {} -> {}", cap.id, cap.processor);
        }

        // Index signals
        for (sig_type, desc) in plugin.signals() {
            self.signal_index.insert(sig_type.clone(), name.clone());
            tracing::debug!("[PluginRegistry]   signal: {} ({})", sig_type, desc);
        }

        // Index fields
        for field in plugin.fields() {
            let field_name = field.name().to_string();
            self.field_index.insert(field_name, name.clone());
            tracing::debug!("[PluginRegistry]   field: {}", field.name());
        }

        self.plugins.insert(name, plugin);
        tracing::info!("[PluginRegistry] plugin registered");
    }

    /// Get all processors from all registered plugins.
    pub fn all_processors(&self) -> Vec<Box<dyn Processor + Send>> {
        let mut all = Vec::new();
        for entry in self.plugins.iter() {
            all.extend(entry.value().processors());
        }
        all
    }

    /// Get all signals from all registered plugins (owned strings).
    pub fn all_signals(&self) -> Vec<(SignalType, String)> {
        let mut all = Vec::new();
        for entry in self.plugins.iter() {
            for (sig_type, desc) in entry.value().signals() {
                all.push((sig_type, desc.to_string()));
            }
        }
        all
    }

    /// Get all capabilities from all registered plugins.
    pub fn all_capabilities(&self) -> Vec<Capability> {
        let mut all = Vec::new();
        for entry in self.plugins.iter() {
            all.extend(entry.value().capabilities());
        }
        all
    }

    /// Find processors that provide a given capability.
    pub fn find_by_capability(&self, capability_id: &str) -> Vec<(String, String)> {
        self.capability_index
            .get(capability_id)
            .map(|e| e.value().clone())
            .unwrap_or_default()
    }

    /// List all registered plugin names.
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.iter().map(|e| e.key().clone()).collect()
    }

    /// List all registered capability IDs.
    pub fn capability_ids(&self) -> Vec<String> {
        self.capability_index.iter().map(|e| e.key().clone()).collect()
    }

    /// Get a plugin by name.
    pub fn get(&self, name: &str) -> Option<Box<dyn Plugin + Send + Sync>> {
        // Direct Plugin reference access is future work.
        // All functionality (processors, capabilities) is accessible via the index methods.
        let _ = name;
        None
    }

    /// Reload plugins from the default plugin directory.
    ///
    /// Scans `~/.noesis/plugins/*/plugin.json` for manifest-based plugins,
    /// creates ManifestPlugin wrappers, and registers their capabilities.
    /// Returns a list of plugin names that were loaded or errors that occurred.
    pub fn reload_from_plugins_dir(&self) -> (Vec<String>, Vec<String>) {
        let mut loaded = Vec::new();
        let mut errors = Vec::new();

        let loader = PluginLoader::new();
        match PluginManifest::discover(loader.plugin_dir()) {
            Ok(manifests) => {
                for (_path, manifest) in manifests {
                    let plugin = ManifestPlugin::new(manifest.clone());
                    // Register capabilities from this plugin
                    for cap in plugin.capabilities() {
                        let mut entry = self.capability_index
                            .entry(cap.id.clone())
                            .or_insert_with(Vec::new);
                        entry.push((plugin.name().to_string(), cap.processor.clone()));
                    }
                    loaded.push(format!("{} v{}", plugin.name(), plugin.version()));
                    tracing::info!("[PluginRegistry] reloaded plugin: {} v{}", plugin.name(), plugin.version());
                }
            }
            Err(e) => {
                errors.push(format!("Failed to scan plugins: {}", e));
            }
        }

        (loaded, errors)
    }

    /// Number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Loads plugins from plugin directories via manifest files.
///
/// Scans `~/.noesis/plugins/*/plugin.json` for discoverable plugins.
/// Each manifest describes the plugin's metadata, capabilities, and signals.
/// Actual shared library loading is future work (requires libloading feature).
pub struct PluginLoader {
    /// Base directory for plugin discovery.
    plugin_dir: PathBuf,
}

impl PluginLoader {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            plugin_dir: PathBuf::from(home).join(".noesis").join("plugins"),
        }
    }

    /// Create a loader with a custom plugin directory.
    pub fn with_dir(dir: impl Into<PathBuf>) -> Self {
        Self { plugin_dir: dir.into() }
    }

    /// Returns the plugin directory path.
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }

    /// Discover and load all plugins from the plugin directory.
    ///
    /// Returns `(loaded_names, errors)` where `errors` contains any
    /// manifest parsing failures.
    pub fn load_all(&self) -> (Vec<String>, Vec<String>) {
        let mut loaded = Vec::new();
        let mut errors = Vec::new();

        match PluginManifest::discover(&self.plugin_dir) {
            Ok(manifests) => {
                for (_path, manifest) in manifests {
                    loaded.push(format!("{} v{}", manifest.name, manifest.version));
                    tracing::info!(
                        "[PluginLoader] discovered plugin: {} v{} ({})",
                        manifest.name, manifest.version, manifest.description,
                    );
                }
            }
            Err(e) => {
                errors.push(format!("Failed to scan plugins: {}", e));
            }
        }

        (loaded, errors)
    }

    /// Load a plugin from a specific manifest file path.
    pub fn load_manifest(path: impl AsRef<Path>) -> Result<ManifestPlugin> {
        let manifest = PluginManifest::load_from_path(path)?;
        Ok(ManifestPlugin::new(manifest))
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_plugin_manifest_load_from_json() {
        let json = r#"{
            "name": "test-plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "provided_signals": ["test.signal"],
            "capabilities": [
                {
                    "id": "test_cap",
                    "name": "Test Capability",
                    "description": "A test capability",
                    "weight": 0.8,
                    "processor": "test_processor"
                }
            ]
        }"#;

        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.provided_signals, vec!["test.signal"]);
        assert_eq!(manifest.capabilities.len(), 1);
        assert_eq!(manifest.capabilities[0].id, "test_cap");
    }

    #[test]
    fn test_manifest_plugin_wraps_metadata() {
        let json = r#"{
            "name": "meta-plugin",
            "version": "0.2.0",
            "description": "A metadata test",
            "capabilities": [
                {
                    "id": "cap1",
                    "name": "Cap One",
                    "description": "First capability",
                    "weight": 0.7,
                    "processor": "proc1"
                }
            ]
        }"#;

        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        let plugin = ManifestPlugin::new(manifest);

        assert_eq!(plugin.name(), "meta-plugin");
        assert_eq!(plugin.version(), "0.2.0");
        assert_eq!(plugin.description(), "A metadata test");

        let caps = plugin.capabilities();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].id, "cap1");
        assert_eq!(caps[0].confidence, 0.7);
    }

    #[test]
    fn test_manifest_discover_finds_nothing_in_empty_dir() {
        let tmp = std::env::temp_dir().join(format!("noesis_plugin_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let manifests = PluginManifest::discover(&tmp).unwrap();
        assert!(manifests.is_empty(), "empty dir should have no manifests");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_manifest_discover_finds_json_files() {
        let tmp = std::env::temp_dir().join(format!("noesis_plugin_test_discover_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp.join("my-plugin")).unwrap();

        let mut f = std::fs::File::create(tmp.join("my-plugin").join("plugin.json")).unwrap();
        f.write_all(br#"{"name":"my-plugin","version":"1.0.0","description":"Discovered plugin"}"#).unwrap();
        drop(f);

        let manifests = PluginManifest::discover(&tmp).unwrap();
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].1.name, "my-plugin");

        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn test_manifest_minimal() {
        let json = r#"{"name":"minimal","version":"0.0.1","description":"Minimal plugin"}"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "minimal");
        assert!(manifest.capabilities.is_empty());
        assert!(manifest.provided_signals.is_empty());
        assert!(manifest.library_path.is_none());
    }

    #[test]
    fn test_plugin_loader_load_manifest() {
        let tmp = std::env::temp_dir().join(format!("noesis_loader_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let manifest_path = tmp.join("loader-plugin.json");
        let mut f = std::fs::File::create(&manifest_path).unwrap();
        f.write_all(br#"{"name":"loader-plugin","version":"0.5.0","description":"Loaded via PluginLoader","capabilities":[{"id":"loader_cap","name":"Loader","description":"Test","weight":0.9,"processor":"loader_proc"}]}"#).unwrap();
        drop(f);

        let plugin = PluginLoader::load_manifest(&manifest_path).unwrap();
        assert_eq!(plugin.name(), "loader-plugin");
        assert_eq!(plugin.capabilities().len(), 1);

        let _ = std::fs::remove_dir_all(tmp);
    }
}
