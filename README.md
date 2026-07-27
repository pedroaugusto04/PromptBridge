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

This will automatically:
- Download the latest binary for your platform
- Install it to the appropriate directory
- Set up the default configuration file
- Add the installation directory to your PATH (if needed)

### Manual Installation

#### Linux

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

### Step 2: Install dependencies (Linux)
*Required for clipboard access, keystroke simulation, and visual feedback dialogs.*
```bash
sudo apt install xclip xdotool zenity -y
```

### Step 3: Run the Auto-Installer
To generate the translation keyboard shortcut runner and default configuration, run:
```bash
promptbridge install-shortcut
```

> **Important**: This command creates your global configuration file at `~/.config/promptbridge/promptbridge.toml`. Make sure to open this file and configure your AI provider (e.g., set your OpenAI API key or custom Ollama URL)

### Step 4: Configure the System Hotkey
* Open your system **Settings** -> **Keyboard** -> **Keyboard Shortcuts** -> **Custom Shortcuts (+)**.
* Create a new shortcut:
  * **Name**: `PromptBridge Translate`
  * **Command**: `pb-translate`
  * **Shortcut**: Set your preferred key combination (e.g., `Ctrl+Alt+T` or `Super+T`).

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

#### Step 2: Run the Auto-Installer
To generate the translation keyboard shortcut runner and default configuration, run:
```powershell
promptbridge install-shortcut
```

> **Important**: This command creates your global configuration file at `%APPDATA%\promptbridge\promptbridge.toml`. Make sure to open this file and configure your AI provider (e.g., set your OpenAI API key or custom Ollama URL)

#### Step 3: Configure the System Hotkey
Windows requires additional setup for global hotkeys. Choose one of the following methods:

**Option A: Using AutoHotkey (Recommended)**
1. Install [AutoHotkey](https://www.autohotkey.com/)
2. Create an AutoHotkey script with:
```autohotkey
^t::  ; Ctrl+T
Run, PowerShell.exe -ExecutionPolicy Bypass -File "%APPDATA%\promptbridge\pb-translate.ps1"
return
```
3. Save as `promptbridge.ahk` and run it

**Option B: Using PowerShell Shortcut**
1. Create a shortcut on your desktop
2. Set target to: `powershell.exe -ExecutionPolicy Bypass -File "%APPDATA%\promptbridge\pb-translate.ps1"`
3. Right-click shortcut -> Properties -> Shortcut Key
4. Set your preferred key combination (e.g., `Ctrl+Alt+T`)

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

#### Step 2: Run the Auto-Installer
To generate the translation keyboard shortcut runner and default configuration, run:
```bash
promptbridge install-shortcut
```

> **Important**: This command creates your global configuration file at `~/Library/Application Support/promptbridge/promptbridge.toml`. Make sure to open this file and configure your AI provider (e.g., set your OpenAI API key or custom Ollama URL)

#### Step 3: Configure the System Hotkey
macOS requires using Automator to create a Quick Action:

1. Open **Automator** (Applications -> Automator)
2. Choose **Quick Action** as the document type
3. Set:
   - Workflow receives current: **text**
   - in: **any application**
4. Add **Run Shell Script** action
5. Set shell to: `/bin/bash`
6. Set script to:
```bash
~/Library/Application\ Support/promptbridge/pb-translate.sh
```
7. Save as `PromptBridge Translate`
8. Open **System Settings** -> **Keyboard** -> **Keyboard Shortcuts** -> **Services**
9. Find `PromptBridge Translate` and assign your shortcut (e.g., `Ctrl+Alt+T`)

> **Note**: On macOS, you need to copy text to the clipboard before pressing the hotkey, as macOS doesn't have a primary selection like Linux.

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
