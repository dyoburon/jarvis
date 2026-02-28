//! Voice and audio configuration types.

use serde::{Deserialize, Serialize};

/// Voice input mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum VoiceMode {
    #[default]
    Ptt,
    Vad,
}

/// Push-to-talk settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PTTConfig {
    pub key: String,
    pub cooldown: f64,
}

impl Default for PTTConfig {
    fn default() -> Self {
        Self {
            key: "Option+Period".into(),
            cooldown: 0.3,
        }
    }
}

/// Voice-activity detection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VADConfig {
    pub silence_threshold: f64,
    pub energy_threshold: u32,
}

impl Default for VADConfig {
    fn default() -> Self {
        Self {
            silence_threshold: 1.0,
            energy_threshold: 300,
        }
    }
}

/// Voice feedback sounds settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VoiceSoundsConfig {
    pub enabled: bool,
    pub volume: f64,
    pub listen_start: bool,
    pub listen_end: bool,
}

impl Default for VoiceSoundsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.5,
            listen_start: true,
            listen_end: true,
        }
    }
}

/// Voice and audio configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VoiceConfig {
    pub enabled: bool,
    pub mode: VoiceMode,
    pub ptt: PTTConfig,
    pub vad: VADConfig,
    pub input_device: String,
    pub sample_rate: u32,
    pub whisper_sample_rate: u32,
    pub sounds: VoiceSoundsConfig,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: VoiceMode::Ptt,
            ptt: PTTConfig::default(),
            vad: VADConfig::default(),
            input_device: "default".into(),
            sample_rate: 24000,
            whisper_sample_rate: 16000,
            sounds: VoiceSoundsConfig::default(),
        }
    }
}
