//! Forecasting processor — extrapolates trends from goals and episodes.
//!
//! On GOAL_CREATED, estimates a completion timeline based on goal
//! priority and complexity. On BEAT_SLOW, emits ForecastReady with
//! the prediction and probability.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;
use uuid::Uuid;

use crate::kernel::signal::{SignalArc, SignalType, SignalMeta};
use crate::signals::types;
use crate::signals::agency::GoalCreated;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// A trend forecast.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForecastReady {
    pub meta: SignalMeta,
    pub forecast_id: Uuid,
    pub prediction: String,
    pub probability: f32,
}

impl ForecastReady {
    pub fn new(prediction: &str, probability: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::FORECAST_READY, "simulation::forecast"),
            forecast_id: Uuid::new_v4(),
            prediction: prediction.to_string(),
            probability,
        }
    }
}

crate::signals::signal_impl!(ForecastReady, FORECAST_READY, "simulation::forecast");

/// Forecasts trends from goal and episode signals.
pub struct ForecastingProcessor {
    forecast_count: usize,
}

impl ForecastingProcessor {
    pub fn new() -> Self {
        Self {
            forecast_count: 0,
        }
    }

    fn predict(&self, description: &str, priority: u8) -> (String, f32) {
        // Higher priority goals are estimated to complete faster
        let base_days = match priority {
            0..=3 => 14,
            4..=6 => 7,
            7..=10 => 3,
            _ => 10,
        };
        let probability = (0.3 + (priority as f32 * 0.06)).min(0.9);
        let prediction = format!(
            "Goal '{}' likely completes in ~{} days (priority: {})",
            description, base_days, priority,
        );
        (prediction, probability)
    }
}

#[async_trait]
impl Processor for ForecastingProcessor {
    fn name(&self) -> &str {
        "forecast"
    }

    fn priority(&self) -> u8 {
        160
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::GOAL_CREATED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::FORECAST_READY]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::GOAL_CREATED {
            if let Some(gc) = signal.as_any().downcast_ref::<GoalCreated>() {
                self.forecast_count += 1;
                let (prediction, probability) = self.predict(&gc.description, gc.priority);

                tracing::info!(
                    "[Forecast] forecast #{}: {} (prob: {:.2})",
                    self.forecast_count, prediction, probability,
                );

                return Ok(vec![Arc::new(ForecastReady::new(&prediction, probability))]);
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for ForecastingProcessor {
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
    fn test_forecast_processor_name() {
        let p = ForecastingProcessor::new();
        assert_eq!(p.name(), "forecast");
    }

    #[tokio::test]
    async fn test_forecast_emits_on_goal_created() {
        let mut p = ForecastingProcessor::new();
        let ctx = test_context();

        let goal = GoalCreated::new("Finish the Noesis project", 8);
        let result = p.process(&ctx, Arc::new(goal)).await.unwrap();
        assert!(!result.is_empty(), "should emit ForecastReady");

        let f = result[0].as_any().downcast_ref::<ForecastReady>().unwrap();
        assert!(!f.prediction.is_empty(), "prediction should not be empty");
        assert!(f.probability > 0.3, "probability should be reasonable");
    }

    #[test]
    fn test_predict_high_priority() {
        let p = ForecastingProcessor::new();
        let (prediction, prob) = p.predict("Critical task", 10);
        assert!(prediction.contains("~3 days"), "high priority should predict ~3 days");
        assert!(prob >= 0.8, "high priority should have high probability");
    }

    #[test]
    fn test_predict_low_priority() {
        let p = ForecastingProcessor::new();
        let (prediction, prob) = p.predict("Low priority task", 1);
        assert!(prediction.contains("~14 days"), "low priority should predict ~14 days");
        assert!(prob < 0.5, "low priority should have lower probability");
    }
}
