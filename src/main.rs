use clap::Parser;
use promptbridge::cli::{Cli, Commands, ConfigSubcommand};
use promptbridge::config::Config;
use promptbridge::engine::{TransformMode, TransformationPipeline};
use promptbridge::exec::ExecGateway;
use promptbridge::messages::{
    format_provider_list_item, MSG_INPUT_PROMPT_EMPTY, MSG_PROMPT_COPIED_CLIPBOARD,
};
use promptbridge::providers::{LlmProvider, ProviderFactory};
use promptbridge::utils::clipboard::copy_to_clipboard;
use promptbridge::utils::error::{PromptBridgeError, Result};
use promptbridge::utils::formatting::{format_diff, print_error, print_success};
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
    }

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
        ProviderFactory::create(provider_config)?
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
        print_success(MSG_PROMPT_COPIED_CLIPBOARD);
    }

    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut buffer)?;
    Ok(buffer)
}
