pub mod redis_backend;
pub mod postgres_backend;

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;

use super::store::Storage;
use super::memory_store::MemoryStore;

/// Composite storage that tries real backends first, falls back to memory.
///
/// Mirrors how curlyos-core uses Postgres as source of truth with
/// Redis as cache/working memory, and Noesis MemoryStore as fallback.
pub struct CompositeStorage {
    pub memory: MemoryStore,
    pub redis: Option<Arc<dyn Storage + Send + Sync>>,
    pub postgres: Option<Arc<dyn Storage + Send + Sync>>,
}

impl CompositeStorage {
    pub fn new() -> Self {
        Self {
            memory: MemoryStore::new(),
            redis: None,
            postgres: None,
        }
    }

    pub fn with_redis(mut self, backend: Arc<dyn Storage + Send + Sync>) -> Self {
        self.redis = Some(backend);
        self
    }

    pub fn with_postgres(mut self, backend: Arc<dyn Storage + Send + Sync>) -> Self {
        self.postgres = Some(backend);
        self
    }

    /// Read from postgres → redis → memory in priority order.
    pub async fn get(&self, namespace: &str, key: &str) -> Result<Option<Value>> {
        if let Some(ref pg) = self.postgres {
            if let Some(val) = pg.get(namespace, key).await? {
                return Ok(Some(val));
            }
        }
        if let Some(ref r) = self.redis {
            if let Some(val) = r.get(namespace, key).await? {
                return Ok(Some(val));
            }
        }
        self.memory.get(namespace, key).await
    }

    /// Write to all available backends.
    pub async fn set(&self, namespace: &str, key: &str, value: Value) -> Result<()> {
        if let Some(ref pg) = self.postgres {
            let _ = pg.set(namespace, key, value.clone()).await;
        }
        if let Some(ref r) = self.redis {
            let _ = r.set(namespace, key, value.clone()).await;
        }
        self.memory.set(namespace, key, value).await
    }

    /// Delete from all backends.
    pub async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        if let Some(ref pg) = self.postgres {
            let _ = pg.delete(namespace, key).await;
        }
        if let Some(ref r) = self.redis {
            let _ = r.delete(namespace, key).await;
        }
        self.memory.delete(namespace, key).await
    }

    /// List keys from memory store (primary listing source).
    pub async fn list(&self, namespace: &str) -> Result<Vec<String>> {
        self.memory.list(namespace).await
    }

    pub async fn is_postgres_connected(&self) -> bool {
        self.postgres.is_some()
    }

    pub async fn is_redis_connected(&self) -> bool {
        self.redis.is_some()
    }
}

impl Default for CompositeStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Storage for CompositeStorage {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Value>> {
        CompositeStorage::get(self, namespace, key).await
    }

    async fn set(&self, namespace: &str, key: &str, value: Value) -> Result<()> {
        CompositeStorage::set(self, namespace, key, value).await
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        CompositeStorage::delete(self, namespace, key).await
    }

    async fn list(&self, namespace: &str) -> Result<Vec<String>> {
        CompositeStorage::list(self, namespace).await
    }
}

impl std::fmt::Debug for CompositeStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeStorage")
            .field("has_redis", &self.redis.is_some())
            .field("has_postgres", &self.postgres.is_some())
            .finish()
    }
}
