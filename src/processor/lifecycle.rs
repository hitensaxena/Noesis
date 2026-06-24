use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use tracing;

use crate::eventbus::bus::EventBus;
use crate::eventbus::signal::{SignalArc, SignalType};
use crate::eventbus::subscription::Subscription;
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// Wraps a processor with its event bus subscriptions and manages its lifecycle.
pub struct ProcessorHandle {
    processor: Box<dyn Processor>,
    subscriptions: Vec<Subscription>,
}

impl ProcessorHandle {
    pub fn new(processor: Box<dyn Processor>) -> Self {
        Self {
            processor,
            subscriptions: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        self.processor.name()
    }

    pub fn subscribed_signals(&self) -> &[SignalType] {
        self.processor.subscribed_signals()
    }

    /// Subscribe this processor to all the signal types it declares.
    pub fn subscribe(&mut self, event_bus: &Arc<EventBus>) {
        for signal_type in self.processor.subscribed_signals() {
            let sub = event_bus.subscribe(signal_type.clone(), self.processor.name());
            self.subscriptions.push(sub);
            tracing::debug!(
                "[Processor] {} subscribed to {}",
                self.processor.name(),
                signal_type
            );
        }
    }

    /// Process a signal and return emitted signals.
    pub async fn process(
        &mut self,
        ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        self.processor.process(ctx, signal).await
    }

    /// Shut down and unsubscribe.
    pub async fn shutdown(&mut self) -> Result<()> {
        for sub in &self.subscriptions {
            sub.unsubscribe();
        }
        self.processor.shutdown().await
    }
}

/// Manages a collection of ProcessorHandles with a fast dispatch map.
///
/// The dispatch map maps each SignalType to the indices of processors
/// that subscribe to it, allowing O(1) lookup when routing signals.
pub struct ProcessorRegistry {
    handles: Vec<ProcessorHandle>,
    /// SignalType → indices into handles[]
    dispatch_map: HashMap<SignalType, Vec<usize>>,
}

impl ProcessorRegistry {
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
            dispatch_map: HashMap::new(),
        }
    }

    /// Register a processor and rebuild the dispatch map.
    pub fn register(&mut self, processor: Box<dyn Processor>) {
        let idx = self.handles.len();
        self.handles.push(ProcessorHandle::new(processor));
        // Add this processor to the dispatch map for each signal it subscribes to
        for signal_type in self.handles[idx].subscribed_signals() {
            self.dispatch_map
                .entry(signal_type.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }
    }

    /// Subscribe all processors to the event bus.
    pub fn subscribe_all(&mut self, event_bus: &Arc<EventBus>) {
        for handle in &mut self.handles {
            handle.subscribe(event_bus);
        }
    }

    /// Dispatch a signal to all processors that subscribe to its type.
    ///
    /// Returns all signals emitted by the processors during this dispatch.
    pub async fn dispatch(
        &mut self,
        ctx: &FieldContext,
        signal: SignalArc,
    ) -> Vec<SignalArc> {
        let signal_type = signal.signal_type();
        let processor_indices = match self.dispatch_map.get(&signal_type) {
            Some(indices) => indices.clone(),
            None => return Vec::new(),
        };

        let mut all_emitted: Vec<SignalArc> = Vec::new();

        for idx in &processor_indices {
            if *idx >= self.handles.len() {
                continue;
            }
            let handle = &mut self.handles[*idx];
            match handle.process(ctx, signal.clone()).await {
                Ok(emitted) => {
                    if !emitted.is_empty() {
                        tracing::debug!(
                            "[Dispatch] {} emitted {} signal(s) from {}",
                            handle.name(),
                            emitted.len(),
                            signal_type
                        );
                        all_emitted.extend(emitted);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "[Dispatch] processor {} failed on {}: {}",
                        handle.name(),
                        signal_type,
                        e
                    );
                }
            }
        }

        all_emitted
    }

    /// Run a full cascade cycle: dispatch a signal, then recursively
    /// process all emitted signals until the network reaches equilibrium.
    ///
    /// Returns the total number of signals processed in the cascade.
    pub async fn dispatch_cascade(
        &mut self,
        ctx: &FieldContext,
        initial_signal: SignalArc,
    ) -> usize {
        let mut total_processed = 1; // the initial signal
        let mut pending = vec![initial_signal];
        let max_depth = 50;

        while let Some(signal) = pending.pop() {
            let depth = signal.meta().depth;
            if depth > max_depth {
                tracing::warn!("[Cascade] depth {} exceeded max, terminating", depth);
                continue;
            }

            let emitted = self.dispatch(ctx, signal).await;
            if emitted.is_empty() {
                continue;
            }

            // Propagate depth: child signals get parent depth + 1
            for sig in emitted {
                total_processed += 1;
                pending.push(sig);
            }
        }

        total_processed
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    pub fn names(&self) -> Vec<String> {
        self.handles
            .iter()
            .map(|h| h.name().to_string())
            .collect()
    }

    pub async fn shutdown_all(&mut self) -> Result<()> {
        for mut handle in self.handles.drain(..) {
            handle.shutdown().await?;
        }
        Ok(())
    }
}

impl Default for ProcessorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
