use crate::constants::{
    APP_NAME, CONFIG_FILE_NAME, DEFAULT_CONFIG_TOML, DEFAULT_TARGET_LANGUAGE, ENV_PREFIX,
};
use crate::utils::error::{PromptBridgeError, Result};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub default_provider: String,
    pub target_language: String,
    pub mode: String, // "preview" or "silent"
    pub auto_copy_clipboard: bool,
    pub preserve_technical_terms: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_provider: "ollama".to_string(),
            target_language: DEFAULT_TARGET_LANGUAGE.to_string(),
            mode: "preview".to_string(),
            auto_copy_clipboard: false,
            preserve_technical_terms: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(rename = "type")]
    pub provider_type: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
}

impl Config {
    pub fn load(custom_path: Option<PathBuf>) -> Result<Self> {
        let mut figment = Figment::new();

        // 1. Default config fallback from constants
        figment = figment.merge(Toml::string(DEFAULT_CONFIG_TOML));

        // 2. Global user config (~/.config/promptbridge/config.toml on Linux,
        //    ~/Library/Application Support/promptbridge/config.toml on macOS,
        //    %APPDATA%\promptbridge\config.toml on Windows)
        if let Some(mut user_config) = dirs::config_dir() {
            user_config.push(APP_NAME);
            user_config.push(CONFIG_FILE_NAME);
            if user_config.exists() {
                figment = figment.merge(Toml::file(user_config));
            }
        }

        // 3. Local directory config (./promptbridge.toml)
        let local_config = PathBuf::from(CONFIG_FILE_NAME);
        if local_config.exists() {
            figment = figment.merge(Toml::file(local_config));
        }

        // 4. Custom specified config path
        if let Some(path) = custom_path {
            if path.exists() {
                figment = figment.merge(Toml::file(path));
            }
        }

        // 5. Environment variables (PROMPTBRIDGE_*)
        figment = figment.merge(Env::prefixed(ENV_PREFIX).split("_"));

        let config: Config = figment
            .extract()
            .map_err(|e| PromptBridgeError::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    pub fn get_default_provider_config(&self) -> Result<&ProviderConfig> {
        self.providers
            .get(&self.general.default_provider)
            .ok_or_else(|| {
                PromptBridgeError::Config(format!(
                    "Default provider '{}' not found in configuration",
                    self.general.default_provider
                ))
            })
    }
}
