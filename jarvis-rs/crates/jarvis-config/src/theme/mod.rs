//! Theme loading and merging.
//!
//! Themes are YAML files that override subsets of the config (colors, font, etc.).
//! Built-in themes are loaded from `resources/themes/` relative to the executable.

mod apply;
mod loader;
mod types;

pub use apply::apply_theme;
pub use loader::{load_theme, load_theme_from_path};
pub use types::{
    ThemeBackgroundOverrides, ThemeFontOverrides, ThemeOverrides, ThemeVisualizerOverrides,
    BUILT_IN_THEMES,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ColorConfig, JarvisConfig};

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
        assert_eq!(
            theme.font.as_ref().unwrap().family,
            Some("Fira Code".into())
        );
    }

    #[test]
    fn nonexistent_theme_returns_error() {
        let result = load_theme("definitely-not-a-real-theme-name");
        assert!(result.is_err());
    }
}
