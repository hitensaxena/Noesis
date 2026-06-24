use async_trait::async_trait;
use anyhow::Result;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::field::context::FieldContext;

/// A Processor performs exactly one cognitive transformation.
///
/// Processors never invoke other processors — they subscribe to signals,
/// perform their transformation, and emit new signals. They are stateless;
/// all persistent state lives in Fields.
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
