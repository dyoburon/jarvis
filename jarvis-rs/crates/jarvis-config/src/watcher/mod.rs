//! File watcher for live config reload.
//!
//! Uses the `notify` crate to watch the config file for changes,
//! with a 500ms debounce to avoid rapid reloads.

mod config_watcher;

#[cfg(test)]
mod tests;

pub use config_watcher::ConfigWatcher;
