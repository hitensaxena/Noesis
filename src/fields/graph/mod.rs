use std::any::Any;
use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::Result;
use chrono::Utc;
use tracing;

use crate::kernel::signal::SignalArc;
use crate::field_runtime::field::Field;
use crate::field_runtime::context::FieldContext;
use crate::engines::graph::types::{Entity, EntityCategory, GraphSnapshot, Relation, RelationType};
use crate::signals::types;
use crate::signals::graph::{EntityCreated, EdgeCreated};

pub mod state;
pub use state::GraphFieldState;

/// The Knowledge Graph Field — owns entities and their relationships.
pub struct GraphField {
    state: GraphFieldState,
}

impl GraphField {
    pub fn new() -> Self {
        Self {
            state: GraphFieldState {
                entities: HashMap::new(),
                relations: Vec::new(),
            },
        }
    }

    /// Find an entity by name (case-insensitive).
    pub fn find_entity(&self, name: &str) -> Option<&Entity> {
        let lower = name.to_lowercase();
        self.state.entities.values()
            .find(|e| e.name.to_lowercase() == lower || e.aliases.iter().any(|a| a.to_lowercase() == lower))
    }

    /// Get entities by category.
    pub fn entities_by_category(&self, category: &EntityCategory) -> Vec<&Entity> {
        self.state.entities.values()
            .filter(|e| e.category == *category)
            .collect()
    }

    /// Get all relations for an entity.
    pub fn relations_for(&self, entity_id: &uuid::Uuid) -> Vec<&Relation> {
        self.state.relations.iter()
            .filter(|r| r.subject_id == *entity_id || r.object_id == *entity_id)
            .collect()
    }

    pub fn snapshot(&self) -> GraphSnapshot {
        let entities: Vec<Entity> = self.state.entities.values().cloned().collect();
        GraphSnapshot {
            entity_count: entities.len(),
            relation_count: self.state.relations.len(),
            entities,
            relations: self.state.relations.clone(),
        }
    }
}

#[async_trait]
impl Field for GraphField {
    fn name(&self) -> &str { "knowledge_graph" }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[GraphField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        let signal_type = signal.signal_type();

        if signal_type == types::ENTITY_CREATED {
            if let Some(ec) = signal.as_any().downcast_ref::<EntityCreated>() {
                let entity = Entity::new(&ec.name, EntityCategory::Other(ec.category.clone()), "Created from extraction");
                self.state.entities.insert(ec.name.to_lowercase(), entity);
                tracing::debug!("[GraphField] stored entity: {}", ec.name);
            }
        } else if signal_type == types::EDGE_CREATED {
            if let Some(ec) = signal.as_any().downcast_ref::<EdgeCreated>() {
                let predicate = parse_relation_type(&ec.predicate);
                let relation = Relation {
                    id: ec.edge_id,
                    subject_id: ec.subject_id,
                    predicate,
                    object_id: ec.object_id,
                    confidence: ec.confidence,
                    created_at: Utc::now(),
                    valid_from: Utc::now(),
                    valid_to: None,
                    source_episode_id: None,
                };
                self.state.relations.push(relation);
                tracing::debug!("[GraphField] stored relation: {} -> {} -> {}", ec.subject_id, ec.predicate, ec.object_id);
            }
        }
        Ok(())
    }

    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(self.snapshot())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[GraphField] shutting down with {} entities, {} relations",
            self.state.entities.len(), self.state.relations.len());
        Ok(())
    }
}

fn parse_relation_type(predicate: &str) -> RelationType {
    match predicate.to_lowercase().replace(' ', "_").as_str() {
        "works_on" | "workson" => RelationType::WorksOn,
        "uses" => RelationType::Uses,
        "part_of" | "partof" => RelationType::PartOf,
        "related_to" | "relatedto" => RelationType::RelatedTo,
        "located_in" | "locatedin" => RelationType::LocatedIn,
        "knows" => RelationType::Knows,
        "created" => RelationType::Created,
        "owns" => RelationType::Owns,
        "mentions" => RelationType::Mentions,
        other => RelationType::Other(other.to_string()),
    }
}

impl Default for GraphField {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::field_runtime::field::Field;
    use crate::field_runtime::context::FieldContext;
    use crate::storage::memory_store::MemoryStore;
    use crate::kernel::bus::EventBus;
    use crate::signals::graph::EntityCreated;
    use crate::kernel::signal::SignalMeta;
    use crate::signals::types;

    #[tokio::test]
    async fn test_graph_field_init() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);
        let mut field = GraphField::new();
        field.init(&ctx).await.unwrap();
        assert_eq!(field.name(), "knowledge_graph");
    }

    #[tokio::test]
    async fn test_graph_field_stores_entity() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);
        let mut field = GraphField::new();
        field.init(&ctx).await.unwrap();

        let entity = EntityCreated {
            meta: SignalMeta::new(types::ENTITY_CREATED, "test"),
            entity_id: uuid::Uuid::new_v4(),
            name: "Noesis".to_string(),
            category: "Project".to_string(),
            source: "test".to_string(),
        };
        field.handle_signal(&ctx, Arc::new(entity)).await.unwrap();
        let found = field.find_entity("Noesis");
        assert!(found.is_some(), "should find entity by name");
        assert_eq!(found.unwrap().name, "Noesis");
    }

    #[tokio::test]
    async fn test_graph_field_snapshot() {
        let field = GraphField::new();
        let snapshot = field.snapshot();
        assert_eq!(snapshot.entity_count, 0);
        assert_eq!(snapshot.relation_count, 0);
    }
}
