use regex::Regex;

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
        if pattern.contains("(?i)((?:key|token|secret|password)=)") {
            result = re.replace_all(&result, "${1}[REDACTED]").into_owned();
        } else {
            result = re.replace_all(&result, "[REDACTED]").into_owned();
        }
    }

    result
}
