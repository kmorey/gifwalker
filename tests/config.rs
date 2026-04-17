use gifwalker::config::{AppConfig, ConfigStore};

#[test]
fn persists_api_key_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(dir.path().join("config.toml"));

    let config = AppConfig {
        giphy_api_key: Some("test-key".into()),
    };

    store.save(&config).unwrap();
    let loaded = store.load().unwrap();

    assert_eq!(loaded.giphy_api_key.as_deref(), Some("test-key"));
}

#[test]
fn missing_file_returns_default_config() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(dir.path().join("missing.toml"));

    let loaded = store.load().unwrap();

    assert_eq!(loaded, AppConfig::default());
}
