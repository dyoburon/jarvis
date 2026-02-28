//! PTY error types.

/// Errors originating from PTY operations.
#[derive(Debug, thiserror::Error)]
pub enum PtyError {
    #[error("failed to spawn process: {0}")]
    SpawnFailed(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("failed to resize PTY: {0}")]
    ResizeFailed(String),
}
