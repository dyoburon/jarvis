use std::backtrace::Backtrace;
use std::panic::PanicHookInfo;
use std::path::PathBuf;

use crate::paths::crash_report_dir;

/// Writes a crash report to disk when a panic occurs.
///
/// Returns the path to the written report, or `None` if writing failed.
/// This function is designed to run inside a panic hook and never panics itself —
/// all errors are silently swallowed.
pub fn write_crash_report(info: &PanicHookInfo) -> Option<PathBuf> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let path = crash_report_dir().join(format!("crash_{timestamp}.json"));

    let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    };

    let location = info.location().map(|loc| {
        serde_json::json!({
            "file": loc.file(),
            "line": loc.line(),
            "column": loc.column(),
        })
    });

    let backtrace = Backtrace::force_capture().to_string();

    let report = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "panic_message": message,
        "location": location,
        "backtrace": backtrace,
    });

    // Ensure directory exists (may not if ensure_dirs wasn't called or failed)
    let _ = std::fs::create_dir_all(crash_report_dir());
    std::fs::write(&path, serde_json::to_string_pretty(&report).ok()?).ok()?;

    Some(path)
}
