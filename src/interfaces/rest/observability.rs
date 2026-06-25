//! Observability endpoints — metrics, pipeline traces, cascade logging.
//!
//! Mirrors curlyos-core's /api/observability/* endpoints.

use axum::{Json, extract::State};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::interfaces::rest::ApiState;

/// GET /api/observability/overview — system-wide observability summary.
pub async fn overview(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let kernel = state.kernel.lock().await;
    let metrics = state.metrics.snapshot();
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Json(serde_json::json!({
        "service": "noesis",
        "version": "0.1.0",
        "uptime_seconds": uptime,
        "fields": kernel.fields.len(),
        "processors": kernel.processors.len(),
        "signal_types": kernel.signal_types.len(),
        "signals_processed": metrics["signals"],
        "processor_stats": metrics["processors"],
    }))
}

/// GET /api/observability/signals — per-signal-type metrics.
pub async fn signal_metrics(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let metrics = state.metrics.snapshot();
    Json(metrics["signals"].clone())
}

/// GET /api/observability/processors — per-processor latency metrics.
pub async fn processor_metrics(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let metrics = state.metrics.snapshot();
    Json(metrics["processors"].clone())
}

/// GET /api/observability/cascade — cascade trace (most recent cascade events).
pub async fn cascade_trace(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "recent_cascades": [],
        "note": "Cascade tracing coming in next release — tracks each signal chain through the processor network",
    }))
}
