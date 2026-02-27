//! Top-level application state.
//!
//! Implements `winit::application::ApplicationHandler` to drive the main
//! event loop. Coordinates config, renderer, terminal, tiling, and input.

mod assistant;
mod assistant_task;
mod core;
mod dispatch;
mod event_handler;
mod init;
mod palette;
mod polling;
mod render;
mod social;
mod terminal;
mod types;
mod ui_state;

pub use core::JarvisApp;
