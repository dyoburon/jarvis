//! Shell detection and argument helpers.
//!
//! Detects the user's default shell from environment variables and provides
//! appropriate command-line arguments for interactive/login sessions.

/// Detect the user's default shell.
///
/// - On Unix: reads the `SHELL` environment variable, falling back to `/bin/sh`.
/// - On Windows: reads the `COMSPEC` environment variable, falling back to `cmd.exe`.
pub fn detect_shell() -> String {
    #[cfg(unix)]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }

    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }

    #[cfg(not(any(unix, windows)))]
    {
        "/bin/sh".to_string()
    }
}

/// Return the appropriate command-line arguments for the given shell binary.
///
/// Interactive login flags are added for shells that support them.
pub fn shell_args(shell: &str) -> Vec<String> {
    if shell.ends_with("zsh") || shell.ends_with("bash") {
        vec!["--login".to_string()]
    } else {
        vec![]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_shell_returns_non_empty() {
        let shell = detect_shell();
        assert!(!shell.is_empty(), "detect_shell() should not be empty");
    }

    #[test]
    fn shell_args_for_zsh() {
        assert_eq!(shell_args("/bin/zsh"), vec!["--login".to_string()]);
        assert_eq!(shell_args("zsh"), vec!["--login".to_string()]);
    }

    #[test]
    fn shell_args_for_bash() {
        assert_eq!(shell_args("/bin/bash"), vec!["--login".to_string()]);
        assert_eq!(shell_args("bash"), vec!["--login".to_string()]);
    }

    #[test]
    fn shell_args_for_fish() {
        let args = shell_args("/usr/bin/fish");
        assert!(args.is_empty());
    }

    #[test]
    fn shell_args_for_unknown() {
        let args = shell_args("/usr/local/bin/something_custom");
        assert!(args.is_empty());
    }
}
