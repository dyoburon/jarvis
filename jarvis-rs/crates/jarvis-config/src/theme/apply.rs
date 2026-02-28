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
