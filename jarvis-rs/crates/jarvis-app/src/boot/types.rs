//! Boot sequence phase definitions.

/// Boot sequence phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootPhase {
    /// Splash screen / loading animation.
    Splash,
    /// Config loaded, subsystems initializing (`BootSequence::update` transitions into this after splash).
    #[allow(dead_code)]
    Initializing,
    /// Application ready for use.
    Ready,
}
