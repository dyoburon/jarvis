//! Validation for smaller config sections: startup, voice, performance,
//! livechat, presence, updates, and logging.

use crate::schema::JarvisConfig;

use super::helpers::{validate_range, validate_range_f64};

/// Validate startup constraints.
pub(crate) fn validate_startup(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range(
        errors,
        "startup.on_ready.panels.count",
        config.startup.on_ready.panels.count,
        1,
        5,
    );
}

/// Validate voice constraints.
pub(crate) fn validate_voice(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range_f64(
        errors,
        "voice.sounds.volume",
        config.voice.sounds.volume,
        0.0,
        1.0,
    );
}

/// Validate performance constraints.
pub(crate) fn validate_performance(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range(
        errors,
        "performance.frame_rate",
        config.performance.frame_rate,
        30,
        120,
    );
    validate_range(
        errors,
        "performance.bloom_passes",
        config.performance.bloom_passes,
        1,
        4,
    );
}

/// Validate livechat constraints.
pub(crate) fn validate_livechat(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range(
        errors,
        "livechat.server_port",
        config.livechat.server_port,
        1024,
        65535,
    );
    validate_range(
        errors,
        "livechat.connection_timeout",
        config.livechat.connection_timeout,
        5,
        60,
    );
    validate_range(
        errors,
        "livechat.nickname.validation.min_length",
        config.livechat.nickname.validation.min_length,
        1,
        10,
    );
    validate_range(
        errors,
        "livechat.nickname.validation.max_length",
        config.livechat.nickname.validation.max_length,
        5,
        50,
    );
    validate_range(
        errors,
        "livechat.automod.rate_limit",
        config.livechat.automod.rate_limit,
        1,
        20,
    );
    validate_range(
        errors,
        "livechat.automod.max_message_length",
        config.livechat.automod.max_message_length,
        100,
        2000,
    );
}

/// Validate presence constraints.
pub(crate) fn validate_presence(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range(
        errors,
        "presence.heartbeat_interval",
        config.presence.heartbeat_interval,
        10,
        300,
    );
}

/// Validate updates constraints.
pub(crate) fn validate_updates(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range(
        errors,
        "updates.check_interval",
        config.updates.check_interval,
        3600,
        604800,
    );
}

/// Validate logging constraints.
pub(crate) fn validate_logging(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range(
        errors,
        "logging.max_file_size_mb",
        config.logging.max_file_size_mb,
        1,
        50,
    );
    validate_range(
        errors,
        "logging.backup_count",
        config.logging.backup_count,
        1,
        10,
    );
}
