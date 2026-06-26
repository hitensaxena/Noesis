use axum::{Json, extract::{Query, State}};
use serde::Deserialize;
use tracing;

use crate::interfaces::rest::ApiState;
use crate::signals::EpisodeRecorded;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct RecallQuery {
    pub q: String,
    pub k: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateMemoryBody {
    pub content: String,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// GET /api/memories — list all stored memories (from memory field state cache).
pub async fn list_memories(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let memory_state = state.field_cache.get("memory");
    if let Some(state_val) = memory_state {
        Json(serde_json::json!({
            "field": "memory",
            "state": state_val.value(),
        }))
    } else {
        Json(serde_json::json!({
            "memories": [],
            "count": 0,
            "note": "No memory field state cached yet — inject an experience first",
        }))
    }
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

/// GET /api/memory/recall?q=query&k=5 — search episodes by content.
pub async fn recall_memories(
    State(state): State<ApiState>,
    Query(params): Query<RecallQuery>,
) -> Json<serde_json::Value> {
    let k = params.k.unwrap_or(10).min(100);
    let memory_state = state.field_cache.get("memory");
    if let Some(state_val) = memory_state {
        let episodes = state_val.value().get("episodes").and_then(|e| e.as_array()).cloned().unwrap_or_default();
        let q = params.q.to_lowercase();
        let matches: Vec<_> = episodes.iter()
            .filter(|ep| {
                ep.get("content").and_then(|c| c.as_str()).unwrap_or("").to_lowercase().contains(&q)
                    || ep.get("source").and_then(|s| s.as_str()).unwrap_or("").to_lowercase().contains(&q)
            })
            .take(k)
            .cloned()
            .collect();

        Json(serde_json::json!({
            "query": params.q,
            "matches": matches.len(),
            "results": matches,
        }))
    } else {
        Json(serde_json::json!({
            "query": params.q,
            "matches": 0,
            "results": [],
            "note": "No memory state cached",
        }))
    }
}

/// GET /api/episodes — list recorded episodes (from field state cache).
pub async fn list_episodes(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let memory_state = state.field_cache.get("memory");
    if let Some(state_val) = memory_state {
        Json(serde_json::json!({
            "episodes_from_cache": state_val.value(),
        }))
    } else {
        Json(serde_json::json!({
            "episodes": [],
            "count": 0,
            "note": "No episode state cached yet",
        }))
    }
}
