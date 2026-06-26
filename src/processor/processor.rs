use async_trait::async_trait;
use anyhow::Result;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::field_runtime::context::FieldContext;

/// A Processor performs exactly one cognitive transformation.
///
/// Processors never invoke other processors — they subscribe to signals,
/// perform their transformation, and emit new signals. They may keep
/// ephemeral runtime state (counters, caches, recent context). All
/// persistent state lives in Fields.
///
/// ## Attention Economy
///
/// Every processor has an `activation_threshold`. The EventBus filters
/// incoming signals by comparing signal activation to this threshold.
/// Signals with `activation < threshold` are silently skipped. This
/// guarantees cascade convergence — no processor sees a signal that
/// has decayed below its threshold.
#[async_trait]
pub trait Processor: Send + Sync {
    /// The unique name of this processor.
    fn name(&self) -> &str;

    /// Semantic version of this processor.
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Lower priority = processed first (0 is highest).
    fn priority(&self) -> u8 {
        100
    }

    /// Minimum activation required for this processor to process a signal.
    ///
    /// Signals with `meta.activation < self.activation_threshold()` are
    /// ignored. Default: 0.1. Critical processors may lower this to 0.05;
    /// opportunistic processors may raise it to 0.2.
    fn activation_threshold(&self) -> f32 {
        0.1
    }

    /// The signal types this processor wants to receive.
    fn subscribed_signals(&self) -> &[SignalType];

    /// The signal types this processor may emit.
    fn emitted_signals(&self) -> &[SignalType];

    /// Initialize the processor.
    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        Ok(())
    }

    /// Process an incoming signal and return any new signals to emit.
    async fn process(
        &mut self,
        ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>>;

    /// Shut down the processor.
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
