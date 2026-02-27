//! PTY management using the `portable-pty` crate.
//!
//! Provides [`PtyManager`] for spawning a shell inside a pseudo-terminal,
//! reading/writing data, resizing, and lifecycle management.

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::mpsc;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors originating from PTY operations.
#[derive(Debug, thiserror::Error)]
pub enum PtyError {
    #[error("failed to spawn process: {0}")]
    SpawnFailed(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("failed to resize PTY: {0}")]
    ResizeFailed(String),
}

// ---------------------------------------------------------------------------
// PtyManager
// ---------------------------------------------------------------------------

/// Owns a pseudo-terminal pair (master + spawned child process) and exposes
/// helpers for reading, writing, resizing, and lifecycle queries.
///
/// PTY output is read on a background thread and buffered in a channel,
/// so [`PtyManager::read`] is always non-blocking.
pub struct PtyManager {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    rx: mpsc::Receiver<Vec<u8>>,
    /// Leftover bytes from a previous channel receive that did not fit in the
    /// caller's buffer. Drained first on the next [`PtyManager::read`] call.
    pending: Vec<u8>,
}

impl PtyManager {
    /// Spawn a new shell process inside a PTY of the given size.
    ///
    /// `env` is an optional list of extra environment variables to set.
    /// `TERM` is always set to `xterm-256color`.
    pub fn spawn(
        shell: &str,
        cols: u16,
        rows: u16,
        env: Option<Vec<(String, String)>>,
    ) -> Result<Self, PtyError> {
        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");

        if let Some(vars) = env {
            for (key, value) in vars {
                cmd.env(key, value);
            }
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        // Spawn a background thread to read from the PTY into a channel.
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("pty-reader".into())
            .spawn(move || {
                let mut buf = [0u8; 8192];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if tx.send(buf[..n].to_vec()).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        Ok(PtyManager {
            master: pair.master,
            writer,
            child,
            rx,
            pending: Vec::new(),
        })
    }

    /// Read bytes produced by the child process (non-blocking).
    ///
    /// Returns the number of bytes placed into `buf`, or `0` if no data
    /// is currently available.  Data that does not fit in `buf` is kept
    /// in an internal buffer and returned on the next call.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, PtyError> {
        // Serve leftover bytes from a previous oversized receive first.
        if !self.pending.is_empty() {
            let n = self.pending.len().min(buf.len());
            buf[..n].copy_from_slice(&self.pending[..n]);
            self.pending.drain(..n);
            return Ok(n);
        }

        match self.rx.try_recv() {
            Ok(data) => {
                let n = data.len().min(buf.len());
                buf[..n].copy_from_slice(&data[..n]);
                if data.len() > buf.len() {
                    self.pending.extend_from_slice(&data[buf.len()..]);
                }
                Ok(n)
            }
            Err(mpsc::TryRecvError::Empty) => Ok(0),
            Err(mpsc::TryRecvError::Disconnected) => Ok(0),
        }
    }

    /// Write data (typically user keystrokes) into the PTY so the child
    /// process receives it as stdin.
    pub fn write(&mut self, data: &[u8]) -> Result<(), PtyError> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Inform the kernel (and thus the child) that the terminal size changed.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), PtyError> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::ResizeFailed(e.to_string()))
    }

    /// Returns `true` if the child process is still running.
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Send SIGKILL (or platform equivalent) to the child process.
    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }

    /// Block until the child process exits, returning its exit status.
    pub fn wait(&mut self) -> Option<portable_pty::ExitStatus> {
        self.child.wait().ok()
    }
}

impl Drop for PtyManager {
    fn drop(&mut self) {
        // Kill the child process so the PTY fd closes and the reader thread
        // exits naturally.  Errors are intentionally ignored — the process may
        // have already exited.
        let _ = self.child.kill();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    #[cfg(unix)]
    fn test_spawn_and_echo() {
        // Spawn /bin/echo which prints "hello" and exits immediately.
        // Spawn a shell, write a command, and read back the output.
        let mut mgr = PtyManager::spawn("/bin/sh", 80, 24, None).expect("spawn sh");

        // Write a command that echoes "hello" and exits.
        mgr.write(b"echo hello\n").expect("write");
        mgr.write(b"exit\n").expect("write exit");

        let mut output = String::new();
        let mut buf = [0u8; 4096];
        let deadline = Instant::now() + Duration::from_secs(5);

        while Instant::now() < deadline {
            match mgr.read(&mut buf) {
                Ok(0) => {
                    // Non-blocking: no data yet, sleep briefly and retry
                    std::thread::sleep(Duration::from_millis(10));
                }
                Ok(n) => {
                    output.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if output.contains("hello") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        assert!(
            output.contains("hello"),
            "expected 'hello' in output, got: {output:?}"
        );
    }
}
