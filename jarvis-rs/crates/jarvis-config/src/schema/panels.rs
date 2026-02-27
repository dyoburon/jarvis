//! Panel configuration types.

use serde::{Deserialize, Serialize};

/// Panel history persistence settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryConfig {
    pub enabled: bool,
    pub max_messages: u32,
    pub restore_on_launch: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_messages: 1000,
            restore_on_launch: true,
        }
    }
}

/// Panel input behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    pub multiline: bool,
    pub auto_grow: bool,
    pub max_height: u32,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            multiline: true,
            auto_grow: true,
            max_height: 300,
        }
    }
}

/// Panel focus behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FocusConfig {
    pub restore_on_activate: bool,
    pub show_indicator: bool,
    pub border_glow: bool,
}

impl Default for FocusConfig {
    fn default() -> Self {
        Self {
            restore_on_activate: true,
            show_indicator: true,
            border_glow: true,
        }
    }
}

/// Panel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct PanelsConfig {
    pub history: HistoryConfig,
    pub input: InputConfig,
    pub focus: FocusConfig,
}
