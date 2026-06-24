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
use crate::signals::BeliefChanged;

/// A single belief held by the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub id: Uuid,
    pub belief: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

/// A detected personality trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,
    pub strength: f32,
}

/// State of the Identity Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityFieldState {
    pub beliefs: Vec<Belief>,
    pub traits: Vec<Trait>,
    pub identity_version: u32,
}

/// The Identity Field — owns beliefs, traits, and the self-model.
pub struct IdentityField {
    state: IdentityFieldState,
}

impl IdentityField {
    pub fn new() -> Self {
        Self {
            state: IdentityFieldState {
                beliefs: Vec::new(),
                traits: Vec::new(),
                identity_version: 0,
            },
        }
    }
}

#[async_trait]
impl Field for IdentityField {
    fn name(&self) -> &str {
        "identity"
    }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[IdentityField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        if signal.signal_type() == types::BELIEF_CHANGED {
            if let Some(bc) = signal.as_any().downcast_ref::<BeliefChanged>() {
                let belief = Belief {
                    id: bc.belief_id,
                    belief: bc.belief.clone(),
                    confidence: bc.confidence,
                    created_at: Utc::now(),
                    is_active: true,
                };
                self.state.beliefs.push(belief);
                self.state.identity_version += 1;
                tracing::debug!(
                    "[IdentityField] stored belief (v{}, total: {})",
                    self.state.identity_version,
                    self.state.beliefs.len()
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
            "[IdentityField] shutting down with {} beliefs, {} traits",
            self.state.beliefs.len(),
            self.state.traits.len()
        );
        Ok(())
    }
}

impl Default for IdentityField {
    fn default() -> Self {
        Self::new()
    }
}
