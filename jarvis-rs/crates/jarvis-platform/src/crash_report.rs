use std::backtrace::Backtrace;
use std::panic::PanicHookInfo;
use std::path::PathBuf;

use regex::Regex;

use crate::paths::crash_report_dir;

/// Redacts known secret patterns from the input string.
///
/// Replaces API keys, tokens, AWS keys, bearer tokens, and generic secrets
/// (after `key=`, `token=`, `secret=`, `password=`) with `[REDACTED]`.
pub fn sanitize_secrets(input: &str) -> String {
    // Order matters: more specific patterns first, generic last.
    let patterns: &[&str] = &[
        // AWS access key IDs
        r"AKIA[0-9A-Z]{16}",
        // Stripe-style live/test keys
        r"sk_live_[a-zA-Z0-9]+",
        r"sk_test_[a-zA-Z0-9]+",
        // Generic sk- keys (OpenAI, etc.)
        r"sk-[a-zA-Z0-9]{20,}",
        // GitHub tokens
        r"ghp_[a-zA-Z0-9]{36}",
        r"gho_[a-zA-Z0-9]+",
        r"ghs_[a-zA-Z0-9]+",
        // Bearer tokens
        r"Bearer [a-zA-Z0-9._\-]+",
        // Generic secrets after key=, token=, secret=, password=
        r"(?i)((?:key|token|secret|password)=)[a-zA-Z0-9]{32,}",
    ];

    let mut result = input.to_string();

    for pattern in patterns {
        // SAFETY rationale: all patterns are static string literals validated at compile-test time.
        // Using expect here is acceptable — these are compile-time-constant regexes that cannot fail.
        let re = Regex::new(pattern).expect("crash_report: static regex pattern must compile");
        // For the generic key=/token=/secret=/password= pattern, preserve the prefix.
        if pattern.contains("(?i)((?:key|token|secret|password)=)") {
            result = re.replace_all(&result, "${1}[REDACTED]").into_owned();
        } else {
            result = re.replace_all(&result, "[REDACTED]").into_owned();
        }
    }

    result
}

/// Writes a crash report to disk when a panic occurs.
///
/// Returns the path to the written report, or `None` if writing failed.
/// This function is designed to run inside a panic hook and never panics itself —
/// all errors are silently swallowed.
///
/// Secret patterns (API keys, tokens, etc.) are redacted before writing.
/// On Unix, the report file is set to mode 0o600 (owner read/write only).
pub fn write_crash_report(info: &PanicHookInfo) -> Option<PathBuf> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let path = crash_report_dir()
        .ok()?
        .join(format!("crash_{timestamp}.json"));

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

    // Sanitize secrets from panic message and backtrace before persisting
    let message = sanitize_secrets(&message);
    let backtrace = sanitize_secrets(&backtrace);

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
    let _ = std::fs::create_dir_all(crash_report_dir().ok()?);
    std::fs::write(&path, serde_json::to_string_pretty(&report).ok()?).ok()?;

    // Restrict file permissions to owner-only on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }

    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_redacts_api_key() {
        // OpenAI-style sk- key
        let input = "error: auth failed with key sk-abc123DEF456ghi789jkl012mno345pq";
        let result = sanitize_secrets(input);
        assert!(
            !result.contains("sk-abc123DEF456ghi789jkl012mno345pq"),
            "sk- key should be redacted, got: {result}"
        );
        assert!(result.contains("[REDACTED]"));

        // Stripe live key
        let input = "token=sk_live_abcdef1234567890XYZ";
        let result = sanitize_secrets(input);
        assert!(
            !result.contains("sk_live_abcdef1234567890XYZ"),
            "sk_live_ key should be redacted, got: {result}"
        );

        // Stripe test key
        let input = "using sk_test_TESTABC123 for sandbox";
        let result = sanitize_secrets(input);
        assert!(
            !result.contains("sk_test_TESTABC123"),
            "sk_test_ key should be redacted, got: {result}"
        );

        // AWS access key
        let input = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";
        let result = sanitize_secrets(input);
        assert!(
            !result.contains("AKIAIOSFODNN7EXAMPLE"),
            "AWS key should be redacted, got: {result}"
        );
    }

    #[test]
    fn sanitize_redacts_github_token() {
        // ghp_ token (classic PAT)
        let input = "Authorization: token ghp_ABCDEFghijklmnopqrstuvwxyz0123456789";
        let result = sanitize_secrets(input);
        assert!(
            !result.contains("ghp_ABCDEFghijklmnopqrstuvwxyz0123456789"),
            "ghp_ token should be redacted, got: {result}"
        );
        assert!(result.contains("[REDACTED]"));

        // gho_ token (OAuth)
        let input = "token: gho_someOAuthToken123";
        let result = sanitize_secrets(input);
        assert!(
            !result.contains("gho_someOAuthToken123"),
            "gho_ token should be redacted, got: {result}"
        );

        // ghs_ token (server-to-server)
        let input = "ghs_installationToken456ABC";
        let result = sanitize_secrets(input);
        assert!(
            !result.contains("ghs_installationToken456ABC"),
            "ghs_ token should be redacted, got: {result}"
        );
    }

    #[test]
    fn sanitize_redacts_bearer() {
        let input = "header: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.signature";
        let result = sanitize_secrets(input);
        assert!(
            !result.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"),
            "Bearer token should be redacted, got: {result}"
        );
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn sanitize_leaves_normal_text() {
        let input =
            "thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 5'";
        let result = sanitize_secrets(input);
        assert_eq!(result, input, "Normal panic text should be unchanged");

        let input2 = "connection refused to localhost:8080";
        let result2 = sanitize_secrets(input2);
        assert_eq!(result2, input2, "Normal error text should be unchanged");

        let input3 = "";
        let result3 = sanitize_secrets(input3);
        assert_eq!(result3, input3, "Empty string should be unchanged");
    }
}
