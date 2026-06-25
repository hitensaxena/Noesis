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
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let graph_state = state.field_cache.get("knowledge_graph");
    if let Some(state_val) = graph_state {
        Json(serde_json::json!({
            "graph": state_val.value(),
        }))
    } else {
        Json(serde_json::json!({
            "entities": [],
            "relations": [],
            "entity_count": 0,
            "relation_count": 0,
            "note": "No graph state cached yet — inject an experience first",
        }))
    }
}

/// GET /api/graph/sources — entity counts by source.
pub async fn graph_sources(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let graph_state = state.field_cache.get("knowledge_graph");
    if let Some(state_val) = graph_state {
        let entities = state_val.value().get("entities").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let mut sources: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for entity in &entities {
            let src = entity.get("source_episode_id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            *sources.entry(src).or_insert(0) += 1;
        }
        Json(serde_json::json!({ "sources": sources }))
    } else {
        Json(serde_json::json!({ "sources": {}, "note": "No graph data yet" }))
    }
}

/// GET /api/graph/expand — expand entity connections.
pub async fn expand_entity(
    State(state): State<ApiState>,
    Query(params): Query<ExpandParams>,
) -> Json<serde_json::Value> {
    let entity_name = params.entity.unwrap_or_else(|| "unknown".to_string());
    let graph_state = state.field_cache.get("knowledge_graph");
    if let Some(state_val) = graph_state {
        let val = state_val.value();
        let entities = val.get("entities").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let relations = val.get("relations").and_then(|v| v.as_array()).cloned().unwrap_or_default();

        // Find the entity by name
        let found_entity = entities.iter().find(|e| {
            e.get("name").and_then(|v| v.as_str()) == Some(entity_name.as_str())
        });

        if let Some(entity) = found_entity {
            let entity_id = entity.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let connected: Vec<&serde_json::Value> = relations.iter().filter(|r| {
                r.get("subject_id").and_then(|v| v.as_str()) == Some(entity_id)
                    || r.get("object_id").and_then(|v| v.as_str()) == Some(entity_id)
            }).collect();

            Json(serde_json::json!({
                "entity": entity,
                "connected_relations": connected,
                "relation_count": connected.len(),
            }))
        } else {
            Json(serde_json::json!({
                "entity": entity_name,
                "relations": [],
                "note": format!("Entity '{}' not found in graph", entity_name),
            }))
        }
    } else {
        Json(serde_json::json!({
            "entity": entity_name,
            "relations": [],
            "note": "No graph data available yet",
        }))
    }
}
