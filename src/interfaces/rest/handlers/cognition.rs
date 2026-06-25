use axum::{Json, extract::State};

use crate::interfaces::rest::ApiState;

/// GET /api/cognition/meta — principles, assumptions, mental models.
pub async fn meta(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    // Read from relevant field caches and compose a meta view
    let identity_state = state.field_cache.get("identity");
    let executive_state = state.field_cache.get("executive");

    let beliefs = identity_state
        .as_ref()
        .and_then(|v| v.value().get("beliefs").cloned())
        .unwrap_or(serde_json::json!([]));
    let goals = executive_state
        .as_ref()
        .and_then(|v| v.value().get("goals").cloned())
        .unwrap_or(serde_json::json!([]));

    Json(serde_json::json!({
        "principles": [],
        "assumptions": [],
        "beliefs": beliefs,
        "goals": goals,
        "note": "Cognition meta composed from field cache — full reflection processor output coming in next release",
    }))
}

/// GET /api/cognition/reflection — reflection reports.
pub async fn reflection(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let identity_state = state.field_cache.get("identity");
    let version = identity_state
        .as_ref()
        .and_then(|v| v.value().get("identity_version").cloned())
        .unwrap_or(serde_json::json!(0));

    Json(serde_json::json!({
        "reports": [],
        "identity_version": version,
        "note": "Reflection processor active — reports displayed via signal cascade. Structured report output coming next release.",
    }))
}

/// GET /api/cognition/narrative — generated narratives.
pub async fn narrative(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let memory_state = state.field_cache.get("memory");
    let episode_count = memory_state
        .as_ref()
        .and_then(|v| v.value().get("episode_count").cloned())
        .unwrap_or(serde_json::json!(0));

    Json(serde_json::json!({
        "narratives": [],
        "episode_count": episode_count,
        "note": "Narrative state from narrative processor coming next release",
    }))
}
