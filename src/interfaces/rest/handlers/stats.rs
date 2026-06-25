use axum::{Json, extract::State};
use crate::interfaces::rest::ApiState;

/// GET /api/stats — full system statistics.
pub async fn get_stats(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let kernel = state.kernel.lock().await;

    Json(serde_json::json!({
        "fields": kernel.fields.len(),
        "processors": kernel.processors.len(),
        "signal_types": kernel.signal_types.len(),
        "field_names": kernel.fields,
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
