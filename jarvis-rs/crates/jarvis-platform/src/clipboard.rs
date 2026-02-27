use jarvis_common::PlatformError;

/// Cross-platform clipboard abstraction backed by `arboard`.
pub struct Clipboard {
    inner: arboard::Clipboard,
}

impl Clipboard {
    /// Creates a new clipboard handle.
    pub fn new() -> Result<Self, PlatformError> {
        let inner =
            arboard::Clipboard::new().map_err(|e| PlatformError::ClipboardError(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Reads text from the system clipboard.
    pub fn get_text(&mut self) -> Result<String, PlatformError> {
        self.inner
            .get_text()
            .map_err(|e| PlatformError::ClipboardError(e.to_string()))
    }

    /// Writes text to the system clipboard.
    pub fn set_text(&mut self, text: &str) -> Result<(), PlatformError> {
        self.inner
            .set_text(text.to_owned())
            .map_err(|e| PlatformError::ClipboardError(e.to_string()))
    }
}
