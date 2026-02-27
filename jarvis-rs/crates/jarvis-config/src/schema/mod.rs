//! Configuration schema types for Jarvis.
//!
//! All structs use `serde(default)` so partial configs work correctly.
//! Missing fields are filled with sensible defaults matching the Python schema.

mod background;
mod effects;
mod font;
mod games;
mod keybind_config;
mod layout;
mod livechat;
mod panels;
mod performance;
mod shell;
mod social;
mod startup;
mod system;
mod terminal;
mod theme;
mod visualizer;
mod voice;
mod window;

pub use background::*;
pub use effects::*;
pub use font::*;
pub use games::*;
pub use keybind_config::*;
pub use layout::*;
pub use livechat::*;
pub use panels::*;
pub use performance::*;
pub use shell::*;
pub use social::*;
pub use startup::*;
pub use system::*;
pub use terminal::*;
pub use theme::*;
pub use visualizer::*;
pub use voice::*;
pub use window::*;

use serde::{Deserialize, Serialize};

/// Current config schema version.
pub const CONFIG_SCHEMA_VERSION: u32 = 1;

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
    pub terminal: TerminalConfig,
    pub shell: ShellConfig,
    pub window: WindowConfig,
    pub effects: EffectsSchemaConfig,
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

    // =========================================================================
    // Phase 7: Terminal, Shell, Window config integration tests
    // =========================================================================

    #[test]
    fn default_config_has_correct_terminal() {
        let config = JarvisConfig::default();
        assert_eq!(config.terminal.scrollback_lines, 10_000);
        assert_eq!(config.terminal.cursor_style, CursorStyle::Block);
        assert!(config.terminal.cursor_blink);
        assert_eq!(config.terminal.cursor_blink_interval_ms, 500);
        assert!(config.terminal.true_color);
        assert!(config.terminal.bell.visual);
        assert!(!config.terminal.bell.audio);
        assert!(!config.terminal.mouse.copy_on_select);
        assert!(config.terminal.mouse.url_detection);
        assert!(config.terminal.search.wrap_around);
        assert!(!config.terminal.search.regex);
    }

    #[test]
    fn default_config_has_correct_shell() {
        let config = JarvisConfig::default();
        assert!(config.shell.program.is_empty());
        assert!(config.shell.args.is_empty());
        assert!(config.shell.working_directory.is_none());
        assert!(config.shell.env.is_empty());
        assert!(config.shell.login_shell);
    }

    #[test]
    fn default_config_has_correct_window() {
        let config = JarvisConfig::default();
        assert_eq!(config.window.decorations, WindowDecorations::Full);
        assert!((config.window.opacity - 1.0).abs() < f64::EPSILON);
        assert!(!config.window.blur);
        assert_eq!(config.window.startup_mode, StartupMode::Windowed);
        assert_eq!(config.window.title, "Jarvis");
        assert!(config.window.dynamic_title);
        assert_eq!(config.window.padding.top, 0);
    }

    #[test]
    fn default_config_has_correct_font_extensions() {
        let config = JarvisConfig::default();
        assert!(config.font.bold_family.is_none());
        assert!(config.font.italic_family.is_none());
        assert!(config.font.nerd_font);
        assert!(!config.font.ligatures);
        assert!(config.font.fallback_families.is_empty());
        assert_eq!(config.font.font_weight, 400);
        assert_eq!(config.font.bold_weight, 700);
    }

    #[test]
    fn terminal_config_in_toml() {
        let toml_str = r#"
[terminal]
scrollback_lines = 50000
cursor_style = "beam"
cursor_blink = false

[terminal.bell]
audio = true

[terminal.mouse]
copy_on_select = true
"#;
        let config: JarvisConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.terminal.scrollback_lines, 50_000);
        assert_eq!(config.terminal.cursor_style, CursorStyle::Beam);
        assert!(!config.terminal.cursor_blink);
        assert!(config.terminal.bell.audio);
        assert!(config.terminal.mouse.copy_on_select);
        // Defaults preserved
        assert!(config.terminal.true_color);
        assert!(config.terminal.bell.visual);
        assert_eq!(config.theme.name, "jarvis-dark");
    }

    #[test]
    fn shell_config_in_toml() {
        let toml_str = r#"
[shell]
program = "/bin/zsh"
args = ["-l"]
login_shell = false

[shell.env]
TERM = "xterm-256color"
"#;
        let config: JarvisConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.shell.program, "/bin/zsh");
        assert_eq!(config.shell.args, vec!["-l"]);
        assert!(!config.shell.login_shell);
        assert_eq!(config.shell.env.get("TERM").unwrap(), "xterm-256color");
    }

    #[test]
    fn window_config_in_toml() {
        let toml_str = r#"
[window]
decorations = "transparent"
opacity = 0.9
blur = true
startup_mode = "maximized"
title = "My Terminal"

[window.padding]
top = 4
bottom = 4
left = 8
right = 8
"#;
        let config: JarvisConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.window.decorations, WindowDecorations::Transparent);
        assert!((config.window.opacity - 0.9).abs() < f64::EPSILON);
        assert!(config.window.blur);
        assert_eq!(config.window.startup_mode, StartupMode::Maximized);
        assert_eq!(config.window.title, "My Terminal");
        assert_eq!(config.window.padding.top, 4);
        assert_eq!(config.window.padding.left, 8);
    }

    #[test]
    fn new_font_fields_in_toml() {
        let toml_str = r#"
[font]
family = "JetBrains Mono"
size = 14
ligatures = true
nerd_font = false
bold_family = "JetBrains Mono Bold"
fallback_families = ["Symbols Nerd Font Mono"]
font_weight = 300
"#;
        let config: JarvisConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.font.family, "JetBrains Mono");
        assert_eq!(config.font.size, 14);
        assert!(config.font.ligatures);
        assert!(!config.font.nerd_font);
        assert_eq!(
            config.font.bold_family.as_deref(),
            Some("JetBrains Mono Bold")
        );
        assert_eq!(config.font.fallback_families.len(), 1);
        assert_eq!(config.font.font_weight, 300);
        // Defaults preserved
        assert_eq!(config.font.bold_weight, 700);
        assert!(config.font.italic_family.is_none());
    }

    #[test]
    fn default_config_has_correct_effects() {
        let config = JarvisConfig::default();
        assert!(config.effects.enabled);
        assert!(config.effects.inactive_pane_dim);
        assert!(config.effects.scanlines.enabled);
        assert!((config.effects.scanlines.intensity - 0.08).abs() < f32::EPSILON);
        assert!(config.effects.vignette.enabled);
        assert!(config.effects.bloom.enabled);
        assert_eq!(config.effects.bloom.passes, 2);
        assert!(config.effects.glow.enabled);
        assert_eq!(config.effects.glow.color, "#00d4ff");
        assert!(config.effects.flicker.enabled);
        assert!(!config.effects.crt_curvature);
    }

    #[test]
    fn effects_config_in_toml() {
        let toml_str = r##"
[effects]
enabled = true
inactive_pane_dim = false

[effects.scanlines]
intensity = 0.15

[effects.bloom]
passes = 3
intensity = 1.5

[effects.glow]
color = "#ff6b00"
width = 4.0

[effects.flicker]
enabled = false
"##;
        let config: JarvisConfig = toml::from_str(toml_str).unwrap();
        assert!(config.effects.enabled);
        assert!(!config.effects.inactive_pane_dim);
        assert!((config.effects.scanlines.intensity - 0.15).abs() < f32::EPSILON);
        assert_eq!(config.effects.bloom.passes, 3);
        assert_eq!(config.effects.glow.color, "#ff6b00");
        assert!(!config.effects.flicker.enabled);
        // Defaults preserved
        assert!(config.effects.vignette.enabled);
        assert_eq!(config.theme.name, "jarvis-dark");
    }

    #[test]
    fn effects_disabled_in_toml() {
        let toml_str = r#"
[effects]
enabled = false
"#;
        let config: JarvisConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.effects.enabled);
        // Sub-configs still have defaults (master toggle is checked at runtime)
        assert!(config.effects.scanlines.enabled);
        assert!(config.effects.bloom.enabled);
    }

    #[test]
    fn empty_toml_still_gives_all_defaults_with_new_fields() {
        let config: JarvisConfig = toml::from_str("").unwrap();
        // New fields have defaults
        assert_eq!(config.terminal.scrollback_lines, 10_000);
        assert!(config.shell.program.is_empty());
        assert_eq!(config.window.title, "Jarvis");
        assert!(config.font.nerd_font);
        assert!(config.effects.enabled);
        assert_eq!(config.effects.bloom.passes, 2);
    }
}
