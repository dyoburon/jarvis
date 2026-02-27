//! Boot sequence timing and phase transitions.

use std::time::Instant;

use jarvis_config::schema::JarvisConfig;

use super::types::BootPhase;

/// Manages the boot sequence timing and phase transitions.
pub struct BootSequence {
    start_time: Instant,
    phase: BootPhase,
    skip_requested: bool,
    splash_duration: f64,
}

impl BootSequence {
    /// Create a new boot sequence. If boot animation is disabled or fast_start
    /// is enabled in config, skips directly to [`BootPhase::Ready`].
    pub fn new(config: &JarvisConfig) -> Self {
        let splash_duration = config.startup.boot_animation.duration;
        let skip = !config.startup.boot_animation.enabled || config.startup.fast_start.enabled;

        Self {
            start_time: Instant::now(),
            phase: if skip {
                BootPhase::Ready
            } else {
                BootPhase::Splash
            },
            skip_requested: false,
            splash_duration,
        }
    }

    /// Skip the splash screen immediately.
    pub fn skip(&mut self) {
        if self.phase == BootPhase::Splash {
            self.skip_requested = true;
            self.phase = BootPhase::Ready;
        }
    }

    /// Advance the boot sequence based on elapsed time.
    pub fn update(&mut self) {
        if self.phase == BootPhase::Splash
            && !self.skip_requested
            && self.start_time.elapsed().as_secs_f64() >= self.splash_duration
        {
            self.phase = BootPhase::Initializing;
        }
        if self.phase == BootPhase::Initializing {
            // Transition to Ready once subsystems are initialized.
            // In the current implementation this is immediate since init
            // happens synchronously in `resumed()`.
            self.phase = BootPhase::Ready;
        }
    }

    /// Current phase.
    pub fn phase(&self) -> BootPhase {
        self.phase
    }

    /// Whether the boot sequence is complete.
    pub fn is_ready(&self) -> bool {
        self.phase == BootPhase::Ready
    }

    /// Progress through the splash screen (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        if self.phase != BootPhase::Splash || self.splash_duration <= 0.0 {
            return 1.0;
        }
        (self.start_time.elapsed().as_secs_f64() / self.splash_duration).min(1.0)
    }

    /// Time elapsed since boot started.
    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}
