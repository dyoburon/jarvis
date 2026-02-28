//! Types for the boot screen renderer.

/// GPU uniforms for the boot screen shader.
///
/// Must match the `Uniforms` struct in `boot.wgsl` exactly.
/// Padded to 16-byte alignment.
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct BootUniforms {
    pub time: f32,
    pub progress: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub accent_r: f32,
    pub accent_g: f32,
    pub accent_b: f32,
    pub bg_r: f32,
    pub bg_g: f32,
    pub bg_b: f32,
    pub opacity: f32,
    pub _pad: f32,
}

/// Configuration for boot screen colors and messages.
///
/// Extracted from `JarvisConfig` at startup.
pub struct BootScreenConfig {
    /// Background color RGB (0.0–1.0).
    pub bg_color: [f32; 3],
    /// Accent color RGB (0.0–1.0) — brackets, scan line, progress bar.
    pub accent_color: [f32; 3],
    /// Muted text color RGB (0.0–1.0) — status messages, percentage.
    pub muted_color: [f32; 3],
    /// Border/track color RGB (0.0–1.0) — progress bar background.
    pub track_color: [f32; 3],
    /// Total boot duration in seconds.
    pub duration: f32,
    /// Seconds between status message changes.
    pub message_interval: f32,
    /// Status messages to cycle through.
    pub messages: Vec<String>,
}

impl Default for BootScreenConfig {
    fn default() -> Self {
        Self {
            // Ayu Mirage defaults
            bg_color: [0.122, 0.141, 0.188],    // #1F2430
            accent_color: [1.0, 0.8, 0.4],      // #FFCC66
            muted_color: [0.439, 0.478, 0.549], // #707A8C
            track_color: [0.090, 0.106, 0.141], // #171B24
            duration: 4.5,
            message_interval: 1.5,
            messages: default_messages(),
        }
    }
}

/// Default military/intelligence-style status messages.
fn default_messages() -> Vec<String> {
    [
        "PROVISIONING ARMAMENTS",
        "CALIBRATING SENSOR ARRAY",
        "ESTABLISHING SECURE CHANNELS",
        "INITIALIZING NEURAL INTERFACE",
        "DEPLOYING COUNTERMEASURES",
        "SYNCHRONIZING THREAT MATRIX",
        "LOADING TACTICAL OVERLAYS",
        "VERIFYING BIOMETRIC CLEARANCE",
        "ACTIVATING PERIMETER DEFENSE",
        "COMPILING INTELLIGENCE BRIEFS",
        "SCANNING FREQUENCY SPECTRUM",
        "ENGAGING QUANTUM ENCRYPTION",
        "BOOTSTRAPPING CORE SYSTEMS",
        "SYSTEM ONLINE",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_uniforms_size_is_48_bytes() {
        // 12 floats * 4 bytes = 48, aligned to 16
        assert_eq!(std::mem::size_of::<BootUniforms>(), 48);
    }

    #[test]
    fn default_config_has_14_messages() {
        let config = BootScreenConfig::default();
        assert_eq!(config.messages.len(), 14);
    }

    #[test]
    fn default_config_last_message_is_system_online() {
        let config = BootScreenConfig::default();
        assert_eq!(config.messages.last().unwrap(), "SYSTEM ONLINE");
    }

    #[test]
    fn default_config_duration_is_4_5() {
        let config = BootScreenConfig::default();
        assert!((config.duration - 4.5).abs() < f32::EPSILON);
    }
}
