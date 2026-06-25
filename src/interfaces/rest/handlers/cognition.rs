use axum::{Json, extract::State};

use crate::interfaces::rest::ApiState;

/// GET /api/cognition/meta — principles, assumptions, mental models.
pub async fn meta(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "principles": [],
        "assumptions": [],
        "mental_models": [],
        "note": "Cognition meta from reflection processor coming soon",
    }))
}

/// GET /api/cognition/reflection — reflection reports.
pub async fn reflection(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "reports": [],
        "note": "Reflection processor not yet implemented",
    }))
}

/// GET /api/cognition/narrative — generated narratives.
pub async fn narrative(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "narratives": [],
        "note": "Narrative state from narrative processor coming soon",
    }))
}
