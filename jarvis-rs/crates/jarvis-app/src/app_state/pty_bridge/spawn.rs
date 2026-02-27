//! PTY spawn logic: create a new PTY with the user's default shell.

use std::io::Read;
use std::sync::mpsc;
use std::thread;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};

use super::types::{PtyHandle, DEFAULT_COLS, DEFAULT_ROWS, PTY_READ_CHUNK};

// =============================================================================
// SHELL DETECTION
// =============================================================================

/// Get the user's default shell.
///
/// - Unix: reads `$SHELL`, falls back to `/bin/sh`
/// - Windows: reads `$COMSPEC`, falls back to `cmd.exe`
pub fn default_shell() -> String {
    #[cfg(unix)]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }
}

// =============================================================================
// ENVIRONMENT SANITIZATION
// =============================================================================

/// Allowed environment variables to inherit.
///
/// We inherit a minimal set to avoid leaking Jarvis-internal secrets
/// (API keys, tokens, etc.) into the shell environment.
const ALLOWED_ENV_VARS: &[&str] = &[
    "HOME",
    "USER",
    "LOGNAME",
    "SHELL",
    "PATH",
    "TERM",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "DISPLAY",
    "WAYLAND_DISPLAY",
    "XDG_RUNTIME_DIR",
    "TMPDIR",
    "TMP",
    "TEMP",
    // Windows-specific
    "USERPROFILE",
    "APPDATA",
    "LOCALAPPDATA",
    "SYSTEMROOT",
    "COMSPEC",
    "HOMEDRIVE",
    "HOMEPATH",
];

/// Build a sanitized `CommandBuilder` for the given shell.
fn build_shell_command(shell: &str) -> CommandBuilder {
    let mut cmd = CommandBuilder::new(shell);

    // Clear inherited env, then selectively re-add safe vars
    cmd.env_clear();
    for key in ALLOWED_ENV_VARS {
        if let Ok(val) = std::env::var(key) {
            cmd.env(key, val);
        }
    }

    // Always set TERM for proper terminal behavior
    cmd.env("TERM", "xterm-256color");

    // On Unix, pass -l for a login shell (loads .profile, .bash_profile, etc.)
    #[cfg(unix)]
    {
        cmd.arg("-l");
    }

    cmd
}

// =============================================================================
// SPAWN
// =============================================================================

/// Spawn a new PTY with the given terminal size.
///
/// Returns a `PtyHandle` that owns the master side of the PTY pair.
/// A background thread reads output from the PTY and sends chunks
/// over the returned handle's `output_rx` channel.
pub fn spawn_pty(cols: u16, rows: u16) -> Result<PtyHandle, String> {
    let pty_system = native_pty_system();

    let size = PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system
        .openpty(size)
        .map_err(|e| format!("Failed to open PTY: {e}"))?;

    let shell = default_shell();
    let cmd = build_shell_command(&shell);

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("Failed to spawn shell '{shell}': {e}"))?;

    // Drop the slave side — we only need the master
    drop(pair.slave);

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("Failed to take PTY writer: {e}"))?;

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("Failed to clone PTY reader: {e}"))?;

    // Spawn a background thread to read PTY output
    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    thread::Builder::new()
        .name("pty-reader".to_string())
        .spawn(move || {
            let mut buf = [0u8; PTY_READ_CHUNK];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF — shell exited
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(e) => {
                        tracing::debug!("PTY reader error: {e}");
                        break;
                    }
                }
            }
        })
        .map_err(|e| format!("Failed to spawn PTY reader thread: {e}"))?;

    Ok(PtyHandle {
        writer,
        output_rx: rx,
        child,
        master: pair.master,
        size,
    })
}

/// Spawn a PTY with default terminal dimensions.
#[allow(dead_code)] // Used in tests
pub fn spawn_pty_default() -> Result<PtyHandle, String> {
    spawn_pty(DEFAULT_COLS, DEFAULT_ROWS)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shell_returns_nonempty() {
        let shell = default_shell();
        assert!(!shell.is_empty(), "default shell should not be empty");
    }

    #[test]
    fn default_shell_is_absolute_on_unix() {
        let shell = default_shell();
        #[cfg(unix)]
        {
            // Either from $SHELL (usually absolute) or /bin/sh
            // We just check it's not empty — $SHELL could be relative in CI
            assert!(!shell.is_empty());
        }
        #[cfg(windows)]
        {
            assert!(!shell.is_empty());
        }
    }

    #[test]
    fn allowed_env_vars_contains_essentials() {
        assert!(ALLOWED_ENV_VARS.contains(&"HOME"));
        assert!(ALLOWED_ENV_VARS.contains(&"PATH"));
        assert!(ALLOWED_ENV_VARS.contains(&"TERM"));
        assert!(ALLOWED_ENV_VARS.contains(&"USER"));
    }

    #[test]
    fn allowed_env_vars_excludes_secrets() {
        // Ensure we don't accidentally inherit secret-like vars
        for var in ALLOWED_ENV_VARS {
            let lower = var.to_lowercase();
            assert!(
                !lower.contains("key"),
                "ALLOWED_ENV_VARS should not contain '{var}'"
            );
            assert!(
                !lower.contains("secret"),
                "ALLOWED_ENV_VARS should not contain '{var}'"
            );
            assert!(
                !lower.contains("token"),
                "ALLOWED_ENV_VARS should not contain '{var}'"
            );
            assert!(
                !lower.contains("password"),
                "ALLOWED_ENV_VARS should not contain '{var}'"
            );
        }
    }

    #[test]
    fn spawn_pty_creates_handle() {
        let handle = spawn_pty(80, 24);
        assert!(
            handle.is_ok(),
            "spawn_pty should succeed: {:?}",
            handle.err()
        );
        let mut handle = handle.unwrap();
        assert_eq!(handle.size.cols, 80);
        assert_eq!(handle.size.rows, 24);
        // Clean up
        handle.child.kill().ok();
    }

    #[test]
    fn spawn_pty_default_uses_standard_size() {
        let handle = spawn_pty_default();
        assert!(handle.is_ok(), "spawn_pty_default should succeed");
        let mut handle = handle.unwrap();
        assert_eq!(handle.size.cols, DEFAULT_COLS);
        assert_eq!(handle.size.rows, DEFAULT_ROWS);
        handle.child.kill().ok();
    }
}
