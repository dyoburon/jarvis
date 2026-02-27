//! Shell process configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Shell process settings.
///
/// Controls which shell to launch, its arguments, working directory,
/// extra environment variables, and login shell behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShellConfig {
    /// Shell program path. Empty string means auto-detect from `$SHELL`.
    pub program: String,
    /// Extra arguments passed to the shell.
    pub args: Vec<String>,
    /// Initial working directory. `None` means inherit from parent.
    pub working_directory: Option<String>,
    /// Extra environment variables injected into the shell.
    pub env: HashMap<String, String>,
    /// Launch as a login shell (prepend `-` to argv[0]).
    pub login_shell: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            program: String::new(),
            args: Vec::new(),
            working_directory: None,
            env: HashMap::new(),
            login_shell: true,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_config_defaults() {
        let config = ShellConfig::default();
        assert!(config.program.is_empty());
        assert!(config.args.is_empty());
        assert!(config.working_directory.is_none());
        assert!(config.env.is_empty());
        assert!(config.login_shell);
    }

    #[test]
    fn shell_config_partial_toml() {
        let toml_str = r#"
program = "/bin/zsh"
args = ["-l", "--no-rcs"]
login_shell = false
"#;
        let config: ShellConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.program, "/bin/zsh");
        assert_eq!(config.args, vec!["-l", "--no-rcs"]);
        assert!(!config.login_shell);
        // Defaults preserved
        assert!(config.working_directory.is_none());
        assert!(config.env.is_empty());
    }

    #[test]
    fn shell_config_with_env_vars() {
        let toml_str = r#"
[env]
TERM = "xterm-256color"
EDITOR = "nvim"
"#;
        let config: ShellConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.env.get("TERM").unwrap(), "xterm-256color");
        assert_eq!(config.env.get("EDITOR").unwrap(), "nvim");
        assert!(config.login_shell); // default preserved
    }

    #[test]
    fn shell_config_serialization_roundtrip() {
        let config = ShellConfig {
            program: "/usr/local/bin/fish".into(),
            args: vec!["--init-command".into(), "echo hi".into()],
            working_directory: Some("/home/user".into()),
            env: HashMap::from([("FOO".into(), "bar".into())]),
            login_shell: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ShellConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.program, config.program);
        assert_eq!(deserialized.args, config.args);
        assert_eq!(deserialized.working_directory, config.working_directory);
        assert_eq!(deserialized.env, config.env);
        assert_eq!(deserialized.login_shell, config.login_shell);
    }
}
