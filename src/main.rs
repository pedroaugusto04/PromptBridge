use clap::Parser;
use promptbridge::cli::{Cli, Commands, ConfigSubcommand};
use promptbridge::config::{Config, ProviderConfig};
use promptbridge::engine::{TransformMode, TransformationPipeline};
use promptbridge::exec::ExecGateway;
use promptbridge::messages::{
    format_provider_list_item, MSG_INPUT_PROMPT_EMPTY,
};
use promptbridge::platform::get_platform;
use promptbridge::providers::{LlmProvider, ProviderFactory};
use promptbridge::utils::clipboard::copy_to_clipboard;
use promptbridge::utils::error::{PromptBridgeError, Result};
use promptbridge::utils::formatting::{format_diff, print_error};
use std::io::{self, Read};

#[tokio::main]
async fn main() {
    // Load .env file automatically if present in workspace
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    if let Err(err) = run_app(cli).await {
        print_error(&err.user_facing_message());
        std::process::exit(1);
    }
}

async fn run_app(cli: Cli) -> Result<()> {
    let mut config = Config::load(cli.config)?;

    // Apply CLI overrides to configuration
    if let Some(target_lang) = cli.target_lang {
        config.general.target_language = target_lang;
    }

    if let Some(provider_name) = &cli.provider {
        config.general.default_provider = provider_name.clone();
    }

    if let Some(model_name) = &cli.model {
        if let Some(provider_config) = config.providers.get_mut(&config.general.default_provider) {
            provider_config.model = Some(model_name.clone());
        }
    }

    match cli.command {
        Commands::Exec {
            args,
            preview,
            silent,
        } => {
            if preview {
                config.general.mode = "preview".to_string();
            } else if silent {
                config.general.mode = "silent".to_string();
            }

            ExecGateway::run(
                &args,
                &config,
                cli.provider.clone(),
                TransformMode::Transform,
                cli.dry_run,
            )
            .await?;
        }

        Commands::Transform { input } => {
            process_single_prompt(
                input,
                &config,
                cli.provider,
                TransformMode::Transform,
                cli.copy,
                cli.dry_run,
            )
            .await?;
        }

        Commands::Translate { input } => {
            process_single_prompt(
                input,
                &config,
                cli.provider,
                TransformMode::Translate,
                cli.copy,
                cli.dry_run,
            )
            .await?;
        }

        Commands::Optimize { input } => {
            process_single_prompt(
                input,
                &config,
                cli.provider,
                TransformMode::Optimize,
                cli.copy,
                cli.dry_run,
            )
            .await?;
        }

        Commands::Config { action } => match action {
            Some(ConfigSubcommand::Show) | None => {
                let toml_str = toml::to_string_pretty(&config)
                    .map_err(|e| PromptBridgeError::Config(e.to_string()))?;
                println!("{}", toml_str);
            }
            Some(ConfigSubcommand::Path) => {
                if let Some(user_config) = dirs::config_dir() {
                    println!("{}", user_config.join("promptbridge").join("config.toml").display());
                }
            }
        },

        Commands::Providers => {
            println!("Configured LLM Providers:");
            for (name, prov) in &config.providers {
                let is_default = name == &config.general.default_provider;
                println!("{}", format_provider_list_item(
                    name,
                    is_default,
                    &prov.provider_type,
                    prov.model.as_deref().unwrap_or("default")
                ));
            }
        }

        Commands::InstallShortcut => {
            install_shortcut()?;
        }

        Commands::InitConfig => {
            init_config_interactive()?;
        }
    }

    Ok(())
}

fn install_shortcut() -> Result<()> {
    let platform = get_platform();
    let result = platform.install_shortcut()?;
    
    println!("✓ Created helper script at: {}", result.script_path);
    println!("\n=== Installation complete ===");
    println!("{}", result.config_instructions);
    
    Ok(())
}

async fn process_single_prompt(
    input: Option<String>,
    config: &Config,
    override_provider: Option<String>,
    mode: TransformMode,
    copy: bool,
    dry_run: bool,
) -> Result<()> {
    let raw_text = match input {
        Some(text) if !text.trim().is_empty() => text,
        _ => read_stdin()?,
    };

    if raw_text.trim().is_empty() {
        return Err(PromptBridgeError::Engine(MSG_INPUT_PROMPT_EMPTY.to_string()));
    }

    let provider_name = override_provider
        .as_deref()
        .unwrap_or(&config.general.default_provider);

    let provider_config = config.providers.get(provider_name).ok_or_else(|| {
        PromptBridgeError::Config(format!("Provider '{}' not configured", provider_name))
    })?;

    let provider: Box<dyn LlmProvider> = if dry_run {
        Box::new(promptbridge::providers::mock::MockProvider::new(None))
    } else {
        ProviderFactory::create(provider_config, config.general.keep_alive_interval_minutes)?
    };

    let result = TransformationPipeline::execute(
        provider.as_ref(),
        &raw_text,
        mode,
        &config.general.target_language,
    )
    .await?;

    if config.general.mode == "preview" {
        eprintln!("{}", format_diff(&result.original_text, &result.final_prompt));
    } else {
        println!("{}", result.final_prompt);
    }

    if copy || config.general.auto_copy_clipboard {
        copy_to_clipboard(&result.final_prompt)?;
    }

    Ok(())
}

fn init_config_interactive() -> Result<()> {
    use dialoguer::{Select, Input, Confirm};
    
    println!("PromptBridge Interactive Configuration\n");
    
    // Get config directory
    let config_dir = dirs::config_dir()
        .ok_or_else(|| PromptBridgeError::Config("Could not determine config directory".to_string()))?
        .join("promptbridge");
    
    std::fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("promptbridge.toml");
    
    // Load existing config or create default
    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str::<Config>(&content).map_err(|e| PromptBridgeError::Config(e.to_string()))?
    } else {
        Config::load(None)?
    };
    
    // Select provider
    let providers = vec!["ollama", "openai", "mock"];
    let selection = Select::new()
        .with_prompt("Select your LLM provider")
        .items(&providers)
        .default(0)
        .interact()
        .map_err(|e| PromptBridgeError::Config(format!("Interactive prompt failed: {}", e)))?;
    
    let selected_provider = providers[selection];
    config.general.default_provider = selected_provider.to_string();
    
    // Provider-specific configuration
    match selected_provider {
        "ollama" => {
            let base_url = Input::new()
                .with_prompt("Ollama base URL")
                .with_initial_text("http://localhost:11434")
                .default("http://localhost:11434".to_string())
                .interact()
                .map_err(|e| PromptBridgeError::Config(format!("Interactive prompt failed: {}", e)))?;
            
            let model = Input::new()
                .with_prompt("Model name")
                .with_initial_text("llama3.2")
                .default("llama3.2".to_string())
                .interact()
                .map_err(|e| PromptBridgeError::Config(format!("Interactive prompt failed: {}", e)))?;
            
            let use_auth = Confirm::new()
                .with_prompt("Does your Ollama instance require authentication?")
                .default(false)
                .interact()
                .map_err(|e| PromptBridgeError::Config(format!("Interactive prompt failed: {}", e)))?;
            
            let api_key = if use_auth {
                Some(Input::new()
                    .with_prompt("API key / Bearer token")
                    .interact()
                    .map_err(|e| PromptBridgeError::Config(format!("Interactive prompt failed: {}", e)))?)
            } else {
                None
            };
            
            let provider_config = ProviderConfig {
                provider_type: "ollama".to_string(),
                base_url: Some(base_url),
                api_key,
                model: Some(model),
                temperature: Some(0.2),
            };
            
            config.providers.insert("ollama".to_string(), provider_config);
        }
        
        "openai" => {
            let base_url = Input::new()
                .with_prompt("OpenAI API base URL")
                .with_initial_text("https://api.openai.com/v1")
                .default("https://api.openai.com/v1".to_string())
                .interact()
                .map_err(|e| PromptBridgeError::Config(format!("Interactive prompt failed: {}", e)))?;
            
            let api_key = Input::new()
                .with_prompt("API key")
                .interact()
                .map_err(|e| PromptBridgeError::Config(format!("Interactive prompt failed: {}", e)))?;
            
            let model = Input::new()
                .with_prompt("Model name")
                .with_initial_text("gpt-4o-mini")
                .default("gpt-4o-mini".to_string())
                .interact()
                .map_err(|e| PromptBridgeError::Config(format!("Interactive prompt failed: {}", e)))?;
            
            let provider_config = ProviderConfig {
                provider_type: "openai".to_string(),
                base_url: Some(base_url),
                api_key: Some(api_key),
                model: Some(model),
                temperature: Some(0.2),
            };
            
            config.providers.insert("openai".to_string(), provider_config);
        }
        
        "mock" => {
            let provider_config = ProviderConfig {
                provider_type: "mock".to_string(),
                base_url: None,
                api_key: None,
                model: None,
                temperature: Some(0.0),
            };
            
            config.providers.insert("mock".to_string(), provider_config);
        }
        
        _ => unreachable!(),
    }
    
    // Target language
    let target_lang = Input::new()
        .with_prompt("Target language for translation")
        .with_initial_text("en")
        .default("en".to_string())
        .interact()
        .map_err(|e| PromptBridgeError::Config(format!("Interactive prompt failed: {}", e)))?;
    
    config.general.target_language = target_lang;
    
    // Keep-alive interval
    let keep_alive = Input::new()
        .with_prompt("Keep-alive interval in minutes (0 to disable)")
        .with_initial_text("60")
        .default("60".to_string())
        .interact()
        .map_err(|e| PromptBridgeError::Config(format!("Interactive prompt failed: {}", e)))?;
    
    config.general.keep_alive_interval_minutes = Some(keep_alive.parse::<u64>().unwrap_or(60));
    
    // Save configuration
    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| PromptBridgeError::Config(e.to_string()))?;
    
    std::fs::write(&config_path, toml_str)?;
    
    println!("\nConfiguration saved to: {}", config_path.display());
    println!("You can edit this file manually if needed.");
    
    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut buffer)?;
    Ok(buffer)
}
