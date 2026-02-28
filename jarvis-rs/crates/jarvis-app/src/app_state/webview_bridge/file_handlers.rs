//! IPC handlers for reading local image files and clipboard images.
//!
//! Used by the chat panel to send images by file path or paste.

use jarvis_webview::IpcPayload;

use crate::app_state::core::JarvisApp;

/// Image file magic bytes for validation.
const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47];
const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];
const GIF_MAGIC: &[u8] = &[0x47, 0x49, 0x46];
const WEBP_MAGIC: &[u8] = b"RIFF";
const BMP_MAGIC: &[u8] = &[0x42, 0x4D];

/// Detect MIME type from magic bytes.
fn detect_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 4 {
        return None;
    }
    if bytes.starts_with(PNG_MAGIC) {
        Some("image/png")
    } else if bytes.starts_with(JPEG_MAGIC) {
        Some("image/jpeg")
    } else if bytes.starts_with(GIF_MAGIC) {
        Some("image/gif")
    } else if bytes.len() >= 12 && bytes.starts_with(WEBP_MAGIC) && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else if bytes.starts_with(BMP_MAGIC) {
        Some("image/bmp")
    } else {
        None
    }
}

/// Maximum file size: 5 MB.
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

impl JarvisApp {
    /// Handle a `read_file` IPC message — read a local image file and
    /// return its contents as a base64 data URL.
    pub(in crate::app_state) fn handle_read_file(
        &mut self,
        pane_id: u32,
        payload: &IpcPayload,
    ) {
        let obj = match payload {
            IpcPayload::Json(v) => v,
            _ => {
                tracing::warn!(pane_id, "read_file: expected JSON payload");
                return;
            }
        };

        let req_id = obj.get("_reqId").and_then(|v| v.as_u64()).unwrap_or(0);

        let path_str = match obj.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                self.read_file_respond(pane_id, req_id, None, Some("missing path"));
                return;
            }
        };

        // Expand ~ to home directory
        let expanded = if path_str.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(&path_str[2..])
            } else {
                std::path::PathBuf::from(path_str)
            }
        } else {
            std::path::PathBuf::from(path_str)
        };

        // Validate path exists and is a file
        let metadata = match std::fs::metadata(&expanded) {
            Ok(m) => m,
            Err(e) => {
                self.read_file_respond(pane_id, req_id, None, Some(&format!("file not found: {e}")));
                return;
            }
        };

        if !metadata.is_file() {
            self.read_file_respond(pane_id, req_id, None, Some("not a regular file"));
            return;
        }

        if metadata.len() > MAX_FILE_SIZE {
            self.read_file_respond(pane_id, req_id, None, Some("file too large (max 5MB)"));
            return;
        }

        // Read the file
        let bytes = match std::fs::read(&expanded) {
            Ok(b) => b,
            Err(e) => {
                self.read_file_respond(pane_id, req_id, None, Some(&format!("read error: {e}")));
                return;
            }
        };

        // Validate it's an image by checking magic bytes
        let mime = match detect_mime(&bytes) {
            Some(m) => m,
            None => {
                self.read_file_respond(pane_id, req_id, None, Some("not a recognized image format"));
                return;
            }
        };

        // Encode as base64 data URL
        use base64::Engine as _;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let data_url = format!("data:{mime};base64,{b64}");

        self.read_file_respond(pane_id, req_id, Some(&data_url), None);
    }

    /// Send a `read_file_response` IPC message back to the webview.
    fn read_file_respond(
        &self,
        pane_id: u32,
        req_id: u64,
        data_url: Option<&str>,
        error: Option<&str>,
    ) {
        let registry = match &self.webviews {
            Some(r) => r,
            None => return,
        };
        let handle = match registry.get(pane_id) {
            Some(h) => h,
            None => return,
        };

        let payload = if let Some(url) = data_url {
            serde_json::json!({
                "_reqId": req_id,
                "data_url": url,
            })
        } else {
            serde_json::json!({
                "_reqId": req_id,
                "error": error.unwrap_or("unknown error"),
            })
        };

        if let Err(e) = handle.send_ipc("read_file_response", &payload) {
            tracing::warn!(pane_id, error = %e, "Failed to send read_file_response");
        }
    }

    /// Handle a `clipboard_paste` IPC request — read the system clipboard
    /// and return image data (as PNG base64 data URL) or text.
    ///
    /// WKWebView doesn't fire DOM `paste` events for image clipboard data,
    /// so the chat panel calls this via IPC when the user presses Cmd+V.
    pub(in crate::app_state) fn handle_clipboard_paste(
        &mut self,
        pane_id: u32,
        payload: &IpcPayload,
    ) {
        let req_id = match payload {
            IpcPayload::Json(v) => v.get("_reqId").and_then(|v| v.as_u64()).unwrap_or(0),
            _ => 0,
        };

        let mut cb = match jarvis_platform::Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(pane_id, error = %e, "clipboard_paste: failed to open clipboard");
                self.clipboard_paste_respond(pane_id, req_id, None, None, Some("clipboard unavailable"));
                return;
            }
        };

        // Try image first
        if let Ok((width, height, rgba)) = cb.get_image() {
            tracing::info!(pane_id, width, height, rgba_len = rgba.len(), "clipboard_paste: image found");
            // Encode RGBA pixels as PNG
            match encode_rgba_as_png(width as u32, height as u32, &rgba) {
                Ok(png_bytes) => {
                    use base64::Engine as _;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
                    let data_url = format!("data:image/png;base64,{b64}");
                    self.clipboard_paste_respond(pane_id, req_id, Some(&data_url), None, None);
                }
                Err(e) => {
                    tracing::warn!(pane_id, error = %e, "clipboard_paste: PNG encode failed");
                    self.clipboard_paste_respond(pane_id, req_id, None, None, Some("failed to encode image"));
                }
            }
            return;
        }

        // Fall back to text
        if let Ok(text) = cb.get_text() {
            if !text.is_empty() {
                let preview: String = text.chars().take(200).collect();
                tracing::info!(pane_id, text_len = text.len(), %preview, "clipboard_paste: text found");
                self.clipboard_paste_respond(pane_id, req_id, None, Some(&text), None);
                return;
            }
        }

        tracing::info!(pane_id, "clipboard_paste: clipboard empty");
        self.clipboard_paste_respond(pane_id, req_id, None, None, Some("clipboard empty"));
    }

    /// Send a `clipboard_paste_response` IPC message back to the webview.
    fn clipboard_paste_respond(
        &self,
        pane_id: u32,
        req_id: u64,
        image_data_url: Option<&str>,
        text: Option<&str>,
        error: Option<&str>,
    ) {
        let registry = match &self.webviews {
            Some(r) => r,
            None => return,
        };
        let handle = match registry.get(pane_id) {
            Some(h) => h,
            None => return,
        };

        let payload = if let Some(url) = image_data_url {
            serde_json::json!({
                "_reqId": req_id,
                "kind": "image",
                "data_url": url,
            })
        } else if let Some(t) = text {
            serde_json::json!({
                "_reqId": req_id,
                "kind": "text",
                "text": t,
            })
        } else {
            serde_json::json!({
                "_reqId": req_id,
                "error": error.unwrap_or("unknown error"),
            })
        };

        if let Err(e) = handle.send_ipc("clipboard_paste_response", &payload) {
            tracing::warn!(pane_id, error = %e, "Failed to send clipboard_paste_response");
        }
    }
}

/// Encode raw RGBA pixels as a PNG byte buffer.
fn encode_rgba_as_png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Cursor;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut encoder = png::Encoder::new(&mut buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    }
    Ok(buf.into_inner())
}
