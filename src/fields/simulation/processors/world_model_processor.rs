//! World model processor — maintains an entity→state view of the world.
//!
//! On EPISODE_RECORDED, extracts domain cues to build a simple world model.
//! On BEAT_SLOW, emits WorldModelUpdated with the current state estimate.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::EpisodeRecorded;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Named domains used in world model building.
const DOMAIN_KEYWORDS: &[(&str, &str)] = &[
    ("rust", "Systems Programming"),
    ("coding", "Software Development"),
    ("cook", "Culinary"),
    ("running", "Fitness"),
    ("trading", "Financial Markets"),
    ("reading", "Learning"),
    ("writing", "Communication"),
    ("research", "Analysis"),
];

/// A simple world model snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldModelUpdated {
    pub meta: crate::kernel::signal::SignalMeta,
    pub model_id: uuid::Uuid,
    pub name: String,
    pub domains: Vec<String>,
    pub confidence: f32,
}

impl WorldModelUpdated {
    pub fn new(name: &str, domains: Vec<String>, confidence: f32) -> Self {
        Self {
            meta: crate::kernel::signal::SignalMeta::new(types::WORLD_MODEL_UPDATED, "simulation::world_model"),
            model_id: uuid::Uuid::new_v4(),
            name: name.to_string(),
            domains,
            confidence,
        }
    }
}

crate::signals::signal_impl!(WorldModelUpdated, WORLD_MODEL_UPDATED, "simulation::world_model");

/// Builds and maintains a world model from episode content.
pub struct WorldModelProcessor {
    domains: std::collections::HashMap<String, usize>,
    episode_count: usize,
}

impl WorldModelProcessor {
    pub fn new() -> Self {
        Self {
            domains: std::collections::HashMap::new(),
            episode_count: 0,
        }
    }

    fn extract_domains(text: &str) -> Vec<String> {
        let lower = text.to_lowercase();
        DOMAIN_KEYWORDS
            .iter()
            .filter(|(keyword, _)| lower.contains(keyword))
            .map(|(_, name)| name.to_string())
            .collect()
    }
}

#[async_trait]
impl Processor for WorldModelProcessor {
    fn name(&self) -> &str {
        "world_model"
    }

    fn priority(&self) -> u8 {
        120
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::WORLD_MODEL_UPDATED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::EPISODE_RECORDED {
            if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
                self.episode_count += 1;
                let domains = Self::extract_domains(&ep.content);

                for domain in domains {
                    *self.domains.entry(domain).or_insert(0) += 1;
                }

                // Emit updated model
                let dominant_domains: Vec<String> = {
                    let mut d: Vec<(String, usize)> = self.domains
                        .iter()
                        .map(|(k, v)| (k.clone(), *v))
                        .collect();
                    d.sort_by(|a, b| b.1.cmp(&a.1));
                    d.truncate(3);
                    d.into_iter().map(|(name, _)| name).collect()
                };

                let total_domains = self.domains.len();
                let confidence = ((self.episode_count as f32) / 30.0).min(1.0);

                if !dominant_domains.is_empty() {
                    let model_name = format!("WorldModel-v{}", self.episode_count);
                    tracing::info!(
                        "[WorldModel] updated: {} domains (top: {:?})",
                        total_domains, dominant_domains,
                    );
                    return Ok(vec![Arc::new(WorldModelUpdated::new(
                        &model_name,
                        dominant_domains,
                        confidence,
                    ))]);
                }
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[WorldModel] shutting down with {} domains", self.domains.len());
        Ok(())
    }
}

impl Default for WorldModelProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_world_model_processor_name() {
        let p = WorldModelProcessor::new();
        assert_eq!(p.name(), "world_model");
    }

    #[test]
    fn test_extract_domains() {
        let domains = WorldModelProcessor::extract_domains(
            "Working on Rust programming project and cooking dinner",
        );
        assert!(domains.contains(&"Systems Programming".to_string()));
        assert!(domains.contains(&"Culinary".to_string()));
    }

    #[tokio::test]
    async fn test_world_model_emits_on_episode() {
        let mut p = WorldModelProcessor::new();
        let ctx = test_context();

        let ep = EpisodeRecorded::new(
            "Working on the Rust compiler project today",
            "test", vec![],
        );
        let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
        assert!(!result.is_empty(), "should emit WorldModelUpdated");

        let sig = result[0].as_any().downcast_ref::<WorldModelUpdated>().unwrap();
        assert!(!sig.domains.is_empty(), "should have domains");
    }
}
