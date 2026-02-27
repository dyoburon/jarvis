//! Local content serving via custom protocol.
//!
//! Registers a `jarvis://` custom protocol so that WebViews can load
//! bundled HTML/JS/CSS assets without needing a local HTTP server.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Serves local files from a base directory via custom protocol.
///
/// When a WebView requests `jarvis://app/index.html`, the content
/// provider resolves it to `{base_dir}/app/index.html` and returns
/// the file contents with the appropriate MIME type.
pub struct ContentProvider {
    /// Base directory for resolving asset paths.
    base_dir: PathBuf,
    /// In-memory overrides (for dynamically generated content).
    overrides: HashMap<String, (String, Vec<u8>)>, // path -> (mime, data)
}

impl ContentProvider {
    /// Create a new content provider rooted at `base_dir`.
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            overrides: HashMap::new(),
        }
    }

    /// Register an in-memory asset override.
    pub fn add_override(
        &mut self,
        path: impl Into<String>,
        mime: impl Into<String>,
        data: impl Into<Vec<u8>>,
    ) {
        self.overrides
            .insert(path.into(), (mime.into(), data.into()));
    }

    /// Resolve a request path to content bytes and MIME type.
    pub fn resolve(&self, path: &str) -> Option<(Cow<'_, str>, Cow<'_, [u8]>)> {
        let clean = path.trim_start_matches('/');

        // Check overrides first
        if let Some((mime, data)) = self.overrides.get(clean) {
            return Some((Cow::Borrowed(mime.as_str()), Cow::Borrowed(data.as_slice())));
        }

        // Resolve from filesystem
        let file_path = self.base_dir.join(clean);

        // Prevent directory traversal
        if !file_path.starts_with(&self.base_dir) {
            return None;
        }

        let data = std::fs::read(&file_path).ok()?;
        let mime = mime_from_extension(&file_path);
        Some((Cow::Owned(mime.to_string()), Cow::Owned(data)))
    }

    /// The base directory for assets.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

/// Guess MIME type from file extension.
fn mime_from_extension(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") | Some("mjs") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("wasm") => "application/wasm",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        Some("wav") => "audio/wav",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("webp") => "image/webp",
        Some("txt") => "text/plain",
        Some("xml") => "application/xml",
        _ => "application/octet-stream",
    }
}
