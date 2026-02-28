//! Boot sequence phase definitions.

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
