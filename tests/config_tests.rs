use promptbridge::config::Config;

#[test]
fn test_default_config_loading() {
    let config = Config::load(None).unwrap();
    assert_eq!(config.general.default_provider, "ollama");
    assert_eq!(config.general.target_language, "en");
    assert_eq!(config.general.mode, "preview");
    assert!(config.providers.contains_key("ollama"));
    assert!(config.providers.contains_key("openai"));
    assert!(config.providers.contains_key("mock"));
}
