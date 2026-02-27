//! Boot sequence manager.
//!
//! Controls the application startup phases: splash screen, config loading,
//! and transition to the ready state.

mod sequence;
mod types;

pub use sequence::BootSequence;
pub use types::BootPhase;

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> jarvis_config::schema::JarvisConfig {
        jarvis_config::schema::JarvisConfig::default()
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
