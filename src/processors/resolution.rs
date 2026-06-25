//! Entity resolution processor — converts extracted triples into graph entities.
//!
//! Subscribes to TriplesExtracted signals and emits EntityCreated and EdgeCreated
//! signals that the GraphField uses to build the knowledge graph.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::graph::{EntityCreated, EdgeCreated, TriplesExtracted};
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// Converts extracted triples into knowledge graph entities and edges.
///
/// For each triple in a TriplesExtracted signal:
/// 1. Creates or resolves entities (subject, object)
/// 2. Creates a relation between them
/// 3. Emits EntityCreated and EdgeCreated signals
/// 4. The GraphField receives these and builds the graph
pub struct EntityResolutionProcessor;

impl EntityResolutionProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for EntityResolutionProcessor {
    fn name(&self) -> &str {
        "resolution"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn priority(&self) -> u8 {
        120 // Runs after extraction (which has default 100)
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::TRIPLES_EXTRACTED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::ENTITY_CREATED, types::EDGE_CREATED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(extracted) = signal.as_any().downcast_ref::<TriplesExtracted>() {
            if extracted.triples.is_empty() {
                return Ok(vec![]);
            }

            tracing::info!(
                "[Resolution] resolving {} triples into graph entities",
                extracted.triples.len()
            );

            let mut emitted: Vec<SignalArc> = Vec::new();
            let mut entity_cache: std::collections::HashMap<String, uuid::Uuid> =
                std::collections::HashMap::new();

            for triple in &extracted.triples {
                // Resolve or create subject entity
                let subject_id = if let Some(id) = entity_cache.get(&triple.subject) {
                    *id
                } else {
                    let id = uuid::Uuid::new_v4();
                    let entity = EntityCreated {
                        meta: signal.meta().child(types::ENTITY_CREATED, "resolution::processor"),
                        entity_id: id,
                        name: triple.subject.clone(),
                        category: triple.subject_category.to_string(),
                        source: extracted.source_episode_id.clone().unwrap_or_default(),
                    };
                    entity_cache.insert(triple.subject.clone(), id);
                    emitted.push(Arc::new(entity));
                    id
                };

                // Resolve or create object entity
                let object_id = if let Some(id) = entity_cache.get(&triple.object) {
                    *id
                } else {
                    let id = uuid::Uuid::new_v4();
                    let entity = EntityCreated {
                        meta: signal.meta().child(types::ENTITY_CREATED, "resolution::processor"),
                        entity_id: id,
                        name: triple.object.clone(),
                        category: triple.object_category.to_string(),
                        source: extracted.source_episode_id.clone().unwrap_or_default(),
                    };
                    entity_cache.insert(triple.object.clone(), id);
                    emitted.push(Arc::new(entity));
                    id
                };

                // Create edge between subject and object
                let edge = EdgeCreated {
                    meta: signal.meta().child(types::EDGE_CREATED, "resolution::processor"),
                    edge_id: uuid::Uuid::new_v4(),
                    subject_id,
                    predicate: triple.predicate.clone(),
                    object_id,
                    confidence: triple.confidence,
                };
                emitted.push(Arc::new(edge));
            }

            tracing::info!(
                "[Resolution] emitted {} graph signals ({} entities, {} edges)",
                emitted.len(),
                entity_cache.len(),
                emitted.len() - entity_cache.len(),
            );

            return Ok(emitted);
        }

        Ok(vec![])
    }
}

impl Default for EntityResolutionProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::graph::ExtractedTriplePayload;
    use crate::eventbus::signal::SignalMeta;

    fn make_triples_extracted(triples: Vec<ExtractedTriplePayload>) -> SignalArc {
        let meta = SignalMeta::new(types::TRIPLES_EXTRACTED, "test");
        let extracted = TriplesExtracted {
            meta,
            source_episode_id: Some("test-episode".to_string()),
            triples,
            source_text: "test".to_string(),
        };
        Arc::new(extracted)
    }

    #[tokio::test]
    async fn test_resolution_creates_entities() {
        let storage = Arc::new(crate::storage::memory_store::MemoryStore::new());
        let event_bus = Arc::new(crate::eventbus::bus::EventBus::new());
        let ctx = crate::field::context::FieldContext::new(event_bus, storage);

        let mut processor = EntityResolutionProcessor::new();

        let triple = ExtractedTriplePayload {
            subject: "Hiten".to_string(),
            subject_category: "Person".to_string(),
            predicate: "works_on".to_string(),
            object: "Noesis".to_string(),
            object_category: "Project".to_string(),
            confidence: 0.9,
        };

        let signal = make_triples_extracted(vec![triple]);
        let emitted = processor.process(&ctx, signal).await.unwrap();

        assert!(!emitted.is_empty(), "should emit signals");
        assert!(emitted.len() >= 2, "should emit at least 1 entity + 1 edge");

        // Check first signal is EntityCreated
        assert_eq!(emitted[0].signal_type(), types::ENTITY_CREATED);
    }

    #[tokio::test]
    async fn test_resolution_dedups_entities() {
        let storage = Arc::new(crate::storage::memory_store::MemoryStore::new());
        let event_bus = Arc::new(crate::eventbus::bus::EventBus::new());
        let ctx = crate::field::context::FieldContext::new(event_bus, storage);

        let mut processor = EntityResolutionProcessor::new();

        let triples = vec![
            ExtractedTriplePayload {
                subject: "Hiten".to_string(),
                subject_category: "Person".to_string(),
                predicate: "works_on".to_string(),
                object: "Noesis".to_string(),
                object_category: "Project".to_string(),
                confidence: 0.9,
            },
            ExtractedTriplePayload {
                subject: "Hiten".to_string(),
                subject_category: "Person".to_string(),
                predicate: "uses".to_string(),
                object: "Rust".to_string(),
                object_category: "Tool".to_string(),
                confidence: 0.8,
            },
        ];

        let signal = make_triples_extracted(triples);
        let emitted = processor.process(&ctx, signal).await.unwrap();

        // Should have 3 entities (Hiten, Noesis, Rust) + 2 edges
        assert_eq!(emitted.len(), 5, "3 entities + 2 edges");

        // Count entity vs edge signals
        let entities: Vec<_> = emitted.iter().filter(|s| s.signal_type() == types::ENTITY_CREATED).collect();
        let edges: Vec<_> = emitted.iter().filter(|s| s.signal_type() == types::EDGE_CREATED).collect();
        assert_eq!(entities.len(), 3, "should create 3 unique entities");
        assert_eq!(edges.len(), 2, "should create 2 edges");
    }
}
