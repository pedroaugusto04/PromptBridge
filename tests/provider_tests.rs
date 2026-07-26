use promptbridge::config::ProviderConfig;
use promptbridge::providers::types::{ChatMessage, CompletionRequest};
use promptbridge::providers::ProviderFactory;

#[tokio::test]
async fn test_mock_provider_factory_creation() {
    let config = ProviderConfig {
        provider_type: "mock".to_string(),
        base_url: None,
        api_key: None,
        model: Some("test-mock".to_string()),
        temperature: Some(0.0),
    };

    let provider = ProviderFactory::create(&config).unwrap();
    assert_eq!(provider.name(), "Mock (Dry-Run)");

    let req = CompletionRequest {
        messages: vec![ChatMessage::user("Hello world")],
        temperature: None,
        max_tokens: None,
        model: None,
    };

    let res = provider.completion(&req).await.unwrap();
    assert!(res.content.contains("Hello world"));
}
