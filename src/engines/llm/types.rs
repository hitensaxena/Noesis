use serde::{Deserialize, Serialize};

/// Role of a message participant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self {
            role: Role::System,
            content: content.to_string(),
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: Role::User,
            content: content.to_string(),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: Role::Assistant,
            content: content.to_string(),
        }
    }
}

/// Token usage information from a completion.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// Request to an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Whether to stream the response token by token
    #[serde(skip)]
    pub stream: bool,
}

impl CompletionRequest {
    pub fn new(model: &str, messages: Vec<Message>) -> Self {
        Self {
            model: model.to_string(),
            messages,
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop: None,
            stream: false,
        }
    }

    pub fn with_temperature(mut self, t: f32) -> Self {
        self.temperature = Some(t);
        self
    }

    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = Some(n);
        self
    }
}

/// Response from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub finish_reason: Option<String>,
    pub usage: Option<Usage>,
    pub model: String,
}

/// Error types for LLM operations.
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Provider error: {status} {message}")]
    Provider { status: u16, message: String },

    #[error("Timeout after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("Rate limited. Retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    #[error("Empty response from provider")]
    EmptyResponse,

    #[error("All models in chain failed: {0}")]
    ChainExhausted(String),

    #[error("JSON parse error: {0}")]
    JsonParse(String),
}
