//! Noesis — a decentralized cognitive architecture.
//!
//! Noesis models cognition as an emergent decentralized network. Fields own state,
//! Processors transform signals, and Signals are the only communication mechanism.
//! No central controller. No god objects. Intelligence emerges from recursive signal propagation.

pub mod core;
pub mod eventbus;
pub mod scheduler;
pub mod field;
pub mod processor;
pub mod plugin;
pub mod signals;
pub mod fields;
pub mod processors;
pub mod storage;
pub mod interfaces;
pub mod metrics;
pub mod engines;

// Re-export key types for convenience
pub use core::kernel::Kernel;
pub use core::registry::Registry;
pub use eventbus::bus::EventBus;
pub use eventbus::signal::{Signal, SignalArc, SignalMeta, SignalType};
pub use field::field::Field;
pub use field::context::FieldContext;
pub use processor::processor::Processor;
