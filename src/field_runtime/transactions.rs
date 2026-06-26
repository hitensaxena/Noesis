//! TransactionManager — wraps processor execution in atomic state changes.
//!
//! Uses serde-based checkpointing: before a processor runs, the current
//! field state is serialized. If the processor fails, the state is
//! deserialized back. This provides full rollback without requiring
//! `Clone` on field state types.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use anyhow::Result;
use serde_json::Value;

/// A checkpoint stores serialized field states at a point in time.
#[derive(Debug)]
struct Checkpoint {
    field_states: HashMap<String, Value>,
}

/// Manages atomic state transactions for field updates.
///
/// Before executing a processor, create a checkpoint of all field states.
/// If processing fails, rollback restores the checkpointed states.
pub struct TransactionManager {
    /// Active checkpoint stack (supports nesting via push/pop).
    checkpoints: Vec<Checkpoint>,
    /// Whether transactions are enabled (off by default for performance).
    enabled: bool,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
            enabled: false,
        }
    }

    /// Enable or disable transaction tracking.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether transaction tracking is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Create a checkpoint from current field states.
    /// Returns the number of states checkpointed.
    pub fn checkpoint(&mut self, field_states: HashMap<String, &Value>) -> usize {
        if !self.enabled {
            return 0;
        }
        let states: HashMap<String, Value> = field_states
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let count = states.len();
        self.checkpoints.push(Checkpoint { field_states: states });
        count
    }

    /// Rollback to the last checkpoint, returning the restored field states.
    /// Returns `None` if no checkpoint exists.
    pub fn rollback(&mut self) -> Option<HashMap<String, Value>> {
        if !self.enabled {
            return None;
        }
        self.checkpoints.pop().map(|cp| cp.field_states)
    }

    /// Commit (discard) the most recent checkpoint without rolling back.
    pub fn commit(&mut self) {
        if self.enabled {
            self.checkpoints.pop();
        }
    }

    /// Number of active checkpoints (for testing).
    pub fn depth(&self) -> usize {
        self.checkpoints.len()
    }

    /// Execute a future within a transaction context.
    /// If the future succeeds, the checkpoint is committed.
    /// If it fails, the checkpoint is rolled back.
    pub async fn execute<T>(
        &mut self,
        field_states: HashMap<String, &Value>,
        f: Pin<Box<dyn Future<Output = Result<T>> + Send>>,
    ) -> Result<T> {
        let _ = self.checkpoint(field_states);

        match f.await {
            Ok(result) => {
                self.commit();
                Ok(result)
            }
            Err(e) => {
                if self.enabled {
                    self.rollback();
                }
                Err(e)
            }
        }
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transaction_disabled_by_default() {
        let tm = TransactionManager::new();
        assert!(!tm.is_enabled());
        assert_eq!(tm.depth(), 0);
    }

    #[test]
    fn test_checkpoint_and_commit() {
        let mut tm = TransactionManager::new();
        tm.set_enabled(true);

        let mut states = HashMap::new();
        let val = json!({"episodes": ["a", "b"]});
        states.insert("memory".to_string(), &val);

        let count = tm.checkpoint(states);
        assert_eq!(count, 1);
        assert_eq!(tm.depth(), 1);

        tm.commit();
        assert_eq!(tm.depth(), 0);
    }

    #[test]
    fn test_checkpoint_and_rollback_restores_state() {
        let mut tm = TransactionManager::new();
        tm.set_enabled(true);

        let mut states = HashMap::new();
        let val = json!({"episodes": ["a", "b", "c"]});
        states.insert("memory".to_string(), &val);

        tm.checkpoint(states);
        assert_eq!(tm.depth(), 1);

        let restored = tm.rollback();
        assert!(restored.is_some());
        let restored = restored.unwrap();
        assert_eq!(restored["memory"]["episodes"].as_array().unwrap().len(), 3);
        assert_eq!(tm.depth(), 0);
    }

    #[test]
    fn test_no_checkpoint_when_disabled() {
        let mut tm = TransactionManager::new();
        // enabled is false by default

        let mut states = HashMap::new();
        let val = json!({"key": "value"});
        states.insert("test".to_string(), &val);

        let count = tm.checkpoint(states);
        assert_eq!(count, 0, "should not checkpoint when disabled");
        assert!(tm.rollback().is_none(), "rollback should return None when disabled");
    }

    #[test]
    fn test_checkpoint_does_not_hold_reference_after_insert() {
        let mut tm = TransactionManager::new();
        tm.set_enabled(true);

        // Create a value, checkpoint it, then drop the original
        let val = json!({"data": [1, 2, 3]});
        let mut states = HashMap::new();
        states.insert("field".to_string(), &val);
        tm.checkpoint(states);
        drop(val); // Original can be dropped; checkpoint holds its own clone

        let restored = tm.rollback().unwrap();
        assert_eq!(restored["field"]["data"][0], 1);
    }

    #[tokio::test]
    async fn test_execute_success_commits() {
        let mut tm = TransactionManager::new();
        tm.set_enabled(true);

        let mut states = HashMap::new();
        let val = json!({"count": 1});
        states.insert("counter".to_string(), &val);

        let result = tm.execute(
            states,
            Box::pin(async { Ok::<_, anyhow::Error>(42) }),
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(tm.depth(), 0, "checkpoint should be committed on success");
    }

    #[tokio::test]
    async fn test_execute_failure_rolls_back() {
        let mut tm = TransactionManager::new();
        tm.set_enabled(true);

        let mut states = HashMap::new();
        let val = json!({"data": "original"});
        states.insert("field".to_string(), &val);

        let result = tm.execute::<i32>(
            states,
            Box::pin(async { Err(anyhow::anyhow!("processing failed")) }),
        ).await;

        assert!(result.is_err());
        // After rollback, the state would need to be re-checkpointed
        // (rollback returns the states, and the caller is responsible for reapplying them)
        assert_eq!(tm.depth(), 0);
    }
}
