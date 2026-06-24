use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use dashmap::DashMap;

/// Per-signal or per-processor metrics.
#[derive(Default)]
pub struct SignalMetrics {
    pub count: AtomicU64,
    pub total_latency_ns: AtomicU64,
}

/// Collects metrics for signals and processors.
pub struct MetricsCollector {
    signal_counts: DashMap<String, Arc<SignalMetrics>>,
    processor_latency: DashMap<String, Arc<SignalMetrics>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            signal_counts: DashMap::new(),
            processor_latency: DashMap::new(),
        }
    }

    /// Record that a signal was published.
    pub fn record_signal(&self, signal_type: &str) {
        let entry = self
            .signal_counts
            .entry(signal_type.to_string())
            .or_insert_with(|| Arc::new(SignalMetrics::default()));
        entry.value().count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record the latency of a processor handling a signal.
    pub fn record_processor_latency(&self, processor: &str, latency_ns: u64) {
        let entry = self
            .processor_latency
            .entry(processor.to_string())
            .or_insert_with(|| Arc::new(SignalMetrics::default()));
        let metrics = entry.value();
        metrics.count.fetch_add(1, Ordering::Relaxed);
        metrics.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);
    }

    /// Take a snapshot of all collected metrics.
    pub fn snapshot(&self) -> serde_json::Value {
        let signals: serde_json::Map<String, serde_json::Value> = self
            .signal_counts
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    serde_json::json!(entry.value().count.load(Ordering::Relaxed)),
                )
            })
            .collect();

        let processors: serde_json::Map<String, serde_json::Value> = self
            .processor_latency
            .iter()
            .map(|entry| {
                let m = entry.value();
                let count = m.count.load(Ordering::Relaxed);
                let total_ns = m.total_latency_ns.load(Ordering::Relaxed);
                let avg_ms = if count > 0 {
                    total_ns / (count as u64 * 1_000_000)
                } else {
                    0
                };
                (
                    entry.key().clone(),
                    serde_json::json!({
                        "count": count,
                        "avg_latency_ms": avg_ms,
                    }),
                )
            })
            .collect();

        serde_json::json!({
            "signals": signals,
            "processors": processors,
        })
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
