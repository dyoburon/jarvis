//! Full configuration validation.
//!
//! Validates all numeric ranges, keybind uniqueness, and color formats.
//! Each domain has its own submodule; this orchestrator calls them all
//! and collects errors into a single `ConfigError`.

mod background;
mod font;
mod helpers;
mod layout;
mod misc;
mod opacity;
mod visualizer;
mod visualizer_effects;

#[cfg(test)]
mod tests;

use crate::keybinds;
use crate::schema::JarvisConfig;
use jarvis_common::ConfigError;

/// Run all validations on a config, collecting all errors.
pub fn validate(config: &JarvisConfig) -> Result<(), ConfigError> {
    let mut errors: Vec<String> = Vec::new();

    // Keybind duplicates
    if let Err(e) = keybinds::validate_no_duplicates(&config.keybinds) {
        errors.push(e.to_string());
    }

    font::validate_font(&mut errors, config);
    layout::validate_layout(&mut errors, config);
    opacity::validate_opacity(&mut errors, config);
    background::validate_background(&mut errors, config);
    visualizer::validate_visualizer(&mut errors, config);
    misc::validate_startup(&mut errors, config);
    misc::validate_voice(&mut errors, config);
    misc::validate_performance(&mut errors, config);
    misc::validate_livechat(&mut errors, config);
    misc::validate_presence(&mut errors, config);
    misc::validate_updates(&mut errors, config);
    misc::validate_logging(&mut errors, config);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::ValidationError(errors.join("; ")))
    }
}
