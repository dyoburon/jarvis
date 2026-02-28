//! Session types and concurrency guards.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::AiError;

/// Callback for executing tool calls. Takes a tool name + arguments,
/// returns the tool's output string.
pub type ToolExecutor = Box<dyn Fn(&str, &serde_json::Value) -> String + Send + Sync>;

/// Guard that clears the `busy` flag on drop, ensuring it is always released
/// even if the future is cancelled or an early return occurs.
pub(crate) struct BusyGuard<'a> {
    flag: &'a AtomicBool,
}

impl<'a> BusyGuard<'a> {
    /// Attempt to acquire the busy lock. Returns `Err` if already busy.
    pub(crate) fn acquire(flag: &'a AtomicBool) -> Result<Self, AiError> {
        if flag
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(AiError::ApiError(
                "Session is busy with another request".into(),
            ));
        }
        Ok(Self { flag })
    }
}

impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::Release);
    }
}
