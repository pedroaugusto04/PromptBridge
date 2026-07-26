use crate::utils::error::{PromptBridgeError, Result};
use arboard::Clipboard;

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to access clipboard: {}", e)))?;
    
    clipboard
        .set_text(text.to_string())
        .map_err(|e| PromptBridgeError::Clipboard(format!("Failed to copy text: {}", e)))?;
    
    Ok(())
}
