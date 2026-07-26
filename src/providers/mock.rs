use crate::providers::types::{CompletionRequest, CompletionResponse};
use crate::providers::LlmProvider;
use crate::utils::error::Result;
use async_trait::async_trait;

pub struct MockProvider {
    model: String,
}

impl MockProvider {
    pub fn new(model: Option<String>) -> Self {
        Self {
            model: model.unwrap_or_else(|| "mock-v1".to_string()),
        }
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    async fn completion(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let user_prompt = request
            .messages
            .iter()
            .rfind(|m| m.role == crate::providers::types::Role::User)
            .map(|m| m.content.as_str())
            .unwrap_or_default();

        // Simulate mock translation / optimization by prefixing and echoing user input
        let content = if user_prompt.contains("Se a requisição falhar") {
            "If the request fails in __PB_PATH_0__, raise a __PB_CODE_INLINE_0__ error using __PB_CODE_INLINE_1__.".to_string()
        } else {
            format!("[Mock Transformed]: {}", user_prompt)
        };

        Ok(CompletionResponse {
            content,
            model: self.model.clone(),
            prompt_tokens: Some(10),
            completion_tokens: Some(15),
        })
    }

    fn name(&self) -> &str {
        "Mock (Dry-Run)"
    }
}
