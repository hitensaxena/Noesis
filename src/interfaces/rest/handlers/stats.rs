use axum::{Json, extract::State};
use crate::interfaces::rest::ApiState;

/// GET /api/stats — full system statistics.
pub async fn get_stats(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let kernel = state.kernel.lock().await;

    // Use field_cache for live field count (kernel.fields may be empty if fields
    // were registered directly through field_runtime rather than kernel.registry)
    let field_names: Vec<String> = state.field_cache.iter().map(|e| e.key().clone()).collect();

    Json(serde_json::json!({
        "fields": field_names.len(),
        "processors": kernel.processors.len(),
        "signal_types": kernel.signal_types.len(),
        "field_names": field_names,
        "processor_names": kernel.processors,
        "signal_names": kernel.signal_types.iter().map(|(t, d)| {
            serde_json::json!({"type": t.0, "description": d})
        }).collect::<Vec<_>>(),
    }))
}

/// GET /api/stats/signals — per-signal-type counts.
pub async fn signal_stats(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(state.metrics.snapshot())
}
