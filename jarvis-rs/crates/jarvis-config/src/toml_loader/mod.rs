//! TOML config file loading and creation.

mod loader;
mod paths;
mod template;

#[cfg(test)]
mod tests;

// Re-export public API — external imports remain unchanged.
pub use loader::{load_default, load_from_path};
pub use paths::{create_default_config, default_config_path};
