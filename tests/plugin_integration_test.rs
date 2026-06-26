//! Plugin loading integration test.
//!
//! Tests the full plugin discovery and registration pipeline using
//! temp directories and manifest files. No external dependencies.

use std::sync::Arc;
use std::io::Write;

use noesis::kernel::plugin::{Plugin, PluginManifest, PluginRegistry, PluginLoader, ManifestPlugin};
use noesis::kernel::capabilities::CapabilityRegistry;

/// Test: create a temp manifest, load it, and verify metadata.
#[test]
fn test_plugin_manifest_roundtrip() {
    let tmp = std::env::temp_dir().join(format!("noesis_plugin_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let manifest_path = tmp.join("test-plugin.json");
    let mut f = std::fs::File::create(&manifest_path).unwrap();
    f.write_all(br#"{
        "name": "roundtrip-plugin",
        "version": "1.2.3",
        "description": "Roundtrip test plugin",
        "provided_signals": ["test.signal.alpha", "test.signal.beta"],
        "capabilities": [
            {
                "id": "roundtrip_cap",
                "name": "Roundtrip",
                "description": "A roundtrip test capability",
                "weight": 0.9,
                "processor": "roundtrip_proc"
            }
        ]
    }"#).unwrap();
    drop(f);

    let manifest = PluginManifest::load_from_path(&manifest_path).unwrap();
    assert_eq!(manifest.name, "roundtrip-plugin");
    assert_eq!(manifest.version, "1.2.3");
    assert_eq!(manifest.provided_signals.len(), 2);
    assert_eq!(manifest.capabilities.len(), 1);
    assert_eq!(manifest.capabilities[0].id, "roundtrip_cap");

    let _ = std::fs::remove_dir_all(tmp);
}

/// Test: PluginManifest::discovers finds manifests in subdirectories.
#[test]
fn test_plugin_discover_temp_dir() {
    let tmp = std::env::temp_dir().join(format!("noesis_discover_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp.join("my-plugin")).unwrap();

    let mut f = std::fs::File::create(tmp.join("my-plugin").join("plugin.json")).unwrap();
    f.write_all(br#"{"name":"my-plugin","version":"0.1.0","description":"Discovered plugin"}"#).unwrap();
    drop(f);

    let manifests = PluginManifest::discover(&tmp).unwrap();
    assert_eq!(manifests.len(), 1, "should discover 1 manifest");
    assert_eq!(manifests[0].1.name, "my-plugin");

    let _ = std::fs::remove_dir_all(tmp);
}

/// Test: PluginRegistry registers a manifest-based plugin.
#[test]
fn test_plugin_registry_with_manifest() {
    let json = r#"{
        "name": "manifest-test",
        "version": "2.0.0",
        "description": "Manifest registration test",
        "capabilities": [
            {
                "id": "manifest_cap",
                "name": "Manifest Cap",
                "description": "A capability from manifest",
                "weight": 0.8,
                "processor": "manifest_proc"
            }
        ]
    }"#;

    let manifest: PluginManifest = serde_json::from_str(json).unwrap();
    let plugin = ManifestPlugin::new(manifest);

    assert_eq!(plugin.name(), "manifest-test");
    assert_eq!(plugin.version(), "2.0.0");
    assert_eq!(plugin.description(), "Manifest registration test");

    let caps = plugin.capabilities();
    assert_eq!(caps.len(), 1);
    assert_eq!(caps[0].id, "manifest_cap");
}

/// Test: PluginRegistry integrates with CapabilityRegistry.
#[test]
fn test_plugin_capability_integration() {
    let registry = PluginRegistry::new();
    let cap_registry = CapabilityRegistry::new();

    let json = r#"{
        "name": "cap-plugin",
        "version": "1.0.0",
        "description": "Capability integration test",
        "capabilities": [
            {
                "id": "integrated_cap",
                "name": "Integrated",
                "description": "Test capability registration",
                "weight": 1.0,
                "processor": "integrated_proc"
            }
        ]
    }"#;

    let manifest: PluginManifest = serde_json::from_str(json).unwrap();
    let plugin = ManifestPlugin::new(manifest);

    // Register capabilities from plugin
    for cap in plugin.capabilities() {
        cap_registry.register(cap);
    }

    assert!(cap_registry.list().contains(&"integrated_cap".to_string()),
            "capability should be registered");

    let providers = cap_registry.find_providers("integrated_cap");
    assert!(!providers.is_empty(), "should have providers");
    assert_eq!(providers[0].processor, "integrated_proc");

    // Verify plugin registry
    assert_eq!(registry.len(), 0, "no plugins directly registered");
}

/// Test: PluginLoader discovers manifests in the default plugin directory.
#[test]
fn test_plugin_loader_discover() {
    let tmp = std::env::temp_dir().join(format!("noesis_loader_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp.join("discovered-plugin")).unwrap();

    let mut f = std::fs::File::create(tmp.join("discovered-plugin").join("plugin.json")).unwrap();
    f.write_all(br#"{"name":"discovered-plugin","version":"3.0.0","description":"Discovered in loader test"}"#).unwrap();
    drop(f);

    let loader = PluginLoader::with_dir(&tmp);
    let (loaded, errors) = loader.load_all();
    assert!(errors.is_empty(), "should have no errors: {:?}", errors);
    assert!(loaded.iter().any(|s| s.contains("discovered-plugin")),
            "should discover the test plugin");

    let _ = std::fs::remove_dir_all(tmp);
}

/// Test: PluginManifest minimal JSON (required fields only).
#[test]
fn test_manifest_minimal_required() {
    let json = r#"{"name":"minimal","version":"0.0.1","description":"Minimal"}"#;
    let manifest: PluginManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.name, "minimal");
    assert!(manifest.capabilities.is_empty());
    assert!(manifest.library_path.is_none());
}

/// Test: PluginManifest with all optional fields.
#[test]
fn test_manifest_full_featured() {
    let json = r#"{
        "name": "full-plugin",
        "version": "99.99.99",
        "description": "A plugin with everything",
        "library_path": "plugins/full/target/release/libfull.so",
        "provided_signals": ["full.signal1", "full.signal2", "full.signal3"],
        "required_signals": ["required.signal"],
        "capabilities": [
            {"id": "cap1", "name": "Cap1", "description": "First", "weight": 0.5, "processor": "proc1"},
            {"id": "cap2", "name": "Cap2", "description": "Second", "weight": 0.7, "processor": "proc2"}
        ],
        "config_schema": {"type": "object", "properties": {"api_key": {"type": "string"}}}
    }"#;

    let manifest: PluginManifest = serde_json::from_str(json).unwrap();
    assert!(manifest.library_path.is_some());
    assert_eq!(manifest.provided_signals.len(), 3);
    assert_eq!(manifest.required_signals.len(), 1);
    assert_eq!(manifest.capabilities.len(), 2);
    assert!(manifest.config_schema.is_some());
}
