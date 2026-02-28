//! Core config file watcher implementation.
//!
//! Contains the [`ConfigWatcher`] struct that monitors a config file
//! for changes using the `notify` crate, with debounced notifications.

use jarvis_common::ConfigError;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Watches a config file for changes and sends notifications.
pub struct ConfigWatcher {
    path: PathBuf,
}

impl ConfigWatcher {
    /// Create a new watcher for the given config file path.
    pub fn new(path: PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            warn!(
                "config file {} does not exist yet, will watch for creation",
                path.display()
            );
        }

        Ok(Self { path })
    }

    /// Watch the config file for changes, sending a signal on the broadcast channel.
    ///
    /// This function runs indefinitely. Changes are debounced with a 500ms window
    /// to avoid rapid reloads when editors do atomic save (write + rename).
    ///
    /// Sends `()` on the broadcast channel when a change is detected.
    pub async fn watch(&self, tx: broadcast::Sender<()>) -> Result<(), ConfigError> {
        let path = self.path.clone();
        let watch_path = if let Some(parent) = path.parent() {
            parent.to_path_buf()
        } else {
            path.clone()
        };

        let file_name = path
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();

        info!("starting config file watcher for {}", path.display());

        // Use a channel to bridge the sync notify callback into async
        let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel::<()>(16);

        let _watcher = {
            let file_name = file_name.clone();
            let notify_tx = notify_tx.clone();

            let mut watcher = RecommendedWatcher::new(
                move |result: Result<Event, notify::Error>| {
                    match result {
                        Ok(event) => {
                            let dominated =
                                matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_));
                            if !dominated {
                                return;
                            }

                            // Check if the changed file matches our config file
                            let is_our_file = event
                                .paths
                                .iter()
                                .any(|p| p.file_name().map(|n| n == file_name).unwrap_or(false));

                            if is_our_file {
                                debug!("config file change detected");
                                let _ = notify_tx.try_send(());
                            }
                        }
                        Err(e) => {
                            error!("file watcher error: {e}");
                        }
                    }
                },
                notify::Config::default(),
            )
            .map_err(|e| ConfigError::WatchError(format!("failed to create watcher: {e}")))?;

            watcher
                .watch(&watch_path, RecursiveMode::NonRecursive)
                .map_err(|e| {
                    ConfigError::WatchError(format!(
                        "failed to watch {}: {e}",
                        watch_path.display()
                    ))
                })?;

            // Keep the watcher alive by moving it into an Arc
            Arc::new(watcher)
        };

        // Keep a reference to prevent the watcher from being dropped
        let _watcher_ref = _watcher;

        // Debounce loop: wait for change signals, coalesce within 500ms
        loop {
            // Wait for the first change signal
            if notify_rx.recv().await.is_none() {
                // Channel closed, watcher dropped
                break;
            }

            // Debounce: drain any additional signals within 500ms
            let debounce = tokio::time::sleep(std::time::Duration::from_millis(500));
            tokio::pin!(debounce);

            loop {
                tokio::select! {
                    _ = &mut debounce => break,
                    msg = notify_rx.recv() => {
                        if msg.is_none() {
                            return Ok(());
                        }
                        // Reset debounce timer by continuing the loop
                    }
                }
            }

            info!("config file changed, sending reload signal");
            if tx.send(()).is_err() {
                debug!("no receivers for config reload signal");
            }
        }

        Ok(())
    }
}
