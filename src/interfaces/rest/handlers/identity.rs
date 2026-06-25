use axum::{Json, extract::State};

use crate::interfaces::rest::ApiState;

/// GET /api/identity — current identity state (beliefs, traits, self-model).
pub async fn get_identity(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let identity_state = state.field_cache.get("identity");
    if let Some(state_val) = identity_state {
        Json(serde_json::json!({
            "identity": state_val.value(),
        }))
    } else {
        Json(serde_json::json!({
            "identity_version": 0,
            "beliefs": [],
            "traits": [],
            "note": "No identity state cached yet",
        }))
    }
}
