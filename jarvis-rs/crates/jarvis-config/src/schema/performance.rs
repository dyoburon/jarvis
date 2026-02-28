//! Performance configuration types.

use serde::{Deserialize, Serialize};

/// Performance quality preset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum PerformancePreset {
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

/// Orb rendering quality.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum OrbQuality {
    Low,
    Medium,
    #[default]
    High,
}

/// Preload settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PreloadConfig {
    pub themes: bool,
    pub games: bool,
    pub fonts: bool,
}

impl Default for PreloadConfig {
    fn default() -> Self {
        Self {
            themes: true,
            games: false,
            fonts: true,
        }
    }
}

/// Performance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PerformanceConfig {
    pub preset: PerformancePreset,
    pub frame_rate: u32,
    pub orb_quality: OrbQuality,
    pub bloom_passes: u32,
    pub preload: PreloadConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            preset: PerformancePreset::High,
            frame_rate: 60,
            orb_quality: OrbQuality::High,
            bloom_passes: 2,
            preload: PreloadConfig::default(),
        }
    }
}
