use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::SignalArc;
use crate::field_runtime::field::Field;
use crate::field_runtime::context::FieldContext;
use crate::signals::types;

pub mod state;
pub mod domains;
pub mod processors;
pub use state::{SimulationFieldState, Scenario};

/// The Simulation Field — holds what-if scenarios.
pub struct SimulationField {
    state: SimulationFieldState,
}

impl SimulationField {
    pub fn new() -> Self {
        Self {
            state: SimulationFieldState {
                scenarios: Vec::new(),
                world_models: Vec::new(),
                assumptions: Vec::new(),
                counterfactuals: Vec::new(),
                forecasts: Vec::new(),
                risk_assessments: Vec::new(),
            },
        }
    }
}

#[async_trait]
impl Field for SimulationField {
    fn name(&self) -> &str { "simulation" }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[SimulationField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        if signal.signal_type() == types::DECISION_EVALUATED {
            tracing::debug!("[SimulationField] received DecisionEvaluated");
        }
        Ok(())
    }

    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[SimulationField] shutting down with {} scenarios, {} world_models, {} assumptions",
            self.state.scenarios.len(), self.state.world_models.len(), self.state.assumptions.len());
        Ok(())
    }
}

impl Default for SimulationField {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::field_runtime::field::Field;
    use crate::field_runtime::context::FieldContext;
    use crate::storage::memory_store::MemoryStore;
    use crate::kernel::bus::EventBus;

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
