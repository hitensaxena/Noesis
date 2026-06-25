use tracing;

use super::client::LLMClient;
use super::types::{CompletionRequest, CompletionResponse, LLMError};

/// A model chain that tries models in order and fails over on error.
///
/// When the primary model fails (rate limit, timeout, server error),
/// the next model in the chain is tried automatically. This matches
/// curlyos-core's model chain pattern where OpenRouter free tiers
/// frequently 429 and need fallback.
pub struct ModelChain {
    /// Ordered list of model names to try.
    models: Vec<String>,
    /// Index into models that we're currently using.
    current_index: usize,
    /// The provider to use for all models in the chain.
    provider: Box<dyn LLMClient>,
}

impl ModelChain {
    /// Create a new model chain.
    ///
    /// # Arguments
    /// * `provider` - The LLM provider (e.g., OpenRouter)
    /// * `models` - Ordered list of model names. The first is primary.
    pub fn new(provider: Box<dyn LLMClient>, models: Vec<String>) -> Self {
        Self {
            models,
            current_index: 0,
            provider,
        }
    }

    /// Get the current primary model name.
    pub fn current_model(&self) -> &str {
        self.models
            .get(self.current_index)
            .map(|s| s.as_str())
            .unwrap_or("unknown")
    }

    /// Send a completion request with automatic failover.
    ///
    /// Tries each model in the chain until one succeeds.
    /// Returns `ChainExhausted` if all models fail.
    pub async fn complete_with_failover(
        &mut self,
        mut request: CompletionRequest,
    ) -> Result<CompletionResponse, LLMError> {
        let start_index = self.current_index;
        let mut last_error: Option<LLMError> = None;

        // Try from current index, then cycle through all models
        for offset in 0..self.models.len() {
            let idx = (start_index + offset) % self.models.len();
            let model = &self.models[idx];
            request.model = model.clone();

            tracing::info!(
                "[ModelChain] trying model: {} (attempt {}/{})",
                model,
                offset + 1,
                self.models.len()
            );

            match self.provider.complete(request.clone()).await {
                Ok(response) => {
                    self.current_index = idx;
                    tracing::info!(
                        "[ModelChain] success with model: {}",
                        model
                    );
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    tracing::warn!(
                        "[ModelChain] model {} failed: {:?}",
                        model,
                        last_error.as_ref().unwrap()
                    );
                }
            }

            // Small delay before retry to avoid hammering the API
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Err(LLMError::ChainExhausted(format!(
            "tried {} models, last error: {:?}",
            self.models.len(),
            last_error
        )))
    }
}
