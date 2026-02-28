//! Jarvis configuration system.
//!
//! Provides TOML-based configuration with theme support, live reload,
//! and full validation. All config sections use sensible defaults so
//! partial configs work out of the box.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use jarvis_config::{load_config, config_to_json};
//!
//! let config = load_config().expect("failed to load config");
//! let json = config_to_json(&config);
//! println!("{json}");
//! ```

pub mod colors;
pub mod keybinds;
pub mod reload;
pub mod schema;
pub mod theme;
pub mod toml_loader;
pub mod toml_writer;
pub mod validation;
pub mod watcher;

// Re-export core types for convenience
pub use reload::ReloadManager;
pub use schema::{JarvisConfig, CONFIG_SCHEMA_VERSION};
pub use theme::{ThemeOverrides, BUILT_IN_THEMES};
pub use toml_writer::{save_config, save_config_to_path};
pub use watcher::ConfigWatcher;

use jarvis_common::ConfigError;

/// Convenience function to load config from the platform default path.
///
/// Loads `config.toml` from the OS config directory, creates a default
/// if none exists, applies the selected theme, and validates the result.
pub fn load_config() -> Result<JarvisConfig, ConfigError> {
    let mut config = toml_loader::load_default()?;

    // Apply theme if not the default
    if config.theme.name != "jarvis-dark" {
        match theme::load_theme(&config.theme.name) {
            Ok(overrides) => theme::apply_theme(&mut config, &overrides),
            Err(e) => {
                tracing::warn!("failed to load theme '{}': {e}", config.theme.name);
            }
        }
    }

    validation::validate(&config)?;
    Ok(config)
}

/// Serialize a config to a pretty-printed JSON string.
pub fn config_to_json(config: &JarvisConfig) -> String {
    serde_json::to_string_pretty(config)
        .unwrap_or_else(|e| format!("{{\"error\": \"failed to serialize config: {e}\"}}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_to_json_contains_theme() {
        let config = JarvisConfig::default();
        let json = config_to_json(&config);
        assert!(json.contains("\"jarvis-dark\""));
        assert!(json.contains("\"theme\""));
    }

    #[test]
    fn config_to_json_contains_all_sections() {
        let config = JarvisConfig::default();
        let json = config_to_json(&config);
        assert!(json.contains("\"colors\""));
        assert!(json.contains("\"font\""));
        assert!(json.contains("\"layout\""));
        assert!(json.contains("\"opacity\""));
        assert!(json.contains("\"background\""));
        assert!(json.contains("\"visualizer\""));
        assert!(json.contains("\"startup\""));
        assert!(json.contains("\"voice\""));
        assert!(json.contains("\"keybinds\""));
        assert!(json.contains("\"panels\""));
        assert!(json.contains("\"games\""));
        assert!(json.contains("\"livechat\""));
        assert!(json.contains("\"presence\""));
        assert!(json.contains("\"performance\""));
        assert!(json.contains("\"updates\""));
        assert!(json.contains("\"logging\""));
        assert!(json.contains("\"advanced\""));
        assert!(json.contains("\"auto_open\""));
    }

    #[test]
    fn config_schema_version_is_1() {
        assert_eq!(CONFIG_SCHEMA_VERSION, 1);
    }

    #[test]
    fn default_config_round_trips_through_json() {
        let config = JarvisConfig::default();
        let json = config_to_json(&config);
        let parsed: JarvisConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.theme.name, "jarvis-dark");
        assert_eq!(parsed.colors.primary, "#cba6f7");
        assert_eq!(parsed.font.size, 13);
    }
}
