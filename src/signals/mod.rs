pub mod memory;
pub mod identity;
pub mod executive;
pub mod awareness;

pub use memory::*;
pub use identity::*;
pub use executive::*;
pub use awareness::*;

/// All signal type constants in the Noesis system.
pub mod types {
    use crate::eventbus::signal::SignalType;

    // Memory
    pub const EPISODE_RECORDED: SignalType = SignalType::new("episode.recorded");
    pub const FACT_EXTRACTED: SignalType = SignalType::new("fact.extracted");
    pub const MEMORY_CONSOLIDATED: SignalType = SignalType::new("memory.consolidated");
    pub const PATTERN_DETECTED: SignalType = SignalType::new("pattern.detected");

    // Identity
    pub const BELIEF_CHANGED: SignalType = SignalType::new("belief.changed");
    pub const TRAIT_DETECTED: SignalType = SignalType::new("trait.detected");
    pub const IDENTITY_UPDATED: SignalType = SignalType::new("identity.updated");

    // Executive
    pub const GOAL_CREATED: SignalType = SignalType::new("goal.created");
    pub const GOAL_COMPLETED: SignalType = SignalType::new("goal.completed");
    pub const DECISION_EVALUATED: SignalType = SignalType::new("decision.evaluated");

    // Awareness
    pub const ATTENTION_SHIFTED: SignalType = SignalType::new("attention.shifted");
    pub const CURIOSITY_DETECTED: SignalType = SignalType::new("curiosity.detected");
    pub const NARRATIVE_GENERATED: SignalType = SignalType::new("narrative.generated");

    // Input
    pub const INGEST_REQUEST: SignalType = SignalType::new("ingest.request");
}

/// Macro to implement the Signal trait for a struct.
macro_rules! signal_impl {
    ($name:ident, $signal_type:ident, $source:expr) => {
        impl crate::eventbus::signal::Signal for $name {
            fn signal_type(&self) -> crate::eventbus::signal::SignalType {
                crate::signals::types::$signal_type
            }
            fn meta(&self) -> &crate::eventbus::signal::SignalMeta {
                &self.meta
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

pub(crate) use signal_impl;
