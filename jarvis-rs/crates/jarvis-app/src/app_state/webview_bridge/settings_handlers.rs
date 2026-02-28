//! Settings IPC handlers — real-time config updates with TOML persistence.
//!
//! Every `settings_update` message updates config in memory, writes TOML
//! to disk, and broadcasts CSS variables to all panels. No save button.

use jarvis_config::schema::{AutoOpenPanel, PanelKind};
use jarvis_webview::IpcPayload;

use crate::app_state::core::JarvisApp;

// =============================================================================
// HANDLERS
// =============================================================================

impl JarvisApp {
    /// Handle `settings_update` — update a single config field, save, broadcast.
    pub(in crate::app_state) fn handle_settings_update(
        &mut self,
        pane_id: u32,
        payload: &IpcPayload,
    ) {
        let (path, value) = match extract_path_value(payload) {
            Some(pv) => pv,
            None => {
                tracing::warn!(pane_id, "settings_update: missing path or value");
                return;
            }
        };

        if !is_valid_settings_path(&path) {
            tracing::warn!(pane_id, path = %path, "settings_update: unknown path");
            return;
        }

        tracing::debug!(pane_id, path = %path, "settings_update");

        let layout_changed = apply_setting(&mut self.config, &path, &value);

        // Persist to disk
        if let Err(e) = jarvis_config::save_config(&self.config) {
            tracing::warn!(error = %e, "Failed to save config to disk");
        }

        // Broadcast theme to all panels
        self.inject_theme_into_all_webviews();

        // If layout changed, update tiling engine + reposition webviews
        if layout_changed {
            self.tiling.set_gap(self.config.layout.panel_gap);
            self.tiling.set_outer_padding(self.config.layout.padding);
            self.sync_webview_bounds();
            self.needs_redraw = true;
        }

        // Confirm save to settings panel
        if let Some(ref registry) = self.webviews {
            if let Some(handle) = registry.get(pane_id) {
                let ack = serde_json::json!({ "path": path });
                if let Err(e) = handle.send_ipc("settings_saved", &ack) {
                    tracing::warn!(pane_id, error = %e, "Failed to send settings_saved");
                }
            }
        }
    }

    /// Handle `settings_reset_section` — reset a config section to defaults.
    pub(in crate::app_state) fn handle_settings_reset_section(
        &mut self,
        pane_id: u32,
        payload: &IpcPayload,
    ) {
        let section = match extract_string_field(payload, "section") {
            Some(s) => s,
            None => {
                tracing::warn!(pane_id, "settings_reset_section: missing 'section'");
                return;
            }
        };

        tracing::info!(pane_id, section = %section, "Resetting config section");
        reset_section(&mut self.config, &section);

        if let Err(e) = jarvis_config::save_config(&self.config) {
            tracing::warn!(error = %e, "Failed to save config after reset");
        }

        self.inject_theme_into_all_webviews();
        self.send_full_config(pane_id);
    }

    /// Handle `settings_get_config` — send full config JSON to the panel.
    pub(in crate::app_state) fn handle_settings_get_config(
        &self,
        pane_id: u32,
        _payload: &IpcPayload,
    ) {
        self.send_full_config(pane_id);
    }

    /// Send the full config JSON to a specific pane.
    fn send_full_config(&self, pane_id: u32) {
        if let Some(ref registry) = self.webviews {
            if let Some(handle) = registry.get(pane_id) {
                let config_json = jarvis_config::config_to_json(&self.config);
                let payload = serde_json::json!({
                    "currentTheme": self.config.theme.name,
                    "availableThemes": jarvis_config::BUILT_IN_THEMES,
                    "config": serde_json::from_str::<serde_json::Value>(&config_json)
                        .unwrap_or(serde_json::Value::Null),
                });
                if let Err(e) = handle.send_ipc("settings_data", &payload) {
                    tracing::warn!(pane_id, error = %e, "Failed to send settings_data");
                }
            }
        }
    }
}

// =============================================================================
// PATH VALIDATION
// =============================================================================

/// Known settings paths that can be updated via IPC.
const VALID_PATHS: &[&str] = &[
    // Colors
    "colors.primary",
    "colors.secondary",
    "colors.background",
    "colors.text",
    "colors.text_muted",
    "colors.success",
    "colors.warning",
    "colors.error",
    // Font
    "font.family",
    "font.size",
    "font.title_size",
    "font.line_height",
    // Opacity
    "opacity.background",
    "opacity.panel",
    "opacity.orb",
    "opacity.hex_grid",
    "opacity.hud",
    // Layout
    "layout.panel_gap",
    "layout.border_radius",
    "layout.padding",
    "layout.max_panels",
    "layout.default_panel_width",
    "layout.scrollbar_width",
    // Background
    "background.mode",
    "background.solid_color",
    "background.hex_grid.color",
    "background.hex_grid.opacity",
    "background.hex_grid.animation_speed",
    "background.hex_grid.glow_intensity",
    // Visualizer
    "visualizer.enabled",
    "visualizer.visualizer_type",
    "visualizer.anchor",
    "visualizer.position_x",
    "visualizer.position_y",
    "visualizer.scale",
    "visualizer.react_to_audio",
    "visualizer.react_to_state",
    "visualizer.orb.color",
    "visualizer.orb.pulse_speed",
    "visualizer.orb.bloom_intensity",
    // Startup
    "startup.boot_animation.enabled",
    "startup.boot_animation.duration",
    "startup.boot_animation.skip_on_key",
    "startup.fast_start.enabled",
    "startup.fast_start.delay",
    "startup.on_ready.action",
    // Voice
    "voice.enabled",
    "voice.mode",
    "voice.input_device",
    "voice.sample_rate",
    // Keybinds
    "keybinds.push_to_talk",
    "keybinds.open_assistant",
    "keybinds.new_panel",
    "keybinds.close_panel",
    "keybinds.toggle_fullscreen",
    "keybinds.open_settings",
    "keybinds.cycle_panels",
    "keybinds.cycle_panels_reverse",
    // Panels
    "panels.history.enabled",
    "panels.history.max_messages",
    "panels.focus.restore_on_activate",
    "panels.focus.show_indicator",
    "panels.focus.border_glow",
    // Games
    "games.enabled.wordle",
    "games.enabled.connections",
    "games.enabled.asteroids",
    "games.enabled.tetris",
    "games.enabled.pinball",
    "games.enabled.doodlejump",
    "games.enabled.minesweeper",
    "games.enabled.draw",
    "games.enabled.subway",
    "games.enabled.videoplayer",
    // Performance
    "performance.preset",
    "performance.frame_rate",
    "performance.bloom_passes",
    // Advanced
    "advanced.developer.show_fps",
    "advanced.developer.show_debug_hud",
    "advanced.developer.inspector_enabled",
    // Auto-open (special: value is an array)
    "auto_open.panels",
];

/// Check if a path is in the known valid set.
pub fn is_valid_settings_path(path: &str) -> bool {
    VALID_PATHS.contains(&path)
}

// =============================================================================
// APPLY SETTING
// =============================================================================

/// Apply a single setting value to the config. Returns `true` if layout changed.
fn apply_setting(
    config: &mut jarvis_config::schema::JarvisConfig,
    path: &str,
    value: &serde_json::Value,
) -> bool {
    let mut layout_changed = false;

    match path {
        // -- Colors --
        "colors.primary" => set_str(&mut config.colors.primary, value),
        "colors.secondary" => set_str(&mut config.colors.secondary, value),
        "colors.background" => set_str(&mut config.colors.background, value),
        "colors.text" => set_str(&mut config.colors.text, value),
        "colors.text_muted" => set_str(&mut config.colors.text_muted, value),
        "colors.success" => set_str(&mut config.colors.success, value),
        "colors.warning" => set_str(&mut config.colors.warning, value),
        "colors.error" => set_str(&mut config.colors.error, value),
        // -- Font --
        "font.family" => set_str(&mut config.font.family, value),
        "font.size" => set_u32(&mut config.font.size, value),
        "font.title_size" => set_u32(&mut config.font.title_size, value),
        "font.line_height" => set_f64(&mut config.font.line_height, value),
        // -- Opacity --
        "opacity.background" => set_f64(&mut config.opacity.background, value),
        "opacity.panel" => set_f64(&mut config.opacity.panel, value),
        "opacity.orb" => set_f64(&mut config.opacity.orb, value),
        "opacity.hex_grid" => set_f64(&mut config.opacity.hex_grid, value),
        "opacity.hud" => set_f64(&mut config.opacity.hud, value),
        // -- Layout --
        "layout.panel_gap" => {
            set_u32(&mut config.layout.panel_gap, value);
            layout_changed = true;
        }
        "layout.border_radius" => set_u32(&mut config.layout.border_radius, value),
        "layout.padding" => {
            set_u32(&mut config.layout.padding, value);
            layout_changed = true;
        }
        "layout.max_panels" => set_u32(&mut config.layout.max_panels, value),
        "layout.default_panel_width" => set_f64(&mut config.layout.default_panel_width, value),
        "layout.scrollbar_width" => set_u32(&mut config.layout.scrollbar_width, value),
        // -- Background --
        "background.mode" => set_str_enum(&mut config.background.mode, value),
        "background.solid_color" => set_str(&mut config.background.solid_color, value),
        "background.hex_grid.color" => set_str(&mut config.background.hex_grid.color, value),
        "background.hex_grid.opacity" => set_f64(&mut config.background.hex_grid.opacity, value),
        "background.hex_grid.animation_speed" => {
            set_f64(&mut config.background.hex_grid.animation_speed, value);
        }
        "background.hex_grid.glow_intensity" => {
            set_f64(&mut config.background.hex_grid.glow_intensity, value);
        }
        // -- Visualizer --
        "visualizer.enabled" => set_bool(&mut config.visualizer.enabled, value),
        "visualizer.visualizer_type" => {
            set_str_enum(&mut config.visualizer.visualizer_type, value);
        }
        "visualizer.anchor" => set_str_enum(&mut config.visualizer.anchor, value),
        "visualizer.position_x" => set_f64(&mut config.visualizer.position_x, value),
        "visualizer.position_y" => set_f64(&mut config.visualizer.position_y, value),
        "visualizer.scale" => set_f64(&mut config.visualizer.scale, value),
        "visualizer.react_to_audio" => set_bool(&mut config.visualizer.react_to_audio, value),
        "visualizer.react_to_state" => set_bool(&mut config.visualizer.react_to_state, value),
        "visualizer.orb.color" => set_str(&mut config.visualizer.orb.color, value),
        "visualizer.orb.pulse_speed" => {
            set_f64(&mut config.visualizer.orb.rotation_speed, value);
        }
        "visualizer.orb.bloom_intensity" => {
            set_f64(&mut config.visualizer.orb.bloom_intensity, value);
        }
        // -- Startup --
        "startup.boot_animation.enabled" => {
            set_bool(&mut config.startup.boot_animation.enabled, value);
        }
        "startup.boot_animation.duration" => {
            set_f64(&mut config.startup.boot_animation.duration, value);
        }
        "startup.boot_animation.skip_on_key" => {
            set_bool(&mut config.startup.boot_animation.skip_on_key, value);
        }
        "startup.fast_start.enabled" => {
            set_bool(&mut config.startup.fast_start.enabled, value);
        }
        "startup.fast_start.delay" => set_f64(&mut config.startup.fast_start.delay, value),
        "startup.on_ready.action" => set_str_enum(&mut config.startup.on_ready.action, value),
        // -- Voice --
        "voice.enabled" => set_bool(&mut config.voice.enabled, value),
        "voice.mode" => set_str_enum(&mut config.voice.mode, value),
        "voice.input_device" => set_str(&mut config.voice.input_device, value),
        "voice.sample_rate" => set_u32(&mut config.voice.sample_rate, value),
        // -- Keybinds --
        "keybinds.push_to_talk" => set_str(&mut config.keybinds.push_to_talk, value),
        "keybinds.open_assistant" => set_str(&mut config.keybinds.open_assistant, value),
        "keybinds.new_panel" => set_str(&mut config.keybinds.new_panel, value),
        "keybinds.close_panel" => set_str(&mut config.keybinds.close_panel, value),
        "keybinds.toggle_fullscreen" => set_str(&mut config.keybinds.toggle_fullscreen, value),
        "keybinds.open_settings" => set_str(&mut config.keybinds.open_settings, value),
        "keybinds.cycle_panels" => set_str(&mut config.keybinds.cycle_panels, value),
        "keybinds.cycle_panels_reverse" => {
            set_str(&mut config.keybinds.cycle_panels_reverse, value);
        }
        // -- Panels --
        "panels.history.enabled" => set_bool(&mut config.panels.history.enabled, value),
        "panels.history.max_messages" => set_u32(&mut config.panels.history.max_messages, value),
        "panels.focus.restore_on_activate" => {
            set_bool(&mut config.panels.focus.restore_on_activate, value);
        }
        "panels.focus.show_indicator" => {
            set_bool(&mut config.panels.focus.show_indicator, value);
        }
        "panels.focus.border_glow" => set_bool(&mut config.panels.focus.border_glow, value),
        // -- Games --
        "games.enabled.wordle" => set_bool(&mut config.games.enabled.wordle, value),
        "games.enabled.connections" => set_bool(&mut config.games.enabled.connections, value),
        "games.enabled.asteroids" => set_bool(&mut config.games.enabled.asteroids, value),
        "games.enabled.tetris" => set_bool(&mut config.games.enabled.tetris, value),
        "games.enabled.pinball" => set_bool(&mut config.games.enabled.pinball, value),
        "games.enabled.doodlejump" => set_bool(&mut config.games.enabled.doodlejump, value),
        "games.enabled.minesweeper" => set_bool(&mut config.games.enabled.minesweeper, value),
        "games.enabled.draw" => set_bool(&mut config.games.enabled.draw, value),
        "games.enabled.subway" => set_bool(&mut config.games.enabled.subway, value),
        "games.enabled.videoplayer" => set_bool(&mut config.games.enabled.videoplayer, value),
        // -- Performance --
        "performance.preset" => set_str_enum(&mut config.performance.preset, value),
        "performance.frame_rate" => set_u32(&mut config.performance.frame_rate, value),
        "performance.bloom_passes" => set_u32(&mut config.performance.bloom_passes, value),
        // -- Advanced --
        "advanced.developer.show_fps" => {
            set_bool(&mut config.advanced.developer.show_fps, value);
        }
        "advanced.developer.show_debug_hud" => {
            set_bool(&mut config.advanced.developer.show_debug_hud, value);
        }
        "advanced.developer.inspector_enabled" => {
            set_bool(&mut config.advanced.developer.inspector_enabled, value);
        }
        // -- Auto-open panels (special: array value) --
        "auto_open.panels" => {
            if let Some(arr) = value.as_array() {
                config.auto_open.panels = arr.iter().filter_map(parse_auto_open_panel).collect();
            }
        }
        _ => {
            tracing::warn!(path, "apply_setting: unhandled path");
        }
    }

    layout_changed
}

// =============================================================================
// RESET SECTION
// =============================================================================

fn reset_section(config: &mut jarvis_config::schema::JarvisConfig, section: &str) {
    match section {
        "colors" => config.colors = Default::default(),
        "font" => config.font = Default::default(),
        "opacity" => config.opacity = Default::default(),
        "layout" => config.layout = Default::default(),
        "background" => config.background = Default::default(),
        "visualizer" => config.visualizer = Default::default(),
        "startup" => config.startup = Default::default(),
        "voice" => config.voice = Default::default(),
        "keybinds" => config.keybinds = Default::default(),
        "panels" => config.panels = Default::default(),
        "games" => config.games = Default::default(),
        "performance" => config.performance = Default::default(),
        "advanced" => config.advanced = Default::default(),
        "auto_open" => config.auto_open = Default::default(),
        _ => tracing::warn!(section, "reset_section: unknown section"),
    }
}

// =============================================================================
// VALUE HELPERS
// =============================================================================

fn set_str(target: &mut String, value: &serde_json::Value) {
    if let Some(s) = value.as_str() {
        *target = s.to_string();
    }
}

fn set_bool(target: &mut bool, value: &serde_json::Value) {
    if let Some(b) = value.as_bool() {
        *target = b;
    }
}

fn set_u32(target: &mut u32, value: &serde_json::Value) {
    if let Some(n) = value.as_u64() {
        *target = n as u32;
    } else if let Some(n) = value.as_f64() {
        *target = n as u32;
    }
}

fn set_f64(target: &mut f64, value: &serde_json::Value) {
    if let Some(n) = value.as_f64() {
        *target = n;
    }
}

/// Set any serde-deserializable enum from a JSON string value.
fn set_str_enum<T>(target: &mut T, value: &serde_json::Value)
where
    T: serde::de::DeserializeOwned,
{
    if let Some(s) = value.as_str() {
        let json_str = serde_json::Value::String(s.to_string());
        if let Ok(parsed) = serde_json::from_value::<T>(json_str) {
            *target = parsed;
        }
    }
}

fn parse_auto_open_panel(v: &serde_json::Value) -> Option<AutoOpenPanel> {
    let kind_str = v.get("kind")?.as_str()?;
    let kind = match kind_str {
        "terminal" => PanelKind::Terminal,
        "assistant" => PanelKind::Assistant,
        "chat" => PanelKind::Chat,
        "settings" => PanelKind::Settings,
        "presence" => PanelKind::Presence,
        _ => return None,
    };
    Some(AutoOpenPanel {
        kind,
        command: v.get("command").and_then(|c| c.as_str()).map(String::from),
        title: v.get("title").and_then(|t| t.as_str()).map(String::from),
        args: v
            .get("args")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        working_directory: v
            .get("working_directory")
            .and_then(|w| w.as_str())
            .map(String::from),
    })
}

// =============================================================================
// PAYLOAD EXTRACTION
// =============================================================================

fn extract_path_value(payload: &IpcPayload) -> Option<(String, serde_json::Value)> {
    match payload {
        IpcPayload::Json(obj) => {
            let path = obj.get("path")?.as_str()?.to_string();
            let value = obj.get("value")?.clone();
            Some((path, value))
        }
        _ => None,
    }
}

fn extract_string_field(payload: &IpcPayload, field: &str) -> Option<String> {
    match payload {
        IpcPayload::Json(obj) => obj.get(field)?.as_str().map(String::from),
        _ => None,
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_settings_paths_accepted() {
        assert!(is_valid_settings_path("colors.primary"));
        assert!(is_valid_settings_path("font.size"));
        assert!(is_valid_settings_path("opacity.panel"));
        assert!(is_valid_settings_path("layout.panel_gap"));
        assert!(is_valid_settings_path("visualizer.enabled"));
        assert!(is_valid_settings_path("keybinds.push_to_talk"));
        assert!(is_valid_settings_path("games.enabled.wordle"));
        assert!(is_valid_settings_path("auto_open.panels"));
        assert!(is_valid_settings_path("advanced.developer.show_fps"));
    }

    #[test]
    fn unknown_paths_rejected() {
        assert!(!is_valid_settings_path(""));
        assert!(!is_valid_settings_path("invalid"));
        assert!(!is_valid_settings_path("colors"));
        assert!(!is_valid_settings_path("colors.nonexistent"));
        assert!(!is_valid_settings_path("system.root_password"));
    }

    #[test]
    fn injection_paths_rejected() {
        assert!(!is_valid_settings_path("colors.primary; rm -rf /"));
        assert!(!is_valid_settings_path("<script>alert(1)</script>"));
        assert!(!is_valid_settings_path("colors.primary\0"));
        assert!(!is_valid_settings_path("../../../etc/passwd"));
    }

    #[test]
    fn apply_setting_colors() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        let val = serde_json::json!("#ff0000");
        apply_setting(&mut config, "colors.primary", &val);
        assert_eq!(config.colors.primary, "#ff0000");
    }

    #[test]
    fn apply_setting_font_size() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        let val = serde_json::json!(16);
        apply_setting(&mut config, "font.size", &val);
        assert_eq!(config.font.size, 16);
    }

    #[test]
    fn apply_setting_opacity() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        let val = serde_json::json!(0.5);
        apply_setting(&mut config, "opacity.panel", &val);
        assert!((config.opacity.panel - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn apply_setting_layout_returns_layout_changed() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        let val = serde_json::json!(10);
        let changed = apply_setting(&mut config, "layout.panel_gap", &val);
        assert!(changed);
        assert_eq!(config.layout.panel_gap, 10);
    }

    #[test]
    fn apply_setting_non_layout_returns_false() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        let val = serde_json::json!("#ff0000");
        let changed = apply_setting(&mut config, "colors.primary", &val);
        assert!(!changed);
    }

    #[test]
    fn apply_setting_toggle() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        let val = serde_json::json!(false);
        apply_setting(&mut config, "visualizer.enabled", &val);
        assert!(!config.visualizer.enabled);
    }

    #[test]
    fn apply_setting_keybind() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        let val = serde_json::json!("Ctrl+Shift+P");
        apply_setting(&mut config, "keybinds.push_to_talk", &val);
        assert_eq!(config.keybinds.push_to_talk, "Ctrl+Shift+P");
    }

    #[test]
    fn apply_setting_auto_open_panels() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        let val = serde_json::json!([
            { "kind": "terminal", "title": "Term", "command": null },
            { "kind": "terminal", "command": "claude", "title": "Claude" }
        ]);
        apply_setting(&mut config, "auto_open.panels", &val);
        assert_eq!(config.auto_open.panels.len(), 2);
        assert_eq!(
            config.auto_open.panels[1].command.as_deref(),
            Some("claude")
        );
    }

    #[test]
    fn reset_section_restores_defaults() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        config.colors.primary = "#ff0000".into();
        config.colors.secondary = "#00ff00".into();
        reset_section(&mut config, "colors");
        assert_eq!(config.colors.primary, "#00d4ff");
        assert_eq!(config.colors.secondary, "#ff6b00");
    }
}
