//! Config path resolution and default file creation.

use jarvis_common::ConfigError;
use std::path::Path;
use tracing::info;

use super::template::default_config_toml;

/// Get the platform-specific default config file path.
pub fn default_config_path() -> Result<std::path::PathBuf, ConfigError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ConfigError::ParseError("could not determine config directory".into()))?;
    Ok(config_dir.join("jarvis").join("config.toml"))
}

/// Create a default TOML config file with documentation comments.
pub fn create_default_config(path: &Path) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ConfigError::ParseError(format!(
                "failed to create config directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    let content = default_config_toml();

    std::fs::write(path, content).map_err(|e| {
        ConfigError::ParseError(format!(
            "failed to write default config to {}: {e}",
            path.display()
        ))
    })?;

    info!("created default config at {}", path.display());
    Ok(())
}
