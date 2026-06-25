use axum::{Json, extract::State};

use crate::interfaces::rest::ApiState;

/// GET /api/identity — current identity state (beliefs, traits, self-model).
pub async fn get_identity(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "identity_version": 1,
        "beliefs": [],
        "traits": [],
        "note": "Identity field state available via field introspection",
    }))
}
