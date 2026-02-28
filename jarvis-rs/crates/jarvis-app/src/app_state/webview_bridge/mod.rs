//! Bridge between the tiling engine and webview panels.
//!
//! Handles coordinate conversion, IPC message dispatch, and
//! synchronizing webview bounds to tiling layout rects.

mod assistant_handlers;
mod bounds;
mod crypto_handlers;
mod ipc_dispatch;
mod lifecycle;
mod presence_handlers;
mod pty_handlers;
mod pty_polling;
mod settings_handlers;
mod status_bar_handlers;
mod theme_handlers;
