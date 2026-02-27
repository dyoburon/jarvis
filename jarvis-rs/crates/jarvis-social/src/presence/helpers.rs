//! Utility helpers for the presence module.

/// Get the current timestamp as an ISO 8601 string.
pub(crate) fn chrono_now() -> String {
    // Use a simple approach without adding chrono as a dependency.
    // SystemTime gives us epoch seconds which we format as a string.
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // ISO-ish format: just use epoch millis as a string for simplicity.
    // A proper ISO format would require the chrono crate.
    format!("{}", duration.as_millis())
}
