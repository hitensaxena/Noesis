use axum::{Json, extract::{State, Query}};
use serde::Deserialize;

use crate::interfaces::rest::ApiState;

#[derive(Deserialize)]
pub struct ExpandParams {
    pub entity: Option<String>,
    #[allow(dead_code)]
    pub limit: Option<usize>,
}

/// GET /api/graph — knowledge graph state overview.
pub async fn get_graph(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "entities": [],
        "relations": [],
        "entity_count": 0,
        "relation_count": 0,
        "note": "Graph field state available via field introspection",
    }))
}

/// GET /api/graph/sources — entity counts by source.
pub async fn graph_sources(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "sources": {},
        "note": "Source tracking coming in next release",
    }))
}

/// GET /api/graph/expand — expand entity connections.
pub async fn expand_entity(
    State(_state): State<ApiState>,
    Query(params): Query<ExpandParams>,
) -> Json<serde_json::Value> {
    let entity = params.entity.unwrap_or_else(|| "unknown".to_string());
    Json(serde_json::json!({
        "entity": entity,
        "relations": [],
        "note": "Entity expansion from graph field coming in next release",
    }))
}
