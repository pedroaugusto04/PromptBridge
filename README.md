# PromptBridge

Translates coding prompts from your native language (e.g., Portuguese) to another language (e.g., English) in place via a global hotkey, eliminating the need to manually open a translation tab. Safely preserves all code blocks, file paths, and terminal commands during translation.

---

## Installation

### Step 1: Install Rust & Cargo
If you don't have Rust installed, install it first via [rustup](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Step 2: Install dependencies (Linux)
*Required to simulate the copy/paste keystrokes in the background when you press the hotkey.*
```bash
sudo apt install xclip xdotool -y
```

### Step 3: Install PromptBridge
```bash
cargo install promptbridge
```

### Step 4: Run the Auto-Installer
To generate the translation keyboard shortcut runner and default configuration, run:
```bash
promptbridge install-shortcut
```

> **Important**: This command creates your global configuration file at `~/.config/promptbridge/promptbridge.toml`. Make sure to open this file and configure your AI provider (e.g., set your OpenAI API key or custom Ollama URL) for it to work!

### Step 5: Configure the System Hotkey
* Open your system **Settings** -> **Keyboard** -> **Keyboard Shortcuts** -> **Custom Shortcuts (+)**.
* Create a new shortcut:
  * **Name**: `PromptBridge Translate`
  * **Command**: `pb-translate`
  * **Shortcut**: Set your preferred key combination (e.g., `Ctrl+Alt+T` or `Super+T`).

---

## Usage

Inside **any** chat interface (terminal TUIs like `opencode`/`aider`, VS Code, or web browsers):
1. Type your prompt in your native language (e.g., `crie uma função em rust em src/main.rs`).
2. Select/highlight the text.
3. Press your hotkey (e.g., `Ctrl+Alt+T`).
4. The text will be instantly translated to English with all pathnames and code blocks protected

---

## Customizing Configuration

You can customize your language, change providers, or adjust model settings at any time by editing the global config file:
```bash
nano ~/.config/promptbridge/promptbridge.toml
```

### Config File Example & Options

```toml
[general]
# The active provider to use ("ollama", "openai", "deepseek", or "mock")
default_provider = "ollama"

# Target language for translation (e.g., "en", "es", "fr", "pt")
target_language = "en"

# Timeout for requests in seconds (increase if model is slow to start/respond)
request_timeout_seconds = 60

# Automatically copy the translated prompt to the system clipboard
auto_copy_clipboard = false

# Safely extract and preserve paths/code during translation
preserve_technical_terms = true

[providers.ollama]
type = "ollama"
base_url = "http://localhost:11434"
model = "llama3.2"
temperature = 0
# api_key = "your-bearer-token-if-using-bearer-auth"

[providers.openai]
type = "openai"
base_url = "https://api.openai.com/v1"
api_key = "env:OPENAI_API_KEY"  
model = "gpt-4o-mini"
temperature = 0.2
```
