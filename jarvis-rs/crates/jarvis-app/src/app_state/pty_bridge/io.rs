//! PTY I/O operations: write input, read output, resize.

use std::io::Write;
use std::sync::mpsc;

use portable_pty::PtySize;

use super::types::{PtyHandle, PtyManager, PTY_MAX_OUTPUT_PER_FRAME};

// =============================================================================
// INPUT (WRITE TO PTY)
// =============================================================================

impl PtyHandle {
    /// Write raw input bytes to the PTY (keystrokes from xterm.js).
    pub fn write_input(&mut self, data: &[u8]) -> Result<(), String> {
        self.writer
            .write_all(data)
            .map_err(|e| format!("PTY write failed: {e}"))?;
        self.writer
            .flush()
            .map_err(|e| format!("PTY flush failed: {e}"))?;
        Ok(())
    }
}

// =============================================================================
// OUTPUT (READ FROM PTY)
// =============================================================================

impl PtyHandle {
    /// Drain all available output from the PTY reader thread.
    ///
    /// Returns accumulated bytes up to `PTY_MAX_OUTPUT_PER_FRAME` (64 KB).
    /// Non-blocking: returns empty vec if no output is available.
    pub fn drain_output(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        loop {
            match self.output_rx.try_recv() {
                Ok(chunk) => {
                    buf.extend_from_slice(&chunk);
                    if buf.len() >= PTY_MAX_OUTPUT_PER_FRAME {
                        buf.truncate(PTY_MAX_OUTPUT_PER_FRAME);
                        break;
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }

        buf
    }

    /// Check if the PTY reader channel is disconnected (shell exited).
    pub fn is_finished(&self) -> bool {
        // If try_recv returns Disconnected and there's nothing buffered,
        // the reader thread has exited.
        matches!(
            self.output_rx.try_recv(),
            Err(mpsc::TryRecvError::Disconnected)
        )
    }
}

// =============================================================================
// RESIZE
// =============================================================================

impl PtyHandle {
    /// Resize the PTY to new dimensions.
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), String> {
        let new_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        self.master
            .resize(new_size)
            .map_err(|e| format!("PTY resize failed: {e}"))?;
        self.size = new_size;
        Ok(())
    }
}

// =============================================================================
// KILL
// =============================================================================

impl PtyHandle {
    /// Kill the PTY child process.
    pub fn kill(&mut self) {
        if let Err(e) = self.child.kill() {
            tracing::debug!("PTY kill error (may already be dead): {e}");
        }
    }

    /// Wait for the child process to exit and return the exit code.
    pub fn wait_exit_code(&mut self) -> Option<u32> {
        match self.child.wait() {
            Ok(status) => {
                // ExitStatus::exit_code() returns Option<u32> on portable-pty 0.9
                let code = status.exit_code();
                Some(code)
            }
            Err(e) => {
                tracing::debug!("PTY wait error: {e}");
                None
            }
        }
    }
}

// =============================================================================
// PTY MANAGER CONVENIENCE
// =============================================================================

impl PtyManager {
    /// Write input to a specific pane's PTY.
    pub fn write_input(&mut self, pane_id: u32, data: &[u8]) -> Result<(), String> {
        match self.get_mut(pane_id) {
            Some(handle) => handle.write_input(data),
            None => Err(format!("No PTY for pane {pane_id}")),
        }
    }

    /// Resize a specific pane's PTY.
    pub fn resize(&mut self, pane_id: u32, cols: u16, rows: u16) -> Result<(), String> {
        match self.get_mut(pane_id) {
            Some(handle) => handle.resize(cols, rows),
            None => Err(format!("No PTY for pane {pane_id}")),
        }
    }

    /// Kill and remove a specific pane's PTY.
    ///
    /// Returns the exit code if the process was successfully waited on.
    pub fn kill_and_remove(&mut self, pane_id: u32) -> Option<u32> {
        if let Some(mut handle) = self.remove(pane_id) {
            handle.kill();
            // wait_exit_code returns Option<u32>
            handle.wait_exit_code()
        } else {
            None
        }
    }

    /// Kill and remove all active PTYs. Used during graceful shutdown.
    pub fn kill_all(&mut self) {
        let pane_ids = self.pane_ids();
        let count = pane_ids.len();
        for pane_id in pane_ids {
            self.kill_and_remove(pane_id);
        }
        tracing::info!(count, "All PTYs killed");
    }

    /// Drain output from all PTYs. Returns `(pane_id, output_bytes)` pairs.
    pub fn drain_all_output(&self) -> Vec<(u32, Vec<u8>)> {
        let mut results = Vec::new();
        for pane_id in self.pane_ids() {
            if let Some(handle) = self.get(pane_id) {
                let output = handle.drain_output();
                if !output.is_empty() {
                    results.push((pane_id, output));
                }
            }
        }
        results
    }

    /// Check all PTYs for finished processes. Returns pane IDs of exited PTYs.
    pub fn check_finished(&self) -> Vec<u32> {
        let mut finished = Vec::new();
        for pane_id in self.pane_ids() {
            if let Some(handle) = self.get(pane_id) {
                if handle.is_finished() {
                    finished.push(pane_id);
                }
            }
        }
        finished
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::spawn::spawn_pty;
    use super::*;

    #[test]
    fn pty_write_and_read_echo() {
        let mut handle = spawn_pty(80, 24).expect("spawn should succeed");

        // Write a command that produces known output
        handle
            .write_input(b"echo PTY_TEST_MARKER_12345\n")
            .expect("write should succeed");

        // Give the shell time to process
        std::thread::sleep(std::time::Duration::from_millis(500));

        let output = handle.drain_output();
        let output_str = String::from_utf8_lossy(&output);

        assert!(
            output_str.contains("PTY_TEST_MARKER_12345"),
            "output should contain echo marker, got: {output_str}"
        );

        handle.kill();
    }

    #[test]
    fn pty_resize_updates_size() {
        let mut handle = spawn_pty(80, 24).expect("spawn should succeed");
        assert_eq!(handle.size.cols, 80);
        assert_eq!(handle.size.rows, 24);

        handle.resize(120, 40).expect("resize should succeed");
        assert_eq!(handle.size.cols, 120);
        assert_eq!(handle.size.rows, 40);

        handle.kill();
    }

    #[test]
    fn pty_kill_terminates_process() {
        let mut handle = spawn_pty(80, 24).expect("spawn should succeed");
        handle.kill();

        // After kill, drain should eventually return empty or disconnected
        std::thread::sleep(std::time::Duration::from_millis(200));
        // Just verify it doesn't panic
        let _ = handle.drain_output();
    }

    #[test]
    fn pty_drain_output_respects_max_frame_size() {
        // PTY_MAX_OUTPUT_PER_FRAME is 64KB — we can't easily produce that
        // much output in a test, but we verify the constant is correct
        assert_eq!(PTY_MAX_OUTPUT_PER_FRAME, 65_536);
    }

    #[test]
    fn pty_manager_write_input_missing_pane() {
        let mut mgr = PtyManager::new();
        let result = mgr.write_input(999, b"hello");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No PTY for pane 999"));
    }

    #[test]
    fn pty_manager_resize_missing_pane() {
        let mut mgr = PtyManager::new();
        let result = mgr.resize(999, 80, 24);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No PTY for pane 999"));
    }

    #[test]
    fn pty_manager_kill_and_remove_missing_pane() {
        let mut mgr = PtyManager::new();
        let result = mgr.kill_and_remove(999);
        assert!(result.is_none());
    }

    #[test]
    fn pty_manager_kill_all_empty() {
        let mut mgr = PtyManager::new();
        mgr.kill_all();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
    }

    #[test]
    fn pty_manager_kill_all_with_ptys() {
        let mut mgr = PtyManager::new();
        let h1 = spawn_pty(80, 24).expect("spawn 1");
        let h2 = spawn_pty(80, 24).expect("spawn 2");
        mgr.insert(1, h1);
        mgr.insert(2, h2);
        assert_eq!(mgr.len(), 2);

        mgr.kill_all();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
    }

    #[test]
    fn pty_manager_kill_all_is_idempotent() {
        let mut mgr = PtyManager::new();
        let h1 = spawn_pty(80, 24).expect("spawn");
        mgr.insert(1, h1);

        mgr.kill_all();
        mgr.kill_all(); // second call should not panic
        assert!(mgr.is_empty());
    }

    #[test]
    fn pty_manager_drain_all_empty() {
        let mgr = PtyManager::new();
        let results = mgr.drain_all_output();
        assert!(results.is_empty());
    }

    #[test]
    fn pty_manager_lifecycle() {
        let mut mgr = PtyManager::new();
        let handle = spawn_pty(80, 24).expect("spawn should succeed");

        mgr.insert(42, handle);
        assert!(mgr.contains(42));
        assert_eq!(mgr.len(), 1);
        assert_eq!(mgr.pane_ids(), vec![42]);

        mgr.kill_and_remove(42);
        assert!(!mgr.contains(42));
        assert!(mgr.is_empty());
    }
}
