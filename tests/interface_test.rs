//! Interface integration tests for REST endpoints.
//!
//! Uses axum's `oneshot` pattern for testing without spawning a server.

use std::sync::Arc;

use noesis::kernel::bus::EventBus;
use noesis::kernel::state::SystemState;
use noesis::kernel::metrics::MetricsCollector;
use noesis::kernel::capabilities::CapabilityRegistry;
use noesis::kernel::plugin::PluginRegistry;
use noesis::interfaces::rest::{self, ApiState};

use axum::body::Body;
use axum::http::{Request, StatusCode, Method};
use tower::ServiceExt;

fn test_api_state() -> ApiState {
    let event_bus = Arc::new(EventBus::new());
    let metrics = Arc::new(MetricsCollector::new());
    let capabilities = Arc::new(CapabilityRegistry::new());
    let field_cache = Arc::new(dashmap::DashMap::new());
    let system_state = Arc::new(SystemState::new(field_cache.clone()));

    let kernel_snapshot = noesis::interfaces::rest::KernelSnapshot {
        fields: vec!["memory".into(), "identity".into()],
        processors: vec!["episode".into()],
        signal_types: vec![],
    };

    ApiState::new(event_bus, metrics, kernel_snapshot, system_state, field_cache, capabilities, Arc::new(PluginRegistry::new()))
}

#[tokio::test]
async fn test_health_endpoint() {
    let _guard = AUTH_LOCK.lock().await;
    let prev = std::env::var("NOESIS_API_KEY").ok();
    std::env::remove_var("NOESIS_API_KEY");

    let app = rest::router(test_api_state());
    let resp = app
        .oneshot(Request::builder().uri("/api/health").method(Method::GET).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    if let Some(k) = prev { std::env::set_var("NOESIS_API_KEY", k); }
}

#[tokio::test]
async fn test_signal_types_endpoint() {
    let _guard = AUTH_LOCK.lock().await;
    let prev = std::env::var("NOESIS_API_KEY").ok();
    std::env::remove_var("NOESIS_API_KEY");

    let app = rest::router(test_api_state());
    let resp = app
        .oneshot(Request::builder().uri("/api/signals").method(Method::GET).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    if let Some(k) = prev { std::env::set_var("NOESIS_API_KEY", k); }
}

#[tokio::test]
async fn test_capabilities_endpoint() {
    let _guard = AUTH_LOCK.lock().await;
    let prev = std::env::var("NOESIS_API_KEY").ok();
    std::env::remove_var("NOESIS_API_KEY");

    let app = rest::router(test_api_state());
    let resp = app
        .oneshot(Request::builder().uri("/api/capabilities").method(Method::GET).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    if let Some(k) = prev { std::env::set_var("NOESIS_API_KEY", k); }
}

#[tokio::test]
async fn test_ingest_endpoint() {
    let _guard = AUTH_LOCK.lock().await;
    let prev = std::env::var("NOESIS_API_KEY").ok();
    std::env::remove_var("NOESIS_API_KEY");

    let app = rest::router(test_api_state());
    let body = serde_json::json!({"text": "Test ingestion via interface test", "source": "test"});
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/ingest")
                .method(Method::POST)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    if let Some(k) = prev { std::env::set_var("NOESIS_API_KEY", k); }
}

// ---- Auth middleware tests ----

/// Serialise auth tests — env vars are process-global.
/// Uses tokio::sync::Mutex so the guard can be held across .await.
static AUTH_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

/// Helper: run a request with NOESIS_API_KEY = "test-key-123".
/// Holds AUTH_LOCK for the entire duration to prevent concurrent env var manipulation.
async fn authed_request(uri: &str, auth_header: Option<(&str, &str)>) -> StatusCode {
    let _guard = AUTH_LOCK.lock().await;
    let prev = std::env::var("NOESIS_API_KEY").ok();
    std::env::set_var("NOESIS_API_KEY", "test-key-123");

    let app = rest::router(test_api_state());
    let mut builder = Request::builder().uri(uri).method(Method::GET);
    if let Some((name, value)) = auth_header {
        builder = builder.header(name, value);
    }
    let resp = app.oneshot(builder.body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();

    match prev {
        Some(k) => std::env::set_var("NOESIS_API_KEY", k),
        None => std::env::remove_var("NOESIS_API_KEY"),
    }
    status
}

#[tokio::test]
async fn test_auth_valid_bearer_token() {
    let status = authed_request("/api/health", Some(("Authorization", "Bearer test-key-123"))).await;
    assert_eq!(status, StatusCode::OK, "valid Bearer token should pass auth");
}

#[tokio::test]
async fn test_auth_valid_api_key_header() {
    let status = authed_request("/api/health", Some(("X-API-Key", "test-key-123"))).await;
    assert_eq!(status, StatusCode::OK, "valid X-API-Key should pass auth");
}

#[tokio::test]
async fn test_auth_invalid_key() {
    let status = authed_request("/api/health", Some(("Authorization", "Bearer wrong-key"))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "wrong key should return 401");
}

#[tokio::test]
async fn test_auth_missing_key() {
    let status = authed_request("/api/health", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "missing key should return 401 when NOESIS_API_KEY is set");
}

#[tokio::test]
async fn test_auth_disabled_when_no_env() {
    let _guard = AUTH_LOCK.lock().await;
    let prev = std::env::var("NOESIS_API_KEY").ok();
    std::env::remove_var("NOESIS_API_KEY");

    let app = rest::router(test_api_state());
    let resp = app
        .oneshot(Request::builder().uri("/api/health").method(Method::GET).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "no auth key configured → open access");

    match prev {
        Some(k) => std::env::set_var("NOESIS_API_KEY", k),
        None => {}
    }
}
