pub mod mock;
pub mod ollama;
pub mod openai;
pub mod types;

use crate::config::ProviderConfig;
use crate::utils::error::{PromptBridgeError, Result};
use async_trait::async_trait;
use types::{CompletionRequest, CompletionResponse};

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send completion request to provider
    async fn completion(&self, request: &CompletionRequest) -> Result<CompletionResponse>;

    /// Returns human-readable name of provider
    fn name(&self) -> &str;
}

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(config: &ProviderConfig) -> Result<Box<dyn LlmProvider>> {
        match config.provider_type.as_str() {
            "ollama" => Ok(Box::new(ollama::OllamaProvider::new(
                config.base_url.clone(),
                config.model.clone(),
                config.temperature,
                config.api_key.clone(),
            )?)),
            "openai" | "openai-compatible" => Ok(Box::new(openai::OpenAiProvider::new(
                config.base_url.clone(),
                config.api_key.clone(),
                config.model.clone(),
                config.temperature,
            )?)),
            "mock" => Ok(Box::new(mock::MockProvider::new(config.model.clone()))),
            unknown => Err(PromptBridgeError::Config(format!(
                "Unknown provider type '{}'. Supported: 'ollama', 'openai', 'mock'",
                unknown
            ))),
        }
    }
}
