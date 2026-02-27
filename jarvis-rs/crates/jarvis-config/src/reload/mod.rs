//! Live config reload manager.
//!
//! Combines the file watcher with config loading to provide automatic
//! config reloading when the config file changes on disk.

mod manager;

#[cfg(test)]
mod tests;

pub use manager::ReloadManager;
