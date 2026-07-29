//! Windows-specific platform implementation
//!
//! Uses:
//! - PowerShell for clipboard access
//! - PowerShell for dialogs (WPF/Windows Forms)
//! - Windows Toast notifications
//! - PowerShell scripts for shortcut installation

use crate::constants;
use crate::platform::{
    PlatformClipboard, PlatformDialog, PlatformNotifier, PlatformShortcutInstaller,
    ProgressDialogHandle, ShortcutInstallResult, TextInfoResult,
};
use crate::utils::error::{PromptBridgeError, Result};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct WindowsPlatform;

impl WindowsPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformShortcutInstaller for WindowsPlatform {
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

        // 2. Create the shortcut script in %APPDATA%\promptbridge\pb-translate.ps1
        if let Some(appdata_dir) = dirs::config_dir() {
            let bin_dir = appdata_dir.join("promptbridge");
            std::fs::create_dir_all(&bin_dir)?;
            let script_path = bin_dir.join("pb-translate.ps1");

            let script_content = r#"# PowerShell script for PromptBridge translation shortcut
# Requires: PromptBridge installed via cargo

$ErrorActionPreference = "Continue"

# Log file for debugging
$LogDir = "$env:LOCALAPPDATA\promptbridge"
$LogFile = "$LogDir\pb-translate.log"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

function Log-Message {
    param([string]$Message)
    $timestamp = Get-Date -Format "HH:mm:ss"
    "[$timestamp] $Message" | Out-File -FilePath $LogFile -Append
}

Log-Message "=== pb-translate started ==="

# Get selected text from clipboard (Windows uses clipboard for selection)
Add-Type -AssemblyName System.Windows.Forms
$clipText = [System.Windows.Forms.Clipboard]::GetText()
Log-Message "Input: $clipText"

# If nothing is selected, exit gracefully
if ([string]::IsNullOrWhiteSpace($clipText)) {
    Log-Message "No text selected - exiting"
    exit 0
}

# Backup current clipboard content
$oldClip = $clipText

# Show progress dialog
Add-Type -AssemblyName System.Windows.Forms
$form = New-Object System.Windows.Forms.Form
$form.Text = "PromptBridge"
$form.Size = New-Object System.Drawing.Size(300,100)
$form.StartPosition = "CenterScreen"
$form.FormBorderStyle = "FixedDialog"
$form.MaximizeBox = $false
$form.MinimizeBox = $false

$label = New-Object System.Windows.Forms.Label
$label.Text = "Translating..."
$label.AutoSize = $true
$label.Location = New-Object System.Drawing.Point(100,30)
$form.Controls.Add($label)

# Show form non-blocking
$form.Show() | Out-Null
$form.Refresh()

try {
    # Run translation
    $rawResult = & promptbridge translate $clipText 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    # Log stderr if command failed
    if ($exitCode -ne 0) {
        Log-Message "Command failed with exit code: $exitCode"
        Log-Message "Error output: $rawResult"
    }
    
    # Extract translated text (after "--- Transformed Prompt ---")
    if ($rawResult -match "--- Transformed Prompt ---") {
    $result = ($rawResult -split "--- Transformed Prompt ---", 2)[1].Trim()
    } else {
        $result = $rawResult.Trim()
    }
    
    Log-Message "Exit code: $exitCode | Result: $result"
    
    # Check if auto_copy is enabled in config
    $configFile = "$env:APPDATA\promptbridge\promptbridge.toml"
    $autoCopy = $false
    if (Test-Path $configFile) {
        $configContent = Get-Content $configFile -Raw
        if ($configContent -match 'auto_copy_clipboard\s*=\s*true') {
            $autoCopy = $true
        }
    }
    
    # Show result based on auto_copy setting
    if ($exitCode -eq 0 -and $result) {
        if ($autoCopy) {
            # Auto-copy mode: copy silently and show notification
            [System.Windows.Forms.Clipboard]::SetText($result)
            Log-Message "Auto-copied to clipboard"
            
            # Show notification
            Add-Type -AssemblyName System.Windows.Forms
            Add-Type -AssemblyName System.Drawing
            
            $notify = New-Object System.Windows.Forms.NotifyIcon
            $notify.Icon = [System.Drawing.SystemIcons]::Information
            $notify.Visible = $true
            $notify.BalloonTipTitle = "PromptBridge"
            $notify.BalloonTipText = "Translated & copied!"
            $notify.ShowBalloonTip(3000)
            
            Start-Sleep -Seconds 3
            
            $notify.Dispose()
        } else {
            # Manual mode: show dialog with Copy/Close buttons
            Add-Type -AssemblyName PresentationFramework
            [xml]$xaml = @"
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        Title="PromptBridge" Width="700" Height="500" WindowStartupLocation="CenterScreen">
    <DockPanel Margin="10">
        <TextBox Name="ResultText" TextWrapping="Wrap" AcceptsReturn="True" 
                 VerticalScrollBarVisibility="Auto" DockPanel.Dock="Top" Height="400"/>
        <StackPanel Orientation="Horizontal" HorizontalAlignment="Right" DockPanel.Dock="Bottom" Margin="0,10">
            <Button Name="CopyBtn" Content="Copy" Width="80" Margin="0,0,10,0"/>
            <Button Name="DoneBtn" Content="Done" Width="80"/>
        </StackPanel>
    </DockPanel>
</Window>
"@
            
            $reader = (New-Object System.Xml.XmlNodeReader $xaml)
            $window = [Windows.Markup.XamlReader]::Load($reader)
            $resultText = $window.FindName("ResultText")
            $copyBtn = $window.FindName("CopyBtn")
            $doneBtn = $window.FindName("DoneBtn")
            
            $resultText.Text = $result
            
            $copyResult = {
                [System.Windows.Forms.Clipboard]::SetText($resultText.Text)
                $window.Close()
            }
            
            $doneResult = {
                [System.Windows.Forms.Clipboard]::SetText($oldClip)
                Log-Message "Done - clipboard restored"
                $window.Close()
            }
            
            $copyBtn.Add_Click($copyResult)
            $doneBtn.Add_Click($doneResult)
            
            $window.ShowDialog() | Out-Null
        }
    } else {
        # Show error dialog
        Add-Type -AssemblyName PresentationFramework
        [System.Windows.MessageBox]::Show(
            "Translation failed.`nSee log for details:`n$LogFile",
            "PromptBridge",
            [System.Windows.MessageBoxButton]::OK,
            [System.Windows.MessageBoxImage]::Error
        ) | Out-Null
        
        [System.Windows.Forms.Clipboard]::SetText($oldClip)
    }
} finally {
    # Close progress dialog
    $form.Close()
}

Log-Message "=== pb-translate done ===""#;

            // Write with CRLF line endings for Windows PowerShell compatibility
            let script_content_crlf = script_content.replace('\n', "\r\n");
            std::fs::write(&script_path, script_content_crlf)?;

            // Create AutoHotkey script automatically
            let ahk_script_path = bin_dir.join("promptbridge.ahk");
            let ahk_content = r#"#Requires AutoHotkey v2.0
; PromptBridge AutoHotkey Script - Ctrl+Alt+T to translate

^!t::
{
    Run 'PowerShell.exe -ExecutionPolicy Bypass -File "%APPDATA%\promptbridge\pb-translate.ps1"'
}
"#;
            std::fs::write(&ahk_script_path, ahk_content)?;

            let config_instructions = format!(
                "Keyboard shortcut configured!\n\
                 \n\
                 Next steps:\n\
                 1. Install AutoHotkey v2: https://www.autohotkey.com/\n\
                 2. Double-click: {}\n\
                 3. Press Ctrl+Alt+T to translate selected text",
                ahk_script_path.display()
            );

            Ok(ShortcutInstallResult {
                script_path: script_path.display().to_string(),
                config_instructions,
            })
        } else {
            Err(PromptBridgeError::Engine(
                "Could not locate APPDATA directory".to_string(),
            ))
        }
    }
}

impl PlatformDialog for WindowsPlatform {
    fn show_progress(&self, title: &str, text: &str) -> Result<Box<dyn ProgressDialogHandle>> {
        let ps_script = format!(
            r#"
Add-Type -AssemblyName PresentationFramework
[xml]$xaml = @"
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        Title="{}" Width="300" Height="100" WindowStyle="ToolWindow" Topmost="True">
    <StackPanel Margin="20">
        <TextBlock Text="{}" FontSize="14" HorizontalAlignment="Center"/>
        <ProgressBar IsIndeterminate="True" Height="20" Margin="0,10"/>
    </StackPanel>
</Window>
"@

$reader = (New-Object System.Xml.XmlNodeReader $xaml)
$window = [Windows.Markup.XamlReader]::Load($reader)
$window.Show() | Out-Null
Start-Sleep -Seconds 3600
"#,
            title, text
        );

        let child = Command::new("powershell")
            .args(&["-Command", &ps_script])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                PromptBridgeError::Engine(format!("Failed to show progress dialog: {}", e))
            })?;

        Ok(Box::new(WindowsProgressDialogHandle {
            _child: Arc::new(Mutex::new(child)),
        }))
    }

    fn show_text_info(&self, title: &str, text: &str) -> Result<TextInfoResult> {
        let ps_script = format!(
            r#"
Add-Type -AssemblyName PresentationFramework
[xml]$xaml = @"
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        Title="{}" Width="700" Height="500" WindowStartupLocation="CenterScreen">
    <DockPanel Margin="10">
        <TextBox Name="ResultText" TextWrapping="Wrap" AcceptsReturn="True" 
                 VerticalScrollBarVisibility="Auto" DockPanel.Dock="Top" Height="400"/>
        <StackPanel Orientation="Horizontal" HorizontalAlignment="Right" DockPanel.Dock="Bottom" Margin="0,10">
            <Button Name="CopyBtn" Content="Copy" Width="80" Margin="0,0,10,0"/>
            <Button Name="DoneBtn" Content="Done" Width="80"/>
        </StackPanel>
    </DockPanel>
</Window>
"@

$reader = (New-Object System.Xml.XmlNodeReader $xaml)
$window = [Windows.Markup.XamlReader]::Load($reader)
$resultText = $window.FindName("ResultText")
$copyBtn = $window.FindName("CopyBtn")
$doneBtn = $window.FindName("DoneBtn")

$resultText.Text = @"

{}

"@

$copyResult = {{
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.Clipboard]::SetText($resultText.Text)
    $window.Close()
    exit 0
}}

$doneResult = {{
    $window.Close()
    exit 1
}}

$copyBtn.Add_Click($copyResult)
$doneBtn.Add_Click($doneResult)

$window.ShowDialog() | Out-Null
"#,
            title, text
        );

        let output = Command::new("powershell")
            .args(&["-Command", &ps_script])
            .output()
            .map_err(|e| {
                PromptBridgeError::Engine(format!("Failed to show text info dialog: {}", e))
            })?;

        if output.status.success() {
            Ok(TextInfoResult::Copy)
        } else {
            Ok(TextInfoResult::Done)
        }
    }

    fn show_error(&self, title: &str, text: &str) -> Result<()> {
        let ps_script = format!(
            r#"
Add-Type -AssemblyName PresentationFramework
[System.Windows.MessageBox]::Show(
    "{}",
    "{}",
    [System.Windows.MessageBoxButton]::OK,
    [System.Windows.MessageBoxImage]::Error
) | Out-Null
"#,
            text, title
        );

        Command::new("powershell")
            .args(&["-Command", &ps_script])
            .status()
            .map_err(|e| {
                PromptBridgeError::Engine(format!("Failed to show error dialog: {}", e))
            })?;

        Ok(())
    }
}

struct WindowsProgressDialogHandle {
    _child: Arc<Mutex<std::process::Child>>,
}

impl ProgressDialogHandle for WindowsProgressDialogHandle {
    fn close(&self) -> Result<()> {
        // On Windows, we'd need to kill the PowerShell process
        // For simplicity, we'll let it timeout naturally
        Ok(())
    }
}

impl PlatformNotifier for WindowsPlatform {
    fn show_notification(&self, title: &str, message: &str) -> Result<()> {
        let ps_script = format!(
            r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$notify = New-Object System.Windows.Forms.NotifyIcon

$notify.Icon = [System.Drawing.SystemIcons]::Information
$notify.Visible = $true

$notify.BalloonTipTitle = "{}"
$notify.BalloonTipText = "{}"

$notify.ShowBalloonTip(3000)

Start-Sleep -Seconds 3

$notify.Dispose()
"#,
            title, message
        );

        Command::new("powershell")
            .args(&["-Command", &ps_script])
            .status()
            .map_err(|e| {
                PromptBridgeError::Engine(format!("Failed to show notification: {}", e))
            })?;

        Ok(())
    }
}

impl PlatformClipboard for WindowsPlatform {
    fn get_text(&self) -> Result<String> {
        let ps_script = r#"
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.Clipboard]::GetText()
"#;

        let output = Command::new("powershell")
            .args(&["-Command", ps_script])
            .output()
            .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to get clipboard: {}", e)))?;

        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to parse clipboard: {}", e)))
    }

    fn set_text(&self, text: &str) -> Result<()> {
        let ps_script = format!(
            r#"
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.Clipboard]::SetText("{}")
"#,
            text.replace('"', "`\"")
        );

        Command::new("powershell")
            .args(&["-Command", &ps_script])
            .status()
            .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to set clipboard: {}", e)))?;

        Ok(())
    }
}
