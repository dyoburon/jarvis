//! Layout configuration validation (gaps, borders, padding, panels, scrollbar).

use crate::schema::JarvisConfig;

use super::helpers::{validate_range, validate_range_f64};

/// Validate all layout-related constraints.
pub(crate) fn validate_layout(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range(errors, "layout.panel_gap", config.layout.panel_gap, 0, 20);
    validate_range(
        errors,
        "layout.border_radius",
        config.layout.border_radius,
        0,
        20,
    );
    validate_range(errors, "layout.padding", config.layout.padding, 0, 40);
    validate_range(errors, "layout.max_panels", config.layout.max_panels, 1, 10);
    validate_range_f64(
        errors,
        "layout.default_panel_width",
        config.layout.default_panel_width,
        0.3,
        1.0,
    );
    validate_range(
        errors,
        "layout.scrollbar_width",
        config.layout.scrollbar_width,
        1,
        10,
    );
}
