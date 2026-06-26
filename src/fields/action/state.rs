use serde::{Deserialize, Serialize};

/// A project tracked by the Action field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
}

/// A task tracked by the Action field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub description: String,
    pub priority: u8,
}

/// State of the Action Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionFieldState {
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
}
