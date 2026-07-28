# PromptBridge

A quick tool that translates coding prompts from your native language (e.g., Portuguese) to another language (e.g., English) via a global hotkey. The translated text is automatically copied to your clipboard, ready to paste anywhere. Safely preserves all code blocks, file paths, and terminal commands during translation.

---

## Why?

**Save money on LLM costs.** English prompts use significantly fewer tokens than other languages:

LLMs are trained primarily on English text, so they tokenize English much more efficiently. Writing prompts in your native language and translating them to English before sending to the AI can reduce your token costs while maintaining technical accuracy.

---

## Installation

### Quick Install (Recommended)

**Linux/macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/pedroaugusto04/PromptBridge/main/scripts/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/pedroaugusto04/PromptBridge/main/scripts/install.ps1 | iex
```

### Setup

The installation script automatically configures everything for you. Just configure your system hotkey:

**1. Configure the system hotkey:**

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
# The active provider to use ("google_translate", "ollama", "openai", or "mock")
default_provider = "google_translate"

# Target language for translation (e.g., "en", "es", "fr", "pt", "zh", "zh-cn")
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

[providers.google_translate]
type = "google_translate"

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

---

## Manual Installation

<details>
<summary>Click to expand manual installation options</summary>

### Linux

**Option 1: Using Cargo (requires Rust)**
```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install PromptBridge
cargo install promptbridge
```

**Option 2: Download Pre-compiled Binary**
```bash
# Download the latest release
wget https://github.com/pedroaugusto04/PromptBridge/releases/latest/download/promptbridge-x86_64-unknown-linux-gnu.tar.gz

# Extract and install
tar -xzf promptbridge-x86_64-unknown-linux-gnu.tar.gz
sudo mv promptbridge /usr/local/bin/
```

**Install Dependencies:**
```bash
sudo apt install xclip xdotool zenity -y
```

### Windows

**Option 1: Using Cargo (requires Rust)**
```powershell
# Install Rust if needed
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe

# Install PromptBridge
cargo install promptbridge
```

**Option 2: Download Pre-compiled Binary**
```powershell
# Download the latest release
Invoke-WebRequest -Uri "https://github.com/pedroaugusto04/PromptBridge/releases/latest/download/promptbridge-x86_64-pc-windows-msvc.zip" -OutFile promptbridge.zip

# Extract and add to PATH
Expand-Archive -Path promptbridge.zip -DestinationPath $env:USERPROFILE\.cargo\bin
```

### macOS

**Option 1: Using Cargo (requires Rust)**
```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install PromptBridge
cargo install promptbridge
```

**Option 2: Download Pre-compiled Binary**
```bash
# For Apple Silicon (M1/M2/M3)
wget https://github.com/pedroaugusto04/PromptBridge/releases/latest/download/promptbridge-aarch64-apple-darwin.tar.gz

# For Intel Macs
wget https://github.com/pedroaugusto04/PromptBridge/releases/latest/download/promptbridge-x86_64-apple-darwin.tar.gz

# Extract and install
tar -xzf promptbridge-*.tar.gz
sudo mv promptbridge /usr/local/bin/
```

</details>
