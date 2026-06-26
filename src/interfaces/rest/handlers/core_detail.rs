//! GET /api/core/detail — deep core system observability.
//!
//! Returns a structured view of the Noesis runtime infrastructure:
//! event_bus, scheduler, registry, plugin_loader, kernel, runtime, config, metrics, permissions.

use axum::{Json, extract::State};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::interfaces::rest::ApiState;

/// GET /api/core/detail — full core system breakdown.
pub async fn core_detail(
    State(state): State<ApiState>,
) -> Json<Value> {
    let kernel = state.kernel.lock().await;
    let metrics = state.metrics.snapshot();
    let system = state.system_state;
    let total_signals = system.signal_count();

    let clock_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    let signals_processed = metrics.get("signals")
        .and_then(|s| s.as_u64())
        .unwrap_or(0);

    let processor_stats = metrics.get("processors")
        .and_then(|p| p.as_object())
        .cloned()
        .unwrap_or_default();

    Json(json!({
        "event_bus": {
            "type": "broadcast",
            "subscribers": kernel.signal_types.len(),
            "channels": kernel.signal_types.iter().map(|(st, _)| st.0).collect::<Vec<_>>(),
            "signal_count": signals_processed,
            "mode": "tokio::sync::broadcast + mpsc fan-in",
            "note": "EventBus uses tokio broadcast channels — one per signal type — fanned into a single mpsc receiver for sequential cascade processing.",
        },
        "scheduler": {
            "status": "initialized",
            "tasks_registered": 2,
            "tasks": [
                {"name": "field-snapshot", "interval_secs": 3, "enabled": true},
                {"name": "event-bridge", "interval_secs": 0, "enabled": true},
            ],
            "note": "Scheduler runs background tasks via tokio::spawn. Cognitive heartbeat (consolidation, reflection) scheduled via processor triggers.",
        },
        "registry": {
            "fields": kernel.fields,
            "processors": kernel.processors,
            "signal_types": kernel.signal_types.iter().map(|(t, d)| {
                json!({"type": t.0, "description": d})
            }).collect::<Vec<_>>(),
            "note": "Registry holds factories and metadata for all fields, processors, and signal types. Thread-safe access via Arc + DashMap.",
        },
        "plugin_loader": {
            "status": "inactive",
            "plugins_loaded": 0,
            "plugins": [],
            "note": "Plugin system scaffolded but not yet wired. Hermes integration available via standalone plugin (Python).",
        },
        "kernel": {
            "status": "running",
            "uptime_secs": clock_now as u64,
            "fields_count": kernel.fields.len(),
            "processors_count": kernel.processors.len(),
            "signal_types_count": kernel.signal_types.len(),
            "note": "Kernel orchestrates lifecycle (init → run → shutdown). Owns EventBus, Registry, and Runtime handles.",
        },
        "runtime": {
            "tasks_count": 3,
            "tasks": [
                "field-snapshot",
                "event-bridge",
                "rest-api",
            ],
            "note": "Runtime manages background task lifecycle via CancellationToken + join handles. Graceful shutdown on Ctrl-C.",
        },
        "config": {
            "rest_api_enabled": true,
            "default_port": 8647,
            "features": ["postgres-redis"],
            "field_cache_interval_secs": 3,
            "note": "Configuration from CLI args + env vars. Postgres/Redis auto-connect on startup.",
        },
        "metrics": {
            "signals_processed": signals_processed,
            "total_all_time": total_signals,
            "processor_stats": processor_stats,
            "signal_by_type": metrics.get("signals").cloned().unwrap_or(json!({})),
            "note": "Metrics collected by MetricsCollector — per-signal-type counts and per-processor latency histograms. Exposed via /api/observability/*.",
        },
        "permissions": {
            "mode": "open",
            "api_key_required": false,
            "auth_enabled": false,
            "note": "Permissions model not yet active. Noesis currently runs in open mode. API key authentication planned.",
        },
        "_meta": {
            "domain": "core",
            "arch": "decentralized-signal-cascade",
            "version": "0.1.0",
            "signals_processed": total_signals,
        }
    }))
}
