use wry::WebView;

/// Handle to a managed WebView instance. Provides methods to interact
/// with the underlying WebView (navigate, evaluate JS, resize, etc.).
pub struct WebViewHandle {
    /// The underlying wry WebView.
    pub(super) webview: WebView,
    /// The pane ID this WebView belongs to.
    pub(super) pane_id: u32,
    /// Current URL (best-effort tracking).
    pub(super) current_url: String,
    /// Current title.
    pub(super) current_title: String,
}

impl WebViewHandle {
    /// Get the pane ID.
    pub fn pane_id(&self) -> u32 {
        self.pane_id
    }

    /// Get the current URL.
    pub fn current_url(&self) -> &str {
        &self.current_url
    }

    /// Get the current title.
    pub fn current_title(&self) -> &str {
        &self.current_title
    }

    /// Navigate to a URL.
    pub fn load_url(&mut self, url: &str) -> Result<(), wry::Error> {
        self.current_url = url.to_string();
        self.webview.load_url(url)
    }

    /// Load raw HTML content.
    pub fn load_html(&mut self, html: &str) -> Result<(), wry::Error> {
        self.current_url = "about:blank".to_string();
        self.webview.load_html(html)
    }

    /// Execute JavaScript in the WebView context.
    pub fn evaluate_script(&self, js: &str) -> Result<(), wry::Error> {
        self.webview.evaluate_script(js)
    }

    /// Send a typed IPC message to JavaScript.
    pub fn send_ipc(&self, kind: &str, payload: &serde_json::Value) -> Result<(), wry::Error> {
        let script = crate::ipc::js_dispatch_message(kind, payload);
        self.webview.evaluate_script(&script)
    }

    /// Set the WebView bounds (position + size) within the parent window.
    pub fn set_bounds(&self, bounds: wry::Rect) -> Result<(), wry::Error> {
        self.webview.set_bounds(bounds)
    }

    /// Show or hide the WebView.
    pub fn set_visible(&self, visible: bool) -> Result<(), wry::Error> {
        self.webview.set_visible(visible)
    }

    /// Focus the WebView.
    pub fn focus(&self) -> Result<(), wry::Error> {
        self.webview.focus()
    }

    /// Return focus to the parent window.
    pub fn focus_parent(&self) -> Result<(), wry::Error> {
        self.webview.focus_parent()
    }

    /// Open devtools (if enabled).
    pub fn open_devtools(&self) {
        self.webview.open_devtools();
    }

    /// Set zoom level.
    pub fn zoom(&self, scale: f64) -> Result<(), wry::Error> {
        self.webview.zoom(scale)
    }

    /// Update the tracked title.
    pub fn set_title(&mut self, title: String) {
        self.current_title = title;
    }

    /// Get a reference to the underlying wry WebView.
    pub fn inner(&self) -> &WebView {
        &self.webview
    }
}
