use crate::constants::{
    PLACEHOLDER_CODE_BLOCK_PREFIX, PLACEHOLDER_INLINE_CODE_PREFIX, PLACEHOLDER_PATH_PREFIX,
    PLACEHOLDER_PREFIX, REGEX_FENCED_CODE, REGEX_FILE_PATH, REGEX_INLINE_CODE,
};
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

pub struct TechnicalParser;

impl TechnicalParser {
    /// Extracts technical elements (code blocks, inline code, file paths, shell commands)
    /// and replaces them with safe unique placeholder strings.
    pub fn extract(input: &str) -> (String, Vec<ExtractedItem>) {
        let mut placeholders = Vec::new();
        let mut placeholder_count = 0;
        let mut processed = input.to_string();

        // 1. Fenced Code Blocks (```rust ... ```)
        let re_fenced = Regex::new(REGEX_FENCED_CODE).unwrap();
        for mat in re_fenced.find_iter(input) {
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
        let re_inline = Regex::new(REGEX_INLINE_CODE).unwrap();
        let input_after_fenced = processed.clone();
        for mat in re_inline.find_iter(&input_after_fenced) {
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
        let re_paths = Regex::new(REGEX_FILE_PATH).unwrap();
        let input_after_inline = processed.clone();
        for mat in re_paths.find_iter(&input_after_inline) {
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

        (processed, placeholders)
    }

    /// Re-hydrates placeholders back to their exact original text
    pub fn restore(transformed: &str, items: &[ExtractedItem]) -> String {
        let mut result = transformed.to_string();
        
        // If no items to restore, return as-is
        if items.is_empty() {
            return result;
        }
        
        let mut item_map: HashMap<String, String> = HashMap::new();

        for item in items {
            item_map.insert(item.placeholder.clone(), item.original_text.clone());
        }

        for (placeholder, original) in item_map {
            result = result.replace(&placeholder, &original);
        }

        result
    }
}
