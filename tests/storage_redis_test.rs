//! Redis storage backend integration tests.
//!
//! Requires a running Redis instance (default: localhost:6379).
//! Set NOESIS_TEST_REDIS_URL to override, or skip if no Redis is available.

use std::sync::Arc;

/// Helper: try to create a RedisBackend from env or default config.
async fn try_connect() -> Option<Arc<dyn noesis::storage::store::Storage + Send + Sync>> {
    let redis_url = std::env::var("NOESIS_TEST_REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    match noesis::storage::backends::redis_backend::RedisBackend::connect(&redis_url, "noesis_test:").await {
        Ok(redis) => Some(Arc::new(redis) as Arc<dyn noesis::storage::store::Storage + Send + Sync>),
        Err(e) => {
            eprintln!("[SKIP] No Redis available: {}. Set NOESIS_TEST_REDIS_URL to test.", e);
            None
        }
    }
}

#[tokio::test]
async fn test_redis_set_get() {
    let storage = match try_connect().await {
        Some(s) => s,
        None => return,
    };

    let value = serde_json::json!({"hello": "redis", "num": 99});
    storage.set("test_ns", "test_key", value.clone()).await.unwrap();

    let retrieved = storage.get("test_ns", "test_key").await.unwrap();
    assert!(retrieved.is_some(), "should find the stored value");
    assert_eq!(retrieved.unwrap(), value);

    storage.delete("test_ns", "test_key").await.unwrap();
}

#[tokio::test]
async fn test_redis_get_missing() {
    let storage = match try_connect().await {
        Some(s) => s,
        None => return,
    };

    let result = storage.get("test_ns", "nonexistent_redis_key").await.unwrap();
    assert!(result.is_none(), "nonexistent key should return None");
}

#[tokio::test]
async fn test_redis_delete() {
    let storage = match try_connect().await {
        Some(s) => s,
        None => return,
    };

    storage.set("test_ns", "del_redis", serde_json::json!("delete_me")).await.unwrap();
    let retrieved = storage.get("test_ns", "del_redis").await.unwrap();
    assert!(retrieved.is_some(), "should exist before delete");

    storage.delete("test_ns", "del_redis").await.unwrap();
    let after = storage.get("test_ns", "del_redis").await.unwrap();
    assert!(after.is_none(), "should be gone after delete");
}

#[tokio::test]
async fn test_redis_overwrite() {
    let storage = match try_connect().await {
        Some(s) => s,
        None => return,
    };

    storage.set("test_ns", "ov_redis", serde_json::json!("v1")).await.unwrap();
    storage.set("test_ns", "ov_redis", serde_json::json!("v2")).await.unwrap();

    let retrieved = storage.get("test_ns", "ov_redis").await.unwrap().unwrap();
    assert_eq!(retrieved, serde_json::json!("v2"), "should return the latest value");

    storage.delete("test_ns", "ov_redis").await.unwrap();
}
