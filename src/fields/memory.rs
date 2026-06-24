use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing;

use crate::eventbus::signal::SignalArc;
use crate::field::field::Field;
use crate::field::context::FieldContext;
use crate::signals::types;
use crate::signals::EpisodeRecorded;

/// A single stored episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: Uuid,
    pub content: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub tags: Vec<String>,
}

/// A consolidated memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub episode_ids: Vec<Uuid>,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

/// State of the Memory Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFieldState {
    pub episodes: Vec<Episode>,
    pub memories: Vec<Memory>,
    pub episode_count: usize,
    pub memory_count: usize,
}

/// The Memory Field — owns episodic and semantic memory state.
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
            },
        }
    }
}

#[async_trait]
impl Field for MemoryField {
    fn name(&self) -> &str {
        "memory"
    }

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
                tracing::debug!(
                    "[MemoryField] stored episode {} (total: {})",
                    ep.episode_id,
                    self.state.episode_count
                );
            }
        }

        Ok(())
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[MemoryField] shutting down with {} episodes, {} memories",
            self.state.episode_count,
            self.state.memory_count
        );
        Ok(())
    }
}

impl Default for MemoryField {
    fn default() -> Self {
        Self::new()
    }
}
