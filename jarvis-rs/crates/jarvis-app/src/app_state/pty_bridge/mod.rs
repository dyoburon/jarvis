//! PTY bridge: connects xterm.js webview panels to real shell processes.
//!
//! Uses `portable-pty` for cross-platform PTY spawning. Each terminal pane
//! gets its own PTY with a background reader thread. Input flows from
//! xterm.js → IPC → PTY writer. Output flows from PTY reader → IPC → xterm.js.

mod io;
mod spawn;
mod types;

pub use spawn::spawn_pty;
pub use types::{PtyManager, DEFAULT_COLS, DEFAULT_ROWS};
