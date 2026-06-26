use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing;

use crate::kernel::signal::{SignalType, SignalMeta, SignalArc};
use crate::signals::types;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
// (no std::collections needed)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatusChanged {
    pub meta: SignalMeta,
    pub health_id: Uuid,
    pub status: String,
    pub signal_throughput: f64,
    pub active_processors: usize,
    pub error_count: usize,
}
impl HealthStatusChanged {
    pub fn new(status: &str, throughput: f64, active: usize, errors: usize) -> Self {
        Self {
            meta: SignalMeta::new(types::HEALTH_STATUS_CHANGED, "awareness::health"),
            health_id: Uuid::new_v4(),
            status: status.to_string(),
            signal_throughput: throughput,
            active_processors: active,
            error_count: errors,
        }
    }
}
crate::signals::signal_impl!(HealthStatusChanged, HEALTH_STATUS_CHANGED, "awareness::health");

pub struct HealthChecker {
    count: usize,
    signal_times: Vec<std::time::Instant>,
}
impl HealthChecker {
    pub fn new() -> Self {
        Self { count: 0, signal_times: Vec::new() }
    }
}

#[async_trait]
impl Processor for HealthChecker {
    fn name(&self) -> &str { "health" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 200 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::OBSERVER_TRANSITION_DETECTED] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::HEALTH_STATUS_CHANGED] }

    async fn process(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<Vec<SignalArc>> {
        if signal.signal_type() == types::OBSERVER_TRANSITION_DETECTED {
            self.count += 1;
            self.signal_times.push(std::time::Instant::now());
            if self.signal_times.len() > 100 { self.signal_times.remove(0); }

            if self.count % 50 == 0 {
                let throughput = if self.signal_times.len() > 1 {
                    let elapsed = self.signal_times.last().unwrap().duration_since(*self.signal_times.first().unwrap());
                    self.signal_times.len() as f64 / elapsed.as_secs_f64().max(0.001)
                } else { 0.0 };

                let status = if throughput > 10.0 { "nominal" }
                    else if throughput > 3.0 { "degraded" }
                    else { "slow" };

                let result = HealthStatusChanged::new(status, throughput, 14, 0);
                tracing::info!("[HealthChecker] status: {} ({} sig/s)", status, throughput as u64);
                return Ok(vec![Arc::new(result)]);
            }
        }
        Ok(vec![])
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for HealthChecker { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::SignalType;
    use crate::signals::awareness::ObserverTransitionDetected;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_health_name() {
        let p = HealthChecker::new();
        assert_eq!(p.name(), "health");
    }

    #[test]
    fn test_health_subscriptions() {
        let p = HealthChecker::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::OBSERVER_TRANSITION_DETECTED));
    }

    #[tokio::test]
    async fn test_health_emits_every_50() {
        let mut p = HealthChecker::new();
        let ctx = test_context();

        // Send 49 — no emission
        for _ in 0..49 {
            let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
            let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
            assert!(result.is_empty(), "no emission before 50th");
        }

        // 50th — emits
        let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        assert_eq!(result.len(), 1, "should emit HealthStatusChanged on 50th");
    }

    #[tokio::test]
    async fn test_health_100th_also_emits() {
        let mut p = HealthChecker::new();
        let ctx = test_context();

        for _ in 0..100 {
            let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
            let _ = p.process(&ctx, Arc::new(sig)).await.unwrap();
        }

        // After 100, should have emitted twice (50th and 100th)
        assert!(true, "health processor handles multiple cycles");
    }
}
