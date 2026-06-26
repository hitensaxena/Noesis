//! Postgres storage backend integration tests.
//!
//! Requires a running Postgres instance (default: curlyos-core on :54321).
//! Set NOESIS_TEST_PG_URL to override, or skip if no Postgres is available.

use std::sync::Arc;

/// Helper: try to create a PostgresBackend from env or default config.
async fn try_connect() -> Option<Arc<dyn noesis::storage::store::Storage + Send + Sync>> {
    let pg_url = std::env::var("NOESIS_TEST_PG_URL").ok();
    let config: tokio_postgres::Config = match pg_url {
        Some(url) => url.parse().ok()?,
        None => {
            let mut c = tokio_postgres::Config::new();
            c.host("127.0.0.1").port(54321).dbname("curlyos");
            c.user("curlyos").password("curlyos");
            c
        }
    };

    match noesis::storage::backends::postgres_backend::PostgresBackend::connect(&config).await {
        Ok(pg) => Some(Arc::new(pg) as Arc<dyn noesis::storage::store::Storage + Send + Sync>),
        Err(e) => {
            eprintln!("[SKIP] No Postgres available: {}. Set NOESIS_TEST_PG_URL to test.", e);
            None
        }
    }
}

#[tokio::test]
async fn test_pg_set_get() {
    let storage = match try_connect().await {
        Some(s) => s,
        None => return, // skip
    };

    let value = serde_json::json!({"hello": "world", "count": 42});
    storage.set("test_ns", "test_key", value.clone()).await.unwrap();

    let retrieved = storage.get("test_ns", "test_key").await.unwrap();
    assert!(retrieved.is_some(), "should find the stored value");
    assert_eq!(retrieved.unwrap(), value);

    // Cleanup
    storage.delete("test_ns", "test_key").await.unwrap();
}

#[tokio::test]
async fn test_pg_get_missing() {
    let storage = match try_connect().await {
        Some(s) => s,
        None => return,
    };

    let result = storage.get("test_ns", "nonexistent_key").await.unwrap();
    assert!(result.is_none(), "nonexistent key should return None");
}

#[tokio::test]
async fn test_pg_delete() {
    let storage = match try_connect().await {
        Some(s) => s,
        None => return,
    };

    storage.set("test_ns", "del_key", serde_json::json!("to_delete")).await.unwrap();
    let retrieved = storage.get("test_ns", "del_key").await.unwrap();
    assert!(retrieved.is_some(), "should exist before delete");

    storage.delete("test_ns", "del_key").await.unwrap();
    let after = storage.get("test_ns", "del_key").await.unwrap();
    assert!(after.is_none(), "should be gone after delete");
}

#[tokio::test]
async fn test_pg_list() {
    let storage = match try_connect().await {
        Some(s) => s,
        None => return,
    };

    // Clean up any leftover test keys
    for key in &["list_a", "list_b", "list_c"] {
        let _ = storage.delete("list_ns", key).await;
    }

    storage.set("list_ns", "list_a", serde_json::json!("a")).await.unwrap();
    storage.set("list_ns", "list_b", serde_json::json!("b")).await.unwrap();
    storage.set("list_ns", "list_c", serde_json::json!("c")).await.unwrap();

    let keys = storage.list("list_ns").await.unwrap();
    assert!(keys.contains(&"list_a".to_string()), "should contain list_a");
    assert!(keys.contains(&"list_b".to_string()), "should contain list_b");
    assert!(keys.contains(&"list_c".to_string()), "should contain list_c");

    // Cleanup
    for key in &["list_a", "list_b", "list_c"] {
        storage.delete("list_ns", key).await.unwrap();
    }
}

#[tokio::test]
async fn test_pg_overwrite() {
    let storage = match try_connect().await {
        Some(s) => s,
        None => return,
    };

    storage.set("test_ns", "overwrite_key", serde_json::json!("original")).await.unwrap();
    storage.set("test_ns", "overwrite_key", serde_json::json!("updated")).await.unwrap();

    let retrieved = storage.get("test_ns", "overwrite_key").await.unwrap().unwrap();
    assert_eq!(retrieved, serde_json::json!("updated"), "should return the latest value");

    storage.delete("test_ns", "overwrite_key").await.unwrap();
}
