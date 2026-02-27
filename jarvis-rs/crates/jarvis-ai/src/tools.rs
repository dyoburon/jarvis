//! Built-in tool definitions for AI assistants.
//!
//! Tools are functions the AI can call to interact with the system
//! (run commands, read files, search, etc.).

use std::path::{Path, PathBuf};

use crate::ToolDefinition;

/// Sandbox that restricts tool operations to a specific directory
/// and validates commands against an allowlist.
pub struct ToolSandbox {
    sandbox_dir: PathBuf,
}

/// Sensitive path segments that must never be accessed, regardless of sandbox location.
const BLOCKED_PATH_SEGMENTS: &[&str] = &[
    ".ssh",
    ".aws",
    ".gnupg",
    ".env",
    "/etc/passwd",
    "/etc/shadow",
];

/// Commands allowed for execution inside the sandbox.
const ALLOWED_COMMANDS: &[&str] = &[
    "ls", "cat", "head", "tail", "wc", "find", "grep", "rg", "git", "cargo", "rustc", "npm",
    "node", "python3", "echo", "mkdir", "cp", "mv", "rm", "touch",
];

impl ToolSandbox {
    /// Create a new sandbox rooted at the given directory.
    ///
    /// The `sandbox_dir` should be a canonicalized absolute path.
    pub fn new(sandbox_dir: PathBuf) -> Self {
        Self { sandbox_dir }
    }

    /// Validate that `path` resolves to a location inside the sandbox
    /// and does not reference any sensitive path segments.
    ///
    /// If the path does not exist yet, the parent directory is canonicalized
    /// instead (to support file-creation use cases).
    ///
    /// Returns the canonicalized path on success.
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf, String> {
        // Canonicalize the path — fall back to the parent when the leaf doesn't exist yet.
        let canonical: PathBuf = std::fs::canonicalize(path).or_else(|_| {
            let parent = path
                .parent()
                .ok_or_else(|| "Access denied: cannot resolve parent directory".to_string())?;
            let canon_parent = std::fs::canonicalize(parent).map_err(|e| {
                format!(
                    "Access denied: cannot resolve path '{}': {e}",
                    parent.display()
                )
            })?;
            let file_name = path
                .file_name()
                .ok_or_else(|| "Access denied: path has no file name".to_string())?;
            // Re-attach the final component so the caller gets the full intended path.
            Ok::<PathBuf, String>(canon_parent.join(file_name))
        })?;

        // The resolved path must live inside the sandbox.
        if !canonical.starts_with(&self.sandbox_dir) {
            return Err(format!(
                "Access denied: path '{}' is outside sandbox '{}'",
                canonical.display(),
                self.sandbox_dir.display(),
            ));
        }

        // Check every component of the path string for blocked segments.
        let path_str = canonical.to_string_lossy();
        for segment in BLOCKED_PATH_SEGMENTS {
            if path_str.contains(segment) {
                return Err(format!(
                    "Access denied: path '{}' contains blocked segment '{segment}'",
                    canonical.display(),
                ));
            }
        }

        Ok(canonical)
    }

    /// Validate that the first word of `cmd` is in the command allowlist.
    pub fn validate_command(&self, cmd: &str) -> Result<(), String> {
        let first_word = cmd.split_whitespace().next().unwrap_or("");

        // Strip any leading path prefix so `/usr/bin/ls` is treated as `ls`.
        let binary_name = first_word.rsplit('/').next().unwrap_or(first_word);

        if ALLOWED_COMMANDS.contains(&binary_name) {
            Ok(())
        } else {
            Err(format!("Command not allowed: {binary_name}"))
        }
    }
}

/// Create the built-in tool definitions that Jarvis exposes to AI models.
pub fn builtin_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "run_command".to_string(),
            description: "Execute a shell command and return its output.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "working_directory": {
                        "type": "string",
                        "description": "Working directory for the command (optional)"
                    }
                },
                "required": ["command"]
            }),
        },
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute or relative path to the file"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file, creating it if it doesn't exist.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        ToolDefinition {
            name: "search_files".to_string(),
            description: "Search for files matching a pattern using glob.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern (e.g., '**/*.rs')"
                    },
                    "directory": {
                        "type": "string",
                        "description": "Root directory to search from"
                    }
                },
                "required": ["pattern"]
            }),
        },
        ToolDefinition {
            name: "search_content".to_string(),
            description: "Search file contents for a regex pattern.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "directory": {
                        "type": "string",
                        "description": "Root directory to search from"
                    },
                    "file_pattern": {
                        "type": "string",
                        "description": "Glob pattern to filter files (e.g., '*.rs')"
                    }
                },
                "required": ["pattern"]
            }),
        },
        ToolDefinition {
            name: "list_directory".to_string(),
            description: "List files and directories at a given path.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path"
                    }
                },
                "required": ["path"]
            }),
        },
    ]
}

/// Convert a tool definition to the Claude API format.
pub fn to_claude_tool(tool: &ToolDefinition) -> serde_json::Value {
    serde_json::json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.parameters,
    })
}

/// Convert a tool definition to the Gemini API format.
pub fn to_gemini_tool(tool: &ToolDefinition) -> serde_json::Value {
    serde_json::json!({
        "name": tool.name,
        "description": tool.description,
        "parameters": tool.parameters,
    })
}

#[cfg(test)]
mod sandbox_tests {
    use super::*;
    use std::fs;

    /// Helper: create a `ToolSandbox` rooted at a temporary directory.
    fn sandbox_in_tmp() -> (ToolSandbox, PathBuf) {
        let dir = std::env::temp_dir().join("jarvis_sandbox_test");
        fs::create_dir_all(&dir).unwrap();
        let canonical = fs::canonicalize(&dir).unwrap();
        (ToolSandbox::new(canonical.clone()), canonical)
    }

    #[test]
    fn blocked_command_rejected() {
        let (sandbox, _dir) = sandbox_in_tmp();

        let result = sandbox.validate_command("curl http://evil.com");
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("Command not allowed"),
            "should reject curl"
        );

        let result = sandbox.validate_command("sudo rm -rf /");
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("Command not allowed"),
            "should reject sudo"
        );

        let result = sandbox.validate_command("wget http://evil.com");
        assert!(result.is_err());

        let result = sandbox.validate_command("bash -c 'echo pwned'");
        assert!(result.is_err());
    }

    #[test]
    fn blocked_path_rejected() {
        let (sandbox, dir) = sandbox_in_tmp();

        // Create a .ssh directory inside the sandbox so canonicalize succeeds.
        let ssh_dir = dir.join(".ssh");
        fs::create_dir_all(&ssh_dir).unwrap();

        let result = sandbox.validate_path(&ssh_dir);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("blocked segment") && err.contains(".ssh"),
            "should block .ssh, got: {err}"
        );

        // .env inside sandbox
        let env_path = dir.join(".env");
        fs::write(&env_path, "SECRET=oops").unwrap();
        let result = sandbox.validate_path(&env_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".env"), "should block .env");
    }

    #[test]
    fn path_traversal_rejected() {
        let (sandbox, dir) = sandbox_in_tmp();

        // Attempt to escape via `..`
        let escaped = dir.join("..").join("..").join("etc").join("hosts");
        let result = sandbox.validate_path(&escaped);
        // Either canonicalize fails (path doesn't exist) or it's outside sandbox.
        assert!(result.is_err(), "path traversal via .. must be rejected");

        // Absolute path outside sandbox
        let outside = Path::new("/tmp");
        // /tmp itself is almost certainly not inside our sandbox sub-dir
        let result = sandbox.validate_path(outside);
        assert!(
            result.is_err(),
            "absolute path outside sandbox must be rejected"
        );
    }

    #[test]
    fn allowed_command_passes() {
        let (sandbox, _dir) = sandbox_in_tmp();

        for cmd in &[
            "ls -la",
            "cat foo.txt",
            "git status",
            "cargo build",
            "echo hello",
            "mkdir -p subdir",
            "rm temp.txt",
            "touch new_file",
            "grep pattern file.rs",
            "python3 script.py",
        ] {
            assert!(
                sandbox.validate_command(cmd).is_ok(),
                "command should be allowed: {cmd}"
            );
        }
    }

    #[test]
    fn allowed_path_passes() {
        let (sandbox, dir) = sandbox_in_tmp();

        // Create a file inside the sandbox
        let test_file = dir.join("allowed_test.txt");
        fs::write(&test_file, "hello").unwrap();

        let result = sandbox.validate_path(&test_file);
        assert!(result.is_ok(), "path inside sandbox should be allowed");
        assert!(
            result.unwrap().starts_with(&dir),
            "returned path should be inside sandbox"
        );

        // Non-existent file whose parent is inside sandbox (creation scenario)
        let new_file = dir.join("will_be_created.txt");
        let result = sandbox.validate_path(&new_file);
        assert!(
            result.is_ok(),
            "non-existent file with valid parent should be allowed"
        );
    }
}
