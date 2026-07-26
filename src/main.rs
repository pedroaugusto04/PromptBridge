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

        Commands::InstallShortcut => {
            install_shortcut()?;
        }
    }

    Ok(())
}

fn install_shortcut() -> Result<()> {
    // 1. Create default config file if it does not exist
    if let Some(mut user_config_dir) = dirs::config_dir() {
        user_config_dir.push("promptbridge");
        std::fs::create_dir_all(&user_config_dir)?;
        let config_file = user_config_dir.join("promptbridge.toml");
        if !config_file.exists() {
            std::fs::write(&config_file, promptbridge::constants::DEFAULT_CONFIG_TOML)?;
            println!("✓ Created global config template at: {}", config_file.display());
        }
    }

    // 2. Create the shortcut script in ~/.local/bin/pb-translate
    if let Some(home_dir) = dirs::home_dir() {
        let bin_dir = home_dir.join(".local").join("bin");
        std::fs::create_dir_all(&bin_dir)?;
        let script_path = bin_dir.join("pb-translate");
        
        let script_content = r#"#!/bin/bash

# Make DBUS available when launched from a keyboard shortcut
export DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$(id -u)/bus"
# Ensure cargo bin is in PATH (may be missing in shortcut context)
export PATH="$HOME/.cargo/bin:$PATH"

# Log file for debugging
LOGFILE="$HOME/.local/share/promptbridge/pb-translate.log"
mkdir -p "$(dirname "$LOGFILE")"
log() { echo "[$(date '+%H:%M:%S')] $1" >> "$LOGFILE"; }

log "=== pb-translate started ==="

# 1. Clear clipboard to detect if text was actually selected
OLD_CLIP=$(xclip -selection clipboard -o 2>/dev/null)
xclip -selection clipboard /dev/null

# 2. Copy selected text
xdotool key ctrl+c
sleep 0.1

# 3. Get text from clipboard
TEXTO=$(xclip -selection clipboard -o 2>/dev/null)
log "Input: $TEXTO"

# If nothing was copied, restore old clipboard and exit gracefully
if [ -z "$TEXTO" ]; then
    log "No text selected — exiting"
    echo -n "$OLD_CLIP" | xclip -selection clipboard
    exit 0
fi

# 4. Show pulsating progress dialog while translating
zenity --progress --pulsate --no-cancel --title="PromptBridge" --text="Translating..." --width=300 &
ZEN_PID=$!

# 5. Run translation (synchronously, capture output)
RESULT=$(promptbridge translate "$TEXTO" 2>>"$LOGFILE")
EXIT_CODE=$?
log "Exit code: $EXIT_CODE | Result: $RESULT"

# 6. Close the progress dialog
kill $ZEN_PID 2>/dev/null
wait $ZEN_PID 2>/dev/null

# 7. Show result in a modal with Paste / Cancel
if [ $EXIT_CODE -eq 0 ] && [ -n "$RESULT" ]; then
    zenity --text-info \
        --title="PromptBridge — Translation Result" \
        --width=600 --height=400 \
        --ok-label="Paste" --cancel-label="Cancel" \
        <<< "$RESULT"
    if [ $? -eq 0 ]; then
        echo -n "$RESULT" | xclip -selection clipboard
        xdotool key ctrl+v
        log "Pasted successfully"
    else
        echo -n "$OLD_CLIP" | xclip -selection clipboard
        log "User cancelled"
    fi
else
    zenity --error \
        --title="PromptBridge" \
        --text="Translation failed.\nSee log for details:\n$LOGFILE" \
        --width=450
    echo -n "$OLD_CLIP" | xclip -selection clipboard
fi

log "=== pb-translate done ==="
"#;

        std::fs::write(&script_path, script_content)?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }
        
        println!("✓ Created helper script at: {}", script_path.display());
        println!("\n=== Installation complete ===");
        println!("Please configure the keyboard shortcut in your OS Settings:");
        println!("  Shortcut Command: pb-translate");
        println!("  Example Shortcut Keys: Ctrl+Alt+T");
    } else {
        return Err(PromptBridgeError::Engine("Could not locate home directory".to_string()));
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
