//! WebView bridge for embedding web content in Jarvis panes.
//!
//! Wraps the `wry` crate to provide:
//! - Managed WebView instances per pane
//! - Bidirectional IPC (Rust <-> JavaScript)
//! - Custom protocol for serving bundled content
//! - Navigation control (URL, HTML, back/forward)
//! - Event handling (page load, title change, navigation)

pub mod content;
pub mod events;
pub mod ipc;
pub mod manager;

pub use events::{PageLoadState, WebViewEvent};
pub use ipc::{IpcMessage, IpcPayload};
pub use manager::{WebViewConfig, WebViewHandle, WebViewManager};
