//! Linux-specific platform implementation
//! 
//! Uses:
//! - xclip for clipboard access
//! - zenity for dialogs
//! - notify-send for notifications
//! - Bash scripts for shortcut installation

use crate::constants;
use crate::platform::{
    PlatformClipboard, PlatformDialog, PlatformNotifier, PlatformShortcutInstaller,
    ProgressDialogHandle, ShortcutInstallResult, TextInfoResult,
};
use crate::utils::error::{PromptBridgeError, Result};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct LinuxPlatform;

impl LinuxPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformShortcutInstaller for LinuxPlatform {
    fn install_shortcut(&self) -> Result<ShortcutInstallResult> {
        // 1. Create default config file if it does not exist
        if let Some(mut user_config_dir) = dirs::config_dir() {
            user_config_dir.push("promptbridge");
            std::fs::create_dir_all(&user_config_dir)?;
            let config_file = user_config_dir.join("promptbridge.toml");
            if !config_file.exists() {
                std::fs::write(&config_file, constants::DEFAULT_CONFIG_TOML)?;
            }
        }

        // 2. Create the shortcut script in ~/.local/bin/pb-translate
        if let Some(home_dir) = dirs::home_dir() {
            let bin_dir = home_dir.join(".local").join("bin");
            std::fs::create_dir_all(&bin_dir)?;
            let script_path = bin_dir.join("pb-translate");
            
            let script_content = r#"#!/bin/bash

# Make DBUS available when launched from a keyboard shortcut
export DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$(id -u)/bus"
# Ensure cargo bin is in PATH (may be missing in shortcut context)
export PATH="$HOME/.cargo/bin:$PATH"

# Log file for debugging
LOGFILE="$HOME/.local/share/promptbridge/pb-translate.log"
mkdir -p "$(dirname "$LOGFILE")"
log() { echo "[$(date '+%H:%M:%S')] $1" >> "$LOGFILE"; }

log "=== pb-translate started ==="

# On Linux/X11, selected text is automatically in the PRIMARY selection.
TEXTO=$(xclip -selection primary -o 2>/dev/null)
log "Input: $TEXTO"

# If nothing is selected, exit gracefully
if [ -z "$TEXTO" ]; then
    log "No text selected — exiting"
    exit 0
fi

# Backup current CLIPBOARD content so we can restore on cancel
OLD_CLIP=$(xclip -selection clipboard -o 2>/dev/null)

# 4. Show pulsating progress dialog while translating
zenity --progress --pulsate --no-cancel --auto-close --title="PromptBridge" --text="Translating..." --width=300 &
ZEN_PID=$!

# 5. Run translation (synchronously, capture output)
RAW_RESULT=$(promptbridge translate "$TEXTO" 2>>"$LOGFILE" 2>&1)
EXIT_CODE=$?
# Extract only the translated text (after "--- Transformed Prompt ---")
if [ -n "$RAW_RESULT" ]; then
    RESULT=$(echo "$RAW_RESULT" | grep -A 100 -- "--- Transformed Prompt ---" | tail -n +2)
else
    RESULT=""
fi
log "Exit code: $EXIT_CODE | Result: $RESULT"

# 6. Close the progress dialog
kill $ZEN_PID 2>/dev/null
wait $ZEN_PID 2>/dev/null

# 7. Check if auto_copy is enabled in config
CONFIG_FILE="$HOME/.config/promptbridge/promptbridge.toml"
AUTO_COPY=$(grep -E "^auto_copy_clipboard\s*=" "$CONFIG_FILE" 2>/dev/null | cut -d= -f2 | tr -d ' "')

# 8. Show result based on auto_copy setting
if [ $EXIT_CODE -eq 0 ] && [ -n "$RESULT" ]; then
    if [ "$AUTO_COPY" = "true" ]; then
        # Auto-copy mode: copy silently and show smooth success notification
        echo -n "$RESULT" | xclip -selection clipboard
        log "Auto-copied to clipboard"
        
        # Show smooth success notification with PromptBridge app name
        (notify-send --app-name="PromptBridge" --icon="info" "Translation" "✓ Translated & copied!" &
        NOTIFY_PID=$!
        sleep 1.5
        kill $NOTIFY_PID 2>/dev/null) &
    else
        # Manual mode: show modal with Copy / Done buttons
        zenity --text-info \
            --title="PromptBridge" \
            --width=700 --height=500 \
            --ok-label="Copy" --cancel-label="Done" \
            --filename=/dev/stdin \
            <<< "$RESULT"
        BUTTON_CODE=$?
        
        if [ $BUTTON_CODE -eq 0 ]; then
            # Copy button clicked - copy to clipboard
            echo -n "$RESULT" | xclip -selection clipboard
        else
            # Done button clicked - just close, restore clipboard
            echo -n "$OLD_CLIP" | xclip -selection clipboard
            log "Done - clipboard restored"
        fi
    fi
else
    zenity --error \
        --title="PromptBridge" \
        --text="Translation failed.\nSee log for details:\n$LOGFILE" \
        --width=450
    echo -n "$OLD_CLIP" | xclip -selection clipboard
fi

log "=== pb-translate done ==="
"#;

            std::fs::write(&script_path, script_content)?;
            
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
            
            let config_instructions = format!(
                "Please configure the keyboard shortcut in your OS Settings:\n\
                 Shortcut Command: pb-translate\n\
                 Example Shortcut Keys: Ctrl+Alt+T"
            );
            
            Ok(ShortcutInstallResult {
                script_path: script_path.display().to_string(),
                config_instructions,
            })
        } else {
            Err(PromptBridgeError::Engine("Could not locate home directory".to_string()))
        }
    }
}

impl PlatformDialog for LinuxPlatform {
    fn show_progress(&self, title: &str, text: &str) -> Result<Box<dyn ProgressDialogHandle>> {
        let child = Command::new("zenity")
            .args(&[
                "--progress",
                "--pulsate",
                "--no-cancel",
                "--auto-close",
                &format!("--title={}", title),
                &format!("--text={}", text),
                "--width=300",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| PromptBridgeError::Engine(format!("Failed to show progress dialog: {}", e)))?;
        
        Ok(Box::new(LinuxProgressDialogHandle {
            pid: child.id(),
            _child: Arc::new(Mutex::new(child)),
        }))
    }
    
    fn show_text_info(&self, title: &str, text: &str) -> Result<TextInfoResult> {
        let mut result = Command::new("zenity")
            .args(&[
                "--text-info",
                &format!("--title={}", title),
                "--width=700",
                "--height=500",
                "--ok-label=Copy",
                "--cancel-label=Done",
                "--filename=/dev/stdin",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| PromptBridgeError::Engine(format!("Failed to show text info dialog: {}", e)))?;
        
        // Write text to stdin
        if let Some(ref mut stdin) = result.stdin {
            stdin.write_all(text.as_bytes()).map_err(|e| {
                PromptBridgeError::Engine(format!("Failed to write to dialog: {}", e))
            })?;
        }
        
        let status = result.wait().map_err(|e| {
            PromptBridgeError::Engine(format!("Failed to wait for dialog: {}", e))
        })?;
        
        // Zenity returns 0 for OK (Copy), 1 for Cancel (Done)
        if status.success() {
            Ok(TextInfoResult::Copy)
        } else {
            Ok(TextInfoResult::Done)
        }
    }
    
    fn show_error(&self, title: &str, text: &str) -> Result<()> {
        Command::new("zenity")
            .args(&[
                "--error",
                &format!("--title={}", title),
                &format!("--text={}", text),
                "--width=450",
            ])
            .status()
            .map_err(|e| PromptBridgeError::Engine(format!("Failed to show error dialog: {}", e)))?;
        
        Ok(())
    }
}

struct LinuxProgressDialogHandle {
    pid: u32,
    _child: Arc<Mutex<std::process::Child>>,
}

impl ProgressDialogHandle for LinuxProgressDialogHandle {
    fn close(&self) -> Result<()> {
        Command::new("kill")
            .arg(self.pid.to_string())
            .status()
            .map_err(|e| PromptBridgeError::Engine(format!("Failed to close progress dialog: {}", e)))?;
        Ok(())
    }
}

impl PlatformNotifier for LinuxPlatform {
    fn show_notification(&self, title: &str, message: &str) -> Result<()> {
        Command::new("notify-send")
            .args(&[
                "--app-name=PromptBridge",
                "--icon=info",
                title,
                message,
            ])
            .status()
            .map_err(|e| PromptBridgeError::Engine(format!("Failed to show notification: {}", e)))?;
        
        Ok(())
    }
}

impl PlatformClipboard for LinuxPlatform {
    fn get_text(&self) -> Result<String> {
        let output = Command::new("xclip")
            .args(&["-selection", "clipboard", "-o"])
            .output()
            .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to get clipboard: {}", e)))?;
        
        String::from_utf8(output.stdout)
            .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to parse clipboard: {}", e)))
    }
    
    fn set_text(&self, text: &str) -> Result<()> {
        let mut child = Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to set clipboard: {}", e)))?;
        
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(text.as_bytes())
                .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to write to clipboard: {}", e)))?;
        }
        
        // Wait for xclip to finish processing
        child.wait()
            .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to wait for clipboard: {}", e)))?;
        
        Ok(())
    }
}
