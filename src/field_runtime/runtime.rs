//! FieldRuntime — the bridge between Kernel and Fields.
//!
//! Owns ProcessorRegistry and SignalDispatcher. Coordinates signal dispatch
//! across fields and their processors in a unified pipeline.
//!
//! Kernel                    (sustains — pure infrastructure)
//!   │
//! Field Runtime             (bridges — knows both kernel and fields)
//!   │
//! Fields                    (cognition — fields remember, processors transform)

use std::sync::Arc;
use anyhow::Result;
use tracing;

use crate::field_runtime::context::FieldContext;
use crate::field_runtime::dispatcher::SignalDispatcher;
use crate::field_runtime::field::Field;
use crate::field_runtime::processor_registry::ProcessorRegistry;
use crate::field_runtime::snapshot::SnapshotManager;
use crate::field_runtime::transactions::TransactionManager;
use crate::kernel::bus::EventBus;
use crate::kernel::signal::SignalArc;

/// The result of processing a single cascade cycle.
#[derive(Debug, Default)]
pub struct CascadeMetrics {
    /// Total signals processed in this cascade (including the root).
    pub total_signals: usize,
    /// Signals emitted as a result of the cascade.
    pub emitted_signals: Vec<SignalArc>,
    /// Signal type names of ALL signals processed in this cascade (for metrics tracking).
    pub signal_types: Vec<String>,
}

/// Coordinates signal dispatch across fields and their processors.
///
/// This is the primary entry point for signal processing in Noesis.
/// It owns the field instances (via SignalDispatcher), the processor registry,
/// and the infrastructure that bridges the Kernel's EventBus to cognition.
pub struct FieldRuntime {
    /// Signal dispatch to field processors (cognitive transformation)
    pub processor_registry: ProcessorRegistry,
    /// Signal dispatch to field handlers (state updates)
    pub field_dispatcher: SignalDispatcher,
    /// Snapshot manager for field state caching
    pub snapshot_manager: SnapshotManager,
    /// Transaction manager for atomic state updates
    pub transaction_manager: TransactionManager,
}

impl FieldRuntime {
    pub fn new() -> Self {
        Self {
            processor_registry: ProcessorRegistry::new(),
            field_dispatcher: SignalDispatcher::new(),
            snapshot_manager: SnapshotManager::new(),
            transaction_manager: TransactionManager::new(),
        }
    }

    /// Register a field, initialize it, and add it to the dispatcher.
    pub async fn register_and_init_field(
        &mut self,
        name: &str,
        mut field: Box<dyn Field + Send>,
        ctx: &FieldContext,
    ) -> Result<()> {
        tracing::info!("[FieldRuntime] initializing field: {}", name);
        field.init(ctx).await?;
        self.field_dispatcher.register_field(name, field);
        Ok(())
    }

    /// Register a processor with the processor registry.
    pub fn register_processor(&mut self, processor: Box<dyn crate::processor::processor::Processor + Send>) {
        tracing::info!("[FieldRuntime] registering processor: {}", processor.name());
        self.processor_registry.register(processor);
    }

    /// Subscribe all processors to the event bus.
    pub fn subscribe_processors(&mut self, event_bus: &Arc<EventBus>) {
        self.processor_registry.subscribe_all(event_bus);
    }

    /// Process a signal through the full pipeline and recursively cascade
    /// all emitted signals until the network reaches equilibrium.
    ///
    /// Convergence is guaranteed by activation decay: each processing hop
    /// reduces signal activation, and processors whose activation threshold
    /// exceeds the signal's activation are skipped. Eventually no processor
    /// activates and the cascade terminates naturally.
    ///
    /// This replaces the old inline main.rs cascade loop with the same
    /// breadth-first semantics — emitted signals go to the back of the
    /// internal queue and are dispatched through both processor logic
    /// and field state update on each iteration.
    pub async fn process_signal_cascade(
        &mut self,
        ctx: &FieldContext,
        root: SignalArc,
    ) -> CascadeMetrics {
        use std::collections::VecDeque;

        let mut total_signals = 0;
        let mut all_emitted: Vec<SignalArc> = Vec::new();
        let mut all_signal_types: Vec<String> = Vec::new();
        let mut queue: VecDeque<SignalArc> = VecDeque::new();
        queue.push_back(root);

        while let Some(signal) = queue.pop_front() {
            total_signals += 1;
            all_signal_types.push(signal.signal_type().to_string());

            // Single dispatch through both processor pipeline and field state update
            let emitted = self.dispatch(ctx, signal).await;

            if !emitted.is_empty() {
                for sig in &emitted {
                    queue.push_back(sig.clone());
                }
                all_emitted.extend(emitted);
            }
        }

        CascadeMetrics {
            total_signals,
            emitted_signals: all_emitted,
            signal_types: all_signal_types,
        }
    }

    /// Dispatch a signal through the full pipeline:
    /// 1. Route to subscribed processors (cognitive transformation)
    /// 2. Route to the matching field's handle_signal (state update)
    ///
    /// Returns emitted signals for the cascade loop.
    pub async fn dispatch(
        &mut self,
        ctx: &FieldContext,
        signal: SignalArc,
    ) -> Vec<SignalArc> {
        let signal_type = signal.signal_type().to_string();

        // Phase 1: Dispatch to processors for cognitive transformation
        let emitted = self.processor_registry.dispatch(ctx, signal.clone()).await;

        // Phase 2: Dispatch to field for state update
        if let Err(e) = self.field_dispatcher.dispatch(&signal, ctx).await {
            tracing::warn!("[FieldRuntime] field dispatch error for {}: {}", signal_type, e);
        }

        emitted
    }

    /// List all registered field names.
    pub fn field_names(&self) -> Vec<String> {
        self.field_dispatcher.field_names()
    }

    /// Collect state snapshots from all registered fields.
    pub fn snapshot_states(&self) -> Vec<(String, Box<dyn std::any::Any + Send>)> {
        self.field_dispatcher.snapshot_states()
    }

    /// List all registered processor names.
    pub fn processor_names(&self) -> Vec<String> {
        self.processor_registry.names()
    }

    /// Shut down all fields and processors.
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[FieldRuntime] shutting down...");
        self.processor_registry.shutdown_all().await?;
        tracing::info!("[FieldRuntime] shutdown complete");
        Ok(())
    }
}

impl Default for FieldRuntime {
    fn default() -> Self {
        Self::new()
    }
}
