use std::sync::Arc;
use axum::{Json, extract::State, extract::Query};
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

    match body.signal_type.as_str() {
        "memory.capture.ingested" => {
            let text = match &body.payload {
                Some(payload) => payload.get("text").and_then(|v| v.as_str()).unwrap_or("injected via API"),
                None => "injected via API",
            };
            let signal = IngestRequest::new(text, "rest-api");
            state.event_bus.publish(Arc::new(signal));
        }
        // Allow direct injection of beat signals for testing/triggering cascade completion
        "kernel.scheduler.beat.slow" => {
            let beat = crate::kernel::beat_coordinator::BeatPulse::new(crate::signals::types::BEAT_SLOW);
            state.event_bus.publish(Arc::new(beat));
        }
        "kernel.scheduler.beat.medium" => {
            let beat = crate::kernel::beat_coordinator::BeatPulse::new(crate::signals::types::BEAT_MEDIUM);
            state.event_bus.publish(Arc::new(beat));
        }
        "kernel.scheduler.beat.fast" => {
            let beat = crate::kernel::beat_coordinator::BeatPulse::new(crate::signals::types::BEAT_FAST);
            state.event_bus.publish(Arc::new(beat));
        }
        "kernel.scheduler.beat.immediate" => {
            let beat = crate::kernel::beat_coordinator::BeatPulse::new(crate::signals::types::BEAT_IMMEDIATE);
            state.event_bus.publish(Arc::new(beat));
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

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub from_seq: Option<u64>,
    pub limit: Option<u64>,
    pub event_type: Option<String>,
    /// Filter by field prefix (e.g. "memory" matches "memory.capture.ingested", "memory.consolidation.consolidated").
    pub field: Option<String>,
}

/// GET /api/signals/history — recent signal history from event store.
pub async fn signal_history(
    State(state): State<ApiState>,
    Query(params): Query<HistoryQuery>,
) -> Json<serde_json::Value> {
    let from_seq = params.from_seq.unwrap_or(1);
    let limit = params.limit.unwrap_or(50).min(500);
    let event_type = params.event_type.as_deref();
    let field_prefix = params.field.as_deref();

    match &state.event_store {
        Some(store) => {
            let events = store.list(from_seq, limit, event_type).await;
            // Apply field prefix filtering in the handler (EventStore trait doesn't support prefix matching)
            let filtered: Vec<_> = if let Some(field) = field_prefix {
                let prefix = format!("{}.", field);
                events.into_iter().filter(|e| e.event_type.starts_with(&prefix)).collect()
            } else {
                events
            };
            let count = store.count().await;
            Json(serde_json::json!({
                "signals": filtered,
                "count": filtered.len(),
                "total": count,
                "from_seq": from_seq,
                "limit": limit,
                "field_filter": field_prefix,
            }))
        }
        None => {
            Json(serde_json::json!({
                "signals": [],
                "count": 0,
                "note": "No event store configured. Start with --event-log <path> to enable persistence.",
            }))
        }
    }
}
