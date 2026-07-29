use crate::constants::{
    get_ollama_base_url, get_ollama_model, get_request_timeout, get_temperature,
};
use crate::providers::types::{CompletionRequest, CompletionResponse, Role};
use crate::providers::LlmProvider;
use crate::utils::error::{PromptBridgeError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
    temperature: f32,
    auth_token: Option<String>,
    keep_alive: Option<String>,
}

impl OllamaProvider {
    pub fn new(
        base_url: Option<String>,
        model: Option<String>,
        temperature: Option<f32>,
        auth_token: Option<String>,
        keep_alive_minutes: Option<u64>,
    ) -> Result<Self> {
        let base_url = base_url.unwrap_or_else(get_ollama_base_url);
        let model = model.unwrap_or_else(get_ollama_model);
        let temperature = temperature.unwrap_or_else(get_temperature);
        let timeout = get_request_timeout();

        let keep_alive = keep_alive_minutes.map(|mins| format!("{}m", mins));

        let client = Client::builder().timeout(timeout).build().map_err(|e| {
            PromptBridgeError::Provider {
                provider: "Ollama".to_string(),
                message: format!("Failed to create HTTP client: {}", e),
            }
        })?;

        Ok(Self {
            client,
            base_url,
            model,
            temperature,
            auth_token,
            keep_alive,
        })
    }
}

#[derive(Serialize)]
struct OllamaChatPayload {
    model: String,
    messages: Vec<OllamaMessagePayload>,
    stream: bool,
    options: OllamaOptionsPayload,
}

#[derive(Serialize)]
struct OllamaMessagePayload {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptionsPayload {
    temperature: f32,
    keep_alive: Option<String>,
}

#[derive(Deserialize)]
struct OllamaResponsePayload {
    model: String,
    message: OllamaResponseMessage,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn completion(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        let messages = request
            .messages
            .iter()
            .map(|m| OllamaMessagePayload {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect();

        let model = request.model.clone().unwrap_or_else(|| self.model.clone());
        let payload = OllamaChatPayload {
            model: model.clone(),
            messages,
            stream: false,
            options: OllamaOptionsPayload {
                temperature: request.temperature.unwrap_or(self.temperature),
                keep_alive: self.keep_alive.clone(),
            },
        };

        let mut request_builder = self.client.post(&url).json(&payload);

        // Add Authorization header if an auth token is configured
        if let Some(token) = &self.auth_token {
            request_builder = request_builder.bearer_auth(token);
        }

        let res = request_builder
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    PromptBridgeError::Provider {
                        provider: "Ollama".to_string(),
                        message: format!("Request timed out. Adjust PROMPTBRIDGE_REQUEST_TIMEOUT_SECONDS if needed: {}", e),
                    }
                } else {
                    PromptBridgeError::Provider {
                        provider: "Ollama".to_string(),
                        message: format!("Failed to connect to Ollama at {}: {}", url, e),
                    }
                }
            })?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(PromptBridgeError::Provider {
                provider: "Ollama".to_string(),
                message: format!("Ollama API returned status {}: {}", status, body),
            });
        }

        let response_payload: OllamaResponsePayload =
            res.json().await.map_err(|e| PromptBridgeError::Provider {
                provider: "Ollama".to_string(),
                message: format!("Failed to deserialize Ollama response: {}", e),
            })?;

        Ok(CompletionResponse {
            content: response_payload.message.content,
            model: response_payload.model,
            prompt_tokens: response_payload.prompt_eval_count,
            completion_tokens: response_payload.eval_count,
        })
    }

    fn name(&self) -> &str {
        "Ollama"
    }
}
