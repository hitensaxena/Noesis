use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::eventbus::signal::SignalMeta;
use crate::signals::types;
use crate::signals::signal_impl;

/// A belief was created, updated, or invalidated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefChanged {
    pub meta: SignalMeta,
    pub belief_id: Uuid,
    pub belief: String,
    pub previous_belief: Option<String>,
    pub confidence: f32,
    pub change_type: BeliefChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeliefChangeType {
    Created,
    Updated,
    Invalidated,
}

impl BeliefChanged {
    pub fn new(belief: &str, change_type: BeliefChangeType, confidence: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::BELIEF_CHANGED, "noesis::signals"),
            belief_id: Uuid::new_v4(),
            belief: belief.to_string(),
            previous_belief: None,
            confidence,
            change_type,
        }
    }
}

signal_impl!(BeliefChanged, BELIEF_CHANGED, "noesis::signals");

/// A personality trait was detected from behavioral patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitDetected {
    pub meta: SignalMeta,
    pub trait_id: Uuid,
    pub trait_name: String,
    pub evidence: String,
    pub strength: f32,
}

signal_impl!(TraitDetected, TRAIT_DETECTED, "noesis::signals");

/// The identity self-model was updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityUpdated {
    pub meta: SignalMeta,
    pub identity_version: u32,
    pub beliefs_count: usize,
    pub traits_count: usize,
    pub summary: String,
}

signal_impl!(IdentityUpdated, IDENTITY_UPDATED, "noesis::signals");
