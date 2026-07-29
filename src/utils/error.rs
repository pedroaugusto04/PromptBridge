use crate::messages::{TIP_CONFIG_ERROR, TIP_EXEC_ERROR, TIP_PROVIDER_ERROR};
use thiserror::Error;

/// Centralized domain error type for PromptBridge
#[derive(Error, Debug)]
pub enum PromptBridgeError {
    #[error("Configuration Error: {0}")]
    Config(String),

    #[error("LLM Provider Error [{provider}]: {message}")]
    Provider { provider: String, message: String },

    #[error("Technical Parser Error: {0}")]
    Parser(String),

    #[error("Prompt Engine Error: {0}")]
    Engine(String),

    #[error("CLI Exec Gateway Error: {0}")]
    Exec(String),

    #[error("Clipboard Operation Error: {0}")]
    Clipboard(String),

    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP Request Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Serialization Error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, PromptBridgeError>;

impl PromptBridgeError {
    /// Formats error message with color and user-friendly guidance for terminal output
    pub fn user_facing_message(&self) -> String {
        match self {
            Self::Config(msg) => format!("Configuration Issue: {}\nTip: {}", msg, TIP_CONFIG_ERROR),
            Self::Provider { provider, message } => format!(
                "Provider '{}' Failed: {}\nTip: {}",
                provider, message, TIP_PROVIDER_ERROR
            ),
            Self::Exec(msg) => format!("Proxy Gateway Error: {}\nTip: {}", msg, TIP_EXEC_ERROR),
            other => format!("{}", other),
        }
    }
}
