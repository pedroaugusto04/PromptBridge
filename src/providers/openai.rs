use crate::constants::{get_openai_base_url, get_openai_model, get_request_timeout, get_temperature};
use crate::providers::types::{CompletionRequest, CompletionResponse, Role};
use crate::providers::LlmProvider;
use crate::utils::error::{PromptBridgeError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAiProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    temperature: f32,
}

impl OpenAiProvider {
    pub fn new(
        base_url: Option<String>,
        api_key: Option<String>,
        model: Option<String>,
        temperature: Option<f32>,
    ) -> Result<Self> {
        let base_url = base_url.unwrap_or_else(get_openai_base_url);

        let api_key = match api_key {
            Some(key) if key.starts_with("env:") => {
                let env_var = key.trim_start_matches("env:");
                std::env::var(env_var).unwrap_or_default()
            }
            Some(key) => key,
            None => std::env::var("OPENAI_API_KEY").unwrap_or_default(),
        };

        let model = model.unwrap_or_else(get_openai_model);
        let temperature = temperature.unwrap_or_else(get_temperature);
        let timeout = get_request_timeout();

        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| PromptBridgeError::Provider {
                provider: "OpenAI".to_string(),
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            client,
            base_url,
            api_key,
            model,
            temperature,
        })
    }
}

#[derive(Serialize)]
struct OpenAiChatPayload {
    model: String,
    messages: Vec<OpenAiMessagePayload>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct OpenAiMessagePayload {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponsePayload {
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(Deserialize)]
struct OpenAiResponseMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn completion(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let messages = request
            .messages
            .iter()
            .map(|m| OpenAiMessagePayload {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect();

        let model = request.model.clone().unwrap_or_else(|| self.model.clone());
        let payload = OpenAiChatPayload {
            model: model.clone(),
            messages,
            temperature: request.temperature.unwrap_or(self.temperature),
            max_tokens: request.max_tokens,
        };

        let mut req = self.client.post(&url).json(&payload);
        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let res = req.send().await.map_err(|e| {
            if e.is_timeout() {
                PromptBridgeError::Provider {
                    provider: "OpenAI".to_string(),
                    message: format!("Request timed out. Adjust PROMPTBRIDGE_REQUEST_TIMEOUT_SECONDS if needed: {}", e),
                }
            } else {
                PromptBridgeError::Provider {
                    provider: "OpenAI".to_string(),
                    message: format!("HTTP request failed to {}: {}", url, e),
                }
            }
        })?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(PromptBridgeError::Provider {
                provider: "OpenAI".to_string(),
                message: format!("API error ({}) from {}: {}", status, url, body),
            });
        }

        let response_payload: OpenAiResponsePayload =
            res.json().await.map_err(|e| PromptBridgeError::Provider {
                provider: "OpenAI".to_string(),
                message: format!("Failed to parse OpenAI JSON response: {}", e),
            })?;

        let choice = response_payload.choices.first().ok_or_else(|| {
            PromptBridgeError::Provider {
                provider: "OpenAI".to_string(),
                message: "API returned zero choices".to_string(),
            }
        })?;

        let content = choice.message.content.clone().unwrap_or_default();

        Ok(CompletionResponse {
            content,
            model: response_payload.model,
            prompt_tokens: response_payload.usage.as_ref().and_then(|u| u.prompt_tokens),
            completion_tokens: response_payload.usage.as_ref().and_then(|u| u.completion_tokens),
        })
    }

    fn name(&self) -> &str {
        "OpenAI-Compatible"
    }
}
