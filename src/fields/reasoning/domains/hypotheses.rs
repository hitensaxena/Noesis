use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A hypothesis — an untested proposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: Uuid,
    pub proposition: String,
    pub supporting: Vec<String>,
    pub contradicting: Vec<String>,
}
