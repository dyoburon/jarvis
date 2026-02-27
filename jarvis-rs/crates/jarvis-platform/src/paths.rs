use std::fs;
use std::path::PathBuf;

use jarvis_common::PlatformError;

const APP_NAME: &str = "jarvis";

/// Returns the platform-specific configuration directory for Jarvis.
///
/// - macOS: `~/Library/Application Support/jarvis`
/// - Linux: `$XDG_CONFIG_HOME/jarvis` (defaults to `~/.config/jarvis`)
/// - Windows: `%APPDATA%\jarvis`
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("could not determine config directory")
        .join(APP_NAME)
}

/// Returns the platform-specific data directory for Jarvis.
///
/// - macOS: `~/Library/Application Support/jarvis`
/// - Linux: `$XDG_DATA_HOME/jarvis` (defaults to `~/.local/share/jarvis`)
/// - Windows: `%APPDATA%\jarvis`
pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .expect("could not determine data directory")
        .join(APP_NAME)
}

/// Returns the platform-specific cache directory for Jarvis.
///
/// - macOS: `~/Library/Caches/jarvis`
/// - Linux: `$XDG_CACHE_HOME/jarvis` (defaults to `~/.cache/jarvis`)
/// - Windows: `%LOCALAPPDATA%\jarvis`
pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .expect("could not determine cache directory")
        .join(APP_NAME)
}

/// Returns the path to the main configuration file.
///
/// Located at `config_dir()/config.toml`.
pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
}

/// Returns the path to the identity file.
///
/// Located at `data_dir()/identity.json`.
pub fn identity_file() -> PathBuf {
    data_dir().join("identity.json")
}

/// Returns the path to the log directory.
///
/// Located at `data_dir()/logs`.
pub fn log_dir() -> PathBuf {
    data_dir().join("logs")
}

/// Returns the path to the crash report directory.
///
/// Located at `log_dir()/crash-reports`.
pub fn crash_report_dir() -> PathBuf {
    log_dir().join("crash-reports")
}

/// Creates all Jarvis directories if they do not already exist.
///
/// Creates: config_dir, data_dir, cache_dir, and log_dir.
pub fn ensure_dirs() -> Result<(), std::io::Error> {
    fs::create_dir_all(config_dir())?;
    fs::create_dir_all(data_dir())?;
    fs::create_dir_all(cache_dir())?;
    fs::create_dir_all(log_dir())?;
    fs::create_dir_all(crash_report_dir())?;
    Ok(())
}

/// Resolves a platform path, returning a `PlatformError` if the base directory
/// cannot be determined.
pub fn resolve_config_dir() -> Result<PathBuf, PlatformError> {
    dirs::config_dir()
        .map(|p| p.join(APP_NAME))
        .ok_or_else(|| PlatformError::PathError("could not determine config directory".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_ends_with_jarvis() {
        let path = config_dir();
        assert!(
            path.ends_with("jarvis"),
            "config_dir should end with 'jarvis', got: {path:?}"
        );
    }

    #[test]
    fn data_dir_ends_with_jarvis() {
        let path = data_dir();
        assert!(
            path.ends_with("jarvis"),
            "data_dir should end with 'jarvis', got: {path:?}"
        );
    }

    #[test]
    fn cache_dir_ends_with_jarvis() {
        let path = cache_dir();
        assert!(
            path.ends_with("jarvis"),
            "cache_dir should end with 'jarvis', got: {path:?}"
        );
    }

    #[test]
    fn config_file_has_correct_name() {
        let path = config_file();
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), "config.toml");
        assert!(
            path.parent().unwrap().ends_with("jarvis"),
            "config_file parent should end with 'jarvis', got: {path:?}"
        );
    }

    #[test]
    fn identity_file_has_correct_name() {
        let path = identity_file();
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), "identity.json");
        assert!(
            path.parent().unwrap().ends_with("jarvis"),
            "identity_file parent should end with 'jarvis', got: {path:?}"
        );
    }

    #[test]
    fn log_dir_is_inside_data_dir() {
        let log = log_dir();
        let data = data_dir();
        assert!(
            log.starts_with(&data),
            "log_dir should be inside data_dir: log={log:?}, data={data:?}"
        );
        assert_eq!(log.file_name().unwrap().to_str().unwrap(), "logs");
    }

    #[test]
    fn resolve_config_dir_returns_ok() {
        let result = resolve_config_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("jarvis"));
    }
}
