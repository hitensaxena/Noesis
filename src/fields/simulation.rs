use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing;

use crate::eventbus::signal::SignalArc;
use crate::field::field::Field;
use crate::field::context::FieldContext;

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

/// The Simulation Field — holds what-if scenarios.
pub struct SimulationField {
    state: SimulationFieldState,
}

impl SimulationField {
    pub fn new() -> Self {
        Self {
            state: SimulationFieldState {
                scenarios: Vec::new(),
            },
        }
    }
}

#[async_trait]
impl Field for SimulationField {
    fn name(&self) -> &str {
        "simulation"
    }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[SimulationField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        if signal.signal_type() == crate::signals::types::DECISION_EVALUATED {
            tracing::debug!("[SimulationField] received DecisionEvaluated");
        }
        Ok(())
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[SimulationField] shutting down with {} scenarios",
            self.state.scenarios.len()
        );
        Ok(())
    }
}

impl Default for SimulationField {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::field::field::Field;
    use crate::field::context::FieldContext;
    use crate::storage::memory_store::MemoryStore;
    use crate::eventbus::bus::EventBus;

    #[tokio::test]
    async fn test_simulation_field_init() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);

        let mut field = SimulationField::new();
        field.init(&ctx).await.unwrap();
        assert_eq!(field.name(), "simulation");
    }
}
