use axum::{Json, extract::State};
use serde::Deserialize;
use tracing;

use crate::interfaces::rest::ApiState;
use crate::signals::EpisodeRecorded;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateMemoryBody {
    pub content: String,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// GET /api/memories — list all stored memories (from memory field).
pub async fn list_memories(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    // In-memory field state would be queried here
    // For now, return structure info
    Json(serde_json::json!({
        "memories": [],
        "count": 0,
        "note": "Memory field state available via field introspection in future release",
    }))
}

/// POST /api/memories — create a memory directly.
pub async fn create_memory(
    State(state): State<ApiState>,
    Json(body): Json<CreateMemoryBody>,
) -> Json<serde_json::Value> {
    let source = body.source.unwrap_or_else(|| "rest".to_string());

    let episode = EpisodeRecorded::new(&body.content, &source, body.tags.unwrap_or_default());
    state.event_bus.publish(Arc::new(episode));

    tracing::info!("[REST] memory created via API");

    Json(serde_json::json!({
        "status": "created",
        "source": source,
    }))
}

/// GET /api/episodes — list recorded episodes.
pub async fn list_episodes(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "episodes": [],
        "count": 0,
        "note": "Episode listing from field state coming in next release",
    }))
}
