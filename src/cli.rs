use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "promptbridge",
    author = "PromptBridge Contributors",
    version = "0.2.2",
    about = "A quick CLI tool that translates coding prompts via global hotkey. Translated text is automatically copied to clipboard."
)]
pub struct Cli {
    /// Custom configuration file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Override LLM provider (ollama, openai, mock)
    #[arg(short, long)]
    pub provider: Option<String>,

    /// Override LLM model
    #[arg(short, long)]
    pub model: Option<String>,

    /// Target language for translation (default: en)
    #[arg(short, long)]
    pub target_lang: Option<String>,

    /// Copy transformed prompt to system clipboard
    #[arg(long)]
    pub copy: bool,

    /// Dry-run mode (simulates parsing and transformation without invoking remote LLMs)
    #[arg(long)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Both translate and optimize prompt (stdin or positional string)
    Transform {
        /// Input prompt text (optional; if omitted, reads from stdin)
        input: Option<String>,
    },

    /// Translate natural language prompt to target language
    Translate {
        /// Input prompt text (optional; if omitted, reads from stdin)
        input: Option<String>,
    },

    /// Optimize developer prompt into structured AI Coding Agent format
    Optimize {
        /// Input prompt text (optional; if omitted, reads from stdin)
        input: Option<String>,
    },

    /// CLI-Agnostic Proxy Gateway mode (e.g. `promptbridge exec -- claude`)
    Exec {
        /// Target CLI binary and arguments to execute
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,

        /// Run in visual preview mode (displays transformed prompt)
        #[arg(long)]
        preview: bool,

        /// Run in silent mode (passes transformed prompt transparently)
        #[arg(long)]
        silent: bool,
    },

    /// Show current loaded configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigSubcommand>,
    },

    /// List configured LLM providers
    Providers,

    /// Install the global keyboard shortcut helper script (Linux/GNOME)
    InstallShortcut,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommand {
    /// Print active configuration TOML
    Show,

    /// Print location of loaded configuration files
    Path,
}
