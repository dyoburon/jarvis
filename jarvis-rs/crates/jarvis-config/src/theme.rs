//! Theme loading and merging.
//!
//! Themes are YAML files that override subsets of the config (colors, font, etc.).
//! Built-in themes are loaded from `resources/themes/` relative to the executable.

use crate::schema::{ColorConfig, JarvisConfig};
use jarvis_common::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

/// Built-in theme names.
pub const BUILT_IN_THEMES: &[&str] = &[
    "jarvis-dark",
    "jarvis-light",
    "catppuccin-mocha",
    "dracula",
    "gruvbox-dark",
    "nord",
    "solarized-dark",
    "tokyo-night",
];

/// Theme override structure.
///
/// All fields are optional; only present fields override the base config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeOverrides {
    pub name: Option<String>,
    pub colors: Option<ColorConfig>,
    pub font: Option<ThemeFontOverrides>,
    pub visualizer: Option<ThemeVisualizerOverrides>,
    pub background: Option<ThemeBackgroundOverrides>,
}

/// Optional font overrides in a theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeFontOverrides {
    pub family: Option<String>,
    pub size: Option<u32>,
    pub title_size: Option<u32>,
    pub line_height: Option<f64>,
}

/// Optional visualizer overrides in a theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeVisualizerOverrides {
    pub orb_color: Option<String>,
    pub orb_secondary_color: Option<String>,
}

/// Optional background overrides in a theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeBackgroundOverrides {
    pub hex_grid_color: Option<String>,
    pub solid_color: Option<String>,
}

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
        ConfigError::ParseError(format!(
            "failed to read theme file {}: {e}",
            path.display()
        ))
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

/// Apply theme overrides to a config, merging only the fields that are present.
pub fn apply_theme(config: &mut JarvisConfig, theme: &ThemeOverrides) {
    // Apply color overrides
    if let Some(ref colors) = theme.colors {
        apply_color_overrides(&mut config.colors, colors);
    }

    // Apply font overrides
    if let Some(ref font) = theme.font {
        if let Some(ref family) = font.family {
            config.font.family = family.clone();
        }
        if let Some(size) = font.size {
            config.font.size = size;
        }
        if let Some(title_size) = font.title_size {
            config.font.title_size = title_size;
        }
        if let Some(line_height) = font.line_height {
            config.font.line_height = line_height;
        }
    }

    // Apply visualizer overrides
    if let Some(ref viz) = theme.visualizer {
        if let Some(ref color) = viz.orb_color {
            config.visualizer.orb.color = color.clone();
        }
        if let Some(ref color) = viz.orb_secondary_color {
            config.visualizer.orb.secondary_color = color.clone();
        }
    }

    // Apply background overrides
    if let Some(ref bg) = theme.background {
        if let Some(ref color) = bg.hex_grid_color {
            config.background.hex_grid.color = color.clone();
        }
        if let Some(ref color) = bg.solid_color {
            config.background.solid_color = color.clone();
        }
    }
}

/// Replace color config fields with theme colors.
/// Since the theme provides a full ColorConfig via serde defaults, we only
/// override if the theme author actually specified values. We do this by
/// replacing the entire colors block when present.
fn apply_color_overrides(target: &mut ColorConfig, source: &ColorConfig) {
    target.primary = source.primary.clone();
    target.secondary = source.secondary.clone();
    target.background = source.background.clone();
    target.panel_bg = source.panel_bg.clone();
    target.text = source.text.clone();
    target.text_muted = source.text_muted.clone();
    target.border = source.border.clone();
    target.border_focused = source.border_focused.clone();
    target.user_text = source.user_text.clone();
    target.tool_read = source.tool_read.clone();
    target.tool_edit = source.tool_edit.clone();
    target.tool_write = source.tool_write.clone();
    target.tool_run = source.tool_run.clone();
    target.tool_search = source.tool_search.clone();
    target.success = source.success.clone();
    target.warning = source.warning.clone();
    target.error = source.error.clone();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_jarvis_dark_returns_default() {
        let theme = load_theme("jarvis-dark").unwrap();
        assert_eq!(theme.name, Some("jarvis-dark".into()));
        assert!(theme.colors.is_none());
    }

    #[test]
    fn built_in_themes_list_has_expected_entries() {
        assert!(BUILT_IN_THEMES.contains(&"jarvis-dark"));
        assert!(BUILT_IN_THEMES.contains(&"jarvis-light"));
        assert!(BUILT_IN_THEMES.contains(&"catppuccin-mocha"));
        assert!(BUILT_IN_THEMES.contains(&"dracula"));
        assert!(BUILT_IN_THEMES.contains(&"gruvbox-dark"));
        assert!(BUILT_IN_THEMES.contains(&"nord"));
        assert!(BUILT_IN_THEMES.contains(&"solarized-dark"));
        assert!(BUILT_IN_THEMES.contains(&"tokyo-night"));
        assert_eq!(BUILT_IN_THEMES.len(), 8);
    }

    #[test]
    fn apply_theme_with_color_overrides() {
        let mut config = JarvisConfig::default();
        let theme = ThemeOverrides {
            colors: Some(ColorConfig {
                primary: "#ff0000".into(),
                secondary: "#00ff00".into(),
                ..Default::default()
            }),
            ..Default::default()
        };

        apply_theme(&mut config, &theme);
        assert_eq!(config.colors.primary, "#ff0000");
        assert_eq!(config.colors.secondary, "#00ff00");
    }

    #[test]
    fn apply_theme_with_font_overrides() {
        let mut config = JarvisConfig::default();
        let theme = ThemeOverrides {
            font: Some(ThemeFontOverrides {
                family: Some("SF Mono".into()),
                size: Some(14),
                ..Default::default()
            }),
            ..Default::default()
        };

        apply_theme(&mut config, &theme);
        assert_eq!(config.font.family, "SF Mono");
        assert_eq!(config.font.size, 14);
        // line_height should be unchanged
        assert!((config.font.line_height - 1.6).abs() < f64::EPSILON);
    }

    #[test]
    fn apply_theme_with_visualizer_overrides() {
        let mut config = JarvisConfig::default();
        let theme = ThemeOverrides {
            visualizer: Some(ThemeVisualizerOverrides {
                orb_color: Some("#ff00ff".into()),
                orb_secondary_color: None,
            }),
            ..Default::default()
        };

        apply_theme(&mut config, &theme);
        assert_eq!(config.visualizer.orb.color, "#ff00ff");
        assert_eq!(config.visualizer.orb.secondary_color, "#0088aa"); // unchanged
    }

    #[test]
    fn apply_theme_with_background_overrides() {
        let mut config = JarvisConfig::default();
        let theme = ThemeOverrides {
            background: Some(ThemeBackgroundOverrides {
                hex_grid_color: Some("#ff0000".into()),
                solid_color: Some("#111111".into()),
            }),
            ..Default::default()
        };

        apply_theme(&mut config, &theme);
        assert_eq!(config.background.hex_grid.color, "#ff0000");
        assert_eq!(config.background.solid_color, "#111111");
    }

    #[test]
    fn apply_empty_theme_changes_nothing() {
        let original = JarvisConfig::default();
        let mut config = JarvisConfig::default();
        let theme = ThemeOverrides::default();

        apply_theme(&mut config, &theme);
        assert_eq!(config.colors.primary, original.colors.primary);
        assert_eq!(config.font.family, original.font.family);
        assert_eq!(config.visualizer.orb.color, original.visualizer.orb.color);
    }

    #[test]
    fn load_theme_from_yaml_string() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test-theme.yaml");
        std::fs::write(
            &path,
            r##"
name: test-theme
colors:
  primary: "#ff00ff"
  background: "#111111"
font:
  family: "Fira Code"
"##,
        )
        .unwrap();

        let theme = load_theme_from_path(&path).unwrap();
        assert_eq!(theme.name, Some("test-theme".into()));
        assert!(theme.colors.is_some());
        assert_eq!(theme.colors.as_ref().unwrap().primary, "#ff00ff");
        assert_eq!(theme.font.as_ref().unwrap().family, Some("Fira Code".into()));
    }

    #[test]
    fn nonexistent_theme_returns_error() {
        let result = load_theme("definitely-not-a-real-theme-name");
        assert!(result.is_err());
    }
}
