use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::eventbus::signal::SignalMeta;
use crate::signals::signal_impl;

/// A knowledge entity was created in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCreated {
    pub meta: SignalMeta,
    pub entity_id: Uuid,
    pub name: String,
    pub category: String,
    pub source: String,
}

signal_impl!(EntityCreated, ENTITY_CREATED, "noesis::signals");

/// A relation was created between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCreated {
    pub meta: SignalMeta,
    pub edge_id: Uuid,
    pub subject_id: Uuid,
    pub predicate: String,
    pub object_id: Uuid,
    pub confidence: f32,
}

signal_impl!(EdgeCreated, EDGE_CREATED, "noesis::signals");

/// Triples were extracted from text and need resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriplesExtracted {
    pub meta: SignalMeta,
    pub source_episode_id: Option<String>,
    pub triples: Vec<ExtractedTriplePayload>,
    pub source_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTriplePayload {
    pub subject: String,
    pub subject_category: String,
    pub predicate: String,
    pub object: String,
    pub object_category: String,
    pub confidence: f32,
}

signal_impl!(TriplesExtracted, TRIPLES_EXTRACTED, "noesis::signals");
