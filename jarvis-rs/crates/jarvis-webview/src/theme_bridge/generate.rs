//! CSS generation from theme variables.
//!
//! Takes validated (name, value) pairs and generates CSS `:root` blocks
//! and JavaScript injection snippets for webview theming.

use super::sanitize::{validate_css_color, validate_css_font_family, validate_css_numeric};

// =============================================================================
// CSS VARIABLE TYPES
// =============================================================================

/// A validated CSS variable ready for injection.
#[derive(Debug, Clone)]
pub struct CssVariable {
    /// CSS custom property name (e.g. `--color-primary`).
    pub name: String,
    /// Validated CSS value (e.g. `#00d4ff`).
    pub value: String,
}

/// The type of validation to apply to a CSS value.
#[derive(Debug, Clone, Copy)]
pub enum CssValueKind {
    /// Hex or rgba() color.
    Color,
    /// Font family name(s).
    FontFamily,
    /// Numeric value with optional unit.
    Numeric,
}

// =============================================================================
// CSS GENERATION
// =============================================================================

/// Generate a CSS `:root { ... }` block from a list of variable definitions.
///
/// Each entry is `(name, value, kind)`. Values are validated according to
/// their kind. Invalid values are skipped with a warning log.
///
/// Returns the CSS string ready for injection via `<style>` or `evaluate_script`.
pub fn generate_css_root(variables: &[(&str, &str, CssValueKind)]) -> String {
    let mut css = String::from(":root {\n");

    for (name, value, kind) in variables {
        let validation = match kind {
            CssValueKind::Color => validate_css_color(value),
            CssValueKind::FontFamily => validate_css_font_family(value),
            CssValueKind::Numeric => validate_css_numeric(value),
        };

        match validation {
            Ok(()) => {
                css.push_str(&format!("  {name}: {value};\n"));
            }
            Err(e) => {
                tracing::warn!(
                    name,
                    value,
                    error = %e,
                    "Theme variable rejected by sanitizer"
                );
            }
        }
    }

    css.push('}');
    css
}

/// Generate a JavaScript snippet that injects CSS variables into the page.
///
/// Uses `document.documentElement.style.setProperty()` for each variable,
/// which updates them live without a page reload.
pub fn generate_css_injection_js(variables: &[(&str, &str, CssValueKind)]) -> String {
    let mut js = String::from("(function() {\n  var s = document.documentElement.style;\n");

    for (name, value, kind) in variables {
        let validation = match kind {
            CssValueKind::Color => validate_css_color(value),
            CssValueKind::FontFamily => validate_css_font_family(value),
            CssValueKind::Numeric => validate_css_numeric(value),
        };

        if validation.is_ok() {
            // Escape for JS string literal — replace \ and ' characters
            let escaped_value = value.replace('\\', "\\\\").replace('\'', "\\'");
            let escaped_name = name.replace('\\', "\\\\").replace('\'', "\\'");
            js.push_str(&format!(
                "  s.setProperty('{escaped_name}', '{escaped_value}');\n"
            ));
        }
    }

    js.push_str("})();");
    js
}

/// Generate a JavaScript snippet for xterm.js theme update.
///
/// Takes a JSON object of xterm theme colors and font settings,
/// dispatches it via the Jarvis IPC bridge.
pub fn generate_xterm_theme_js(xterm_theme: &serde_json::Value) -> String {
    let json_str = serde_json::to_string(xterm_theme).unwrap_or_else(|_| "{}".to_string());
    format!(
        "window.__jarvis_theme = {json_str}; \
        if (window.jarvis && window.jarvis.ipc) {{ \
            window.jarvis.ipc._dispatch('theme', {json_str}); \
        }}"
    )
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_css_root_basic() {
        let vars = vec![
            ("--color-primary", "#00d4ff", CssValueKind::Color),
            ("--color-background", "#000000", CssValueKind::Color),
        ];
        let css = generate_css_root(&vars);

        assert!(css.starts_with(":root {"));
        assert!(css.ends_with('}'));
        assert!(css.contains("--color-primary: #00d4ff;"));
        assert!(css.contains("--color-background: #000000;"));
    }

    #[test]
    fn generate_css_root_with_rgba() {
        let vars = vec![("--color-panel-bg", "rgba(0,0,0,0.93)", CssValueKind::Color)];
        let css = generate_css_root(&vars);
        assert!(css.contains("--color-panel-bg: rgba(0,0,0,0.93);"));
    }

    #[test]
    fn generate_css_root_with_font() {
        let vars = vec![
            (
                "--font-family",
                "'Courier New', monospace",
                CssValueKind::FontFamily,
            ),
            ("--font-size", "14px", CssValueKind::Numeric),
            ("--line-height", "1.6", CssValueKind::Numeric),
        ];
        let css = generate_css_root(&vars);
        assert!(css.contains("--font-family: 'Courier New', monospace;"));
        assert!(css.contains("--font-size: 14px;"));
        assert!(css.contains("--line-height: 1.6;"));
    }

    #[test]
    fn generate_css_root_skips_invalid() {
        let vars = vec![
            ("--color-primary", "#00d4ff", CssValueKind::Color),
            (
                "--color-bad",
                "red; } body { color: evil",
                CssValueKind::Color,
            ),
            ("--color-success", "#00ff88", CssValueKind::Color),
        ];
        let css = generate_css_root(&vars);
        assert!(css.contains("--color-primary: #00d4ff;"));
        assert!(css.contains("--color-success: #00ff88;"));
        assert!(!css.contains("--color-bad"));
        assert!(!css.contains("evil"));
    }

    #[test]
    fn generate_css_root_empty_input() {
        let vars: Vec<(&str, &str, CssValueKind)> = vec![];
        let css = generate_css_root(&vars);
        assert_eq!(css, ":root {\n}");
    }

    #[test]
    fn generate_css_injection_js_basic() {
        let vars = vec![
            ("--color-primary", "#00d4ff", CssValueKind::Color),
            ("--font-size", "14px", CssValueKind::Numeric),
        ];
        let js = generate_css_injection_js(&vars);

        assert!(js.contains("setProperty('--color-primary', '#00d4ff')"));
        assert!(js.contains("setProperty('--font-size', '14px')"));
    }

    #[test]
    fn generate_css_injection_js_skips_invalid() {
        let vars = vec![
            ("--ok", "#fff", CssValueKind::Color),
            ("--bad", "expression(evil)", CssValueKind::Color),
        ];
        let js = generate_css_injection_js(&vars);

        assert!(js.contains("--ok"));
        assert!(!js.contains("--bad"));
        assert!(!js.contains("expression"));
    }

    #[test]
    fn generate_css_injection_js_escapes_quotes() {
        let vars = vec![(
            "--font-family",
            "'Menlo', monospace",
            CssValueKind::FontFamily,
        )];
        let js = generate_css_injection_js(&vars);
        // Single quotes in value should be escaped
        assert!(js.contains("\\'Menlo\\'"));
    }

    #[test]
    fn generate_xterm_theme_js_basic() {
        let theme = serde_json::json!({
            "xterm": {
                "background": "#0a0a0a",
                "foreground": "#c0c8d0"
            }
        });
        let js = generate_xterm_theme_js(&theme);
        assert!(js.contains("_dispatch('theme'"));
        assert!(js.contains("#0a0a0a"));
    }

    #[test]
    fn generate_xterm_theme_js_empty() {
        let theme = serde_json::json!({});
        let js = generate_xterm_theme_js(&theme);
        assert!(js.contains("_dispatch('theme', {})"));
    }
}
