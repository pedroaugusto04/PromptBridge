# PromptBridge

Simple tool that translates prompts for AI coding agents (Claude, Aider, OpenCode, etc.) while preserving code, paths, and technical terms.

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

## Configuration

To change LLM models, API keys, or target endpoints, simply edit your global config file:
```bash
nano ~/.config/promptbridge/promptbridge.toml
```
