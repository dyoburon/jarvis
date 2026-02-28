//! Theme bridge: CSS generation and sanitization for webview theming.
//!
//! Converts theme variables into safe CSS for injection into webview panels.
//! All CSS values are validated to prevent CSS injection attacks.

mod generate;
mod sanitize;

pub use generate::{
    generate_css_injection_js, generate_css_root, generate_xterm_theme_js, CssValueKind,
    CssVariable,
};
pub use sanitize::{validate_css_color, validate_css_font_family, validate_css_numeric};
