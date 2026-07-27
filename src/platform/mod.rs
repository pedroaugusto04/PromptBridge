//! Platform abstraction layer for cross-platform support
//! 
//! This module provides trait-based abstractions for platform-specific operations:
//! - Shortcut installation
//! - Dialog display (progress, text info, error)
//! - Desktop notifications
//! - Clipboard operations

use crate::utils::error::Result;

/// Platform-specific shortcut installer
pub trait PlatformShortcutInstaller {
    /// Install the platform-specific shortcut script
    fn install_shortcut(&self) -> Result<ShortcutInstallResult>;
}

/// Result of shortcut installation
#[derive(Debug)]
pub struct ShortcutInstallResult {
    pub script_path: String,
    pub config_instructions: String,
}

/// Platform-specific dialog operations
pub trait PlatformDialog {
    /// Show a pulsating progress dialog (non-blocking)
    fn show_progress(&self, title: &str, text: &str) -> Result<Box<dyn ProgressDialogHandle>>;
    
    /// Show a text info dialog with Copy/Done buttons
    fn show_text_info(&self, title: &str, text: &str) -> Result<TextInfoResult>;
    
    /// Show an error dialog
    fn show_error(&self, title: &str, text: &str) -> Result<()>;
}

/// Handle for a progress dialog
pub trait ProgressDialogHandle: Send {
    /// Close the progress dialog
    fn close(&self) -> Result<()>;
}

/// Result from text info dialog
#[derive(Debug, PartialEq)]
pub enum TextInfoResult {
    Copy,
    Done,
}

/// Platform-specific notification operations
pub trait PlatformNotifier {
    /// Show a desktop notification
    fn show_notification(&self, title: &str, message: &str) -> Result<()>;
}

/// Platform-specific clipboard operations
pub trait PlatformClipboard {
    /// Get text from clipboard
    fn get_text(&self) -> Result<String>;
    
    /// Set text to clipboard
    fn set_text(&self, text: &str) -> Result<()>;
}

// Platform-specific implementations
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxPlatform;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacosPlatform;

/// Get the platform-specific implementation
pub fn get_platform() -> Box<dyn PlatformShortcutInstaller> {
    #[cfg(target_os = "linux")]
    {
        Box::new(LinuxPlatform::new())
    }
    
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsPlatform::new())
    }
    
    #[cfg(target_os = "macos")]
    {
        Box::new(MacosPlatform::new())
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        compile_error!("Unsupported platform");
    }
}

/// Get the platform-specific dialog implementation
pub fn get_platform_dialog() -> Box<dyn PlatformDialog> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatform::new())
    }

    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatform::new())
    }

    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacosPlatform::new())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        compile_error!("Unsupported platform");
    }
}

/// Get the platform-specific notification implementation
pub fn get_platform_notifier() -> Box<dyn PlatformNotifier> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatform::new())
    }

    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPlatform::new())
    }

    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacosPlatform::new())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        compile_error!("Unsupported platform");
    }
}
