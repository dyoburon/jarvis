//! Background display configuration types.

use serde::{Deserialize, Serialize};

/// Background display mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum BackgroundMode {
    #[default]
    HexGrid,
    Solid,
    Image,
    Video,
    Gradient,
    None,
}

/// Hex grid background settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HexGridConfig {
    pub color: String,
    pub opacity: f64,
    pub animation_speed: f64,
    pub glow_intensity: f64,
}

impl Default for HexGridConfig {
    fn default() -> Self {
        Self {
            color: "#00d4ff".into(),
            opacity: 0.08,
            animation_speed: 1.0,
            glow_intensity: 0.5,
        }
    }
}

/// Image fit mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ImageFit {
    #[default]
    Cover,
    Contain,
    Fill,
    Tile,
}

/// Image background settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ImageBackgroundConfig {
    pub path: String,
    pub fit: ImageFit,
    pub blur: u32,
    pub opacity: f64,
}

impl Default for ImageBackgroundConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            fit: ImageFit::Cover,
            blur: 0,
            opacity: 1.0,
        }
    }
}

/// Video fit mode (no tile variant).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum VideoFit {
    #[default]
    Cover,
    Contain,
    Fill,
}

/// Video background settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VideoBackgroundConfig {
    pub path: String,
    #[serde(rename = "loop")]
    pub loop_video: bool,
    pub muted: bool,
    pub fit: VideoFit,
}

impl Default for VideoBackgroundConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            loop_video: true,
            muted: true,
            fit: VideoFit::Cover,
        }
    }
}

/// Gradient type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum GradientType {
    Linear,
    #[default]
    Radial,
}

/// Gradient background settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GradientBackgroundConfig {
    #[serde(rename = "type")]
    pub gradient_type: GradientType,
    pub colors: Vec<String>,
    pub angle: u32,
}

impl Default for GradientBackgroundConfig {
    fn default() -> Self {
        Self {
            gradient_type: GradientType::Radial,
            colors: vec!["#000000".into(), "#0a1520".into()],
            angle: 180,
        }
    }
}

/// Background system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BackgroundConfig {
    pub mode: BackgroundMode,
    pub solid_color: String,
    pub image: ImageBackgroundConfig,
    pub video: VideoBackgroundConfig,
    pub gradient: GradientBackgroundConfig,
    pub hex_grid: HexGridConfig,
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            mode: BackgroundMode::HexGrid,
            solid_color: "#000000".into(),
            image: ImageBackgroundConfig::default(),
            video: VideoBackgroundConfig::default(),
            gradient: GradientBackgroundConfig::default(),
            hex_grid: HexGridConfig::default(),
        }
    }
}
