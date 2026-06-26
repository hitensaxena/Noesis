use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use tracing;

use crate::kernel::signal::SignalArc;
use crate::field_runtime::field::Field;
use crate::field_runtime::context::FieldContext;
use crate::signals::types;
use crate::signals::{EpisodeRecorded, MemoryConsolidated};
use crate::signals::graph::{EntityCreated, EdgeCreated};

pub mod state;
pub mod processors;
pub use state::{MemoryFieldState, Episode, Memory, KnowledgeEntity, KnowledgeRelation};

pub struct MemoryField {
    state: MemoryFieldState,
}

impl MemoryField {
    pub fn new() -> Self {
        Self {
            state: MemoryFieldState {
                episodes: Vec::new(),
                memories: Vec::new(),
                episode_count: 0,
                memory_count: 0,
                knowledge_entities: Vec::new(),
                knowledge_relations: Vec::new(),
                entity_count: 0,
            },
        }
    }

    /// Search episodes by content query, returning up to k matches.
    pub fn recall(&self, query: &str, k: usize) -> Vec<Episode> {
        let q = query.to_lowercase();
        let mut matches: Vec<Episode> = self.state.episodes.iter()
            .filter(|e| e.content.to_lowercase().contains(&q)
                || e.source.to_lowercase().contains(&q)
                || e.tags.iter().any(|t| t.to_lowercase().contains(&q)))
            .cloned()
            .collect();
        matches.truncate(k);
        matches
    }

    /// Return the most recent N episodes as a context window.
    pub fn context(&self, n: usize) -> Vec<Episode> {
        let start = if self.state.episodes.len() > n {
            self.state.episodes.len() - n
        } else {
            0
        };
        self.state.episodes[start..].to_vec()
    }
}

#[async_trait]
impl Field for MemoryField {
    fn name(&self) -> &str { "memory" }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[MemoryField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        let signal_type = signal.signal_type();

        if signal_type == types::EPISODE_RECORDED {
            if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
                self.state.episodes.push(Episode {
                    id: ep.episode_id,
                    content: ep.content.clone(),
                    source: ep.source.clone(),
                    timestamp: ep.timestamp,
                    tags: ep.tags.clone(),
                });
                self.state.episode_count = self.state.episodes.len();
                tracing::debug!("[MemoryField] stored episode {} (total: {})", ep.episode_id, self.state.episode_count);
            }
        } else if signal_type == types::MEMORY_CONSOLIDATED {
            if let Some(mc) = signal.as_any().downcast_ref::<MemoryConsolidated>() {
                let memory = Memory {
                    id: Uuid::new_v4(),
                    episode_ids: mc.episode_ids.clone(),
                    summary: mc.summary.clone(),
                    created_at: Utc::now(),
                };
                self.state.memories.push(memory);
                self.state.memory_count = self.state.memories.len();
                tracing::debug!("[MemoryField] stored consolidated memory from {} episodes (total: {})",
                    mc.episode_ids.len(), self.state.memory_count);
            }
        } else if signal_type == types::PATTERN_DETECTED {
            tracing::trace!("[MemoryField] pattern detected signal received");
        } else if signal_type == types::ENTITY_CREATED {
            if let Some(ec) = signal.as_any().downcast_ref::<EntityCreated>() {
                self.state.knowledge_entities.push(KnowledgeEntity {
                    name: ec.name.clone(),
                    category: ec.category.clone(),
                    confidence: 0.7,
                });
                self.state.entity_count = self.state.knowledge_entities.len();
                tracing::debug!("[MemoryField] stored entity: {} (total: {})", ec.name, self.state.entity_count);
            }
        } else if signal_type == types::EDGE_CREATED {
            if let Some(ec) = signal.as_any().downcast_ref::<EdgeCreated>() {
                self.state.knowledge_relations.push(KnowledgeRelation {
                    subject: ec.subject_id.to_string(),
                    predicate: ec.predicate.clone(),
                    object: ec.object_id.to_string(),
                    confidence: ec.confidence,
                });
                tracing::debug!("[MemoryField] stored relation: {}", ec.predicate);
            }
        }

        Ok(())
    }

    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[MemoryField] shutting down with {} episodes, {} memories",
            self.state.episode_count, self.state.memory_count);
        Ok(())
    }
}

impl Default for MemoryField {
    fn default() -> Self { Self::new() }
}
