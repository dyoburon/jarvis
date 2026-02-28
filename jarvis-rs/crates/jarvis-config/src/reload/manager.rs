//! Core reload manager implementation.
//!
//! Contains the [`ReloadManager`] struct and its methods for loading,
//! watching, and reloading configuration from disk.

use crate::schema::JarvisConfig;
use crate::theme;
use crate::toml_loader;
use crate::validation;
use crate::watcher::ConfigWatcher;
use std::path::PathBuf;
use tokio::sync::{broadcast, watch};
use tracing::{error, info, warn};

/// Manages live config reloading.
///
/// Watches the config file for changes and publishes new configs
/// via a [`tokio::sync::watch`] channel.
pub struct ReloadManager {
    config_path: PathBuf,
}

impl ReloadManager {
    /// Load the initial config from the given path and start watching for changes.
    ///
    /// Returns the initial config and a watch receiver that will receive
    /// updated configs whenever the file changes on disk.
    ///
    /// The watcher runs in a background task. If the config path does not exist,
    /// it will be created with defaults.
    pub async fn start(config_path: PathBuf) -> (JarvisConfig, watch::Receiver<JarvisConfig>) {
        // Load initial config
        let initial_config = match toml_loader::load_from_path(&config_path) {
            Ok(mut config) => {
                // Apply theme if set
                if config.theme.name != "jarvis-dark" {
                    match theme::load_theme(&config.theme.name) {
                        Ok(theme_overrides) => {
                            theme::apply_theme(&mut config, &theme_overrides);
                        }
                        Err(e) => {
                            warn!("failed to load theme '{}': {e}", config.theme.name);
                        }
                    }
                }
                config
            }
            Err(e) => {
                warn!("failed to load config: {e}, using defaults");
                JarvisConfig::default()
            }
        };

        let (config_tx, config_rx) = watch::channel(initial_config.clone());

        // Spawn the watcher task
        let watch_path = config_path.clone();
        tokio::spawn(async move {
            let manager = ReloadManager {
                config_path: watch_path,
            };
            manager.run_watch_loop(config_tx).await;
        });

        (initial_config, config_rx)
    }

    /// Internal watch loop that reloads config on file changes.
    async fn run_watch_loop(&self, config_tx: watch::Sender<JarvisConfig>) {
        let watcher = match ConfigWatcher::new(self.config_path.clone()) {
            Ok(w) => w,
            Err(e) => {
                error!("failed to create config watcher: {e}");
                return;
            }
        };

        let (change_tx, mut change_rx) = broadcast::channel::<()>(16);

        // Spawn the file watcher
        let _watcher_path = self.config_path.clone();
        tokio::spawn(async move {
            if let Err(e) = watcher.watch(change_tx).await {
                error!("config watcher error: {e}");
            }
        });

        // Listen for change signals and reload
        loop {
            match change_rx.recv().await {
                Ok(()) => {
                    info!("reloading config from {}", self.config_path.display());
                    match self.reload_config() {
                        Ok(config) => {
                            if config_tx.send(config).is_err() {
                                info!("all config receivers dropped, stopping reload manager");
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("config reload failed: {e}");
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("config watcher lagged by {n} events");
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("config watcher channel closed");
                    break;
                }
            }
        }
    }

    /// Reload config from disk, applying theme and validation.
    fn reload_config(&self) -> Result<JarvisConfig, jarvis_common::ConfigError> {
        let mut config = toml_loader::load_from_path(&self.config_path)?;

        // Apply theme
        if config.theme.name != "jarvis-dark" {
            match theme::load_theme(&config.theme.name) {
                Ok(theme_overrides) => {
                    theme::apply_theme(&mut config, &theme_overrides);
                }
                Err(e) => {
                    warn!("failed to load theme '{}': {e}", config.theme.name);
                }
            }
        }

        // Validate
        validation::validate(&config)?;

        Ok(config)
    }
}
