use promptbridge::engine::parser::{TechnicalKind, TechnicalParser};

#[test]
fn test_extract_fenced_code_blocks() {
    let input = "Corrija o código abaixo:\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\nE adicione erro.";
    let (sanitized, items) = TechnicalParser::extract(input).unwrap();

    assert!(sanitized.contains("__PB_CODE_BLOCK_0__"));
    assert!(!sanitized.contains("fn main()"));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].kind, TechnicalKind::FencedCodeBlock);

    let restored = TechnicalParser::restore(&sanitized, &items).unwrap();
    assert_eq!(restored, input);
}

#[test]
fn test_extract_inline_code_and_paths() {
    let input = "Se a função `parse_request` em src/api/v1.rs falhar, lance erro.";
    let (sanitized, items) = TechnicalParser::extract(input).unwrap();

    assert!(sanitized.contains("__PB_CODE_INLINE_"));
    assert!(sanitized.contains("__PB_PATH_"));
    assert!(!sanitized.contains("`parse_request`"));
    assert!(!sanitized.contains("src/api/v1.rs"));

    let restored = TechnicalParser::restore(&sanitized, &items).unwrap();
    assert_eq!(restored, input);
}

#[test]
fn test_preserves_code_without_corruption() {
    let input = "Refatore `user_controller` para usar `tokio::spawn` em `src/controllers/user.rs`.";
    let (sanitized, items) = TechnicalParser::extract(input).unwrap();

    // Simulate LLM returning translated text with intact placeholders
    // The sanitized text contains placeholders like __PB_CODE_INLINE_0__, __PB_CODE_INLINE_1__, __PB_PATH_2__
    // We simulate the LLM translating the Portuguese text but keeping placeholders
    let mock_llm_output = sanitized
        .replace("Refatore", "Refactor")
        .replace("para usar", "to use")
        .replace("em", "in");
    let restored = TechnicalParser::restore(&mock_llm_output, &items).unwrap();

    assert_eq!(
        restored,
        "Refactor `user_controller` to use `tokio::spawn` in `src/controllers/user.rs`."
    );
}

#[test]
fn test_rejects_oversized_input() {
    // Create input larger than MAX_PROMPT_SIZE_BYTES (100KB)
    let oversized_input = "a".repeat(101_000);
    let result = TechnicalParser::extract(&oversized_input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("exceeds maximum size"));
}

#[test]
fn test_rejects_oversized_response() {
    let input = "Test input with `code`";
    let (_sanitized, items) = TechnicalParser::extract(input).unwrap();

    // Create response larger than MAX_RESPONSE_SIZE_BYTES (10KB)
    let oversized_response = "a".repeat(11_000);
    let result = TechnicalParser::restore(&oversized_response, &items);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("exceeds maximum size"));
}

#[test]
fn test_rejects_excessive_extracted_items() {
    // Create input with many code blocks to trigger MAX_EXTRACTED_ITEMS limit
    let many_code_blocks = "```rust\ncode\n```".repeat(1001);
    let result = TechnicalParser::extract(&many_code_blocks);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Too many technical elements"));
}
