//! Boot sequence manager.
//!
//! Controls the application startup phases: splash screen, config loading,
//! and transition to the ready state.

use std::time::Instant;

use jarvis_config::schema::JarvisConfig;

/// Boot sequence phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootPhase {
    /// Splash screen / loading animation.
    Splash,
    /// Config loaded, subsystems initializing.
    Initializing,
    /// Application ready for use.
    Ready,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> JarvisConfig {
        JarvisConfig::default()
    }

    #[test]
    fn fast_start_skips_to_ready() {
        let mut config = default_config();
        config.startup.fast_start.enabled = true;
        let boot = BootSequence::new(&config);
        assert!(boot.is_ready());
        assert_eq!(boot.phase(), BootPhase::Ready);
    }

    #[test]
    fn animation_disabled_skips_to_ready() {
        let mut config = default_config();
        config.startup.boot_animation.enabled = false;
        let boot = BootSequence::new(&config);
        assert!(boot.is_ready());
    }

    #[test]
    fn skip_transitions_to_ready() {
        let mut config = default_config();
        config.startup.boot_animation.enabled = true;
        config.startup.fast_start.enabled = false;
        config.startup.boot_animation.duration = 10.0; // long splash
        let mut boot = BootSequence::new(&config);
        assert_eq!(boot.phase(), BootPhase::Splash);

        boot.skip();
        assert!(boot.is_ready());
    }

    #[test]
    fn progress_starts_at_zero() {
        let mut config = default_config();
        config.startup.boot_animation.enabled = true;
        config.startup.fast_start.enabled = false;
        config.startup.boot_animation.duration = 100.0;
        let boot = BootSequence::new(&config);
        // Progress should be very close to 0 right after creation
        assert!(boot.progress() < 0.1);
    }

    #[test]
    fn progress_is_one_when_ready() {
        let config = default_config();
        let boot = BootSequence::new(&config);
        if boot.is_ready() {
            assert!((boot.progress() - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn elapsed_increases() {
        let config = default_config();
        let boot = BootSequence::new(&config);
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(boot.elapsed_secs() > 0.0);
    }
}
