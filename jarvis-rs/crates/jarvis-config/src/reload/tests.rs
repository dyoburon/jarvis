//! Tests for the reload manager.

use super::*;
use std::path::PathBuf;

#[tokio::test]
async fn start_with_nonexistent_path_uses_defaults() {
    let path = PathBuf::from("/tmp/nonexistent_jarvis_reload_test.toml");
    let (config, _rx) = ReloadManager::start(path).await;
    assert_eq!(config.theme.name, "jarvis-dark");
    assert_eq!(config.colors.primary, "#ffcc66");
}

#[tokio::test]
async fn start_with_valid_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        r#"
[font]
family = "Fira Code"
"#,
    )
    .unwrap();

    let (config, _rx) = ReloadManager::start(path).await;
    assert_eq!(config.font.family, "Fira Code");
    assert_eq!(config.colors.primary, "#ffcc66"); // default
}
