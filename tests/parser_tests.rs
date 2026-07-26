use promptbridge::engine::parser::{ExtractedItem, TechnicalKind, TechnicalParser};

#[test]
fn test_extract_fenced_code_blocks() {
    let input = "Corrija o código abaixo:\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\nE adicione erro.";
    let (sanitized, items) = TechnicalParser::extract(input);

    assert!(sanitized.contains("__PB_CODE_BLOCK_0__"));
    assert!(!sanitized.contains("fn main()"));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].kind, TechnicalKind::FencedCodeBlock);

    let restored = TechnicalParser::restore(&sanitized, &items);
    assert_eq!(restored, input);
}

#[test]
fn test_extract_inline_code_and_paths() {
    let input = "Se a função `parse_request` em src/api/v1.rs falhar, lance erro.";
    let (sanitized, items) = TechnicalParser::extract(input);

    assert!(sanitized.contains("__PB_CODE_INLINE_"));
    assert!(sanitized.contains("__PB_PATH_"));
    assert!(!sanitized.contains("`parse_request`"));
    assert!(!sanitized.contains("src/api/v1.rs"));

    let restored = TechnicalParser::restore(&sanitized, &items);
    assert_eq!(restored, input);
}

#[test]
fn test_preserves_code_without_corruption() {
    let input = "Refatore `user_controller` para usar `tokio::spawn` em `src/controllers/user.rs`.";
    let (sanitized, items) = TechnicalParser::extract(input);

    // Simulate LLM returning translated text with intact placeholders
    let mock_llm_output = "Refactor __PB_CODE_INLINE_0__ to use __PB_CODE_INLINE_1__ in __PB_PATH_2__.";
    let restored = TechnicalParser::restore(mock_llm_output, &items);

    assert_eq!(
        restored,
        "Refactor `user_controller` to use `tokio::spawn` in `src/controllers/user.rs`."
    );
}
