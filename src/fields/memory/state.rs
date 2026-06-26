use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// A lightweight entity reference stored within memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntity {
    pub name: String,
    pub category: String,
    pub confidence: f32,
}

/// A lightweight relation reference stored within memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRelation {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
}

/// State of the Memory Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFieldState {
    pub episodes: Vec<Episode>,
    pub memories: Vec<Memory>,
    pub episode_count: usize,
    pub memory_count: usize,
    pub knowledge_entities: Vec<KnowledgeEntity>,
    pub knowledge_relations: Vec<KnowledgeRelation>,
    pub entity_count: usize,
}
