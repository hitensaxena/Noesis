use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;

use crate::kernel::signal::SignalArc;
use crate::field_runtime::context::FieldContext;

/// A Field is a persistent cognitive space that owns state.
///
/// Fields never call each other directly — they only receive signals
/// through the event bus and update their internal state.
#[async_trait]
pub trait Field: Send + Sync {
    /// The unique name of this field.
    fn name(&self) -> &str;

    /// Initialize the field with the given context.
    async fn init(&mut self, ctx: &FieldContext) -> Result<()>;

    /// Handle an incoming signal. Fields update their state in response.
    async fn handle_signal(&mut self, _ctx: &FieldContext, _signal: SignalArc) -> Result<()> {
        Ok(())
    }

    /// Return a snapshot of the field's current state for inspection.
    fn state(&self) -> Box<dyn Any + Send>;

    /// Shut down the field, releasing any resources.
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
