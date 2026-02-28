//! Write JarvisConfig to TOML on disk.
//!
//! Supports atomic writes (write to `.tmp`, then rename) to prevent
//! corruption if the process crashes mid-write.

use std::path::Path;

use jarvis_common::ConfigError;

use crate::schema::JarvisConfig;
use crate::toml_loader::default_config_path;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Write config to the platform default path (`~/.config/jarvis/config.toml`).
pub fn save_config(config: &JarvisConfig) -> Result<(), ConfigError> {
    let path = default_config_path()?;
    save_config_to_path(config, &path)
}

/// Write config to a specific path.
///
/// Creates parent directories if they don't exist. Uses atomic write
/// (write to `.tmp` file, then rename) to prevent partial writes.
pub fn save_config_to_path(config: &JarvisConfig, path: &Path) -> Result<(), ConfigError> {
    let toml_str = toml::to_string_pretty(config)
        .map_err(|e| ConfigError::ParseError(format!("failed to serialize config to TOML: {e}")))?;

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ConfigError::ParseError(format!(
                "failed to create config directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    // Atomic write: write to .tmp, then rename
    let tmp_path = path.with_extension("toml.tmp");
    std::fs::write(&tmp_path, &toml_str).map_err(|e| {
        ConfigError::ParseError(format!(
            "failed to write config to {}: {e}",
            tmp_path.display()
        ))
    })?;

    if let Err(e) = std::fs::rename(&tmp_path, path) {
        // Rename failed — try direct write as fallback (Windows compat)
        tracing::warn!("atomic rename failed ({}), falling back to direct write", e);
        std::fs::write(path, &toml_str).map_err(|e2| {
            ConfigError::ParseError(format!(
                "failed to write config to {}: {e2}",
                path.display()
            ))
        })?;
    }

    tracing::debug!(path = %path.display(), "Config saved to disk");
    Ok(())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn save_config_writes_valid_toml() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let config = JarvisConfig::default();
        save_config_to_path(&config, &path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let parsed: JarvisConfig = toml::from_str(&contents).unwrap();
        assert_eq!(parsed.theme.name, "jarvis-dark");
        assert_eq!(parsed.colors.primary, "#00d4ff");
    }

    #[test]
    fn save_config_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let config = JarvisConfig::default();
        save_config_to_path(&config, &path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let parsed: JarvisConfig = toml::from_str(&contents).unwrap();

        assert_eq!(parsed.theme.name, config.theme.name);
        assert_eq!(parsed.colors.primary, config.colors.primary);
        assert_eq!(parsed.colors.secondary, config.colors.secondary);
        assert_eq!(parsed.font.family, config.font.family);
        assert_eq!(parsed.font.size, config.font.size);
        assert_eq!(parsed.layout.panel_gap, config.layout.panel_gap);
        assert_eq!(parsed.auto_open.panels.len(), config.auto_open.panels.len());
    }

    #[test]
    fn save_config_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("deep").join("config.toml");

        let config = JarvisConfig::default();
        save_config_to_path(&config, &path).unwrap();

        assert!(path.exists());
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("jarvis-dark"));
    }

    #[test]
    fn save_config_preserves_auto_open() {
        use crate::schema::{AutoOpenPanel, PanelKind};

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let mut config = JarvisConfig::default();
        config.auto_open.panels = vec![
            AutoOpenPanel {
                kind: PanelKind::Terminal,
                command: Some("claude".into()),
                title: Some("Claude Code".into()),
                ..Default::default()
            },
            AutoOpenPanel {
                kind: PanelKind::Terminal,
                title: Some("Terminal".into()),
                ..Default::default()
            },
        ];

        save_config_to_path(&config, &path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let parsed: JarvisConfig = toml::from_str(&contents).unwrap();
        assert_eq!(parsed.auto_open.panels.len(), 2);
        assert_eq!(
            parsed.auto_open.panels[0].command.as_deref(),
            Some("claude")
        );
        assert_eq!(
            parsed.auto_open.panels[1].title.as_deref(),
            Some("Terminal")
        );
    }

    #[test]
    fn save_config_cleans_up_tmp_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let config = JarvisConfig::default();
        save_config_to_path(&config, &path).unwrap();

        let tmp_path = path.with_extension("toml.tmp");
        assert!(
            !tmp_path.exists(),
            "tmp file should be cleaned up after rename"
        );
    }
}
