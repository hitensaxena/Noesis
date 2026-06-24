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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignalMeta {
    pub id: Uuid,
    pub signal_type: SignalType,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub depth: u32,
}

impl SignalMeta {
    pub fn new(signal_type: SignalType, source: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            signal_type,
            timestamp: Utc::now(),
            source: source.to_string(),
            depth: 0,
        }
    }

    pub fn child(&self, signal_type: SignalType, source: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            signal_type,
            timestamp: Utc::now(),
            source: source.to_string(),
            depth: self.depth + 1,
        }
    }
}

/// The Signal trait — everything that travels the event bus.
pub trait Signal: Debug + Send + Sync {
    fn signal_type(&self) -> SignalType;
    fn meta(&self) -> &SignalMeta;
    fn as_any(&self) -> &dyn Any;
}

pub type SignalArc = Arc<dyn Signal>;
