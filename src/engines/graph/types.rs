use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Entity types in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityCategory {
    Person,
    Project,
    Tool,
    Concept,
    Health,
    Organization,
    Location,
    Event,
    Resource,
    Other(String),
}

impl std::fmt::Display for EntityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityCategory::Person => write!(f, "Person"),
            EntityCategory::Project => write!(f, "Project"),
            EntityCategory::Tool => write!(f, "Tool"),
            EntityCategory::Concept => write!(f, "Concept"),
            EntityCategory::Health => write!(f, "Health"),
            EntityCategory::Organization => write!(f, "Organization"),
            EntityCategory::Location => write!(f, "Location"),
            EntityCategory::Event => write!(f, "Event"),
            EntityCategory::Resource => write!(f, "Resource"),
            EntityCategory::Other(s) => write!(f, "{}", s),
        }
    }
}

/// A node in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub category: EntityCategory,
    pub description: String,
    pub aliases: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub source_episode_id: Option<String>,
    pub metadata: serde_json::Value,
}

impl Entity {
    pub fn new(name: &str, category: EntityCategory, description: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            category,
            description: description.to_string(),
            aliases: Vec::new(),
            created_at: Utc::now(),
            valid_from: Utc::now(),
            valid_to: None,
            source_episode_id: None,
            metadata: serde_json::json!({}),
        }
    }
}

/// Type of relation between two entities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    WorksOn,
    Uses,
    PartOf,
    RelatedTo,
    LocatedIn,
    Knows,
    Created,
    Owns,
    Mentions,
    Other(String),
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationType::WorksOn => write!(f, "works_on"),
            RelationType::Uses => write!(f, "uses"),
            RelationType::PartOf => write!(f, "part_of"),
            RelationType::RelatedTo => write!(f, "related_to"),
            RelationType::LocatedIn => write!(f, "located_in"),
            RelationType::Knows => write!(f, "knows"),
            RelationType::Created => write!(f, "created"),
            RelationType::Owns => write!(f, "owns"),
            RelationType::Mentions => write!(f, "mentions"),
            RelationType::Other(s) => write!(f, "{}", s),
        }
    }
}

/// An edge in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub predicate: RelationType,
    pub object_id: Uuid,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub source_episode_id: Option<String>,
}

/// A triple extracted from text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triple {
    pub subject: String,
    pub subject_category: EntityCategory,
    pub predicate: String,
    pub object: String,
    pub object_category: EntityCategory,
    pub confidence: f32,
}

/// Snapshot of the knowledge graph state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub entity_count: usize,
    pub relation_count: usize,
}
