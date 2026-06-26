use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::engines::graph::types::{Entity, Relation};

/// State of the Knowledge Graph Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphFieldState {
    pub entities: HashMap<String, Entity>,
    pub relations: Vec<Relation>,
}
