//! Configuration schema types for Jarvis.
//!
//! All structs use `serde(default)` so partial configs work correctly.
//! Missing fields are filled with sensible defaults matching the Python schema.

use serde::{Deserialize, Serialize};

/// Current config schema version.
pub const CONFIG_SCHEMA_VERSION: u32 = 1;

// =============================================================================
// Theme Config
// =============================================================================

/// Theme selection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    /// Built-in theme name or path to custom theme YAML.
    pub name: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "jarvis-dark".into(),
        }
    }
}

// =============================================================================
// Color Config
// =============================================================================

/// Color palette configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorConfig {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub panel_bg: String,
    pub text: String,
    pub text_muted: String,
    pub border: String,
    pub border_focused: String,
    pub user_text: String,
    pub tool_read: String,
    pub tool_edit: String,
    pub tool_write: String,
    pub tool_run: String,
    pub tool_search: String,
    pub success: String,
    pub warning: String,
    pub error: String,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            primary: "#00d4ff".into(),
            secondary: "#ff6b00".into(),
            background: "#000000".into(),
            panel_bg: "rgba(0,0,0,0.93)".into(),
            text: "#f0ece4".into(),
            text_muted: "#888888".into(),
            border: "rgba(0,212,255,0.12)".into(),
            border_focused: "rgba(0,212,255,0.5)".into(),
            user_text: "rgba(140,190,220,0.65)".into(),
            tool_read: "rgba(100,180,255,0.9)".into(),
            tool_edit: "rgba(255,180,80,0.9)".into(),
            tool_write: "rgba(255,180,80,0.9)".into(),
            tool_run: "rgba(80,220,120,0.9)".into(),
            tool_search: "rgba(200,150,255,0.9)".into(),
            success: "#00ff88".into(),
            warning: "#ff6b00".into(),
            error: "#ff4444".into(),
        }
    }
}

// =============================================================================
// Font Config
// =============================================================================

/// Typography configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FontConfig {
    pub family: String,
    /// Font size in points (valid range: 8-32).
    pub size: u32,
    /// Title font size in points (valid range: 8-48).
    pub title_size: u32,
    /// Line height multiplier (valid range: 1.0-3.0).
    pub line_height: f64,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "Menlo".into(),
            size: 13,
            title_size: 15,
            line_height: 1.6,
        }
    }
}

// =============================================================================
// Layout Config
// =============================================================================

/// Panel layout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    /// Gap between panels in pixels (valid range: 0-20).
    pub panel_gap: u32,
    /// Border radius in pixels (valid range: 0-20).
    pub border_radius: u32,
    /// Padding in pixels (valid range: 0-40).
    pub padding: u32,
    /// Maximum number of panels (valid range: 1-10).
    pub max_panels: u32,
    /// Default panel width as fraction of screen (valid range: 0.3-1.0).
    pub default_panel_width: f64,
    /// Scrollbar width in pixels (valid range: 1-10).
    pub scrollbar_width: u32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            panel_gap: 2,
            border_radius: 4,
            padding: 14,
            max_panels: 5,
            default_panel_width: 0.72,
            scrollbar_width: 3,
        }
    }
}

// =============================================================================
// Opacity Config
// =============================================================================

/// Transparency settings (all values in range 0.0-1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OpacityConfig {
    pub background: f64,
    pub panel: f64,
    pub orb: f64,
    pub hex_grid: f64,
    pub hud: f64,
}

impl Default for OpacityConfig {
    fn default() -> Self {
        Self {
            background: 1.0,
            panel: 0.93,
            orb: 1.0,
            hex_grid: 0.8,
            hud: 1.0,
        }
    }
}

// =============================================================================
// Background Config
// =============================================================================

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

// =============================================================================
// Visualizer Config
// =============================================================================

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

// =============================================================================
// Startup Config
// =============================================================================

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
            duration: 27.0,
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

// =============================================================================
// Voice Config
// =============================================================================

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

// =============================================================================
// Keybind Config
// =============================================================================

/// Keyboard shortcuts configuration.
///
/// Format: "Modifier+Key" where Modifier is one of: Cmd, Option, Control, Shift.
/// Multiple modifiers: "Cmd+Shift+G".
/// Double press: "Escape+Escape".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindConfig {
    pub push_to_talk: String,
    pub open_assistant: String,
    pub new_panel: String,
    pub close_panel: String,
    pub toggle_fullscreen: String,
    pub open_settings: String,
    pub focus_panel_1: String,
    pub focus_panel_2: String,
    pub focus_panel_3: String,
    pub focus_panel_4: String,
    pub focus_panel_5: String,
    pub cycle_panels: String,
    pub cycle_panels_reverse: String,
    pub split_vertical: String,
    pub split_horizontal: String,
    pub close_pane: String,
    pub command_palette: String,
}

impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            push_to_talk: "Option+Period".into(),
            open_assistant: "Cmd+G".into(),
            new_panel: "Cmd+T".into(),
            close_panel: "Escape+Escape".into(),
            toggle_fullscreen: "Cmd+F".into(),
            open_settings: "Cmd+,".into(),
            focus_panel_1: "Cmd+1".into(),
            focus_panel_2: "Cmd+2".into(),
            focus_panel_3: "Cmd+3".into(),
            focus_panel_4: "Cmd+4".into(),
            focus_panel_5: "Cmd+5".into(),
            cycle_panels: "Tab".into(),
            cycle_panels_reverse: "Shift+Tab".into(),
            split_vertical: "Cmd+D".into(),
            split_horizontal: "Cmd+Shift+D".into(),
            close_pane: "Cmd+W".into(),
            command_palette: "Cmd+P".into(),
        }
    }
}

// =============================================================================
// Panels Config
// =============================================================================

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

// =============================================================================
// Games Config
// =============================================================================

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

// =============================================================================
// Livechat Config
// =============================================================================

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

// =============================================================================
// Presence Config
// =============================================================================

/// Presence system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PresenceConfig {
    pub enabled: bool,
    pub server_url: String,
    pub heartbeat_interval: u32,
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            server_url: String::new(),
            heartbeat_interval: 30,
        }
    }
}

// =============================================================================
// Performance Config
// =============================================================================

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

// =============================================================================
// Updates Config
// =============================================================================

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

// =============================================================================
// Logging Config
// =============================================================================

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

// =============================================================================
// Advanced Config
// =============================================================================

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

// =============================================================================
// Root Config
// =============================================================================

/// Root configuration for Jarvis.
///
/// All options have sensible defaults matching current behavior.
/// Only override what you want to change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct JarvisConfig {
    pub theme: ThemeConfig,
    pub colors: ColorConfig,
    pub font: FontConfig,
    pub layout: LayoutConfig,
    pub opacity: OpacityConfig,
    pub background: BackgroundConfig,
    pub visualizer: VisualizerConfig,
    pub startup: StartupConfig,
    pub voice: VoiceConfig,
    pub keybinds: KeybindConfig,
    pub panels: PanelsConfig,
    pub games: GamesConfig,
    pub livechat: LivechatConfig,
    pub presence: PresenceConfig,
    pub performance: PerformanceConfig,
    pub updates: UpdatesConfig,
    pub logging: LoggingConfig,
    pub advanced: AdvancedConfig,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_correct_theme() {
        let config = JarvisConfig::default();
        assert_eq!(config.theme.name, "jarvis-dark");
    }

    #[test]
    fn default_config_has_correct_colors() {
        let config = JarvisConfig::default();
        assert_eq!(config.colors.primary, "#00d4ff");
        assert_eq!(config.colors.secondary, "#ff6b00");
        assert_eq!(config.colors.background, "#000000");
        assert_eq!(config.colors.panel_bg, "rgba(0,0,0,0.93)");
        assert_eq!(config.colors.text, "#f0ece4");
        assert_eq!(config.colors.text_muted, "#888888");
        assert_eq!(config.colors.border, "rgba(0,212,255,0.12)");
        assert_eq!(config.colors.border_focused, "rgba(0,212,255,0.5)");
        assert_eq!(config.colors.user_text, "rgba(140,190,220,0.65)");
        assert_eq!(config.colors.tool_read, "rgba(100,180,255,0.9)");
        assert_eq!(config.colors.tool_edit, "rgba(255,180,80,0.9)");
        assert_eq!(config.colors.tool_write, "rgba(255,180,80,0.9)");
        assert_eq!(config.colors.tool_run, "rgba(80,220,120,0.9)");
        assert_eq!(config.colors.tool_search, "rgba(200,150,255,0.9)");
        assert_eq!(config.colors.success, "#00ff88");
        assert_eq!(config.colors.warning, "#ff6b00");
        assert_eq!(config.colors.error, "#ff4444");
    }

    #[test]
    fn default_config_has_correct_font() {
        let config = JarvisConfig::default();
        assert_eq!(config.font.family, "Menlo");
        assert_eq!(config.font.size, 13);
        assert_eq!(config.font.title_size, 15);
        assert!((config.font.line_height - 1.6).abs() < f64::EPSILON);
    }

    #[test]
    fn default_config_has_correct_layout() {
        let config = JarvisConfig::default();
        assert_eq!(config.layout.panel_gap, 2);
        assert_eq!(config.layout.border_radius, 4);
        assert_eq!(config.layout.padding, 14);
        assert_eq!(config.layout.max_panels, 5);
        assert!((config.layout.default_panel_width - 0.72).abs() < f64::EPSILON);
        assert_eq!(config.layout.scrollbar_width, 3);
    }

    #[test]
    fn default_config_has_correct_opacity() {
        let config = JarvisConfig::default();
        assert!((config.opacity.background - 1.0).abs() < f64::EPSILON);
        assert!((config.opacity.panel - 0.93).abs() < f64::EPSILON);
        assert!((config.opacity.orb - 1.0).abs() < f64::EPSILON);
        assert!((config.opacity.hex_grid - 0.8).abs() < f64::EPSILON);
        assert!((config.opacity.hud - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_config_has_correct_background() {
        let config = JarvisConfig::default();
        assert_eq!(config.background.mode, BackgroundMode::HexGrid);
        assert_eq!(config.background.solid_color, "#000000");
        assert_eq!(config.background.hex_grid.color, "#00d4ff");
        assert!((config.background.hex_grid.opacity - 0.08).abs() < f64::EPSILON);
    }

    #[test]
    fn default_config_has_correct_visualizer() {
        let config = JarvisConfig::default();
        assert!(config.visualizer.enabled);
        assert_eq!(config.visualizer.visualizer_type, VisualizerType::Orb);
        assert_eq!(config.visualizer.orb.color, "#00d4ff");
        assert_eq!(config.visualizer.anchor, VisualizerAnchor::Center);
        assert!(config.visualizer.react_to_audio);
        assert!(config.visualizer.react_to_state);
    }

    #[test]
    fn default_config_has_correct_visualizer_states() {
        let config = JarvisConfig::default();
        // listening = default
        assert!((config.visualizer.state_listening.scale - 1.0).abs() < f64::EPSILON);
        assert!((config.visualizer.state_listening.intensity - 1.0).abs() < f64::EPSILON);
        // speaking
        assert!((config.visualizer.state_speaking.scale - 1.1).abs() < f64::EPSILON);
        assert!((config.visualizer.state_speaking.intensity - 1.4).abs() < f64::EPSILON);
        // skill
        assert!((config.visualizer.state_skill.scale - 0.9).abs() < f64::EPSILON);
        assert_eq!(config.visualizer.state_skill.color, Some("#ffaa00".into()));
        // chat
        assert!((config.visualizer.state_chat.scale - 0.55).abs() < f64::EPSILON);
        assert_eq!(config.visualizer.state_chat.position_x, Some(0.10));
        assert_eq!(config.visualizer.state_chat.position_y, Some(0.30));
        // idle
        assert!((config.visualizer.state_idle.scale - 0.8).abs() < f64::EPSILON);
        assert_eq!(config.visualizer.state_idle.color, Some("#444444".into()));
    }

    #[test]
    fn default_config_has_correct_startup() {
        let config = JarvisConfig::default();
        assert!(config.startup.boot_animation.enabled);
        assert!((config.startup.boot_animation.duration - 27.0).abs() < f64::EPSILON);
        assert!(!config.startup.fast_start.enabled);
        assert_eq!(config.startup.on_ready.action, OnReadyAction::Listening);
    }

    #[test]
    fn default_config_has_correct_voice() {
        let config = JarvisConfig::default();
        assert!(config.voice.enabled);
        assert_eq!(config.voice.mode, VoiceMode::Ptt);
        assert_eq!(config.voice.sample_rate, 24000);
        assert_eq!(config.voice.whisper_sample_rate, 16000);
        assert_eq!(config.voice.ptt.key, "Option+Period");
    }

    #[test]
    fn default_config_has_correct_keybinds() {
        let config = JarvisConfig::default();
        assert_eq!(config.keybinds.push_to_talk, "Option+Period");
        assert_eq!(config.keybinds.open_assistant, "Cmd+G");
        assert_eq!(config.keybinds.new_panel, "Cmd+T");
        assert_eq!(config.keybinds.close_panel, "Escape+Escape");
        assert_eq!(config.keybinds.toggle_fullscreen, "Cmd+F");
        assert_eq!(config.keybinds.open_settings, "Cmd+,");
        assert_eq!(config.keybinds.cycle_panels, "Tab");
        assert_eq!(config.keybinds.cycle_panels_reverse, "Shift+Tab");
    }

    #[test]
    fn default_config_has_correct_panels() {
        let config = JarvisConfig::default();
        assert!(config.panels.history.enabled);
        assert_eq!(config.panels.history.max_messages, 1000);
        assert!(config.panels.input.multiline);
        assert!(config.panels.focus.border_glow);
    }

    #[test]
    fn default_config_has_correct_games() {
        let config = JarvisConfig::default();
        assert!(config.games.enabled.wordle);
        assert!(config.games.enabled.tetris);
        assert!(config.games.fullscreen.escape_to_exit);
        assert!(config.games.custom_paths.is_empty());
    }

    #[test]
    fn default_config_has_correct_livechat() {
        let config = JarvisConfig::default();
        assert!(config.livechat.enabled);
        assert_eq!(config.livechat.server_port, 19847);
        assert_eq!(config.livechat.connection_timeout, 10);
        assert!(config.livechat.automod.enabled);
        assert_eq!(config.livechat.automod.rate_limit, 5);
    }

    #[test]
    fn default_config_has_correct_presence() {
        let config = JarvisConfig::default();
        assert!(config.presence.enabled);
        assert!(config.presence.server_url.is_empty());
        assert_eq!(config.presence.heartbeat_interval, 30);
    }

    #[test]
    fn default_config_has_correct_performance() {
        let config = JarvisConfig::default();
        assert_eq!(config.performance.preset, PerformancePreset::High);
        assert_eq!(config.performance.frame_rate, 60);
        assert_eq!(config.performance.orb_quality, OrbQuality::High);
        assert_eq!(config.performance.bloom_passes, 2);
        assert!(config.performance.preload.themes);
        assert!(!config.performance.preload.games);
    }

    #[test]
    fn default_config_has_correct_updates() {
        let config = JarvisConfig::default();
        assert!(config.updates.check_automatically);
        assert_eq!(config.updates.channel, UpdateChannel::Stable);
        assert_eq!(config.updates.check_interval, 86400);
        assert!(!config.updates.auto_download);
    }

    #[test]
    fn default_config_has_correct_logging() {
        let config = JarvisConfig::default();
        assert_eq!(config.logging.level, LogLevel::Info);
        assert!(config.logging.file_logging);
        assert_eq!(config.logging.max_file_size_mb, 5);
        assert!(config.logging.redact_secrets);
    }

    #[test]
    fn default_config_has_correct_advanced() {
        let config = JarvisConfig::default();
        assert!(!config.advanced.experimental.web_rendering);
        assert!(!config.advanced.experimental.metal_debug);
        assert!(!config.advanced.developer.show_fps);
        assert!(!config.advanced.developer.inspector_enabled);
    }

    #[test]
    fn partial_toml_deserializes_with_defaults() {
        let toml_str = r##"
[font]
family = "SF Mono"
size = 14

[colors]
primary = "#ff0000"
"##;
        let config: JarvisConfig = toml::from_str(toml_str).unwrap();
        // Overridden values
        assert_eq!(config.font.family, "SF Mono");
        assert_eq!(config.font.size, 14);
        assert_eq!(config.colors.primary, "#ff0000");
        // Defaults preserved
        assert!((config.font.line_height - 1.6).abs() < f64::EPSILON);
        assert_eq!(config.font.title_size, 15);
        assert_eq!(config.colors.background, "#000000");
        assert_eq!(config.colors.text, "#f0ece4");
        assert_eq!(config.theme.name, "jarvis-dark");
        assert_eq!(config.background.mode, BackgroundMode::HexGrid);
        assert!(config.visualizer.enabled);
    }

    #[test]
    fn empty_toml_gives_all_defaults() {
        let config: JarvisConfig = toml::from_str("").unwrap();
        let default = JarvisConfig::default();
        assert_eq!(config.theme.name, default.theme.name);
        assert_eq!(config.colors.primary, default.colors.primary);
        assert_eq!(config.font.size, default.font.size);
        assert_eq!(config.layout.max_panels, default.layout.max_panels);
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = JarvisConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: JarvisConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.theme.name, config.theme.name);
        assert_eq!(deserialized.colors.primary, config.colors.primary);
        assert_eq!(deserialized.font.size, config.font.size);
    }

    #[test]
    fn toml_serialization_roundtrip() {
        let config = JarvisConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: JarvisConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.theme.name, config.theme.name);
        assert_eq!(deserialized.colors.primary, config.colors.primary);
    }

    #[test]
    fn background_mode_serialization() {
        let config = BackgroundConfig {
            mode: BackgroundMode::Solid,
            solid_color: "#1a1a1a".into(),
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"solid\""));
        let deserialized: BackgroundConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.mode, BackgroundMode::Solid);
    }

    #[test]
    fn visualizer_type_serialization() {
        let config = VisualizerConfig {
            visualizer_type: VisualizerType::Particle,
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"particle\""));
    }

    #[test]
    fn voice_mode_serialization() {
        let config = VoiceConfig {
            mode: VoiceMode::Vad,
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"vad\""));
    }

    #[test]
    fn log_level_serialization() {
        let config = LoggingConfig {
            level: LogLevel::Debug,
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"DEBUG\""));
    }

    #[test]
    fn anchor_kebab_case_serialization() {
        let config = VisualizerConfig {
            anchor: VisualizerAnchor::TopLeft,
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"top-left\""));
    }

    #[test]
    fn partial_nested_toml_preserves_sibling_defaults() {
        let toml_str = r##"
[background]
mode = "solid"
solid_color = "#1a1a1a"
"##;
        let config: JarvisConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.background.mode, BackgroundMode::Solid);
        assert_eq!(config.background.solid_color, "#1a1a1a");
        // Nested sub-configs should still have defaults
        assert_eq!(config.background.hex_grid.color, "#00d4ff");
        assert_eq!(config.background.image.fit, ImageFit::Cover);
    }

    #[test]
    fn custom_games_in_toml() {
        let toml_str = r#"
[games.enabled]
wordle = false

[[games.custom_paths]]
name = "my-game"
path = "/path/to/game"
"#;
        let config: JarvisConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.games.enabled.wordle);
        assert!(config.games.enabled.tetris); // default preserved
        assert_eq!(config.games.custom_paths.len(), 1);
        assert_eq!(config.games.custom_paths[0].name, "my-game");
    }
}
