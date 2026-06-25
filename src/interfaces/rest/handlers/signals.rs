use std::sync::Arc;
use axum::{Json, extract::State};
use serde::Deserialize;
use tracing;

use crate::interfaces::rest::ApiState;
use crate::signals::IngestRequest;

#[derive(Deserialize)]
pub struct InjectBody {
    pub signal_type: String,
    pub payload: Option<serde_json::Value>,
}

/// GET /api/signals — list all registered signal types.
pub async fn list_signal_types(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let kernel = state.kernel.lock().await;
    let signals: Vec<serde_json::Value> = kernel
        .signal_types
        .iter()
        .map(|(t, d)| {
            serde_json::json!({
                "type": t.0,
                "description": d,
            })
        })
        .collect();

    Json(serde_json::json!({
        "signal_types": signals,
        "count": signals.len(),
    }))
}

/// POST /api/signals/inject — inject an arbitrary signal into the bus.
pub async fn inject_signal(
    State(state): State<ApiState>,
    Json(body): Json<InjectBody>,
) -> Json<serde_json::Value> {
    tracing::info!("[REST] inject signal: {}", body.signal_type);

    // For now, handle common signal types
    match body.signal_type.as_str() {
        "ingest.request" => {
            let text = match &body.payload {
                Some(payload) => payload.get("text").and_then(|v| v.as_str()).unwrap_or("injected via API"),
                None => "injected via API",
            };
            let signal = IngestRequest::new(text, "rest-api");
            state.event_bus.publish(Arc::new(signal));
        }
        _ => {
            return Json(serde_json::json!({
                "error": format!("Unknown signal type: {}", body.signal_type),
                "known_types": state.kernel.lock().await.signal_types.iter().map(|(t, _)| t.0).collect::<Vec<_>>(),
            }));
        }
    }

    Json(serde_json::json!({
        "status": "injected",
        "signal_type": body.signal_type,
    }))
}

/// GET /api/signals/history — recent signal history.
pub async fn signal_history(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    // Would query EventStore for recent events
    Json(serde_json::json!({
        "signals": [],
        "count": 0,
        "note": "Signal history from EventStore coming soon",
    }))
}
