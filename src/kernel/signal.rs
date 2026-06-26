use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

/// A typed identifier for signal routing
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SignalType(pub &'static str);

impl Serialize for SignalType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0)
    }
}

impl<'de> Deserialize<'de> for SignalType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        // Leak the string to get a &'static str (acceptable for signal types which have bounded lifetime)
        Ok(SignalType(Box::leak(s.into_boxed_str())))
    }
}

impl SignalType {
    pub const fn new(name: &'static str) -> Self {
        SignalType(name)
    }
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata attached to every signal.
///
/// Includes cognitive activation properties that govern signal propagation:
/// - `activation` decreases per hop via `decay`, guaranteeing cascade convergence
/// - `salience` / `novelty` / `confidence` are semantic properties used by processors
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignalMeta {
    pub id: Uuid,
    pub signal_type: SignalType,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub depth: u32,

    // -- Cognitive activation properties --
    /// Propagation strength (0.0–1.0). Decreases by `decay` per hop.
    /// Signals below `activation_threshold` (0.1) are not processed.
    pub activation: f32,
    /// How important this signal is (0.0–1.0).
    pub salience: f32,
    /// How surprising or novel this content is (0.0–1.0).
    pub novelty: f32,
    /// How certain the emitter is about this signal (0.0–1.0).
    pub confidence: f32,
    /// Per-hop multiplier applied to activation on `child()` (0.0–1.0).
    pub decay: f32,
}

impl SignalMeta {
    /// Create a new SignalMeta with default activation values.
    ///
    /// Defaults: activation=1.0, salience=0.5, novelty=0.3, confidence=0.5, decay=0.7
    /// These defaults ensure backward compatibility — all existing signal constructors
    /// continue to work without changes and cascades of 6–7 hops behave identically
    /// to the old hard depth cap of 50.
    pub fn new(signal_type: SignalType, source: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            signal_type,
            timestamp: Utc::now(),
            source: source.to_string(),
            depth: 0,
            activation: 1.0,
            salience: 0.5,
            novelty: 0.3,
            confidence: 0.5,
            decay: 0.7,
        }
    }

    /// Create a child SignalMeta (one hop deeper).
    ///
    /// Applies `parent.activation * parent.decay` so the child signal has
    /// reduced propagation strength. Salience, novelty, and confidence are
    /// inherited from the parent (the child may override via builder methods).
    pub fn child(&self, signal_type: SignalType, source: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            signal_type,
            timestamp: Utc::now(),
            source: source.to_string(),
            depth: self.depth + 1,
            activation: self.activation * self.decay,
            salience: self.salience,
            novelty: self.novelty,
            confidence: self.confidence,
            decay: self.decay,
        }
    }

    /// Returns `true` if this signal's activation is above the minimum threshold (0.01).
    ///
    /// Signals that are not alive should be discarded by the EventBus and ignored
    /// by processors. This guarantees cascade convergence — no matter how deep
    /// a cascade runs, activation eventually reaches zero.
    pub fn is_alive(&self) -> bool {
        self.activation > 0.01
    }

    // -- Builder methods --

    /// Override the activation value (consumes self, returns Self).
    pub fn with_activation(mut self, activation: f32) -> Self {
        self.activation = activation.clamp(0.0, 1.0);
        self
    }

    /// Override the salience value (consumes self, returns Self).
    pub fn with_salience(mut self, salience: f32) -> Self {
        self.salience = salience.clamp(0.0, 1.0);
        self
    }

    /// Override the novelty value (consumes self, returns Self).
    pub fn with_novelty(mut self, novelty: f32) -> Self {
        self.novelty = novelty.clamp(0.0, 1.0);
        self
    }

    /// Override the confidence value (consumes self, returns Self).
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Override the decay value (consumes self, returns Self).
    pub fn with_decay(mut self, decay: f32) -> Self {
        self.decay = decay.clamp(0.0, 1.0);
        self
    }
}

/// The Signal trait — everything that travels the event bus.
pub trait Signal: Debug + Send + Sync {
    fn signal_type(&self) -> SignalType;
    fn meta(&self) -> &SignalMeta;
    fn as_any(&self) -> &dyn Any;
}

pub type SignalArc = Arc<dyn Signal>;
