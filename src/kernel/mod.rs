//! The Kernel — runtime infrastructure for Noesis.
//!
//! The Kernel provides the environment in which cognition runs: event bus,
//! registry, scheduler, lifecycle, metrics, and plugin loading. It never
//! performs cognition — it sustains the cognitive fields and processors.

pub mod beat_coordinator;
pub mod bus;
pub mod capabilities;
pub mod catalog;
pub mod cloud_event;
pub mod kernel;
pub mod lifecycle;
pub mod metrics;
pub mod plugin;
pub mod registry;
pub mod runtime;
pub mod scheduler;
pub mod signal;
pub mod state;
pub mod subscription;
