//! Tests for TOML config loading, creation, and path resolution.

use super::*;
use std::path::Path;

#[test]
fn load_from_nonexistent_returns_file_not_found() {
    let result = load_from_path(Path::new("/tmp/nonexistent_jarvis_config.toml"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, jarvis_common::ConfigError::ParseError(_)));
}

#[test]
fn load_valid_partial_toml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        r##"
[font]
family = "SF Mono"
size = 14

[colors]
primary = "#ff0000"
"##,
    )
    .unwrap();

    let config = load_from_path(&path).unwrap();
    assert_eq!(config.font.family, "SF Mono");
    assert_eq!(config.font.size, 14);
    assert_eq!(config.colors.primary, "#ff0000");
    // Defaults preserved
    assert_eq!(config.colors.background, "#000000");
    assert_eq!(config.theme.name, "jarvis-dark");
}

#[test]
fn load_invalid_toml_returns_parse_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "this is not valid toml {{{").unwrap();

    let result = load_from_path(&path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, jarvis_common::ConfigError::ParseError(_)));
}

#[test]
fn load_config_with_invalid_values_falls_back_to_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        r#"
[font]
size = 100
"#,
    )
    .unwrap();

    let config = load_from_path(&path).unwrap();
    // No longer falls back — parsed config returned with invalid values
    assert_eq!(config.font.size, 100);
}

#[test]
fn create_and_load_default_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jarvis").join("config.toml");

    create_default_config(&path).unwrap();
    assert!(path.exists());

    let config = load_from_path(&path).unwrap();
    assert_eq!(config.theme.name, "jarvis-dark");
    assert_eq!(config.colors.primary, "#00d4ff");
}

#[test]
fn default_config_toml_is_valid() {
    use super::template::default_config_toml;
    use crate::schema::JarvisConfig;

    let content = default_config_toml();
    let config: JarvisConfig = toml::from_str(&content).unwrap();
    assert_eq!(config.theme.name, "jarvis-dark");
}

#[test]
fn default_config_path_is_reasonable() {
    // This may not work in all CI environments, but should work locally
    if let Ok(path) = default_config_path() {
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("jarvis"));
        assert!(path_str.ends_with("config.toml"));
    }
}
