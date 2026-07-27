# PromptBridge

A quick CLI tool that translates coding prompts from your native language (e.g., Portuguese) to another language (e.g., English) via a global hotkey. The translated text is automatically copied to your clipboard, ready to paste anywhere. Safely preserves all code blocks, file paths, and terminal commands during translation.

---

## Installation

### Quick Install (Recommended)

**Linux/macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/pedroaugusto04/PromptBridge/main/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/pedroaugusto04/PromptBridge/main/install.ps1 | iex
```

### Setup

**1. Configure your AI provider:**
```bash
promptbridge init-config
```

This interactive wizard will guide you through:
- Selecting your LLM provider (Ollama, OpenAI, or Mock)
- Configuring provider-specific settings (URL, model, API keys)
- Setting your target language
- Configuring keep-alive interval

**2. Install the keyboard shortcut helper:**
```bash
promptbridge install-shortcut
```

**3. Configure the system hotkey:**

**Linux:**
- Open **Settings** → **Keyboard** → **Keyboard Shortcuts** → **Custom Shortcuts (+)**
- Name: `PromptBridge Translate`
- Command: `pb-translate`
- Shortcut: Set your preferred key combination (e.g., `Ctrl+Alt+T`)

**Windows:**
- Install [AutoHotkey](https://www.autohotkey.com/)
- Create a script with:
```autohotkey
^t::  ; Ctrl+T
Run, PowerShell.exe -ExecutionPolicy Bypass -File "%APPDATA%\promptbridge\pb-translate.ps1"
return
```

**macOS:**
- Open **Automator** → **Quick Action**
- Add **Run Shell Script** with: `pb-translate "$@"`
- Set keyboard shortcut in **System Settings** → **Keyboard** → **Keyboard Shortcuts** → **Services**

### Manual Installation

<details>
<summary>Click to expand manual installation options</summary>

**Option 1: Using Cargo (requires Rust)**
```bash
cargo install promptbridge
```

**Option 2: Download Pre-compiled Binaries**
Choose your platform and download the latest release:
- [Linux x86_64](https://github.com/pedroaugusto04/PromptBridge/releases/latest/download/promptbridge-x86_64-unknown-linux-gnu.tar.gz)
- [Linux ARM64](https://github.com/pedroaugusto04/PromptBridge/releases/latest/download/promptbridge-aarch64-unknown-linux-gnu.tar.gz)
- [Windows x86_64](https://github.com/pedroaugusto04/PromptBridge/releases/latest/download/promptbridge-x86_64-pc-windows-msvc.zip)
- [macOS Apple Silicon](https://github.com/pedroaugusto04/PromptBridge/releases/latest/download/promptbridge-aarch64-apple-darwin.tar.gz)
- [macOS Intel](https://github.com/pedroaugusto04/PromptBridge/releases/latest/download/promptbridge-x86_64-apple-darwin.tar.gz)

Extract and move to your PATH:
```bash
# Linux/macOS
tar -xzf promptbridge-*.tar.gz
sudo mv promptbridge /usr/local/bin/

# Windows
Expand-Archive -Path promptbridge.zip -DestinationPath $env:USERPROFILE\.cargo\bin
```

**Linux Dependencies (required for all installation methods):**
```bash
sudo apt install xclip xdotool zenity -y
```

</details>

---

## Usage

Inside **any** chat interface (terminal TUIs like `opencode`/`aider`, VS Code, or web browsers):
1. Type your prompt in your native language (e.g., `crie uma função em rust em src/main.rs`).
2. Select/highlight the text.
3. Press your hotkey (e.g., `Ctrl+Alt+T`).
4. The text will be instantly translated to English with all pathnames and code blocks protected.
5. The translated text is automatically copied to your clipboard (just like Ctrl+C).
6. Paste the translated text wherever you need it (Ctrl+V).

---

## Customizing Configuration

You can customize your language, change providers, or adjust model settings at any time by editing the global config file:

**Linux:**
```bash
nano ~/.config/promptbridge/promptbridge.toml
```

**Windows:**
```powershell
notepad $env:APPDATA\promptbridge\promptbridge.toml
```

**macOS:**
```bash
nano ~/Library/Application\ Support/promptbridge/promptbridge.toml
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

# Keep Ollama model loaded in memory for this many minutes (prevents cold starts)
# Set to 0 to disable, or increase for longer keep-alive periods
keep_alive_interval_minutes = 60

# Automatically copy the translated prompt to the system clipboard
auto_copy_clipboard = true

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
