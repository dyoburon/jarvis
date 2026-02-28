//! Theme file resolution and loading.
//!
//! Resolves theme names to filesystem paths and parses YAML theme files
//! into [`ThemeOverrides`].

use super::types::ThemeOverrides;
use jarvis_common::ConfigError;
use std::path::{Path, PathBuf};
use tracing::info;

/// Theme file extensions to search for, in priority order.
const THEME_EXTENSIONS: &[&str] = &["toml", "yaml", "yml"];

/// Resolve the filesystem path for a theme by name.
///
/// Built-in themes are looked up in `resources/themes/` relative to the
/// executable directory. If the name looks like a file path (contains `/`
/// or ends in a known extension), it is used directly.
fn resolve_theme_path(name: &str) -> Result<PathBuf, ConfigError> {
    // If the name looks like a direct path, use it as-is
    let is_path = name.contains('/')
        || name.ends_with(".yaml")
        || name.ends_with(".yml")
        || name.ends_with(".toml");
    if is_path {
        let path = PathBuf::from(name);
        if path.exists() {
            return Ok(path);
        }
        return Err(ConfigError::FileNotFound(path));
    }

    // Search directories in priority order
    let search_dirs = search_directories();

    for dir in &search_dirs {
        for ext in THEME_EXTENSIONS {
            let theme_path = dir.join(format!("{name}.{ext}"));
            if theme_path.exists() {
                return Ok(theme_path);
            }
        }
    }

    Err(ConfigError::FileNotFound(PathBuf::from(format!(
        "theme '{name}' not found in any search path"
    ))))
}

/// Collect theme search directories in priority order.
fn search_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // 1. Config directory (~/.config/jarvis/themes/)
    if let Some(config_dir) = dirs::config_dir() {
        dirs.push(config_dir.join("jarvis").join("themes"));
    }

    // 2. Relative to executable (resources/themes/)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            dirs.push(exe_dir.join("resources").join("themes"));
        }
    }

    // 3. Relative to current working directory
    dirs.push(PathBuf::from("resources").join("themes"));

    dirs
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
///
/// Detects format by extension: `.toml` → TOML, `.yaml`/`.yml` → YAML.
pub fn load_theme_from_path(path: &Path) -> Result<ThemeOverrides, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::FileNotFound(path.to_path_buf()));
    }

    let content = std::fs::read_to_string(path).map_err(|e| {
        ConfigError::ParseError(format!("failed to read theme file {}: {e}", path.display()))
    })?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("yaml");

    let theme: ThemeOverrides = match ext {
        "toml" => toml::from_str(&content).map_err(|e| {
            ConfigError::ParseError(format!(
                "failed to parse theme TOML {}: {e}",
                path.display()
            ))
        })?,
        _ => serde_yaml::from_str(&content).map_err(|e| {
            ConfigError::ParseError(format!(
                "failed to parse theme YAML {}: {e}",
                path.display()
            ))
        })?,
    };

    info!("loaded theme from {}", path.display());
    Ok(theme)
}
