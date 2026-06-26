//! Noesis — a decentralized cognitive architecture.
//!
//! Noesis models cognition as an emergent decentralized network. Fields own state,
//! Processors transform signals, and Signals are the only communication mechanism.
//! No central controller. No god objects. Intelligence emerges from recursive signal propagation.

#![recursion_limit = "256"]

pub mod kernel;
pub mod field_runtime;
pub mod processor;
pub mod signals;
pub mod fields;
pub mod processors;
pub mod storage;
pub mod interfaces;
pub mod engines;
pub mod tui;
pub mod docs;

// Re-export key types for convenience
pub use kernel::kernel::Kernel;
pub use kernel::registry::Registry;
pub use kernel::bus::EventBus;
pub use kernel::signal::{Signal, SignalArc, SignalMeta, SignalType};
pub use field_runtime::field::Field;
pub use field_runtime::context::FieldContext;
pub use processor::processor::Processor;
