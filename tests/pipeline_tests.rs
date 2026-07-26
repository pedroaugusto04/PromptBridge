use promptbridge::engine::{TransformMode, TransformationPipeline};
use promptbridge::providers::mock::MockProvider;

#[tokio::test]
async fn test_pipeline_transform_mode_with_mock_provider() {
    let mock_provider = MockProvider::new(None);
    let input = "Se a requisição falhar em src/api.rs, lance erro `ApiError`.";

    let result = TransformationPipeline::execute(
        &mock_provider,
        input,
        TransformMode::Transform,
        "en",
    )
    .await
    .unwrap();

    assert_eq!(result.original_text, input);
    assert!(result.sanitized_text.contains("__PB_PATH_"));
    assert!(!result.final_prompt.is_empty());
}
