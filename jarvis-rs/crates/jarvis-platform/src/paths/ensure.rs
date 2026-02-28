use std::fs;

use jarvis_common::PlatformError;

use super::resolve::{cache_dir, config_dir, crash_report_dir, data_dir, log_dir};

/// Creates all Jarvis directories if they do not already exist.
///
/// Creates: config_dir, data_dir, cache_dir, log_dir, and crash_report_dir.
pub fn ensure_dirs() -> Result<(), PlatformError> {
    fs::create_dir_all(config_dir()?).map_err(|e| PlatformError::PathError(e.to_string()))?;
    fs::create_dir_all(data_dir()?).map_err(|e| PlatformError::PathError(e.to_string()))?;
    fs::create_dir_all(cache_dir()?).map_err(|e| PlatformError::PathError(e.to_string()))?;
    fs::create_dir_all(log_dir()?).map_err(|e| PlatformError::PathError(e.to_string()))?;
    fs::create_dir_all(crash_report_dir()?).map_err(|e| PlatformError::PathError(e.to_string()))?;
    Ok(())
}
