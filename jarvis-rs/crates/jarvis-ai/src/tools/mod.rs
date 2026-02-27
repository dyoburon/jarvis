//! Built-in tool definitions for AI assistants.
//!
//! Tools are functions the AI can call to interact with the system
//! (run commands, read files, search, etc.).

mod definitions;
mod sandbox;

pub use definitions::{builtin_tools, to_claude_tool, to_gemini_tool};
pub use sandbox::ToolSandbox;

#[cfg(test)]
mod sandbox_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Helper: create a `ToolSandbox` rooted at a temporary directory.
    fn sandbox_in_tmp() -> (ToolSandbox, std::path::PathBuf) {
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
