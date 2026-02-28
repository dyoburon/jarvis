//! Core TOML config loading: read from path or platform default.

use crate::schema::JarvisConfig;
use crate::validation;
use jarvis_common::ConfigError;
use std::path::Path;
use tracing::{info, warn};

use super::paths::{create_default_config, default_config_path};

/// Load config from a specific TOML file path.
///
/// Deserializes the file using serde defaults for any missing fields.
/// After loading, the config is validated; if validation fails, a warning
/// is logged and the parsed config is returned as-is.
pub fn load_from_path(path: &Path) -> Result<JarvisConfig, ConfigError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::ParseError(format!("failed to read {}: {e}", path.display())))?;

    let config: JarvisConfig = toml::from_str(&content)
        .map_err(|e| ConfigError::ParseError(format!("failed to parse TOML: {e}")))?;

    if let Err(e) = validation::validate(&config) {
        warn!(
            "config validation warning: {e} — using parsed config with potentially invalid values"
        );
    }

    info!("loaded config from {}", path.display());
    Ok(config)
}

/// Load config from the platform-specific default path.
///
/// On macOS: `~/Library/Application Support/jarvis/config.toml`
/// On Linux: `~/.config/jarvis/config.toml`
///
/// If the file does not exist, creates a default config file and returns defaults.
pub fn load_default() -> Result<JarvisConfig, ConfigError> {
    let path = default_config_path()?;

    match load_from_path(&path) {
        Ok(config) => Ok(config),
        Err(ConfigError::ParseError(msg)) if msg.contains("failed to read") => {
            info!("no config found at {}, creating default", path.display());
            create_default_config(&path)?;
            Ok(JarvisConfig::default())
        }
        Err(e) => Err(e),
    }
}
