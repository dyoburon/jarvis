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
        // Font
        css_font("--font-family", &f.family),
        css_numeric("--font-size", &format!("{}px", f.size)),
        css_numeric("--font-title-size", &format!("{}px", f.title_size)),
        css_numeric("--line-height", &format!("{}", f.line_height)),
    ]
}

/// Map a `JarvisConfig` to an xterm.js theme JSON object.
///
/// The terminal HTML listens for `theme` IPC messages and applies the
/// xterm.js `ITheme` object plus font settings.
pub fn config_to_xterm_theme(config: &JarvisConfig) -> serde_json::Value {
    let c = &config.colors;
    let f = &config.font;

    serde_json::json!({
        "xterm": {
            "background": c.background,
            "foreground": c.text,
            "cursor": c.primary,
            "cursorAccent": c.background,
            "selectionBackground": format!("rgba({}, 0.2)",
                hex_to_rgb_args(&c.primary).unwrap_or_else(|| "0, 229, 255".to_string())
            ),
            "selectionForeground": "#ffffff",
            "black": "#000000",
            "red": c.error,
            "green": c.success,
            "yellow": c.warning,
            "blue": c.primary,
            "magenta": "#ff44ff",
            "cyan": c.primary,
            "white": c.text
        },
        "fontSize": f.size,
        "fontFamily": format!("'{}', monospace", f.family)
    })
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
                // Send current theme name and available themes
                let payload = serde_json::json!({
                    "currentTheme": self.config.theme.name,
                    "availableThemes": jarvis_config::BUILT_IN_THEMES,
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
    }

    #[test]
    fn config_to_css_variables_has_correct_values() {
        let config = JarvisConfig::default();
        let vars = config_to_css_variables(&config);
        let map: std::collections::HashMap<&str, &str> = vars
            .iter()
            .map(|(n, v, _)| (n.as_str(), v.as_str()))
            .collect();

        assert_eq!(map["--color-primary"], "#00d4ff");
        assert_eq!(map["--color-background"], "#000000");
        assert_eq!(map["--font-size"], "13px");
        assert_eq!(map["--line-height"], "1.6");
        assert_eq!(map["--font-family"], "Menlo");
    }

    #[test]
    fn config_to_css_variables_count() {
        let config = JarvisConfig::default();
        let vars = config_to_css_variables(&config);
        assert_eq!(vars.len(), 16);
    }

    #[test]
    fn config_to_xterm_theme_has_required_fields() {
        let config = JarvisConfig::default();
        let theme = config_to_xterm_theme(&config);

        assert!(theme.get("xterm").is_some());
        assert!(theme.get("fontSize").is_some());
        assert!(theme.get("fontFamily").is_some());

        let xterm = &theme["xterm"];
        assert!(xterm.get("background").is_some());
        assert!(xterm.get("foreground").is_some());
        assert!(xterm.get("cursor").is_some());
    }

    #[test]
    fn config_to_xterm_theme_uses_config_colors() {
        let config = JarvisConfig::default();
        let theme = config_to_xterm_theme(&config);

        assert_eq!(theme["xterm"]["background"], "#000000");
        assert_eq!(theme["xterm"]["foreground"], "#f0ece4");
        assert_eq!(theme["xterm"]["cursor"], "#00d4ff");
        assert_eq!(theme["fontSize"], 13);
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
