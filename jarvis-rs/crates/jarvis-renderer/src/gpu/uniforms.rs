//! GPU uniform buffer types shared across shader passes.
//!
//! `GpuUniforms` is the single uniform block uploaded each frame.
//! All shaders (background, sphere, bloom, composite) read from it.

use jarvis_config::schema::JarvisConfig;

use super::super::background::hex_to_rgb;

/// GPU-side uniform buffer matching the WGSL `Uniforms` struct.
///
/// Layout: 20 × f32 = 80 bytes, 16-byte aligned (wgpu requirement).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuUniforms {
    /// Elapsed time in seconds (wraps at ~6 hours to avoid precision loss).
    pub time: f32,
    /// Audio input level (0.0 = silence, 1.0 = peak).
    pub audio_level: f32,
    /// Orb power/energy level (0.0-1.0).
    pub power_level: f32,
    /// Global intensity multiplier.
    pub intensity: f32,

    /// CRT scanline darkness (0.0 = off, 1.0 = full).
    pub scanline_intensity: f32,
    /// Vignette edge darkening strength.
    pub vignette_intensity: f32,
    /// Viewport width in pixels.
    pub screen_width: f32,
    /// Viewport height in pixels.
    pub screen_height: f32,

    /// Viewport aspect ratio (width / height).
    pub aspect_ratio: f32,
    /// Orb center X in NDC (-1..1).
    pub orb_center_x: f32,
    /// Orb center Y in NDC (-1..1).
    pub orb_center_y: f32,
    /// Orb scale multiplier.
    pub orb_scale: f32,

    /// Background opacity (0.0-1.0).
    pub bg_opacity: f32,
    /// Background alpha (0.0-1.0).
    pub bg_alpha: f32,
    /// Hex grid color — red channel.
    pub hex_color_r: f32,
    /// Hex grid color — green channel.
    pub hex_color_g: f32,

    /// Hex grid color — blue channel.
    pub hex_color_b: f32,
    /// Brightness flicker amplitude.
    pub flicker_amplitude: f32,
    /// Padding to reach 80 bytes (16-byte alignment).
    pub _padding: [f32; 2],
}

impl GpuUniforms {
    /// Create uniforms from application config with default runtime values.
    ///
    /// Runtime-varying fields (`time`, `audio_level`, `power_level`) start at
    /// zero and are updated each frame via [`Self::update_time`].
    pub fn from_config(config: &JarvisConfig) -> Self {
        let [hex_r, hex_g, hex_b] = hex_to_rgb(&config.background.hex_grid.color)
            .map(|[r, g, b]| [r as f32, g as f32, b as f32])
            .unwrap_or([0.0, 0.83, 1.0]);

        Self {
            time: 0.0,
            audio_level: 0.0,
            power_level: 0.0,
            intensity: config.visualizer.state_listening.intensity as f32,
            scanline_intensity: config.effects.scanlines.intensity,
            vignette_intensity: config.effects.vignette.intensity,
            screen_width: 0.0,
            screen_height: 0.0,
            aspect_ratio: 1.0,
            orb_center_x: 0.0,
            orb_center_y: 0.0,
            orb_scale: config.visualizer.state_listening.scale as f32,
            bg_opacity: config.background.hex_grid.opacity as f32,
            bg_alpha: config.opacity.background as f32,
            hex_color_r: hex_r,
            hex_color_g: hex_g,
            hex_color_b: hex_b,
            flicker_amplitude: config.effects.flicker.amplitude,
            _padding: [0.0; 2],
        }
    }

    /// Update per-frame time. Wraps at ~6 hours to avoid f32 precision loss.
    pub fn update_time(&mut self, dt: f32) {
        self.time = (self.time + dt) % 21600.0;
    }

    /// Update viewport dimensions and recompute aspect ratio.
    pub fn update_viewport(&mut self, width: u32, height: u32) {
        self.screen_width = width as f32;
        self.screen_height = height as f32;
        self.aspect_ratio = if height > 0 {
            width as f32 / height as f32
        } else {
            1.0
        };
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniforms_size_is_80_bytes() {
        assert_eq!(std::mem::size_of::<GpuUniforms>(), 80);
    }

    #[test]
    fn uniforms_alignment_is_4_bytes() {
        assert_eq!(std::mem::align_of::<GpuUniforms>(), 4);
    }

    #[test]
    fn uniforms_from_default_config() {
        let config = JarvisConfig::default();
        let u = GpuUniforms::from_config(&config);
        assert!((u.time - 0.0).abs() < f32::EPSILON);
        assert!((u.audio_level - 0.0).abs() < f32::EPSILON);
        assert!((u.scanline_intensity - 0.08).abs() < f32::EPSILON);
        assert!((u.vignette_intensity - 1.2).abs() < f32::EPSILON);
        assert!((u.bg_opacity - 0.08).abs() < f32::EPSILON);
        assert!((u.flicker_amplitude - 0.004).abs() < f32::EPSILON);
        // Default hex grid color is "#00d4ff" → r≈0, g≈0.83, b≈1.0
        assert!(u.hex_color_r < 0.01);
        assert!(u.hex_color_g > 0.8);
        assert!(u.hex_color_b > 0.99);
    }

    #[test]
    fn uniforms_from_config_red_hex_grid() {
        let mut config = JarvisConfig::default();
        config.background.hex_grid.color = "#ff0000".into();
        let u = GpuUniforms::from_config(&config);
        assert!((u.hex_color_r - 1.0).abs() < 1e-3);
        assert!((u.hex_color_g - 0.0).abs() < 1e-3);
        assert!((u.hex_color_b - 0.0).abs() < 1e-3);
    }

    #[test]
    fn uniforms_from_config_invalid_hex_uses_fallback() {
        let mut config = JarvisConfig::default();
        config.background.hex_grid.color = "not-a-color".into();
        let u = GpuUniforms::from_config(&config);
        // Fallback: cyan-ish (0.0, 0.83, 1.0)
        assert!((u.hex_color_r - 0.0).abs() < 1e-3);
        assert!(u.hex_color_g > 0.8);
        assert!(u.hex_color_b > 0.99);
    }

    #[test]
    fn update_time_wraps() {
        let mut u = GpuUniforms::from_config(&JarvisConfig::default());
        u.time = 21599.0;
        u.update_time(2.0);
        // Should wrap: (21599 + 2) % 21600 = 1.0
        assert!((u.time - 1.0).abs() < 1e-3);
    }

    #[test]
    fn update_viewport_computes_aspect_ratio() {
        let mut u = GpuUniforms::from_config(&JarvisConfig::default());
        u.update_viewport(1920, 1080);
        assert!((u.screen_width - 1920.0).abs() < f32::EPSILON);
        assert!((u.screen_height - 1080.0).abs() < f32::EPSILON);
        assert!((u.aspect_ratio - (1920.0 / 1080.0)).abs() < 1e-4);
    }

    #[test]
    fn update_viewport_zero_height_gives_aspect_one() {
        let mut u = GpuUniforms::from_config(&JarvisConfig::default());
        u.update_viewport(800, 0);
        assert!((u.aspect_ratio - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bytemuck_cast_works() {
        let u = GpuUniforms::from_config(&JarvisConfig::default());
        let bytes: &[u8] = bytemuck::bytes_of(&u);
        assert_eq!(bytes.len(), 80);
    }
}
