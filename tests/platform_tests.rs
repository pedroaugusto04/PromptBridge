//! Platform-specific module tests
//! 
//! Tests for the cross-platform abstraction layer including:
//! - Shortcut installation
//! - Dialog operations
//! - Notifications
//! - Clipboard operations

#[cfg(test)]
mod platform_tests {
    // Note: These tests are designed to be run on their respective platforms
    // They will be conditionally compiled based on the target OS
    
    #[cfg(target_os = "linux")]
    mod linux {
        use promptbridge::platform::linux::LinuxPlatform;
        use promptbridge::platform::{PlatformClipboard, PlatformNotifier, PlatformShortcutInstaller};
        
        #[test]
        fn test_linux_platform_creation() {
            let _platform = LinuxPlatform::new();
            // Test that platform can be created without panicking
            assert!(true);
        }
        
        #[test]
        fn test_linux_clipboard_set_text() {
            let test_text = "Test text for clipboard";
            
            // This test requires xclip to be installed
            // Skip if not available
            let result = std::process::Command::new("which")
                .arg("xclip")
                .output();
            
            if result.is_err() || !result.unwrap().status.success() {
                println!("Skipping clipboard test: xclip not installed");
                return;
            }
            
            // Clear clipboard first by setting empty string
            let _ = std::process::Command::new("sh")
                .args(&["-c", "echo -n | xclip -selection clipboard"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            
            let platform = LinuxPlatform::new();
            let set_result = platform.set_text(test_text);
            assert!(set_result.is_ok(), "Failed to set clipboard text");
            
            let get_result = platform.get_text();
            assert!(get_result.is_ok(), "Failed to get clipboard text");
            
            let retrieved_text = get_result.unwrap();
            assert_eq!(retrieved_text, test_text, "Clipboard text mismatch");
        }
        
        #[test]
        fn test_linux_notification() {
            let platform = LinuxPlatform::new();
            
            // This test requires notify-send to be installed
            let result = std::process::Command::new("which")
                .arg("notify-send")
                .output();
            
            if result.is_err() || !result.unwrap().status.success() {
                println!("Skipping notification test: notify-send not installed");
                return;
            }
            
            let notification_result = platform.show_notification("Test Title", "Test Message");
            assert!(notification_result.is_ok(), "Failed to show notification");
        }
        
        #[test]
        fn test_linux_shortcut_install_config_creation() {
            let platform = LinuxPlatform::new();
            
            // Test that shortcut installation creates config directory
            let result = platform.install_shortcut();
            
            // Clean up after test
            if let Ok(_) = result {
                let config_dir = dirs::config_dir()
                    .map(|d| d.join("promptbridge"));
                
                if let Some(config_path) = config_dir {
                    if config_path.exists() {
                        let _ = std::fs::remove_dir_all(&config_path);
                    }
                }
                
                let bin_dir = dirs::home_dir()
                    .map(|d| d.join(".local/bin"));
                
                if let Some(bin_path) = bin_dir {
                    let script_path = bin_path.join("pb-translate");
                    if script_path.exists() {
                        let _ = std::fs::remove_file(&script_path);
                    }
                }
            }
            
            assert!(result.is_ok(), "Failed to install shortcut");
        }
    }
    
    #[cfg(target_os = "windows")]
    mod windows {
        use promptbridge::platform::windows::WindowsPlatform;
        use promptbridge::platform::{PlatformClipboard, PlatformNotifier, PlatformShortcutInstaller};
        
        #[test]
        fn test_windows_platform_creation() {
            let platform = WindowsPlatform::new();
            // Test that platform can be created without panicking
            assert!(true);
        }
        
        #[test]
        fn test_windows_clipboard_operations() {
            let platform = WindowsPlatform::new();
            let test_text = "Test text for clipboard";
            
            // This test requires PowerShell
            let set_result = platform.set_text(test_text);
            assert!(set_result.is_ok(), "Failed to set clipboard text");
            
            let get_result = platform.get_text();
            assert!(get_result.is_ok(), "Failed to get clipboard text");
            
            let retrieved_text = get_result.unwrap();
            assert_eq!(retrieved_text, test_text, "Clipboard text mismatch");
        }
        
        #[test]
        fn test_windows_notification() {
            let platform = WindowsPlatform::new();
            
            let notification_result = platform.show_notification("Test Title", "Test Message");
            assert!(notification_result.is_ok(), "Failed to show notification");
        }
        
        #[test]
        fn test_windows_shortcut_install_config_creation() {
            let platform = WindowsPlatform::new();
            
            let result = platform.install_shortcut();
            
            // Clean up after test
            if let Ok(_) = result {
                let config_dir = dirs::config_dir()
                    .map(|d| d.join("promptbridge"));
                
                if let Some(config_path) = config_dir {
                    if config_path.exists() {
                        let _ = std::fs::remove_dir_all(&config_path);
                    }
                }
            }
            
            assert!(result.is_ok(), "Failed to install shortcut");
        }
    }
    
    #[cfg(target_os = "macos")]
    mod macos {
        use promptbridge::platform::macos::MacosPlatform;
        use promptbridge::platform::{PlatformClipboard, PlatformNotifier, PlatformShortcutInstaller};
        
        #[test]
        fn test_macos_platform_creation() {
            let platform = MacosPlatform::new();
            // Test that platform can be created without panicking
            assert!(true);
        }
        
        #[test]
        fn test_macos_clipboard_operations() {
            let platform = MacosPlatform::new();
            let test_text = "Test text for clipboard";
            
            // This test requires osascript
            let set_result = platform.set_text(test_text);
            assert!(set_result.is_ok(), "Failed to set clipboard text");
            
            let get_result = platform.get_text();
            assert!(get_result.is_ok(), "Failed to get clipboard text");
            
            let retrieved_text = get_result.unwrap();
            assert_eq!(retrieved_text, test_text, "Clipboard text mismatch");
        }
        
        #[test]
        fn test_macos_notification() {
            let platform = MacosPlatform::new();
            
            let notification_result = platform.show_notification("Test Title", "Test Message");
            assert!(notification_result.is_ok(), "Failed to show notification");
        }
        
        #[test]
        fn test_macos_shortcut_install_config_creation() {
            let platform = MacosPlatform::new();
            
            let result = platform.install_shortcut();
            
            // Clean up after test
            if let Ok(_) = result {
                let config_dir = dirs::home_dir()
                    .map(|d| d.join("Library/Application Support/promptbridge"));
                
                if let Some(config_path) = config_dir {
                    if config_path.exists() {
                        let _ = std::fs::remove_dir_all(&config_path);
                    }
                }
            }
            
            assert!(result.is_ok(), "Failed to install shortcut");
        }
    }
    
    #[test]
    fn test_platform_detection() {
        // Test that get_platform returns the correct platform implementation
        let _platform = promptbridge::platform::get_platform();
        // Just ensure it doesn't panic
        assert!(true);
    }
}

#[cfg(test)]
mod platform_trait_tests {
    // Tests for trait implementations and interface consistency
    
    #[test]
    fn test_shortcut_install_result_structure() {
        use promptbridge::platform::ShortcutInstallResult;
        
        let result = ShortcutInstallResult {
            script_path: "/test/path/script.sh".to_string(),
            config_instructions: "Test instructions".to_string(),
        };
        
        assert_eq!(result.script_path, "/test/path/script.sh");
        assert_eq!(result.config_instructions, "Test instructions");
    }
    
    #[test]
    fn test_text_info_result_variants() {
        use promptbridge::platform::TextInfoResult;
        
        let copy_result = TextInfoResult::Copy;
        let done_result = TextInfoResult::Done;
        
        assert_eq!(copy_result, TextInfoResult::Copy);
        assert_eq!(done_result, TextInfoResult::Done);
        assert_ne!(copy_result, done_result);
    }
}
