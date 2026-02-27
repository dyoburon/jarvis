//! Bridge between the tiling engine and webview panels.
//!
//! Handles coordinate conversion, IPC message dispatch, and
//! synchronizing webview bounds to tiling layout rects.

mod bounds;
mod ipc_dispatch;
mod lifecycle;
mod pty_handlers;
mod pty_polling;
