//! Recovery processor — generates recovery strategies from risk assessments.
//!
//! On RISK_ASSESSED, formulates a recovery strategy appropriate to the
//! risk level. High-probability risks get proactive strategies;
//! high-impact risks get conservative, multi-step recovery plans.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::processors::risk::RiskAssessed;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecoveryStarted {
    pub meta: crate::kernel::signal::SignalMeta,
    pub recovery_id: uuid::Uuid,
    pub strategy: String,
    pub failed_item: String,
}

impl RecoveryStarted {
    pub fn new(strategy: &str, failed_item: &str) -> Self {
        Self {
            meta: crate::kernel::signal::SignalMeta::new(types::RECOVERY_STARTED, "action::recovery"),
            recovery_id: uuid::Uuid::new_v4(),
            strategy: strategy.to_string(),
            failed_item: failed_item.to_string(),
        }
    }
}

crate::signals::signal_impl!(RecoveryStarted, RECOVERY_STARTED, "action::recovery");

pub struct RecoveryProcessor;

impl RecoveryProcessor {
    pub fn new() -> Self { Self }

    fn formulate_strategy(description: &str, probability: f32, impact: f32) -> (String, String) {
        let risk_level = probability * impact;
        let strategy = if risk_level > 0.5 {
            format!("Proactive mitigation: reduce probability (currently {:.0}%) and impact ({:.0}%)",
                probability * 100.0, impact * 100.0)
        } else if risk_level > 0.25 {
            format!("Contingency plan: monitor and respond to potential issue: {}", description)
        } else {
            format!("Acceptance: low-risk item (score {:.2}), no action needed", risk_level)
        };
        (strategy, description.to_string())
    }
}

#[async_trait]
impl Processor for RecoveryProcessor {
    fn name(&self) -> &str { "recovery" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 180 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::RISK_ASSESSED] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::RECOVERY_STARTED] }

    async fn process(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<Vec<SignalArc>> {
        if signal.signal_type() == types::RISK_ASSESSED {
            if let Some(risk) = signal.as_any().downcast_ref::<RiskAssessed>() {
                let (strategy, item) = Self::formulate_strategy(&risk.description, risk.probability, risk.impact);
                tracing::info!("[Recovery] strategy: {}", strategy);
                return Ok(vec![Arc::new(RecoveryStarted::new(&strategy, &item))]);
            }
        }
        Ok(vec![])
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for RecoveryProcessor { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_ctx() -> FieldContext {
        FieldContext::new(Arc::new(EventBus::new()), Arc::new(MemoryStore::new()))
    }

    #[test]
    fn test_recovery_processor_name() { assert_eq!(RecoveryProcessor::new().name(), "recovery"); }

    #[test]
    fn test_recovery_high_risk_strategy() {
        let (strat, _) = RecoveryProcessor::formulate_strategy("Critical system risk", 0.8, 0.9);
        assert!(strat.contains("mitigation"), "high risk should suggest mitigation");
    }

    #[test]
    fn test_recovery_low_risk_strategy() {
        let (strat, _) = RecoveryProcessor::formulate_strategy("Minor concern", 0.1, 0.2);
        assert!(strat.contains("acceptance") || strat.contains("Acceptance"),
            "low risk should suggest acceptance");
    }

    #[tokio::test]
    async fn test_recovery_emits_on_risk() {
        let mut p = RecoveryProcessor::new();
        let ctx = test_ctx();
        let risk = RiskAssessed::new("Test risk scenario", 0.6, 0.7);
        let result = p.process(&ctx, Arc::new(risk)).await.unwrap();
        assert!(!result.is_empty());
        let rec = result[0].as_any().downcast_ref::<RecoveryStarted>().unwrap();
        assert!(!rec.strategy.is_empty());
    }
}
