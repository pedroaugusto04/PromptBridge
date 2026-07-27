//! MacOS-specific platform implementation
//! 
//! Uses:
//! - osascript for clipboard access
//! - AppleScript for dialogs
//! - osascript for notifications
//! - AppleScript/Automator for shortcut installation

use crate::constants;
use crate::platform::{
    PlatformClipboard, PlatformDialog, PlatformNotifier, PlatformShortcutInstaller,
    ProgressDialogHandle, ShortcutInstallResult, TextInfoResult,
};
use crate::utils::error::{PromptBridgeError, Result};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct MacosPlatform;

impl MacosPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformShortcutInstaller for MacosPlatform {
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

        // 2. Create the shortcut script in ~/Library/Application Support/promptbridge/pb-translate.sh
        if let Some(home_dir) = dirs::home_dir() {
            let bin_dir = home_dir.join("Library").join("Application Support").join("promptbridge");
            std::fs::create_dir_all(&bin_dir)?;
            let script_path = bin_dir.join("pb-translate.sh");
            
            let script_content = r#"#!/bin/bash

# Log file for debugging
LOGDIR="$HOME/Library/Application Support/promptbridge"
LOGFILE="$LOGDIR/pb-translate.log"
mkdir -p "$LOGDIR"
log() { echo "[$(date '+%H:%M:%S')] $1" >> "$LOGFILE"; }

log "=== pb-translate started ==="

# On macOS, get selected text from clipboard (user must copy first)
TEXTO=$(osascript -e 'tell application "System Events" to get the clipboard' 2>/dev/null)
log "Input: $TEXTO"

# If nothing is selected, exit gracefully
if [ -z "$TEXTO" ]; then
    log "No text selected — exiting"
    exit 0
fi

# Backup current clipboard content
OLD_CLIP="$TEXTO"

# Show progress dialog using AppleScript
osascript -e 'tell application "System Events" to display dialog "Translating..." with title "PromptBridge" buttons {"OK"} default button "OK" giving up after 3600' > /dev/null 2>&1 &
PROG_PID=$!

# Run translation (synchronously, capture output)
RAW_RESULT=$(promptbridge translate "$TEXTO" 2>&1)
EXIT_CODE=$?

# Log stderr if command failed
if [ $EXIT_CODE -ne 0 ]; then
    log "Command failed with exit code: $EXIT_CODE"
    log "Error output: $RAW_RESULT"
fi

# Extract only the translated text (after "--- Transformed Prompt ---")
if [ -n "$RAW_RESULT" ]; then
    RESULT=$(echo "$RAW_RESULT" | grep -A 100 -- "--- Transformed Prompt ---" | tail -n +2)
else
    RESULT=""
fi
log "Exit code: $EXIT_CODE | Result: $RESULT"

# Close progress dialog
kill $PROG_PID 2>/dev/null
wait $PROG_PID 2>/dev/null

# Check if auto_copy is enabled in config
CONFIG_FILE="$HOME/Library/Application Support/promptbridge/promptbridge.toml"
AUTO_COPY=$(grep -E "^auto_copy_clipboard\s*=" "$CONFIG_FILE" 2>/dev/null | cut -d= -f2 | tr -d ' "')

# Show result based on auto_copy setting
if [ $EXIT_CODE -eq 0 ] && [ -n "$RESULT" ]; then
    if [ "$AUTO_COPY" = "true" ]; then
        # Auto-copy mode: copy silently and show notification
        echo "$RESULT" | pbcopy
        log "Auto-copied to clipboard"
        
        # Show notification
        osascript -e 'display notification "✓ Translated & copied!" with title "PromptBridge" subtitle "Translation"' &
        sleep 1.5
    else
        # Manual mode: show dialog with Copy/Done buttons
        BUTTON=$(osascript -e 'tell application "System Events"
            set result to button returned of (display dialog "'" "$RESULT" '" with title "PromptBridge" buttons {"Copy", "Done"} default button "Copy" with icon note)
        end tell')
        
        if [ "$BUTTON" = "Copy" ]; then
            echo "$RESULT" | pbcopy
        else
            echo "$OLD_CLIP" | pbcopy
            log "Done - clipboard restored"
        fi
    fi
else
    osascript -e 'tell application "System Events" to display dialog "Translation failed.\nSee log for details:\n'"$LOGFILE"'" with title "PromptBridge" buttons {"OK"} with icon stop'
    echo "$OLD_CLIP" | pbcopy
fi

log "=== pb-translate done ==="
"#;

            std::fs::write(&script_path, script_content)?;
            
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
            
            let config_instructions = format!(
                "Please configure the keyboard shortcut:\n\
                 1. Open System Settings -> Keyboard -> Keyboard Shortcuts -> App Shortcuts\n\
                 2. Click + to add a new shortcut\n\
                 3. Application: All Applications\n\
                 4. Menu Title: pb-translate\n\
                 5. Keyboard Shortcut: Ctrl+Alt+T (or your preferred keys)\n\
                 \n\
                 Alternatively, use Automator to create a Quick Action:\n\
                 1. Open Automator -> Quick Action\n\
                 2. Add 'Run Shell Script' action\n\
                 3. Set script to: {}\n\
                 4. Save as 'PromptBridge Translate'\n\
                 5. Assign keyboard shortcut in System Settings",
                script_path.display()
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

impl PlatformDialog for MacosPlatform {
    fn show_progress(&self, title: &str, text: &str) -> Result<Box<dyn ProgressDialogHandle>> {
        let applescript = format!(
            r#"tell application "System Events"
                display dialog "{}" with title "{}" buttons {"OK"} default button "OK" giving up after 3600
            end tell"#,
            text, title
        );
        
        let child = Command::new("osascript")
            .args(&["-e", &applescript])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| PromptBridgeError::Engine(format!("Failed to show progress dialog: {}", e)))?;
        
        Ok(Box::new(MacosProgressDialogHandle {
            _child: Arc::new(Mutex::new(child)),
        }))
    }
    
    fn show_text_info(&self, title: &str, text: &str) -> Result<TextInfoResult> {
        let applescript = format!(
            r#"tell application "System Events"
                set result to button returned of (display dialog "{}" with title "{}" buttons {"Copy", "Done"} default button "Copy" with icon note)
            end tell"#,
            text.replace('"', r#"\\""#), title
        );
        
        let output = Command::new("osascript")
            .args(&["-e", &applescript])
            .output()
            .map_err(|e| PromptBridgeError::Engine(format!("Failed to show text info dialog: {}", e)))?;
        
        let result = String::from_utf8_lossy(&output.stdout);
        if result.contains("Copy") {
            Ok(TextInfoResult::Copy)
        } else {
            Ok(TextInfoResult::Done)
        }
    }
    
    fn show_error(&self, title: &str, text: &str) -> Result<()> {
        let applescript = format!(
            r#"tell application "System Events"
                display dialog "{}" with title "{}" buttons {"OK"} with icon stop
            end tell"#,
            text.replace('"', r#"\\""#), title
        );
        
        Command::new("osascript")
            .args(&["-e", &applescript])
            .status()
            .map_err(|e| PromptBridgeError::Engine(format!("Failed to show error dialog: {}", e)))?;
        
        Ok(())
    }
}

struct MacosProgressDialogHandle {
    _child: Arc<Mutex<std::process::Child>>,
}

impl ProgressDialogHandle for MacosProgressDialogHandle {
    fn close(&self) -> Result<()> {
        // On macOS, we'd need to kill the osascript process
        // For simplicity, we'll let it timeout naturally
        Ok(())
    }
}

impl PlatformNotifier for MacosPlatform {
    fn show_notification(&self, title: &str, message: &str) -> Result<()> {
        let applescript = format!(
            r#"display notification "{}" with title "{}""#,
            message.replace('"', r#"\\""#), title
        );
        
        Command::new("osascript")
            .args(&["-e", &applescript])
            .status()
            .map_err(|e| PromptBridgeError::Engine(format!("Failed to show notification: {}", e)))?;
        
        Ok(())
    }
}

impl PlatformClipboard for MacosPlatform {
    fn get_text(&self) -> Result<String> {
        let applescript = r#"tell application "System Events" to get the clipboard"#;
        
        let output = Command::new("osascript")
            .args(&["-e", applescript])
            .output()
            .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to get clipboard: {}", e)))?;
        
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to parse clipboard: {}", e)))
    }
    
    fn set_text(&self, text: &str) -> Result<()> {
        let applescript = format!(
            r#"set the clipboard to "{}""#,
            text.replace('"', r#"\\""#)
        );
        
        Command::new("osascript")
            .args(&["-e", &applescript])
            .status()
            .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to set clipboard: {}", e)))?;
        
        Ok(())
    }
}
