//! Theme file resolution and loading.
//!
//! Resolves theme names to filesystem paths and parses YAML theme files
//! into [`ThemeOverrides`].

use super::types::ThemeOverrides;
use jarvis_common::ConfigError;
use std::path::{Path, PathBuf};
use tracing::info;

/// Resolve the filesystem path for a theme by name.
///
/// Built-in themes are looked up in `resources/themes/` relative to the
/// executable directory. If the name looks like a file path (contains `/`
/// or ends in `.yaml`/`.yml`), it is used directly.
fn resolve_theme_path(name: &str) -> Result<PathBuf, ConfigError> {
    // If the name looks like a direct path, use it as-is
    if name.contains('/') || name.ends_with(".yaml") || name.ends_with(".yml") {
        let path = PathBuf::from(name);
        if path.exists() {
            return Ok(path);
        }
        return Err(ConfigError::FileNotFound(path));
    }

    // Look for built-in themes relative to the executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let theme_path = exe_dir
                .join("resources")
                .join("themes")
                .join(format!("{name}.yaml"));
            if theme_path.exists() {
                return Ok(theme_path);
            }
        }
    }

    // Also try relative to the current working directory
    let local_path = PathBuf::from("resources")
        .join("themes")
        .join(format!("{name}.yaml"));
    if local_path.exists() {
        return Ok(local_path);
    }

    // Try config directory
    if let Some(config_dir) = dirs::config_dir() {
        let config_theme = config_dir
            .join("jarvis")
            .join("themes")
            .join(format!("{name}.yaml"));
        if config_theme.exists() {
            return Ok(config_theme);
        }
    }

    Err(ConfigError::FileNotFound(PathBuf::from(format!(
        "theme '{name}' not found in any search path"
    ))))
}

/// Load a theme from a YAML file by name.
///
/// Returns the parsed theme overrides. If the theme file is not found,
/// returns an error. The special name "jarvis-dark" always succeeds
/// (returns empty overrides since it is the default).
pub fn load_theme(name: &str) -> Result<ThemeOverrides, ConfigError> {
    // jarvis-dark is the default; no overrides needed
    if name == "jarvis-dark" {
        return Ok(ThemeOverrides {
            name: Some("jarvis-dark".into()),
            ..Default::default()
        });
    }

    let path = resolve_theme_path(name)?;
    load_theme_from_path(&path)
}

/// Load a theme from a specific filesystem path.
pub fn load_theme_from_path(path: &Path) -> Result<ThemeOverrides, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::FileNotFound(path.to_path_buf()));
    }

    let content = std::fs::read_to_string(path).map_err(|e| {
        ConfigError::ParseError(format!("failed to read theme file {}: {e}", path.display()))
    })?;

    let theme: ThemeOverrides = serde_yaml::from_str(&content).map_err(|e| {
        ConfigError::ParseError(format!(
            "failed to parse theme YAML {}: {e}",
            path.display()
        ))
    })?;

    info!("loaded theme from {}", path.display());
    Ok(theme)
}
