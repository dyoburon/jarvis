//! Tests for the config file watcher.

use super::*;
use std::path::PathBuf;

#[test]
fn watcher_new_with_nonexistent_path_succeeds() {
    // Watcher should be created even if the file doesn't exist yet
    let watcher = ConfigWatcher::new(PathBuf::from("/tmp/nonexistent_jarvis_test.toml"));
    assert!(watcher.is_ok());
}

#[test]
fn watcher_new_with_existing_path_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "# test").unwrap();

    let watcher = ConfigWatcher::new(path);
    assert!(watcher.is_ok());
}
