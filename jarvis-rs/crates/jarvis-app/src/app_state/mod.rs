//! Top-level application state.
//!
//! Implements `winit::application::ApplicationHandler` to drive the main
//! event loop. Coordinates config, renderer, webview panels, tiling, and input.

mod assistant;
mod assistant_task;
mod core;
mod dispatch;
mod event_handler;
mod init;
mod palette;
mod polling;
mod social;
mod types;
mod ui_state;
mod webview_bridge;

pub use core::JarvisApp;
