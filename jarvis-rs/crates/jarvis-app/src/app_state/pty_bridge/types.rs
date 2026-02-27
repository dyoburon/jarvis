//! PTY bridge types: per-pane PTY handle and the manager that owns them all.

use std::collections::HashMap;
use std::io::Write;
use std::sync::mpsc;

use portable_pty::{Child, MasterPty, PtySize};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Maximum bytes to read from a PTY in a single poll (8 KB).
pub const PTY_READ_CHUNK: usize = 8_192;

/// Maximum bytes to send to a webview per frame (64 KB).
pub const PTY_MAX_OUTPUT_PER_FRAME: usize = 65_536;

/// Default terminal columns.
pub const DEFAULT_COLS: u16 = 80;

/// Default terminal rows.
pub const DEFAULT_ROWS: u16 = 24;

// =============================================================================
// PTY HANDLE
// =============================================================================

/// A single PTY instance bound to a pane.
///
/// Owns the master side of the PTY pair: a writer for input, a reader
/// thread that sends output chunks over an `mpsc` channel, and a child
/// process handle for lifecycle management.
pub struct PtyHandle {
    /// Writer to send input bytes to the PTY.
    pub(super) writer: Box<dyn Write + Send>,
    /// Receiver for output chunks from the reader thread.
    pub(super) output_rx: mpsc::Receiver<Vec<u8>>,
    /// Child process handle (for wait / kill).
    pub(super) child: Box<dyn Child + Send + Sync>,
    /// Master PTY handle (for resize).
    pub(super) master: Box<dyn MasterPty + Send>,
    /// Current terminal size.
    pub(super) size: PtySize,
}

// =============================================================================
// PTY MANAGER
// =============================================================================

/// Manages all PTY instances, keyed by pane ID.
pub struct PtyManager {
    /// Active PTY handles, one per terminal pane.
    handles: HashMap<u32, PtyHandle>,
}

impl PtyManager {
    /// Create an empty PTY manager.
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    /// Insert a PTY handle for a pane.
    pub fn insert(&mut self, pane_id: u32, handle: PtyHandle) {
        self.handles.insert(pane_id, handle);
    }

    /// Remove and return the PTY handle for a pane.
    pub fn remove(&mut self, pane_id: u32) -> Option<PtyHandle> {
        self.handles.remove(&pane_id)
    }

    /// Get a mutable reference to a PTY handle.
    pub fn get_mut(&mut self, pane_id: u32) -> Option<&mut PtyHandle> {
        self.handles.get_mut(&pane_id)
    }

    /// Get an immutable reference to a PTY handle.
    pub fn get(&self, pane_id: u32) -> Option<&PtyHandle> {
        self.handles.get(&pane_id)
    }

    /// Check if a pane has an active PTY.
    pub fn contains(&self, pane_id: u32) -> bool {
        self.handles.contains_key(&pane_id)
    }

    /// Return all pane IDs with active PTYs.
    pub fn pane_ids(&self) -> Vec<u32> {
        self.handles.keys().copied().collect()
    }

    /// Number of active PTYs.
    #[allow(dead_code)] // Used in tests and future UI
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// Whether there are no active PTYs.
    #[allow(dead_code)] // Used in tests and future UI
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pty_manager_insert_and_lookup() {
        let mgr = PtyManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert!(!mgr.contains(1));
    }

    #[test]
    fn pty_manager_default_is_empty() {
        let mgr = PtyManager::default();
        assert!(mgr.is_empty());
        assert_eq!(mgr.pane_ids(), Vec::<u32>::new());
    }

    #[test]
    fn pty_constants_are_sane() {
        assert_eq!(PTY_READ_CHUNK, 8_192);
        assert_eq!(PTY_MAX_OUTPUT_PER_FRAME, 65_536);
        assert_eq!(DEFAULT_COLS, 80);
        assert_eq!(DEFAULT_ROWS, 24);
    }
}
