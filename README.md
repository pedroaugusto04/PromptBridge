# PromptBridge

Tired of opening a browser tab just to translate your coding prompt before sending it to your agent?

PromptBridge lets you write prompts in your native language (e.g., Portuguese) and translates them to English in place (via a system hotkey) for AI coding agents (Claude, Aider, OpenCode, etc.), keeping code blocks, file paths, and terminal commands completely untouched.

---

## Installation

### Step 1: Install dependencies
```bash
sudo apt install xclip xdotool -y
```

### Step 2: Install PromptBridge

#### Option A: Via Cargo (Rust Package Manager)
```bash
cargo install promptbridge
```

#### Option B: Via Homebrew (macOS / Linux)
```bash
brew tap pedroaugusto04/tap
brew install promptbridge
```

### Step 3: Run the Auto-Installer
To configure your local configuration and generate the translation keyboard shortcut runner, run:
```bash
promptbridge install-shortcut
```

*This command automatically creates your configuration file at `~/.config/promptbridge/promptbridge.toml` and installs the helper script.*

### Step 4: Configure the System Hotkey
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
4. The text will be instantly translated to English with all pathnames and code blocks protected!

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
