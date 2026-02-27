//! Sandbox that restricts tool operations to a specific directory.

use std::path::{Path, PathBuf};

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

/// Sandbox that restricts tool operations to a specific directory
/// and validates commands against an allowlist.
pub struct ToolSandbox {
    sandbox_dir: PathBuf,
}

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
