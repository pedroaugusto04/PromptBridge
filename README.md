# PromptBridge

A quick CLI tool that translates coding prompts from your native language (e.g., Portuguese) to another language (e.g., English) via a global hotkey. The translated text is automatically copied to your clipboard, ready to paste anywhere. Safely preserves all code blocks, file paths, and terminal commands during translation.

---

## Installation

### Linux

### Step 1: Install Rust & Cargo
If you don't have Rust installed, install it first via [rustup](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Step 2: Install dependencies (Linux)
*Required for clipboard access, keystroke simulation, and visual feedback dialogs.*
```bash
sudo apt install xclip xdotool zenity -y
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

> **Important**: This command creates your global configuration file at `~/.config/promptbridge/promptbridge.toml`. Make sure to open this file and configure your AI provider (e.g., set your OpenAI API key or custom Ollama URL)

### Step 5: Configure the System Hotkey
* Open your system **Settings** -> **Keyboard** -> **Keyboard Shortcuts** -> **Custom Shortcuts (+)**.
* Create a new shortcut:
  * **Name**: `PromptBridge Translate`
  * **Command**: `pb-translate`
  * **Shortcut**: Set your preferred key combination (e.g., `Ctrl+Alt+T` or `Super+T`).

### Windows

#### Step 1: Install Rust & Cargo
If you don't have Rust installed, install it first via [rustup](https://rustup.rs/):
```powershell
# In PowerShell
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe
```

#### Step 2: Install PromptBridge
```powershell
cargo install promptbridge
```

#### Step 3: Run the Auto-Installer
To generate the translation keyboard shortcut runner and default configuration, run:
```powershell
promptbridge install-shortcut
```

> **Important**: This command creates your global configuration file at `%APPDATA%\promptbridge\promptbridge.toml`. Make sure to open this file and configure your AI provider (e.g., set your OpenAI API key or custom Ollama URL)

#### Step 4: Configure the System Hotkey
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

#### Step 1: Install Rust & Cargo
If you don't have Rust installed, install it first via [rustup](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Step 2: Install PromptBridge
```bash
cargo install promptbridge
```

#### Step 3: Run the Auto-Installer
To generate the translation keyboard shortcut runner and default configuration, run:
```bash
promptbridge install-shortcut
```

> **Important**: This command creates your global configuration file at `~/Library/Application Support/promptbridge/promptbridge.toml`. Make sure to open this file and configure your AI provider (e.g., set your OpenAI API key or custom Ollama URL)

#### Step 4: Configure the System Hotkey
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
