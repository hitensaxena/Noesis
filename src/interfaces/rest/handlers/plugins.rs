//! Plugin management API handlers.
//!
//! Provides endpoints for listing, inspecting, and reloading plugins
//! at runtime. These power the Plugin Manager view in the web dashboard.

use axum::{Json, extract::State, extract::Path};
use tracing;

use crate::interfaces::rest::ApiState;

/// GET /api/plugins — list all loaded plugins with version/status.
pub async fn list_plugins(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let names = state.plugin_registry.plugin_names();
    let total_caps = state.capability_registry.list().len();
    let cap_ids: Vec<String> = state.capability_registry.list();

    let plugins: Vec<serde_json::Value> = names.iter().map(|name| {
        serde_json::json!({
            "name": name,
            "version": "0.1.0",
            "status": "loaded",
            "capabilities": cap_ids.iter().filter(|cid| {
                !state.capability_registry.find_providers(cid).is_empty()
            }).count(),
        })
    }).collect();

    Json(serde_json::json!({
        "plugins": plugins,
        "total": names.len(),
        "capability_count": total_caps,
    }))
}

/// GET /api/plugins/:name — detailed view of a single plugin.
pub async fn plugin_detail(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Json<serde_json::Value> {
    // Check if the plugin exists in the registry
    let all_names = state.plugin_registry.plugin_names();
    if !all_names.contains(&name) {
        return Json(serde_json::json!({
            "error": format!("Plugin '{}' not found", name),
            "known_plugins": all_names,
        }));
    }

    // Gather capabilities from this plugin (via capability_registry)
    let caps: Vec<serde_json::Value> = state.capability_registry.list().iter().filter_map(|cid| {
        let providers = state.capability_registry.find_providers(cid);
        if providers.is_empty() {
            None
        } else {
            Some(serde_json::json!({
                "id": cid,
                "providers": providers.iter().map(|p| serde_json::json!({
                    "name": p.name,
                    "processor": p.processor,
                    "confidence": p.confidence,
                })).collect::<Vec<_>>(),
            }))
        }
    }).collect();

    Json(serde_json::json!({
        "name": name,
        "version": "0.1.0",
        "status": "loaded",
        "capabilities": caps,
        "capability_count": caps.len(),
    }))
}

/// POST /api/plugins/reload — trigger plugin manifest re-scan.
///
/// Scans `~/.noesis/plugins/*/plugin.json` for new plugin manifests,
/// registers their capabilities, and returns what was loaded.
pub async fn reload_plugins(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    tracing::info!("[REST] plugin reload requested");

    let (loaded, errors) = state.plugin_registry.reload_from_plugins_dir();

    // Emit a plugin.loaded signal for any newly discovered plugins
    for plugin_name in &loaded {
        tracing::info!("[REST] plugin loaded: {}", plugin_name);
    }
    for err in &errors {
        tracing::warn!("[REST] plugin reload error: {}", err);
    }

    Json(serde_json::json!({
        "status": "reload_complete",
        "loaded": loaded,
        "errors": errors,
        "total_plugins": state.plugin_registry.plugin_names().len(),
    }))
}

/// GET /api/config — current runtime configuration overview.
pub async fn system_config(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let kernel = state.kernel.lock().await;
    let has_auth = std::env::var("NOESIS_API_KEY").ok().filter(|k| !k.is_empty()).is_some();
    let has_event_store = state.event_store.is_some();

    Json(serde_json::json!({
        "service": "noesis",
        "version": "0.1.0",
        "auth_enabled": has_auth,
        "event_persistence": has_event_store,
        "fields": kernel.fields.len(),
        "processors": kernel.processors.len(),
        "signal_types": kernel.signal_types.len(),
        "capability_count": state.capability_registry.list().len(),
        "plugins_count": state.plugin_registry.plugin_names().len(),
        "field_names": kernel.fields,
    }))
}
