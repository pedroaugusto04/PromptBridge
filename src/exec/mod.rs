use crate::config::Config;
use crate::engine::{TransformMode, TransformationPipeline};
use crate::messages::{
    format_child_exit_warning, format_dry_run_info, format_exec_argv_info, MSG_NO_COMMAND_SPECIFIED,
};
use crate::providers::{LlmProvider, ProviderFactory};
use crate::utils::error::{PromptBridgeError, Result};
use crate::utils::formatting::{print_info, print_warning};
use std::process::{Command, Stdio};

pub struct ExecGateway;

impl ExecGateway {
    /// Executes a target CLI command (e.g. `claude`, `codex`, `opencode`, `aider`)
    /// with transparent prompt interception & transformation.
    pub async fn run(
        cmd_args: &[String],
        config: &Config,
        override_provider: Option<String>,
        mode: TransformMode,
        dry_run: bool,
    ) -> Result<()> {
        if cmd_args.is_empty() {
            return Err(PromptBridgeError::Exec(
                MSG_NO_COMMAND_SPECIFIED.to_string(),
            ));
        }

        let binary = &cmd_args[0];
        let args = &cmd_args[1..];

        // Instantiate provider
        let provider_name = override_provider
            .as_deref()
            .unwrap_or(&config.general.default_provider);

        let provider_config = config.providers.get(provider_name).ok_or_else(|| {
            PromptBridgeError::Config(format!("Provider '{}' not configured", provider_name))
        })?;

        let provider: Box<dyn LlmProvider> = if dry_run {
            Box::new(crate::providers::mock::MockProvider::new(None))
        } else {
            ProviderFactory::create(provider_config, config.general.keep_alive_interval_minutes)?
        };

        // Inspect argv positional arguments to see if a raw prompt argument was passed
        let mut modified_args: Vec<String> = Vec::new();
        for arg in args {
            if !arg.starts_with('-') && arg.contains(' ') && arg.chars().any(|c| c.is_alphabetic())
            {
                let pipeline_res = TransformationPipeline::execute(
                    provider.as_ref(),
                    arg,
                    mode,
                    &config.general.target_language,
                )
                .await?;

                if config.general.mode == "preview" {
                    print_info(&format_exec_argv_info(&pipeline_res.final_prompt));
                }

                modified_args.push(pipeline_res.final_prompt);
            } else {
                modified_args.push(arg.clone());
            }
        }

        if dry_run {
            print_info(&format_dry_run_info(binary, &modified_args));
            return Ok(());
        }

        // Spawn child process seamlessly
        let mut child = Command::new(binary)
            .args(&modified_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                PromptBridgeError::Exec(format!(
                    "Failed to execute command '{}': {}. Make sure binary is installed in PATH.",
                    binary, e
                ))
            })?;

        let status = child.wait()?;
        if !status.success() {
            if let Some(code) = status.code() {
                print_warning(&format_child_exit_warning(code));
            }
        }

        Ok(())
    }
}
