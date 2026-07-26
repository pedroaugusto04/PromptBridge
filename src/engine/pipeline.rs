use crate::engine::parser::{ExtractedItem, TechnicalParser};
use crate::engine::templates::{TemplateEngine, TransformMode};
use crate::providers::types::{ChatMessage, CompletionRequest};
use crate::providers::LlmProvider;
use crate::utils::error::Result;

pub struct TransformationPipeline;

#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub original_text: String,
    pub sanitized_text: String,
    pub extracted_items: Vec<ExtractedItem>,
    pub raw_llm_response: String,
    pub final_prompt: String,
}

impl TransformationPipeline {
    pub async fn execute(
        provider: &dyn LlmProvider,
        input: &str,
        mode: TransformMode,
        target_language: &str,
    ) -> Result<PipelineResult> {
        // 1. Sanitize & extract technical elements
        let (sanitized_text, extracted_items) = TechnicalParser::extract(input);

        // 2. Build system prompt
        let system_prompt = TemplateEngine::get_system_prompt(mode, target_language);

        // 3. Prepare completion request
        let request = CompletionRequest {
            messages: vec![
                ChatMessage::system(system_prompt),
                ChatMessage::user(&sanitized_text),
            ],
            temperature: Some(0.2),
            max_tokens: None,
            model: None,
        };

        // 4. Invoke LLM provider
        let response = provider.completion(&request).await?;
        let raw_llm_response = response.content.trim().to_string();

        // 5. Restore technical elements
        let final_prompt = TechnicalParser::restore(&raw_llm_response, &extracted_items);

        Ok(PipelineResult {
            original_text: input.to_string(),
            sanitized_text,
            extracted_items,
            raw_llm_response,
            final_prompt,
        })
    }
}
