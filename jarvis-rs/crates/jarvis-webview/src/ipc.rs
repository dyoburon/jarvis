//! IPC (Inter-Process Communication) protocol between Rust and JavaScript.
//!
//! Messages flow in both directions:
//! - **JS -> Rust**: JavaScript calls `window.ipc.postMessage(JSON.stringify({...}))`,
//!   which triggers the `ipc_handler` registered on the WebView.
//! - **Rust -> JS**: Rust calls `webview.evaluate_script("...")` to invoke
//!   JavaScript functions in the WebView context.

use serde::{Deserialize, Serialize};

/// A typed IPC message from JavaScript to Rust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// The message type / command name.
    pub kind: String,
    /// The message payload (arbitrary JSON).
    pub payload: IpcPayload,
}

/// Payload of an IPC message — either a simple string or structured JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IpcPayload {
    Text(String),
    Json(serde_json::Value),
    None,
}

impl IpcMessage {
    /// Parse an IPC message from a raw JSON string (from JS postMessage).
    pub fn from_json(raw: &str) -> Option<Self> {
        serde_json::from_str(raw).ok()
    }

    /// Create a simple text message.
    pub fn text(kind: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            payload: IpcPayload::Text(text.into()),
        }
    }

    /// Create a JSON message.
    pub fn json(kind: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            kind: kind.into(),
            payload: IpcPayload::Json(value),
        }
    }
}

/// JavaScript snippet that sets up the IPC bridge on the JS side.
/// This is injected as an initialization script into every WebView.
pub const IPC_INIT_SCRIPT: &str = r#"
(function() {
    // Jarvis IPC bridge
    window.jarvis = window.jarvis || {};
    window.jarvis.ipc = {
        postMessage: function(msg) {
            window.ipc.postMessage(JSON.stringify(msg));
        },
        send: function(kind, payload) {
            window.ipc.postMessage(JSON.stringify({
                kind: kind,
                payload: payload || null
            }));
        },
        // Callbacks registered by JS code to handle messages from Rust
        _handlers: {},
        on: function(kind, callback) {
            this._handlers[kind] = callback;
        },
        _dispatch: function(kind, payload) {
            var handler = this._handlers[kind];
            if (handler) {
                handler(payload);
            }
        }
    };
})();
"#;

/// Generate a JS snippet that dispatches a message to the JS IPC handler.
pub fn js_dispatch_message(kind: &str, payload: &serde_json::Value) -> String {
    let payload_json = serde_json::to_string(payload).unwrap_or_else(|_| "null".to_string());
    format!(
        "window.jarvis.ipc._dispatch({}, {});",
        serde_json::to_string(kind).unwrap_or_else(|_| "\"unknown\"".to_string()),
        payload_json,
    )
}
