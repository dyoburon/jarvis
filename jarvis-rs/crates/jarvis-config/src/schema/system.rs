//! System configuration types: updates, logging, and advanced settings.

use serde::{Deserialize, Serialize};

/// Update channel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum UpdateChannel {
    #[default]
    Stable,
    Beta,
}

/// Auto-update configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdatesConfig {
    pub check_automatically: bool,
    pub channel: UpdateChannel,
    /// Check interval in seconds (valid range: 3600-604800).
    pub check_interval: u32,
    pub auto_download: bool,
    pub auto_install: bool,
}

impl Default for UpdatesConfig {
    fn default() -> Self {
        Self {
            check_automatically: true,
            channel: UpdateChannel::Stable,
            check_interval: 86400,
            auto_download: false,
            auto_install: false,
        }
    }
}

/// Log level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
#[derive(Default)]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warning,
    Error,
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub file_logging: bool,
    pub max_file_size_mb: u32,
    pub backup_count: u32,
    pub redact_secrets: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            file_logging: true,
            max_file_size_mb: 5,
            backup_count: 3,
            redact_secrets: true,
        }
    }
}

/// Experimental features.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct ExperimentalConfig {
    pub web_rendering: bool,
    pub metal_debug: bool,
}

/// Developer options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct DeveloperConfig {
    pub show_fps: bool,
    pub show_debug_hud: bool,
    pub inspector_enabled: bool,
}

/// Advanced configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct AdvancedConfig {
    pub experimental: ExperimentalConfig,
    pub developer: DeveloperConfig,
}
