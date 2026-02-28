//! Startup sequence configuration types.

use serde::{Deserialize, Serialize};

/// Boot animation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BootAnimationConfig {
    pub enabled: bool,
    pub duration: f64,
    pub skip_on_key: bool,
    pub music_enabled: bool,
    pub voiceover_enabled: bool,
}

impl Default for BootAnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 4.5,
            skip_on_key: true,
            music_enabled: true,
            voiceover_enabled: true,
        }
    }
}

/// Fast-start mode settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FastStartConfig {
    pub enabled: bool,
    pub delay: f64,
}

impl Default for FastStartConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            delay: 0.5,
        }
    }
}

/// Panel action configuration for on_ready.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelActionConfig {
    pub count: u32,
    pub titles: Vec<String>,
    pub auto_create: bool,
}

impl Default for PanelActionConfig {
    fn default() -> Self {
        Self {
            count: 1,
            titles: vec!["Bench 1".into()],
            auto_create: true,
        }
    }
}

/// Chat action configuration for on_ready.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatActionConfig {
    pub room: String,
}

impl Default for ChatActionConfig {
    fn default() -> Self {
        Self {
            room: "general".into(),
        }
    }
}

/// Game action configuration for on_ready.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameActionConfig {
    pub name: String,
}

impl Default for GameActionConfig {
    fn default() -> Self {
        Self {
            name: "wordle".into(),
        }
    }
}

/// Skill action configuration for on_ready.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SkillActionConfig {
    pub name: String,
}

impl Default for SkillActionConfig {
    fn default() -> Self {
        Self {
            name: "code_assistant".into(),
        }
    }
}

/// On-ready action type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum OnReadyAction {
    #[default]
    Listening,
    Panels,
    Chat,
    Game,
    Skill,
}

/// What to show after boot/skip.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OnReadyConfig {
    pub action: OnReadyAction,
    pub panels: PanelActionConfig,
    pub chat: ChatActionConfig,
    pub game: GameActionConfig,
    pub skill: SkillActionConfig,
}

impl Default for OnReadyConfig {
    fn default() -> Self {
        Self {
            action: OnReadyAction::Listening,
            panels: PanelActionConfig::default(),
            chat: ChatActionConfig::default(),
            game: GameActionConfig::default(),
            skill: SkillActionConfig::default(),
        }
    }
}

/// Startup sequence configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct StartupConfig {
    pub boot_animation: BootAnimationConfig,
    pub fast_start: FastStartConfig,
    pub on_ready: OnReadyConfig,
}
