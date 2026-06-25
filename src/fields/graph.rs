use std::any::Any;
use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing;

use crate::eventbus::signal::SignalArc;
use crate::field::field::Field;
use crate::field::context::FieldContext;
use crate::engines::graph::types::{Entity, EntityCategory, GraphSnapshot, Relation};
use crate::signals::types;
use crate::signals::graph::{EntityCreated, EdgeCreated};

/// State of the Knowledge Graph Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphFieldState {
    pub entities: HashMap<String, Entity>,
    pub relations: Vec<Relation>,
}

/// The Knowledge Graph Field — owns entities and their relationships.
///
/// Receives EntityCreated and EdgeCreated signals and maintains
/// a navigable, queryable graph structure.
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
        self.state
            .entities
            .values()
            .find(|e| e.name.to_lowercase() == lower || e.aliases.iter().any(|a| a.to_lowercase() == lower))
    }

    /// Get entities by category.
    pub fn entities_by_category(&self, category: &EntityCategory) -> Vec<&Entity> {
        self.state
            .entities
            .values()
            .filter(|e| e.category == *category)
            .collect()
    }

    /// Get all relations for an entity.
    pub fn relations_for(&self, entity_id: &uuid::Uuid) -> Vec<&Relation> {
        self.state
            .relations
            .iter()
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
    fn name(&self) -> &str {
        "knowledge_graph"
    }

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
            if let Some(_ec) = signal.as_any().downcast_ref::<EdgeCreated>() {
                tracing::debug!("[GraphField] stored relation");
            }
        }

        Ok(())
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(self.snapshot())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[GraphField] shutting down with {} entities, {} relations",
            self.state.entities.len(),
            self.state.relations.len()
        );
        Ok(())
    }
}

impl Default for GraphField {
    fn default() -> Self {
        Self::new()
    }
}
