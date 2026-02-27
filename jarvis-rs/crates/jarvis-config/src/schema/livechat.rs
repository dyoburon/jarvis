//! Livechat configuration types.

use serde::{Deserialize, Serialize};

/// Nickname validation rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NicknameValidationConfig {
    pub min_length: u32,
    pub max_length: u32,
    pub pattern: String,
}

impl Default for NicknameValidationConfig {
    fn default() -> Self {
        Self {
            min_length: 1,
            max_length: 20,
            pattern: r"^[a-zA-Z0-9_\- ]+$".into(),
        }
    }
}

/// Nickname settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NicknameConfig {
    pub default: String,
    pub persist: bool,
    pub allow_change: bool,
    pub validation: NicknameValidationConfig,
}

impl Default for NicknameConfig {
    fn default() -> Self {
        Self {
            default: String::new(),
            persist: true,
            allow_change: true,
            validation: NicknameValidationConfig::default(),
        }
    }
}

/// Auto-moderation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoModConfig {
    pub enabled: bool,
    pub filter_profanity: bool,
    pub rate_limit: u32,
    pub max_message_length: u32,
    pub spam_detection: bool,
}

impl Default for AutoModConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            filter_profanity: true,
            rate_limit: 5,
            max_message_length: 500,
            spam_detection: true,
        }
    }
}

/// Livechat configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LivechatConfig {
    pub enabled: bool,
    pub server_port: u32,
    pub connection_timeout: u32,
    pub nickname: NicknameConfig,
    pub automod: AutoModConfig,
}

impl Default for LivechatConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            server_port: 19847,
            connection_timeout: 10,
            nickname: NicknameConfig::default(),
            automod: AutoModConfig::default(),
        }
    }
}
