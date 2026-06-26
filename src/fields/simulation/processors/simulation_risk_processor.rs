//! Simulation risk processor — cross-references scenarios with risk profiles.
//!
//! On SCENARIO_READY, evaluates the scenario against known risk factors
//! and emits SimulationRiskAssessed with probability and impact scores.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;
use uuid::Uuid;

use crate::kernel::signal::{SignalArc, SignalType, SignalMeta};
use crate::signals::types;
use crate::processors::scenario::ScenarioReady;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// A risk assessment for a scenario.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimulationRiskAssessed {
    pub meta: SignalMeta,
    pub risk_id: Uuid,
    pub scenario: String,
    pub probability: f32,
    pub impact: f32,
    pub risk_score: f32,
}

impl SimulationRiskAssessed {
    pub fn new(scenario: &str, probability: f32, impact: f32) -> Self {
        let risk_score = probability * impact;
        Self {
            meta: SignalMeta::new(types::SIMULATION_RISK_ASSESSED, "simulation::risk"),
            risk_id: Uuid::new_v4(),
            scenario: scenario.to_string(),
            probability,
            impact,
            risk_score,
        }
    }
}

crate::signals::signal_impl!(SimulationRiskAssessed, SIMULATION_RISK_ASSESSED, "simulation::risk");

/// Assesses risk from scenario projections.
pub struct SimulationRiskProcessor;

impl SimulationRiskProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for SimulationRiskProcessor {
    fn name(&self) -> &str {
        "simulation_risk"
    }

    fn priority(&self) -> u8 {
        170
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::SCENARIO_READY]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::SIMULATION_RISK_ASSESSED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::SCENARIO_READY {
            if let Some(sc) = signal.as_any().downcast_ref::<ScenarioReady>() {
                // Risk = probability of negative outcome × potential impact
                // Invert scenario probability: unlikely scenarios are higher risk
                // (less predictable outcomes)
                let probability = 1.0 - sc.probability;
                let impact = 0.3 + (sc.probability * 0.5); // higher prob scenarios have higher impact
                let risk_score = probability * impact;

                let scenario = &sc.description;
                tracing::info!(
                    "[SimulationRisk] assessed: {} (risk: {:.2}, prob: {:.2}, impact: {:.2})",
                    scenario, risk_score, probability, impact,
                );

                return Ok(vec![Arc::new(SimulationRiskAssessed::new(
                    scenario,
                    probability,
                    impact,
                ))]);
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for SimulationRiskProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_simulation_risk_processor_name() {
        let p = SimulationRiskProcessor::new();
        assert_eq!(p.name(), "simulation_risk");
    }

    #[tokio::test]
    async fn test_risk_assesses_scenario() {
        let mut p = SimulationRiskProcessor::new();
        let ctx = test_context();

        let scenario = ScenarioReady::new("Test scenario with high probability", 0.8);
        let result = p.process(&ctx, Arc::new(scenario)).await.unwrap();
        assert!(!result.is_empty(), "should emit SimulationRiskAssessed");

        let risk = result[0].as_any().downcast_ref::<SimulationRiskAssessed>().unwrap();
        assert!(risk.probability >= 0.0 && risk.probability <= 1.0, "probability must be valid");
        assert!(risk.impact >= 0.0 && risk.impact <= 1.0, "impact must be valid");
        assert!((risk.risk_score - risk.probability * risk.impact).abs() < 0.001,
            "risk_score should be prob * impact");
    }
}
