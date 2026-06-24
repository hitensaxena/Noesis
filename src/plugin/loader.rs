use std::path::Path;
use anyhow::Result;
use tracing;

/// Loads plugins from dynamic libraries or plugin directories.
pub struct PluginLoader;

impl PluginLoader {
    pub fn new() -> Self {
        Self
    }

    /// Load a plugin from a dynamic library path.
    pub fn load_plugin(&self, path: &str) -> Result<()> {
        tracing::info!("[PluginLoader] loading plugin from: {}", path);
        // Placeholder for dynamic loading via libloading
        Ok(())
    }

    /// Scan a directory for plugin files (.so, .dylib).
    pub fn scan_directory(&self, dir: &str) -> Result<Vec<String>> {
        let mut plugins = Vec::new();
        let path = Path::new(dir);
        if !path.exists() {
            return Ok(plugins);
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if let Some(ext) = entry_path.extension() {
                if ext == "so" || ext == "dylib" {
                    if let Some(p) = entry_path.to_str() {
                        plugins.push(p.to_string());
                    }
                }
            }
        }
        Ok(plugins)
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
