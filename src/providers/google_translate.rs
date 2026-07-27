use crate::providers::types::{CompletionRequest, CompletionResponse};
use crate::providers::LlmProvider;
use crate::utils::error::{PromptBridgeError, Result};
use async_trait::async_trait;
use reqwest::Client;

pub struct GoogleTranslateProvider {
    client: Client,
    base_url: String,
}

impl GoogleTranslateProvider {
    pub fn new() -> Result<Self> {
        let timeout = std::time::Duration::from_secs(30);
        
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| PromptBridgeError::Provider {
                provider: "GoogleTranslate".to_string(),
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            client,
            base_url: "https://translate.googleapis.com/translate_a/single".to_string(),
        })
    }
}


// Google Translate free API returns a nested array structure:
// [[["translated", "original", null, 5], ...], null, "target_lang", ...]
type GoogleTranslateResponse = Vec<Vec<Vec<String>>>;

#[async_trait]
impl LlmProvider for GoogleTranslateProvider {
    async fn completion(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        // Extract the user message content (we only care about the last user message for translation)
        let user_content = request
            .messages
            .iter()
            .filter(|m| m.role == crate::providers::types::Role::User)
            .last()
            .map(|m| m.content.clone())
            .ok_or_else(|| PromptBridgeError::Provider {
                provider: "GoogleTranslate".to_string(),
                message: "No user message found in request".to_string(),
            })?;

        // Extract target language from system message or default to English
        let target_lang = request
            .messages
            .iter()
            .filter(|m| m.role == crate::providers::types::Role::System)
            .find_map(|m| {
                // Try to extract language from system prompt like "Translate to: en"
                if m.content.contains("Translate to:") {
                    m.content
                        .split("Translate to:")
                        .nth(1)
                        .map(|s| s.trim().to_string())
                } else if m.content.contains("target language:") {
                    m.content
                        .split("target language:")
                        .nth(1)
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "en".to_string());

        // Use auto-detection for source language
        let source_lang = "auto";

        let url = format!("{}?client=gtx", self.base_url);

        let params = [
            ("sl", source_lang),
            ("tl", &target_lang),
            ("dt", "t"),
            ("q", &user_content),
        ];

        let res = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    PromptBridgeError::Provider {
                        provider: "GoogleTranslate".to_string(),
                        message: format!("Request timed out: {}", e),
                    }
                } else {
                    PromptBridgeError::Provider {
                        provider: "GoogleTranslate".to_string(),
                        message: format!("Failed to connect to Google Translate: {}", e),
                    }
                }
            })?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(PromptBridgeError::Provider {
                provider: "GoogleTranslate".to_string(),
                message: format!("Google Translate API returned status {}: {}", status, body),
            });
        }

        let response_text = res.text().await.map_err(|e| PromptBridgeError::Provider {
            provider: "GoogleTranslate".to_string(),
            message: format!("Failed to read Google Translate response: {}", e),
        })?;

        // Parse the JSON response (Google Translate returns nested array structure)
        let response: GoogleTranslateResponse = serde_json::from_str(&response_text)
            .map_err(|e| PromptBridgeError::Provider {
                provider: "GoogleTranslate".to_string(),
                message: format!("Failed to parse Google Translate response: {}", e),
            })?;

        // Extract translated text from the nested array structure
        // Format: [[["translated", "original", null, 5], ...], ...]
        let translated_text = if let Some(first_level) = response.get(0) {
            first_level
                .iter()
                .filter_map(|sentence| sentence.first())
                .cloned()
                .collect::<Vec<String>>()
                .join("")
        } else {
            return Err(PromptBridgeError::Provider {
                provider: "GoogleTranslate".to_string(),
                message: "Invalid response structure from Google Translate".to_string(),
            });
        };

        Ok(CompletionResponse {
            content: translated_text,
            model: "google-translate".to_string(),
            prompt_tokens: None,
            completion_tokens: None,
        })
    }

    fn name(&self) -> &str {
        "GoogleTranslate"
    }
}
