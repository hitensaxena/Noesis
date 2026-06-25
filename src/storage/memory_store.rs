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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_set_and_get() {
        let store = MemoryStore::new();
        store.set("test", "key1", json!({"value": 42})).await.unwrap();
        let val = store.get("test", "key1").await.unwrap();
        assert!(val.is_some());
        assert_eq!(val.unwrap()["value"], 42);
    }

    #[tokio::test]
    async fn test_get_missing() {
        let store = MemoryStore::new();
        let val = store.get("test", "nonexistent").await.unwrap();
        assert!(val.is_none());
    }

    #[tokio::test]
    async fn test_delete() {
        let store = MemoryStore::new();
        store.set("test", "key1", json!("value")).await.unwrap();
        store.delete("test", "key1").await.unwrap();
        let val = store.get("test", "key1").await.unwrap();
        assert!(val.is_none());
    }

    #[tokio::test]
    async fn test_list_namespace() {
        let store = MemoryStore::new();
        store.set("ns1", "a", json!(1)).await.unwrap();
        store.set("ns1", "b", json!(2)).await.unwrap();
        store.set("ns2", "c", json!(3)).await.unwrap();

        let ns1_keys = store.list("ns1").await.unwrap();
        assert_eq!(ns1_keys.len(), 2);
        assert!(ns1_keys.contains(&"a".to_string()));

        let ns2_keys = store.list("ns2").await.unwrap();
        assert_eq!(ns2_keys.len(), 1);
    }
}
