use crate::constants::{
    MAX_EXTRACTED_ITEMS, MAX_PROMPT_SIZE_BYTES, MAX_RESPONSE_SIZE_BYTES,
    PLACEHOLDER_CODE_BLOCK_PREFIX, PLACEHOLDER_INLINE_CODE_PREFIX, PLACEHOLDER_PATH_PREFIX,
    PLACEHOLDER_PREFIX, REGEX_FENCED_CODE, REGEX_FILE_PATH, REGEX_INLINE_CODE,
};
use crate::utils::error::{PromptBridgeError, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TechnicalKind {
    FencedCodeBlock,
    InlineCode,
    FilePath,
    ShellCommand,
}

#[derive(Debug, Clone)]
pub struct ExtractedItem {
    pub placeholder: String,
    pub original_text: String,
    pub kind: TechnicalKind,
}

// Lazy-initialized regex patterns with proper error handling at compile time
lazy_static! {
    static ref RE_FENCED_CODE: Regex = Regex::new(REGEX_FENCED_CODE)
        .expect("Failed to compile fenced code regex");
    static ref RE_INLINE_CODE: Regex = Regex::new(REGEX_INLINE_CODE)
        .expect("Failed to compile inline code regex");
    static ref RE_FILE_PATH: Regex = Regex::new(REGEX_FILE_PATH)
        .expect("Failed to compile file path regex");
}

pub struct TechnicalParser;

impl TechnicalParser {
    /// Extracts technical elements (code blocks, inline code, file paths, shell commands)
    /// and replaces them with safe unique placeholder strings.
    pub fn extract(input: &str) -> Result<(String, Vec<ExtractedItem>)> {
        // Validate input size to prevent memory exhaustion
        if input.len() > MAX_PROMPT_SIZE_BYTES {
            return Err(PromptBridgeError::Parser(format!(
                "Input prompt exceeds maximum size of {} bytes (got {} bytes)",
                MAX_PROMPT_SIZE_BYTES,
                input.len()
            )));
        }

        let mut placeholders = Vec::new();
        let mut placeholder_count = 0;
        let mut processed = input.to_string();

        // 1. Fenced Code Blocks (```rust ... ```)
        for mat in RE_FENCED_CODE.find_iter(input) {
            let original = mat.as_str().to_string();
            let placeholder = format!("{}{}{}__", PLACEHOLDER_CODE_BLOCK_PREFIX, placeholder_count, "");
            placeholder_count += 1;

            placeholders.push(ExtractedItem {
                placeholder: placeholder.clone(),
                original_text: original.clone(),
                kind: TechnicalKind::FencedCodeBlock,
            });

            processed = processed.replace(&original, &placeholder);
        }

        // 2. Inline Code (`code_snippet`)
        let input_after_fenced = processed.clone();
        for mat in RE_INLINE_CODE.find_iter(&input_after_fenced) {
            let original = mat.as_str().to_string();
            let placeholder = format!("{}{}{}__", PLACEHOLDER_INLINE_CODE_PREFIX, placeholder_count, "");
            placeholder_count += 1;

            placeholders.push(ExtractedItem {
                placeholder: placeholder.clone(),
                original_text: original.clone(),
                kind: TechnicalKind::InlineCode,
            });

            processed = processed.replace(&original, &placeholder);
        }

        // 3. File Paths (e.g. src/api/v1.rs, C:\Users\path\file.txt, ./config.toml)
        let input_after_inline = processed.clone();
        for mat in RE_FILE_PATH.find_iter(&input_after_inline) {
            let original = mat.as_str().to_string();
            if original.starts_with(PLACEHOLDER_PREFIX) {
                continue;
            }
            let placeholder = format!("{}{}{}__", PLACEHOLDER_PATH_PREFIX, placeholder_count, "");
            placeholder_count += 1;

            placeholders.push(ExtractedItem {
                placeholder: placeholder.clone(),
                original_text: original.clone(),
                kind: TechnicalKind::FilePath,
            });

            processed = processed.replace(&original, &placeholder);
        }

        // Validate number of extracted items to prevent excessive processing
        if placeholders.len() > MAX_EXTRACTED_ITEMS {
            return Err(PromptBridgeError::Parser(format!(
                "Too many technical elements extracted ({}), maximum allowed is {}",
                placeholders.len(),
                MAX_EXTRACTED_ITEMS
            )));
        }

        Ok((processed, placeholders))
    }

    /// Re-hydrates placeholders back to their exact original text
    pub fn restore(transformed: &str, items: &[ExtractedItem]) -> Result<String> {
        // Validate response size to prevent memory exhaustion
        if transformed.len() > MAX_RESPONSE_SIZE_BYTES {
            return Err(PromptBridgeError::Parser(format!(
                "LLM response exceeds maximum size of {} bytes (got {} bytes)",
                MAX_RESPONSE_SIZE_BYTES,
                transformed.len()
            )));
        }

        let mut result = transformed.to_string();

        // If no items to restore, return as-is
        if items.is_empty() {
            return Ok(result);
        }

        let mut item_map: HashMap<String, String> = HashMap::new();

        for item in items {
            item_map.insert(item.placeholder.clone(), item.original_text.clone());
        }

        for (placeholder, original) in item_map {
            result = result.replace(&placeholder, &original);
        }

        Ok(result)
    }
}
