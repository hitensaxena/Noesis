use axum::Json;

/// GET /api/health
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "noesis",
        "version": "0.1.0",
        "architecture": "decentralized-signal-cascade",
    }))
}
