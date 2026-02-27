//! Games configuration types.

use serde::{Deserialize, Serialize};

/// Enabled games configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GamesEnabledConfig {
    pub wordle: bool,
    pub connections: bool,
    pub asteroids: bool,
    pub tetris: bool,
    pub pinball: bool,
    pub doodlejump: bool,
    pub minesweeper: bool,
    pub draw: bool,
    pub subway: bool,
    pub videoplayer: bool,
}

impl Default for GamesEnabledConfig {
    fn default() -> Self {
        Self {
            wordle: true,
            connections: true,
            asteroids: true,
            tetris: true,
            pinball: true,
            doodlejump: true,
            minesweeper: true,
            draw: true,
            subway: true,
            videoplayer: true,
        }
    }
}

/// Game fullscreen settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FullscreenConfig {
    pub keyboard_passthrough: bool,
    pub escape_to_exit: bool,
}

impl Default for FullscreenConfig {
    fn default() -> Self {
        Self {
            keyboard_passthrough: true,
            escape_to_exit: true,
        }
    }
}

/// Custom game definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGameConfig {
    pub name: String,
    pub path: String,
}

/// Games configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct GamesConfig {
    pub enabled: GamesEnabledConfig,
    pub fullscreen: FullscreenConfig,
    pub custom_paths: Vec<CustomGameConfig>,
}
