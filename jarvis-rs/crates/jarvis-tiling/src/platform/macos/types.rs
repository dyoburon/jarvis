//! macOS WindowManager types and constructors.

/// macOS window manager backed by Core Graphics and osascript.
pub struct MacOsWindowManager {
    /// Minimum window layer to consider (0 = normal windows).
    pub(super) min_layer: i32,
}

impl MacOsWindowManager {
    pub fn new() -> Self {
        Self { min_layer: 0 }
    }
}

impl Default for MacOsWindowManager {
    fn default() -> Self {
        Self::new()
    }
}
