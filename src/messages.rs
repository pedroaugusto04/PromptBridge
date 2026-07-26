//! Centralized user-facing messages, CLI feedback strings, and error guidance tips.

/// UI Feedback Messages
pub const MSG_PROMPT_COPIED_CLIPBOARD: &str = "Prompt copied to clipboard!";
pub const MSG_NO_COMMAND_SPECIFIED: &str = "No target CLI command specified. Example: `promptbridge exec -- claude`";
pub const MSG_INPUT_PROMPT_EMPTY: &str = "Input prompt is empty. Provide text as argument or pipe via stdin.";

/// Error Guidance Tips for User Facing Display
pub const TIP_CONFIG_ERROR: &str = "Check your `promptbridge.toml` or environment variables.";
pub const TIP_PROVIDER_ERROR: &str = "Verify endpoint URL, model availability, or API keys.";
pub const TIP_EXEC_ERROR: &str = "Ensure the target CLI binary is installed and executable in your PATH.";
pub const TIP_TIMEOUT_ERROR: &str = "Adjust PROMPTBRIDGE_REQUEST_TIMEOUT_SECONDS if needed.";

/// Formatted CLI feedback functions
pub fn format_exec_argv_info(transformed_prompt: &str) -> String {
    format!("[PromptBridge] Transformed argv prompt -> \"{}\"", transformed_prompt)
}

pub fn format_dry_run_info(binary: &str, args: &[String]) -> String {
    format!("[Dry-Run] Executing target: {} {}", binary, args.join(" "))
}

pub fn format_child_exit_warning(code: i32) -> String {
    format!("Child process exited with status code {}", code)
}

pub fn format_provider_list_item(name: &str, is_default: bool, provider_type: &str, model: &str) -> String {
    let default_marker = if is_default { " (Default)" } else { "" };
    format!(" - {}{}: type={}, model={}", name, default_marker, provider_type, model)
}
