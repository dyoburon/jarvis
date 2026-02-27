mod report;
mod sanitize;

pub use report::write_crash_report;
pub use sanitize::sanitize_secrets;

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
