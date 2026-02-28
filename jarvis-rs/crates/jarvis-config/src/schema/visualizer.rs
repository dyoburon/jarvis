//! Visualizer system configuration types.

use serde::{Deserialize, Serialize};

use super::background::VideoFit;

/// Visualizer type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum VisualizerType {
    #[default]
    Orb,
    Image,
    Video,
    Particle,
    Waveform,
    None,
}

/// Orb mesh detail level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum MeshDetail {
    Low,
    Medium,
    #[default]
    High,
}

/// Orb visualizer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OrbVisualizerConfig {
    pub color: String,
    pub secondary_color: String,
    pub intensity_base: f64,
    pub bloom_intensity: f64,
    pub rotation_speed: f64,
    pub mesh_detail: MeshDetail,
    pub wireframe: bool,
    pub inner_core: bool,
    pub outer_shell: bool,
}

impl Default for OrbVisualizerConfig {
    fn default() -> Self {
        Self {
            color: "#00d4ff".into(),
            secondary_color: "#0088aa".into(),
            intensity_base: 1.0,
            bloom_intensity: 1.0,
            rotation_speed: 1.0,
            mesh_detail: MeshDetail::High,
            wireframe: false,
            inner_core: true,
            outer_shell: true,
        }
    }
}

/// Image visualizer animation style.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ImageAnimation {
    #[default]
    None,
    Pulse,
    Rotate,
    Bounce,
    Float,
}

/// Image visualizer fit mode (no tile).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum VisualizerFit {
    #[default]
    Contain,
    Cover,
    Fill,
}

/// Image visualizer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ImageVisualizerConfig {
    pub path: String,
    pub fit: VisualizerFit,
    pub opacity: f64,
    pub animation: ImageAnimation,
    pub animation_speed: f64,
}

impl Default for ImageVisualizerConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            fit: VisualizerFit::Contain,
            opacity: 1.0,
            animation: ImageAnimation::None,
            animation_speed: 1.0,
        }
    }
}

/// Video visualizer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VideoVisualizerConfig {
    pub path: String,
    #[serde(rename = "loop")]
    pub loop_video: bool,
    pub muted: bool,
    pub fit: VideoFit,
    pub opacity: f64,
    pub sync_to_audio: bool,
}

impl Default for VideoVisualizerConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            loop_video: true,
            muted: true,
            fit: VideoFit::Cover,
            opacity: 1.0,
            sync_to_audio: false,
        }
    }
}

/// Particle effect style.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ParticleStyle {
    #[default]
    Swirl,
    Fountain,
    Fire,
    Snow,
    Stars,
    Custom,
}

/// Particle visualizer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ParticleVisualizerConfig {
    pub style: ParticleStyle,
    pub count: u32,
    pub color: String,
    pub size: f64,
    pub speed: f64,
    pub lifetime: f64,
    pub custom_shader: String,
}

impl Default for ParticleVisualizerConfig {
    fn default() -> Self {
        Self {
            style: ParticleStyle::Swirl,
            count: 500,
            color: "#00d4ff".into(),
            size: 2.0,
            speed: 1.0,
            lifetime: 3.0,
            custom_shader: String::new(),
        }
    }
}

/// Waveform display style.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum WaveformStyle {
    #[default]
    Bars,
    Line,
    Circular,
    Mirror,
}

/// Waveform visualizer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WaveformVisualizerConfig {
    pub style: WaveformStyle,
    pub color: String,
    pub bar_count: u32,
    pub bar_width: f64,
    pub bar_gap: f64,
    pub height: u32,
    pub smoothing: f64,
}

impl Default for WaveformVisualizerConfig {
    fn default() -> Self {
        Self {
            style: WaveformStyle::Bars,
            color: "#00d4ff".into(),
            bar_count: 64,
            bar_width: 3.0,
            bar_gap: 2.0,
            height: 100,
            smoothing: 0.8,
        }
    }
}

/// Visualizer anchor position.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum VisualizerAnchor {
    #[default]
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Per-state visualizer overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualizerStateConfig {
    pub scale: f64,
    pub intensity: f64,
    pub color: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

impl Default for VisualizerStateConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            intensity: 1.0,
            color: None,
            position_x: None,
            position_y: None,
        }
    }
}

impl VisualizerStateConfig {
    /// State config for "speaking" state.
    pub fn speaking() -> Self {
        Self {
            scale: 1.1,
            intensity: 1.4,
            ..Default::default()
        }
    }

    /// State config for "skill" state.
    pub fn skill() -> Self {
        Self {
            scale: 0.9,
            intensity: 1.2,
            color: Some("#ffaa00".into()),
            ..Default::default()
        }
    }

    /// State config for "chat" state.
    pub fn chat() -> Self {
        Self {
            scale: 0.55,
            intensity: 1.3,
            position_x: Some(0.10),
            position_y: Some(0.30),
            ..Default::default()
        }
    }

    /// State config for "idle" state.
    pub fn idle() -> Self {
        Self {
            scale: 0.8,
            intensity: 0.6,
            color: Some("#444444".into()),
            ..Default::default()
        }
    }
}

/// Visualizer system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualizerConfig {
    pub enabled: bool,
    #[serde(rename = "type")]
    pub visualizer_type: VisualizerType,
    pub position_x: f64,
    pub position_y: f64,
    pub scale: f64,
    pub anchor: VisualizerAnchor,
    pub react_to_audio: bool,
    pub react_to_state: bool,
    pub orb: OrbVisualizerConfig,
    pub image: ImageVisualizerConfig,
    pub video: VideoVisualizerConfig,
    pub particle: ParticleVisualizerConfig,
    pub waveform: WaveformVisualizerConfig,
    pub state_listening: VisualizerStateConfig,
    pub state_speaking: VisualizerStateConfig,
    pub state_skill: VisualizerStateConfig,
    pub state_chat: VisualizerStateConfig,
    pub state_idle: VisualizerStateConfig,
}

impl Default for VisualizerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            visualizer_type: VisualizerType::Orb,
            position_x: 0.0,
            position_y: 0.0,
            scale: 1.0,
            anchor: VisualizerAnchor::Center,
            react_to_audio: true,
            react_to_state: true,
            orb: OrbVisualizerConfig::default(),
            image: ImageVisualizerConfig::default(),
            video: VideoVisualizerConfig::default(),
            particle: ParticleVisualizerConfig::default(),
            waveform: WaveformVisualizerConfig::default(),
            state_listening: VisualizerStateConfig::default(),
            state_speaking: VisualizerStateConfig::speaking(),
            state_skill: VisualizerStateConfig::skill(),
            state_chat: VisualizerStateConfig::chat(),
            state_idle: VisualizerStateConfig::idle(),
        }
    }
}
