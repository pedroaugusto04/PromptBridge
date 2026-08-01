use crate::providers::types::{CompletionRequest, CompletionResponse};
use crate::providers::LlmProvider;
use crate::utils::error::{PromptBridgeError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct GoogleTranslateProvider {
    client: Client,
    base_url: String,
}

impl GoogleTranslateProvider {
    pub fn new() -> Result<Self> {
        let timeout = std::time::Duration::from_secs(30);

        let client = Client::builder().timeout(timeout).build().map_err(|e| {
            PromptBridgeError::Provider {
                provider: "GoogleTranslate".to_string(),
                message: format!("Failed to create HTTP client: {}", e),
            }
        })?;

        Ok(Self {
            client,
            base_url: "https://translate.googleapis.com/translate_a/single".to_string(),
        })
    }
}

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

        // Use target_language from request or default to English
        let target_lang = request
            .target_language
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("en");

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
        // Use Value to handle complex structure with nulls
        let response: Value = serde_json::from_str(&response_text).map_err(|e| PromptBridgeError::Provider {
            provider: "GoogleTranslate".to_string(),
            message: format!("Failed to parse Google Translate response: {}", e),
        })?;

        // Extract translated text from the nested array structure
        // Format: [[["translated", "original", null, 5], ...], null, "target_lang", ...]
        let translated_text = if let Some(first_level) = response.get(0) {
            if let Some(array) = first_level.as_array() {
                array
                    .iter()
                    .filter_map(|sentence| {
                        // Each sentence is an array where first element is translated text
                        if let Some(sentence_array) = sentence.as_array() {
                            if let Some(translated) = sentence_array.get(0) {
                                translated.as_str()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<&str>>()
                    .join("")
            } else {
                return Err(PromptBridgeError::Provider {
                    provider: "GoogleTranslate".to_string(),
                    message: "Invalid response structure from Google Translate".to_string(),
                });
            }
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
