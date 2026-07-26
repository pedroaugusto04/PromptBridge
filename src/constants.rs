//! Centralized constants, default values, magic strings, and configuration templates.

use std::time::Duration;

/// Application metadata
pub const APP_NAME: &str = "promptbridge";
pub const CONFIG_FILE_NAME: &str = "promptbridge.toml";
pub const ENV_PREFIX: &str = "PROMPTBRIDGE_";
pub const DEFAULT_OLLAMA_MODEL: &str = "llama3.2";
pub const DEFAULT_TARGET_LANGUAGE: &str = "en";

/// Dynamic getters with Environment Variable overrides & fallback defaults

pub fn get_ollama_base_url() -> String {
    std::env::var("PROMPTBRIDGE_OLLAMA_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string())
}

pub fn get_ollama_model() -> String {
    std::env::var("PROMPTBRIDGE_OLLAMA_MODEL")
        .unwrap_or_else(|_| "llama3.2".to_string())
}

pub fn get_openai_base_url() -> String {
    std::env::var("PROMPTBRIDGE_OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string())
}

pub fn get_openai_model() -> String {
    std::env::var("PROMPTBRIDGE_OPENAI_MODEL")
        .unwrap_or_else(|_| "gpt-4o-mini".to_string())
}

pub fn get_target_language() -> String {
    std::env::var("PROMPTBRIDGE_TARGET_LANGUAGE")
        .unwrap_or_else(|_| "en".to_string())
}

pub fn get_temperature() -> f32 {
    std::env::var("PROMPTBRIDGE_TEMPERATURE")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.2)
}

pub fn get_display_mode() -> String {
    std::env::var("PROMPTBRIDGE_MODE")
        .unwrap_or_else(|_| "preview".to_string())
}

pub fn get_request_timeout() -> Duration {
    let secs = std::env::var("PROMPTBRIDGE_REQUEST_TIMEOUT_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30);
    Duration::from_secs(secs)
}

/// Placeholder token prefixes for technical content preservation
pub const PLACEHOLDER_PREFIX: &str = "__PB_";
pub const PLACEHOLDER_CODE_BLOCK_PREFIX: &str = "__PB_CODE_BLOCK_";
pub const PLACEHOLDER_INLINE_CODE_PREFIX: &str = "__PB_CODE_INLINE_";
pub const PLACEHOLDER_PATH_PREFIX: &str = "__PB_PATH_";

/// Regex patterns for technical content identification
pub const REGEX_FENCED_CODE: &str = r"(?s)```[a-zA-Z0-9_-]*\n.*?```|```.*?```";
pub const REGEX_INLINE_CODE: &str = r"`[^`\n]+`";
pub const REGEX_FILE_PATH: &str = r#"(?i)\b(?:[a-z]:\\|/|\./|\.\./|[a-z0-9_.-]+/[a-z0-9_.-]+(?:\.[a-z0-9]+)?|[a-z0-9_.-]+\.(?:rs|ts|js|py|json|toml|yaml|yml|md|html|css|cpp|c|h|go|java|sh|ps1))\b"#;

/// Default configuration template
pub const DEFAULT_CONFIG_TOML: &str = r#"
[general]
default_provider = "ollama"
target_language = "en"
mode = "preview"
auto_copy_clipboard = false
preserve_technical_terms = true
request_timeout_seconds = 60

[providers.ollama]
type = "ollama"
base_url = "http://localhost:11434"
model = "llama3.2"
temperature = 0.2

[providers.openai]
type = "openai"
base_url = "https://api.openai.com/v1"
api_key = "env:OPENAI_API_KEY"
model = "gpt-4o-mini"
temperature = 0.2

[providers.mock]
type = "mock"
temperature = 0.0
"#;

/// Base system prompt instructions
pub const SYSTEM_PROMPT_BASE_INSTRUCTIONS: &str = concat!(
    "You are PromptBridge, an expert AI prompt engineering system.\n",
    "STRICT RULES:\n",
    "1. Preserved Tokens: All placeholders formatted like `__PB_CODE_BLOCK_X__`, `__PB_CODE_INLINE_X__`, `__PB_PATH_X__` MUST BE PRESERVED EXACTLY AS WRITTEN. Do not alter, translate, remove, or modify them.\n",
    "2. Technical Terms: Do NOT translate programming languages, framework names (e.g., Tokio, React, Serde, Cargo), library functions, API routes, or CLI flags.\n",
    "3. Output Format: Return ONLY the transformed prompt text. Do NOT include markdown code blocks around the entire output, meta commentary, or greetings.\n",
    "4. Target Language: The natural language portion of the prompt must be in '{target_language}'."
);
