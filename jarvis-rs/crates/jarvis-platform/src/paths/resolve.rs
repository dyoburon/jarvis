use std::path::PathBuf;

use jarvis_common::PlatformError;

pub(super) const APP_NAME: &str = "jarvis";

/// Returns the platform-specific configuration directory for Jarvis.
///
/// - macOS: `~/Library/Application Support/jarvis`
/// - Linux: `$XDG_CONFIG_HOME/jarvis` (defaults to `~/.config/jarvis`)
/// - Windows: `%APPDATA%\jarvis`
pub fn config_dir() -> Result<PathBuf, PlatformError> {
    Ok(dirs::config_dir()
        .ok_or_else(|| PlatformError::PathError("could not determine config directory".into()))?
        .join(APP_NAME))
}

/// Returns the platform-specific data directory for Jarvis.
///
/// - macOS: `~/Library/Application Support/jarvis`
/// - Linux: `$XDG_DATA_HOME/jarvis` (defaults to `~/.local/share/jarvis`)
/// - Windows: `%APPDATA%\jarvis`
pub fn data_dir() -> Result<PathBuf, PlatformError> {
    Ok(dirs::data_dir()
        .ok_or_else(|| PlatformError::PathError("could not determine data directory".into()))?
        .join(APP_NAME))
}

/// Returns the platform-specific cache directory for Jarvis.
///
/// - macOS: `~/Library/Caches/jarvis`
/// - Linux: `$XDG_CACHE_HOME/jarvis` (defaults to `~/.cache/jarvis`)
/// - Windows: `%LOCALAPPDATA%\jarvis`
pub fn cache_dir() -> Result<PathBuf, PlatformError> {
    Ok(dirs::cache_dir()
        .ok_or_else(|| PlatformError::PathError("could not determine cache directory".into()))?
        .join(APP_NAME))
}

/// Returns the path to the main configuration file.
///
/// Located at `config_dir()/config.toml`.
pub fn config_file() -> Result<PathBuf, PlatformError> {
    Ok(config_dir()?.join("config.toml"))
}

/// Returns the path to the identity file.
///
/// Located at `data_dir()/identity.json`.
pub fn identity_file() -> Result<PathBuf, PlatformError> {
    Ok(data_dir()?.join("identity.json"))
}

/// Returns the path to the log directory.
///
/// Located at `data_dir()/logs`.
pub fn log_dir() -> Result<PathBuf, PlatformError> {
    Ok(data_dir()?.join("logs"))
}

/// Returns the path to the crash report directory.
///
/// Located at `log_dir()/crash-reports`.
pub fn crash_report_dir() -> Result<PathBuf, PlatformError> {
    Ok(log_dir()?.join("crash-reports"))
}

/// Resolves a platform path, returning a `PlatformError` if the base directory
/// cannot be determined.
pub fn resolve_config_dir() -> Result<PathBuf, PlatformError> {
    dirs::config_dir()
        .map(|p| p.join(APP_NAME))
        .ok_or_else(|| PlatformError::PathError("could not determine config directory".into()))
}
