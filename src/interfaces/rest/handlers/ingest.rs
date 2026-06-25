use std::sync::Arc;
use axum::{Json, extract::State};
use serde::Deserialize;
use tracing;

use crate::interfaces::rest::ApiState;
use crate::signals::IngestRequest;

#[derive(Deserialize)]
pub struct IngestBody {
    pub text: String,
    pub source: Option<String>,
}

/// POST /api/ingest — inject raw text into the cognition pipeline.
pub async fn ingest(
    State(state): State<ApiState>,
    Json(body): Json<IngestBody>,
) -> Json<serde_json::Value> {
    let source = body.source.unwrap_or_else(|| "rest".to_string());
    let signal = IngestRequest::new(&body.text, &source);

    tracing::info!(
        "[REST] ingest: {} '{}'",
        source,
        &body.text[..60.min(body.text.len())]
    );

    state.event_bus.publish(Arc::new(signal));

    Json(serde_json::json!({
        "status": "accepted",
        "source": source,
        "text_length": body.text.len(),
    }))
}
