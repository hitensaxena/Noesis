//! BeatCoordinator — emits cognitive beat signals at defined intervals.
//!
//! Cognitive beats replace hard-coded counter-based scheduling (e.g. "every 3
//! episodes, generate a narrative") with time-based heartbeats that processors
//! subscribe to. This decouples scheduling from cognitive logic.
//!
//! | Beat | Period | Example Consumers |
//! |------|--------|-------------------|
//! | Fast | ~1s | Extraction, identity updates |
//! | Medium | ~60s | Curiosity, reflection, planning |
//! | Slow | ~900s (15min) | Narrative, consolidation, identity evolution |

use std::any::Any;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing;

use crate::kernel::bus::EventBus;
use crate::kernel::signal::{Signal, SignalMeta, SignalType};

// ---------------------------------------------------------------------------
// Beat signal types — registered in signals/mod.rs types block
// ---------------------------------------------------------------------------

/// Kernel beat signal strings used to construct SignalType constants.
pub mod beat_types {
    /// Emitted on every signal — working memory, attention
    pub const IMMEDIATE: &str = "kernel.scheduler.beat.immediate";
    /// ~1s interval — extraction, identity updates
    pub const FAST: &str = "kernel.scheduler.beat.fast";
    /// ~60s interval — curiosity, reflection, planning
    pub const MEDIUM: &str = "kernel.scheduler.beat.medium";
    /// ~900s (15min) interval — narrative, consolidation
    pub const SLOW: &str = "kernel.scheduler.beat.slow";
    /// Session idle — consolidation, model stabilization
    pub const SLEEP: &str = "kernel.scheduler.beat.sleep";
    /// On demand — re-indexing, optimization
    pub const OFFLINE: &str = "kernel.scheduler.beat.offline";
}

/// A minimal kernel-internal signal representing a cognitive beat.
/// The payload carries no semantic data — the signal_type alone identifies
/// which beat fired.
#[derive(Debug, Clone)]
pub struct BeatPulse {
    pub meta: SignalMeta,
}

impl BeatPulse {
    pub fn new(beat_signal_type: SignalType) -> Self {
        Self {
            meta: SignalMeta::new(beat_signal_type, "noesis::kernel::beat_coordinator"),
        }
    }
}

impl Signal for BeatPulse {
    fn signal_type(&self) -> SignalType {
        self.meta.signal_type.clone()
    }
    fn meta(&self) -> &SignalMeta {
        &self.meta
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Display for BeatPulse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BeatPulse({})", self.meta.signal_type)
    }
}

// ---------------------------------------------------------------------------
// BeatCoordinator
// ---------------------------------------------------------------------------

/// Spawns tokio tasks that emit beat signals into the EventBus at defined intervals.
///
/// Each beat task runs independently and emits its signal type into the kernel's
/// EventBus. The signal flows through the cascade loop like any other signal,
/// reaching processors that subscribe to that beat type.
#[allow(dead_code)]
pub struct BeatCoordinator {
    last_fast: Option<std::time::Instant>,
    last_medium: Option<std::time::Instant>,
    last_slow: Option<std::time::Instant>,
}

impl BeatCoordinator {
    pub fn new() -> Self {
        Self {
            last_fast: None,
            last_medium: None,
            last_slow: None,
        }
    }

    /// Spawn beat-emitting tasks. Returns JoinHandles for each spawned task.
    pub fn spawn(
        &mut self,
        event_bus: Arc<EventBus>,
        cancellation_token: CancellationToken,
        fast_signal: SignalType,
        medium_signal: SignalType,
        slow_signal: SignalType,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();

        // Fast beat (~1s)
        let bus = event_bus.clone();
        let token = cancellation_token.clone();
        let sig = fast_signal;
        handles.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            interval.tick().await; // skip first immediate tick
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let pulse = BeatPulse::new(sig.clone());
                        bus.publish(Arc::new(pulse));
                        tracing::trace!("[BeatCoordinator] fast beat");
                    }
                    _ = token.cancelled() => break,
                }
            }
        }));

        // Medium beat (~60s)
        let bus = event_bus.clone();
        let token = cancellation_token.clone();
        let sig = medium_signal;
        handles.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            interval.tick().await;
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let pulse = BeatPulse::new(sig.clone());
                        bus.publish(Arc::new(pulse));
                        tracing::debug!("[BeatCoordinator] medium beat");
                    }
                    _ = token.cancelled() => break,
                }
            }
        }));

        // Slow beat (~15min)
        let bus = event_bus.clone();
        let token = cancellation_token.clone();
        let sig = slow_signal;
        handles.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(900));
            interval.tick().await;
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let pulse = BeatPulse::new(sig.clone());
                        bus.publish(Arc::new(pulse));
                        tracing::debug!("[BeatCoordinator] slow beat");
                    }
                    _ = token.cancelled() => break,
                }
            }
        }));

        handles
    }
}

impl Default for BeatCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
