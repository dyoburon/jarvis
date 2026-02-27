//! Theme application and merging.
//!
//! Applies [`ThemeOverrides`] to a [`JarvisConfig`], merging only the
//! fields that are present in the theme.

use super::types::ThemeOverrides;
use crate::schema::{ColorConfig, JarvisConfig};

/// Apply theme overrides to a config, merging only the fields that are present.
pub fn apply_theme(config: &mut JarvisConfig, theme: &ThemeOverrides) {
    // Apply color overrides
    if let Some(ref colors) = theme.colors {
        apply_color_overrides(&mut config.colors, colors);
    }

    // Apply font overrides
    if let Some(ref font) = theme.font {
        if let Some(ref family) = font.family {
            config.font.family = family.clone();
        }
        if let Some(size) = font.size {
            config.font.size = size;
        }
        if let Some(title_size) = font.title_size {
            config.font.title_size = title_size;
        }
        if let Some(line_height) = font.line_height {
            config.font.line_height = line_height;
        }
    }

    // Apply visualizer overrides
    if let Some(ref viz) = theme.visualizer {
        if let Some(ref color) = viz.orb_color {
            config.visualizer.orb.color = color.clone();
        }
        if let Some(ref color) = viz.orb_secondary_color {
            config.visualizer.orb.secondary_color = color.clone();
        }
    }

    // Apply background overrides
    if let Some(ref bg) = theme.background {
        if let Some(ref color) = bg.hex_grid_color {
            config.background.hex_grid.color = color.clone();
        }
        if let Some(ref color) = bg.solid_color {
            config.background.solid_color = color.clone();
        }
    }

    // Apply effects overrides
    if let Some(ref fx) = theme.effects {
        if let Some(v) = fx.scanline_intensity {
            config.effects.scanlines.intensity = v;
        }
        if let Some(v) = fx.vignette_intensity {
            config.effects.vignette.intensity = v;
        }
        if let Some(v) = fx.bloom_intensity {
            config.effects.bloom.intensity = v;
        }
        if let Some(ref color) = fx.glow_color {
            config.effects.glow.color = color.clone();
        }
        if let Some(v) = fx.glow_width {
            config.effects.glow.width = v;
        }
    }

    // Apply terminal overrides
    if let Some(ref term) = theme.terminal {
        if let Some(ref style) = term.cursor_style {
            if let Ok(parsed) =
                serde_json::from_str::<crate::schema::CursorStyle>(&format!("\"{style}\""))
            {
                config.terminal.cursor_style = parsed;
            }
        }
        if let Some(blink) = term.cursor_blink {
            config.terminal.cursor_blink = blink;
        }
    }

    // Apply window overrides
    if let Some(ref win) = theme.window {
        if let Some(opacity) = win.opacity {
            config.window.opacity = opacity;
        }
        if let Some(blur) = win.blur {
            config.window.blur = blur;
        }
    }

    // Apply extended font overrides
    if let Some(ref font) = theme.font {
        if let Some(nerd) = font.nerd_font {
            config.font.nerd_font = nerd;
        }
        if let Some(lig) = font.ligatures {
            config.font.ligatures = lig;
        }
        if let Some(w) = font.font_weight {
            config.font.font_weight = w;
        }
        if let Some(w) = font.bold_weight {
            config.font.bold_weight = w;
        }
    }
}

/// Replace color config fields with theme colors.
/// Since the theme provides a full ColorConfig via serde defaults, we only
/// override if the theme author actually specified values. We do this by
/// replacing the entire colors block when present.
fn apply_color_overrides(target: &mut ColorConfig, source: &ColorConfig) {
    target.primary = source.primary.clone();
    target.secondary = source.secondary.clone();
    target.background = source.background.clone();
    target.panel_bg = source.panel_bg.clone();
    target.text = source.text.clone();
    target.text_muted = source.text_muted.clone();
    target.border = source.border.clone();
    target.border_focused = source.border_focused.clone();
    target.user_text = source.user_text.clone();
    target.tool_read = source.tool_read.clone();
    target.tool_edit = source.tool_edit.clone();
    target.tool_write = source.tool_write.clone();
    target.tool_run = source.tool_run.clone();
    target.tool_search = source.tool_search.clone();
    target.success = source.success.clone();
    target.warning = source.warning.clone();
    target.error = source.error.clone();
}
