use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export domain types for cohesive state access
pub use super::domains::world_models::WorldModel;
pub use super::domains::assumptions::Assumption;
pub use super::domains::counterfactuals::Counterfactual;
pub use super::domains::forecasting::Forecast;
pub use super::domains::risk::RiskAssessment;

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
    pub world_models: Vec<WorldModel>,
    pub assumptions: Vec<Assumption>,
    pub counterfactuals: Vec<Counterfactual>,
    pub forecasts: Vec<Forecast>,
    pub risk_assessments: Vec<RiskAssessment>,
}
