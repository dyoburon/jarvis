//! PTY management using the `portable-pty` crate.
//!
//! Provides [`PtyManager`] for spawning a shell inside a pseudo-terminal,
//! reading/writing data, resizing, and lifecycle management.

mod manager;
mod types;

pub use manager::*;
pub use types::*;

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
