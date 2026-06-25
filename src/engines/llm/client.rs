use async_trait::async_trait;
use crate::engines::llm::types::{CompletionRequest, CompletionResponse, LLMError};

/// A generic LLM client.
///
/// Implementations can wrap OpenAI-compatible APIs (OpenRouter, Anthropic, Ollama, etc.)
/// or any other LLM provider. The trait is designed to be provider-agnostic.
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// The name of this provider (e.g., "openrouter", "anthropic", "ollama").
    fn provider_name(&self) -> &str;

    /// Send a completion request and return the full response.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError>;

    /// Check whether the provider is reachable and healthy.
    async fn health_check(&self) -> Result<bool, LLMError> {
        Ok(true)
    }

    /// Return the current configured model for this client.
    fn model(&self) -> &str;
}
