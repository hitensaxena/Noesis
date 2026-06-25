use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;
use tracing;

use super::super::store::Storage;

/// Redis storage backend.
///
/// Connects to the existing curlyos-core Redis container (:6379).
/// Uses namespaced keys (namespace:key) for isolation.
pub struct RedisBackend {
    client: redis::aio::ConnectionManager,
    namespace_prefix: String,
}

impl std::fmt::Debug for RedisBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisBackend")
            .field("namespace_prefix", &self.namespace_prefix)
            .finish()
    }
}

impl RedisBackend {
    /// Connect to Redis.
    ///
    /// # Arguments
    /// * `url` - Redis URL (e.g., "redis://127.0.0.1:6379")
    /// * `namespace_prefix` - Key prefix for isolation (e.g., "noesis:")
    pub async fn connect(url: &str, namespace_prefix: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let mgr = redis::aio::ConnectionManager::new(client).await?;
        tracing::info!("[RedisBackend] connected to {}", url);
        Ok(Self {
            client: mgr,
            namespace_prefix: namespace_prefix.to_string(),
        })
    }

    fn prefixed(&self, namespace: &str, key: &str) -> String {
        format!("{}{}:{}", self.namespace_prefix, namespace, key)
    }
}

#[async_trait]
impl Storage for RedisBackend {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Value>> {
        let redis_key = self.prefixed(namespace, key);
        let mut conn = self.client.clone();
        let result: Option<String> = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await?;
        match result {
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
            None => Ok(None),
        }
    }

    async fn set(&self, namespace: &str, key: &str, value: Value) -> Result<()> {
        let redis_key = self.prefixed(namespace, key);
        let mut conn = self.client.clone();
        let json_str = serde_json::to_string(&value)?;
        redis::cmd("SET")
            .arg(&redis_key)
            .arg(&json_str)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        let redis_key = self.prefixed(namespace, key);
        let mut conn = self.client.clone();
        redis::cmd("DEL")
            .arg(&redis_key)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    async fn list(&self, _namespace: &str) -> Result<Vec<String>> {
        // Redis requires SCAN for key listing — skip for now
        Ok(Vec::new())
    }
}
