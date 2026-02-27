use jarvis_common::PlatformError;
use tracing::info;

/// Sends a native notification to the user.
///
/// - macOS: Uses `osascript` to display a native notification.
/// - Other platforms: Logs the notification (stub).
pub fn notify(title: &str, body: &str) -> Result<(), PlatformError> {
    platform_notify(title, body)
}

// TODO: replace osascript with notify-rust crate for safety
#[cfg(target_os = "macos")]
fn platform_notify(title: &str, body: &str) -> Result<(), PlatformError> {
    let escaped_title = title
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\'', "\\'");
    let escaped_body = body
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\'', "\\'");

    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        escaped_body, escaped_title
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| PlatformError::NotificationError(format!("failed to run osascript: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PlatformError::NotificationError(format!(
            "osascript failed: {stderr}"
        )));
    }

    info!("native notification sent");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn platform_notify(title: &str, body: &str) -> Result<(), PlatformError> {
    info!("notification (stub): would display native notification");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify_returns_ok() {
        let result = notify("Test Title", "Test body content");
        assert!(result.is_ok());
    }

    #[test]
    fn notify_with_empty_strings() {
        let result = notify("", "");
        assert!(result.is_ok());
    }
}
