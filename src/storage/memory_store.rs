use async_trait::async_trait;
use anyhow::Result;
use dashmap::DashMap;
use serde_json::Value;

use super::store::Storage;

/// In-memory storage implementation using DashMap.
#[derive(Debug, Default)]
pub struct MemoryStore {
    data: DashMap<String, DashMap<String, Value>>,
}

#[async_trait]
impl Storage for MemoryStore {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Value>> {
        Ok(self
            .data
            .get(namespace)
            .and_then(|ns| ns.get(key).map(|v| v.clone())))
    }

    async fn set(&self, namespace: &str, key: &str, value: Value) -> Result<()> {
        self.data
            .entry(namespace.to_string())
            .or_insert_with(DashMap::new)
            .insert(key.to_string(), value);
        Ok(())
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        if let Some(ns) = self.data.get(namespace) {
            ns.remove(key);
        }
        Ok(())
    }

    async fn list(&self, namespace: &str) -> Result<Vec<String>> {
        Ok(self
            .data
            .get(namespace)
            .map(|ns| ns.iter().map(|e| e.key().clone()).collect())
            .unwrap_or_default())
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}
