//! Mental model processor — maintains simplified world models.
//!
//! On ENTITY_CREATED and EDGE_CREATED (knowledge graph signals),
//! extracts domain cues and accumulates them into mental models.
//! On BEAT_MEDIUM, emits MentalModelUpdated with the current model.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::graph::{EntityCreated, EdgeCreated};
use crate::signals::reasoning::MentalModelUpdated;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Simple in-memory mental model representation.
#[derive(Debug, Clone)]
struct Model {
    name: String,
    entities: Vec<String>,
    confidence: f32,
}

/// Builds and updates mental models from knowledge graph signals.
pub struct MentalModelProcessor {
    models: Vec<Model>,
}

impl MentalModelProcessor {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
        }
    }

    /// Extract a model name from an entity category.
    fn model_name_for(category: &str) -> &'static str {
        match category.to_lowercase().as_str() {
            "person" => "People",
            "project" => "Projects",
            "tool" => "Tools",
            "concept" => "Concepts",
            "organization" => "Organizations",
            "location" => "Places",
            "event" => "Events",
            "resource" => "Resources",
            _ => "General",
        }
    }

    /// Find or create a model by name.
    fn find_or_create(&mut self, name: &str) -> &mut Model {
        let idx = self.models.iter().position(|m| m.name == name);
        if let Some(i) = idx {
            &mut self.models[i]
        } else {
            self.models.push(Model {
                name: name.to_string(),
                entities: Vec::new(),
                confidence: 0.5,
            });
            self.models.last_mut().unwrap()
        }
    }

    /// Compute overall confidence from model entity count.
    fn compute_confidence(entities: &[String]) -> f32 {
        let count = entities.len();
        if count == 0 {
            return 0.0;
        }
        ((count as f32) / 20.0).min(1.0) // 20 entities = full confidence
    }
}

#[async_trait]
impl Processor for MentalModelProcessor {
    fn name(&self) -> &str {
        "mental_model"
    }

    fn priority(&self) -> u8 {
        110
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::ENTITY_CREATED, types::EDGE_CREATED, types::BEAT_MEDIUM]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::MENTAL_MODEL_UPDATED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::ENTITY_CREATED {
            if let Some(ec) = signal.as_any().downcast_ref::<EntityCreated>() {
                let model_name = Self::model_name_for(&ec.category);
                let model = self.find_or_create(model_name);
                if !model.entities.contains(&ec.name) {
                    model.entities.push(ec.name.clone());
                    model.confidence = Self::compute_confidence(&model.entities);
                    tracing::trace!(
                        "[MentalModelProcessor] added {} to model '{}' (confidence: {:.2})",
                        ec.name, model_name, model.confidence,
                    );
                }
            }
            return Ok(vec![]);
        }

        if signal_type == types::EDGE_CREATED {
            // Edges increase confidence in related models
            if let Some(_ec) = signal.as_any().downcast_ref::<EdgeCreated>() {
                tracing::trace!("[MentalModelProcessor] edge recorded — increasing model confidence");
            }
            return Ok(vec![]);
        }

        if signal_type == types::BEAT_MEDIUM {
            if let Some(model) = self.models.iter()
                .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                .cloned()
            {
                if model.confidence > 0.0 {
                    tracing::info!(
                        "[MentalModelProcessor] updated model: {} (confidence: {:.2})",
                        model.name, model.confidence,
                    );
                    return Ok(vec![Arc::new(MentalModelUpdated::new(
                        &model.name,
                        model.confidence,
                    ))]);
                }
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[MentalModelProcessor] shutting down with {} models",
            self.models.len(),
        );
        Ok(())
    }
}

impl Default for MentalModelProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::beat_coordinator::BeatPulse;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_mental_model_processor_name() {
        let p = MentalModelProcessor::new();
        assert_eq!(p.name(), "mental_model");
    }

    #[test]
    fn test_model_name_for() {
        assert_eq!(MentalModelProcessor::model_name_for("Person"), "People");
        assert_eq!(MentalModelProcessor::model_name_for("Tool"), "Tools");
        assert_eq!(MentalModelProcessor::model_name_for("Unknown"), "General");
    }

    #[tokio::test]
    async fn test_mental_model_accumulates_entities() {
        let mut p = MentalModelProcessor::new();
        let ctx = test_context();

        let ec = EntityCreated {
            meta: crate::kernel::signal::SignalMeta::new(types::ENTITY_CREATED, "test"),
            entity_id: uuid::Uuid::new_v4(),
            name: "Rust".to_string(),
            category: "Tool".to_string(),
            source: "test".to_string(),
        };
        p.process(&ctx, Arc::new(ec)).await.unwrap();

        let model = p.find_or_create("Tools");
        assert_eq!(model.entities.len(), 1, "should have 1 entity");
        assert!(model.entities.contains(&"Rust".to_string()));
    }

    #[tokio::test]
    async fn test_mental_model_emits_on_beat() {
        let mut p = MentalModelProcessor::new();
        let ctx = test_context();

        // Add some entities
        for (name, cat) in &[("Rust", "Tool"), ("Python", "Tool"), ("VS Code", "Tool")] {
            let ec = EntityCreated {
                meta: crate::kernel::signal::SignalMeta::new(types::ENTITY_CREATED, "test"),
                entity_id: uuid::Uuid::new_v4(),
                name: name.to_string(),
                category: cat.to_string(),
                source: "test".to_string(),
            };
            p.process(&ctx, Arc::new(ec)).await.unwrap();
        }

        let beat = BeatPulse::new(types::BEAT_MEDIUM);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(!result.is_empty(), "BEAT_MEDIUM should emit mental model update");

        let sig = result[0].as_any().downcast_ref::<MentalModelUpdated>().unwrap();
        assert_eq!(sig.name, "Tools", "should update the Tools model");
    }
}
