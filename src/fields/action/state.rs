use serde::{Deserialize, Serialize};

// Re-export domain types for cohesive state access
pub use super::domains::projects::Project;
pub use super::domains::plans::{Plan, PlanStatus};
pub use super::domains::tasks::Task;
pub use super::domains::executions::Execution;
pub use super::domains::evaluations::Evaluation;
pub use super::domains::risk::RiskAssessment;

/// State of the Action Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionFieldState {
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
}
