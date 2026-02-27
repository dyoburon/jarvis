//! Typography configuration types.

use serde::{Deserialize, Serialize};

/// Typography configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FontConfig {
    pub family: String,
    /// Font size in points (valid range: 8-32).
    pub size: u32,
    /// Title font size in points (valid range: 8-48).
    pub title_size: u32,
    /// Line height multiplier (valid range: 1.0-3.0).
    pub line_height: f64,
    /// Override font family for bold text. `None` uses `family` with bold weight.
    pub bold_family: Option<String>,
    /// Override font family for italic text. `None` uses `family` with italic style.
    pub italic_family: Option<String>,
    /// Enable Nerd Font glyph rendering.
    pub nerd_font: bool,
    /// Enable font ligatures (e.g. `->`, `=>`, `!=`).
    pub ligatures: bool,
    /// Fallback font families tried in order when glyphs are missing.
    pub fallback_families: Vec<String>,
    /// Font weight for normal text (valid range: 100-900).
    pub font_weight: u32,
    /// Font weight for bold text (valid range: 100-900).
    pub bold_weight: u32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "Menlo".into(),
            size: 13,
            title_size: 15,
            line_height: 1.6,
            bold_family: None,
            italic_family: None,
            nerd_font: true,
            ligatures: false,
            fallback_families: Vec::new(),
            font_weight: 400,
            bold_weight: 700,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_config_defaults() {
        let config = FontConfig::default();
        assert_eq!(config.family, "Menlo");
        assert_eq!(config.size, 13);
        assert_eq!(config.title_size, 15);
        assert!((config.line_height - 1.6).abs() < f64::EPSILON);
        assert!(config.bold_family.is_none());
        assert!(config.italic_family.is_none());
        assert!(config.nerd_font);
        assert!(!config.ligatures);
        assert!(config.fallback_families.is_empty());
        assert_eq!(config.font_weight, 400);
        assert_eq!(config.bold_weight, 700);
    }

    #[test]
    fn font_config_partial_toml_preserves_new_defaults() {
        let toml_str = r#"
family = "SF Mono"
size = 14
ligatures = true
"#;
        let config: FontConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.family, "SF Mono");
        assert_eq!(config.size, 14);
        assert!(config.ligatures);
        // New defaults preserved
        assert!(config.nerd_font);
        assert_eq!(config.font_weight, 400);
        assert_eq!(config.bold_weight, 700);
        assert!(config.bold_family.is_none());
        // Old defaults preserved
        assert!((config.line_height - 1.6).abs() < f64::EPSILON);
        assert_eq!(config.title_size, 15);
    }

    #[test]
    fn font_config_with_overrides() {
        let toml_str = r#"
bold_family = "Menlo Bold"
italic_family = "Menlo Italic"
fallback_families = ["Symbols Nerd Font Mono", "Apple Color Emoji"]
font_weight = 300
bold_weight = 800
"#;
        let config: FontConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.bold_family.as_deref(), Some("Menlo Bold"));
        assert_eq!(config.italic_family.as_deref(), Some("Menlo Italic"));
        assert_eq!(config.fallback_families.len(), 2);
        assert_eq!(config.fallback_families[0], "Symbols Nerd Font Mono");
        assert_eq!(config.font_weight, 300);
        assert_eq!(config.bold_weight, 800);
    }
}
