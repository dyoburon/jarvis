//! Theme injection: maps JarvisConfig colors/fonts to CSS variables,
//! injects into all webview panels, and handles settings IPC.

use jarvis_config::schema::JarvisConfig;
use jarvis_webview::theme_bridge::{
    generate_css_injection_js, generate_xterm_theme_js, CssValueKind,
};
use jarvis_webview::IpcPayload;

use crate::app_state::core::JarvisApp;

// =============================================================================
// CONFIG → CSS VARIABLE MAPPING
// =============================================================================

/// Map a `JarvisConfig` to CSS variable triples `(name, value, kind)`.
///
/// Returns the standard CSS custom property names matching the original
/// ThemeManager.swift variables used by all panel HTML files.
pub fn config_to_css_variables(config: &JarvisConfig) -> Vec<(String, String, CssValueKind)> {
    let c = &config.colors;
    let f = &config.font;
    let l = &config.layout;
    let e = &config.effects;

    vec![
        // Colors
        css_color("--color-primary", &c.primary),
        css_color("--color-secondary", &c.secondary),
        css_color("--color-background", &c.background),
        css_color("--color-panel-bg", &c.panel_bg),
        css_color("--color-text", &c.text),
        css_color("--color-text-muted", &c.text_muted),
        css_color("--color-border", &c.border),
        css_color("--color-border-focused", &c.border_focused),
        css_color("--color-user-text", &c.user_text),
        css_color("--color-success", &c.success),
        css_color("--color-warning", &c.warning),
        css_color("--color-error", &c.error),
        // Font (monospace for code/terminal)
        css_font("--font-family", &f.family),
        css_numeric("--font-size", &format!("{}px", f.size)),
        css_numeric("--font-title-size", &format!("{}px", f.title_size)),
        css_numeric("--line-height", &format!("{}", f.line_height)),
        // Font (sans-serif for UI text)
        css_font("--font-ui", &f.ui_family),
        css_numeric("--font-ui-size", &format!("{}px", f.ui_size)),
        // Layout
        css_numeric("--border-radius", &format!("{}px", l.border_radius)),
        css_numeric("--panel-padding", &format!("{}px", l.padding)),
        css_numeric("--panel-gap", &format!("{}px", l.panel_gap)),
        css_numeric("--scrollbar-width", &format!("{}px", l.scrollbar_width)),
        css_numeric("--border-width", &format!("{}px", l.border_width)),
        css_numeric("--outer-padding", &format!("{}px", l.outer_padding)),
        css_numeric("--inactive-opacity", &format!("{}", l.inactive_opacity)),
        // Effects (glassmorphic)
        css_numeric("--blur-radius", &format!("{}px", e.blur_radius)),
        css_numeric("--saturate", &format!("{}", e.saturate)),
        css_numeric("--transition-speed", &format!("{}ms", e.transition_speed)),
        css_numeric("--glow-intensity", &format!("{}", e.glow.intensity)),
        // Opacity
        css_numeric("--panel-opacity", &format!("{}", config.opacity.panel)),
        // Window
        css_numeric(
            "--titlebar-height",
            &format!("{}px", config.window.titlebar_height),
        ),
        // Status bar
        css_numeric(
            "--status-bar-height",
            &format!("{}px", config.status_bar.height),
        ),
        css_color("--status-bar-bg", &config.status_bar.bg),
    ]
}

/// Map a `JarvisConfig` to an xterm.js theme JSON object.
///
/// The terminal HTML listens for `theme` IPC messages and applies the
/// xterm.js `ITheme` object plus font settings.
pub fn config_to_xterm_theme(config: &JarvisConfig) -> serde_json::Value {
    let c = &config.colors;
    let f = &config.font;
    let t = &config.terminal;

    serde_json::json!({
        "xterm": {
            "background": c.background,
            "foreground": c.text,
            "cursor": c.primary,
            "cursorAccent": c.background,
            "selectionBackground": format!("rgba({}, 0.25)",
                hex_to_rgb_args(&c.primary).unwrap_or_else(|| "255, 204, 102".to_string())
            ),
            "selectionForeground": "#ffffff",
            // Ayu Mirage ANSI palette
            "black": "#171b24",
            "red": "#f28779",
            "green": "#bae67e",
            "yellow": "#ffd580",
            "blue": "#73d0ff",
            "magenta": "#d4bfff",
            "cyan": "#95e6cb",
            "white": c.text,
            "brightBlack": "#707a8c",
            "brightRed": "#f28779",
            "brightGreen": "#bae67e",
            "brightYellow": "#ffd580",
            "brightBlue": "#73d0ff",
            "brightMagenta": "#d4bfff",
            "brightCyan": "#95e6cb",
            "brightWhite": "#f3f4f5"
        },
        "fontSize": f.size,
        "fontFamily": format!("'{}', monospace", f.family),
        "lineHeight": f.line_height,
        "fontWeight": f.font_weight,
        "fontWeightBold": f.bold_weight,
        "cursorStyle": cursor_style_to_xterm(&t.cursor_style),
        "cursorBlink": t.cursor_blink,
        "scrollback": t.scrollback_lines
    })
}

/// Map config CursorStyle to xterm.js cursor style string.
/// xterm.js uses "bar" where our config uses "beam".
fn cursor_style_to_xterm(style: &jarvis_config::schema::CursorStyle) -> &'static str {
    use jarvis_config::schema::CursorStyle;
    match style {
        CursorStyle::Block => "block",
        CursorStyle::Underline => "underline",
        CursorStyle::Beam => "bar",
    }
}

// =============================================================================
// INJECTION
// =============================================================================

impl JarvisApp {
    /// Inject the current theme into all webview panels.
    ///
    /// Generates CSS variable injection JS and evaluates it on every panel.
    /// Also sends xterm.js theme update to terminal panels.
    pub(in crate::app_state) fn inject_theme_into_all_webviews(&self) {
        let vars = config_to_css_variables(&self.config);
        let var_refs: Vec<(&str, &str, CssValueKind)> = vars
            .iter()
            .map(|(n, v, k)| (n.as_str(), v.as_str(), *k))
            .collect();
        let css_js = generate_css_injection_js(&var_refs);

        let xterm_theme = config_to_xterm_theme(&self.config);
        let xterm_js = generate_xterm_theme_js(&xterm_theme);

        if let Some(ref registry) = self.webviews {
            for pane_id in registry.active_panes() {
                if let Some(handle) = registry.get(pane_id) {
                    // Inject CSS variables into all panels
                    if let Err(e) = handle.evaluate_script(&css_js) {
                        tracing::warn!(pane_id, error = %e, "Failed to inject theme CSS");
                    }

                    // Send xterm theme to terminal panels
                    if let Err(e) = handle.evaluate_script(&xterm_js) {
                        tracing::warn!(pane_id, error = %e, "Failed to inject xterm theme");
                    }
                }
            }
            tracing::debug!("Theme injected into all webviews");
        }
    }

    /// Handle `settings_init` — send current config to the settings panel.
    pub(in crate::app_state) fn handle_settings_init(&self, pane_id: u32, _payload: &IpcPayload) {
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

    /// Handle `settings_set_theme` — switch theme and re-inject into all panels.
    pub(in crate::app_state) fn handle_settings_set_theme(
        &mut self,
        pane_id: u32,
        payload: &IpcPayload,
    ) {
        let theme_name = match payload {
            IpcPayload::Json(obj) => obj.get("name").and_then(|v| v.as_str()),
            IpcPayload::Text(s) => Some(s.as_str()),
            _ => None,
        };

        let theme_name = match theme_name {
            Some(name) => name.to_string(),
            None => {
                tracing::warn!(pane_id, "settings_set_theme: missing 'name' field");
                return;
            }
        };

        tracing::info!(pane_id, theme = %theme_name, "Switching theme");

        // Load and apply the theme
        match jarvis_config::theme::load_theme(&theme_name) {
            Ok(overrides) => {
                self.config.theme.name = theme_name;
                jarvis_config::theme::apply_theme(&mut self.config, &overrides);
                self.inject_theme_into_all_webviews();
            }
            Err(e) => {
                tracing::warn!(theme = %theme_name, error = %e, "Failed to load theme");
            }
        }
    }
}

// =============================================================================
// HELPERS
// =============================================================================

fn css_color(name: &str, value: &str) -> (String, String, CssValueKind) {
    (name.to_string(), value.to_string(), CssValueKind::Color)
}

fn css_font(name: &str, value: &str) -> (String, String, CssValueKind) {
    (
        name.to_string(),
        value.to_string(),
        CssValueKind::FontFamily,
    )
}

fn css_numeric(name: &str, value: &str) -> (String, String, CssValueKind) {
    (name.to_string(), value.to_string(), CssValueKind::Numeric)
}

/// Convert a hex color `#rrggbb` to `r, g, b` for use in `rgba()`.
fn hex_to_rgb_args(hex: &str) -> Option<String> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() < 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(format!("{r}, {g}, {b}"))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_to_css_variables_has_all_standard_names() {
        let config = JarvisConfig::default();
        let vars = config_to_css_variables(&config);
        let names: Vec<&str> = vars.iter().map(|(n, _, _)| n.as_str()).collect();

        assert!(names.contains(&"--color-primary"));
        assert!(names.contains(&"--color-secondary"));
        assert!(names.contains(&"--color-background"));
        assert!(names.contains(&"--color-panel-bg"));
        assert!(names.contains(&"--color-text"));
        assert!(names.contains(&"--color-text-muted"));
        assert!(names.contains(&"--color-border"));
        assert!(names.contains(&"--color-border-focused"));
        assert!(names.contains(&"--color-user-text"));
        assert!(names.contains(&"--color-success"));
        assert!(names.contains(&"--color-warning"));
        assert!(names.contains(&"--color-error"));
        assert!(names.contains(&"--font-family"));
        assert!(names.contains(&"--font-size"));
        assert!(names.contains(&"--font-title-size"));
        assert!(names.contains(&"--line-height"));
        assert!(names.contains(&"--border-radius"));
        assert!(names.contains(&"--panel-padding"));
        assert!(names.contains(&"--panel-gap"));
        assert!(names.contains(&"--scrollbar-width"));
        // New glassmorphic + UI font variables
        assert!(names.contains(&"--font-ui"));
        assert!(names.contains(&"--font-ui-size"));
        assert!(names.contains(&"--border-width"));
        assert!(names.contains(&"--outer-padding"));
        assert!(names.contains(&"--blur-radius"));
        assert!(names.contains(&"--saturate"));
        assert!(names.contains(&"--transition-speed"));
        assert!(names.contains(&"--glow-intensity"));
        assert!(names.contains(&"--panel-opacity"));
        assert!(names.contains(&"--titlebar-height"));
        assert!(names.contains(&"--status-bar-height"));
        assert!(names.contains(&"--status-bar-bg"));
        assert!(names.contains(&"--inactive-opacity"));
    }

    #[test]
    fn config_to_css_variables_has_correct_values() {
        let config = JarvisConfig::default();
        let vars = config_to_css_variables(&config);
        let map: std::collections::HashMap<&str, &str> = vars
            .iter()
            .map(|(n, v, _)| (n.as_str(), v.as_str()))
            .collect();

        assert_eq!(map["--color-primary"], "#ffcc66");
        assert_eq!(map["--color-background"], "#1f2430");
        assert_eq!(map["--font-size"], "13px");
        assert_eq!(map["--line-height"], "1.6");
        assert_eq!(map["--font-family"], "Menlo");
    }

    #[test]
    fn config_to_css_variables_count() {
        let config = JarvisConfig::default();
        let vars = config_to_css_variables(&config);
        assert_eq!(vars.len(), 33);
    }

    #[test]
    fn config_to_xterm_theme_has_required_fields() {
        let config = JarvisConfig::default();
        let theme = config_to_xterm_theme(&config);

        assert!(theme.get("xterm").is_some());
        assert!(theme.get("fontSize").is_some());
        assert!(theme.get("fontFamily").is_some());
        assert!(theme.get("lineHeight").is_some());
        assert!(theme.get("fontWeight").is_some());
        assert!(theme.get("fontWeightBold").is_some());
        assert!(theme.get("cursorStyle").is_some());
        assert!(theme.get("cursorBlink").is_some());
        assert!(theme.get("scrollback").is_some());

        let xterm = &theme["xterm"];
        assert!(xterm.get("background").is_some());
        assert!(xterm.get("foreground").is_some());
        assert!(xterm.get("cursor").is_some());
    }

    #[test]
    fn config_to_xterm_theme_uses_config_colors() {
        let config = JarvisConfig::default();
        let theme = config_to_xterm_theme(&config);

        assert_eq!(theme["xterm"]["background"], "#1f2430");
        assert_eq!(theme["xterm"]["foreground"], "#cccac2");
        assert_eq!(theme["xterm"]["cursor"], "#ffcc66");
        assert_eq!(theme["fontSize"], 13);
        assert_eq!(theme["cursorStyle"], "block");
        assert_eq!(theme["cursorBlink"], true);
        assert_eq!(theme["scrollback"], 10_000);
    }

    #[test]
    fn cursor_style_to_xterm_maps_correctly() {
        use jarvis_config::schema::CursorStyle;
        assert_eq!(cursor_style_to_xterm(&CursorStyle::Block), "block");
        assert_eq!(cursor_style_to_xterm(&CursorStyle::Underline), "underline");
        assert_eq!(cursor_style_to_xterm(&CursorStyle::Beam), "bar");
    }

    #[test]
    fn hex_to_rgb_args_valid() {
        assert_eq!(hex_to_rgb_args("#00d4ff"), Some("0, 212, 255".to_string()));
        assert_eq!(hex_to_rgb_args("#ff0000"), Some("255, 0, 0".to_string()));
        assert_eq!(hex_to_rgb_args("#000000"), Some("0, 0, 0".to_string()));
    }

    #[test]
    fn hex_to_rgb_args_invalid() {
        assert_eq!(hex_to_rgb_args("#fff"), None); // too short
        assert_eq!(hex_to_rgb_args("not-hex"), None);
        assert_eq!(hex_to_rgb_args(""), None);
    }

    #[test]
    fn hex_to_rgb_args_with_8_digit() {
        // 8-digit hex — still extracts first 6 chars
        assert_eq!(
            hex_to_rgb_args("#00d4ff80"),
            Some("0, 212, 255".to_string())
        );
    }
}
