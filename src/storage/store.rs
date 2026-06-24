use std::fmt::Debug;
use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;

/// Abstraction over the backing store.
///
/// Can be swapped between in-memory (default), Postgres, Redis, etc.
#[async_trait]
pub trait Storage: Send + Sync + Debug {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Value>>;
    async fn set(&self, namespace: &str, key: &str, value: Value) -> Result<()>;
    async fn delete(&self, namespace: &str, key: &str) -> Result<()>;
    async fn list(&self, namespace: &str) -> Result<Vec<String>>;
}
