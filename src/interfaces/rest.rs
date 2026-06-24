use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
    Json,
    extract::State,
};
use serde::Deserialize;
use tracing;

use crate::eventbus::bus::EventBus;
use crate::signals::IngestRequest;

/// Shared state for the REST API.
#[derive(Clone)]
pub struct ApiState {
    pub event_bus: Arc<EventBus>,
}

/// Build the REST API router.
pub fn router(event_bus: Arc<EventBus>) -> Router {
    let state = ApiState { event_bus };
    Router::new()
        .route("/health", get(health))
        .route("/ingest", post(ingest))
        .route("/signals/count", get(signal_count))
        .with_state(state)
}

/// Health check endpoint.
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "noesis",
        "version": "0.1.0"
    }))
}

/// Ingest a raw experience via HTTP.
#[derive(Deserialize)]
struct IngestBody {
    text: String,
    source: Option<String>,
}

async fn ingest(
    State(state): State<ApiState>,
    Json(body): Json<IngestBody>,
) -> Json<serde_json::Value> {
    let source = body.source.unwrap_or_else(|| "rest".to_string());
    let signal = IngestRequest::new(&body.text, &source);

    tracing::info!("[REST] ingest request: {}", &body.text[..30.min(body.text.len())]);
    state.event_bus.publish(Arc::new(signal));

    Json(serde_json::json!({
        "status": "accepted",
        "text_length": body.text.len(),
        "source": source,
    }))
}

/// Get signal count (placeholder).
async fn signal_count() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "count": 0,
        "note": "real-time metrics coming soon"
    }))
}
