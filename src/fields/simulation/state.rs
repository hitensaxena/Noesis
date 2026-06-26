use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A simulated scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: Uuid,
    pub description: String,
    pub outcome: Option<String>,
}

/// State of the Simulation Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationFieldState {
    pub scenarios: Vec<Scenario>,
}
