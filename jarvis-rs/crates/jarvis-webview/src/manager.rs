//! WebView lifecycle management.
//!
//! `WebViewManager` creates, tracks, and destroys `wry::WebView` instances,
//! one per pane that needs embedded web content (games, chat, docs, etc.).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tracing::{debug, warn};
use wry::raw_window_handle;
use wry::{WebView, WebViewBuilder};

use crate::content::ContentProvider;
use crate::events::{PageLoadState, WebViewEvent};
use crate::ipc::IPC_INIT_SCRIPT;

/// Configuration for creating a new WebView instance.
#[derive(Debug, Clone)]
pub struct WebViewConfig {
    /// Initial URL to load (mutually exclusive with `html`).
    pub url: Option<String>,
    /// Initial HTML content to render (mutually exclusive with `url`).
    pub html: Option<String>,
    /// Whether the WebView background should be transparent.
    pub transparent: bool,
    /// Whether to enable dev tools (always on in debug builds).
    pub devtools: bool,
    /// Custom user agent string.
    pub user_agent: Option<String>,
    /// Whether to enable clipboard access.
    pub clipboard: bool,
    /// Whether to enable autoplay for media.
    pub autoplay: bool,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            url: None,
            html: None,
            transparent: false,
            devtools: cfg!(debug_assertions),
            user_agent: Some("Jarvis/0.1".to_string()),
            clipboard: true,
            autoplay: true,
        }
    }
}

impl WebViewConfig {
    /// Create a config that loads a URL.
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            ..Default::default()
        }
    }

    /// Create a config that renders inline HTML.
    pub fn with_html(html: impl Into<String>) -> Self {
        Self {
            html: Some(html.into()),
            ..Default::default()
        }
    }
}

/// Handle to a managed WebView instance. Provides methods to interact
/// with the underlying WebView (navigate, evaluate JS, resize, etc.).
pub struct WebViewHandle {
    /// The underlying wry WebView.
    webview: WebView,
    /// The pane ID this WebView belongs to.
    pane_id: u32,
    /// Current URL (best-effort tracking).
    current_url: String,
    /// Current title.
    current_title: String,
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

/// Manages all WebView instances across tiling panes.
pub struct WebViewManager {
    /// Event sink — events are pushed here for the main event loop to consume.
    events: Arc<Mutex<Vec<WebViewEvent>>>,
    /// Optional content provider for the `jarvis://` custom protocol.
    content_provider: Option<Arc<ContentProvider>>,
}

impl WebViewManager {
    /// Create a new WebView manager.
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            content_provider: None,
        }
    }

    /// Set the content provider for serving bundled assets via `jarvis://`.
    pub fn set_content_provider(&mut self, provider: ContentProvider) {
        self.content_provider = Some(Arc::new(provider));
    }

    /// Drain all pending events.
    pub fn drain_events(&self) -> Vec<WebViewEvent> {
        let mut events = self.events.lock().unwrap();
        std::mem::take(&mut *events)
    }

    /// Create a new WebView as a child of the given window.
    ///
    /// The `window` must implement `raw_window_handle::HasWindowHandle`.
    /// The WebView is positioned at `bounds` within the parent window.
    pub fn create<W: raw_window_handle::HasWindowHandle>(
        &self,
        pane_id: u32,
        window: &W,
        bounds: wry::Rect,
        config: WebViewConfig,
    ) -> Result<WebViewHandle, wry::Error> {
        let events = Arc::clone(&self.events);
        let pid = pane_id;

        // Start building the WebView
        let mut builder = WebViewBuilder::new()
            .with_bounds(bounds)
            .with_transparent(config.transparent)
            .with_devtools(config.devtools)
            .with_clipboard(config.clipboard)
            .with_autoplay(config.autoplay)
            .with_focused(false);

        // Initialization script for IPC bridge
        builder = builder.with_initialization_script(IPC_INIT_SCRIPT);

        // User agent
        if let Some(ua) = &config.user_agent {
            builder = builder.with_user_agent(ua);
        }

        // IPC handler: JS -> Rust
        let ipc_events = Arc::clone(&events);
        builder = builder.with_ipc_handler(move |request| {
            let body = request.body().to_string();
            debug!(pane_id = pid, body = %body, "IPC message from JS");
            if let Ok(mut evts) = ipc_events.lock() {
                evts.push(WebViewEvent::IpcMessage { pane_id: pid, body });
            }
        });

        // Page load handler
        let load_events = Arc::clone(&events);
        builder = builder.with_on_page_load_handler(move |event, url| {
            let state = PageLoadState::from(event);
            debug!(pane_id = pid, ?state, url = %url, "page load");
            if let Ok(mut evts) = load_events.lock() {
                evts.push(WebViewEvent::PageLoad {
                    pane_id: pid,
                    state,
                    url,
                });
            }
        });

        // Title change handler
        let title_events = Arc::clone(&events);
        builder = builder.with_document_title_changed_handler(move |title| {
            debug!(pane_id = pid, title = %title, "title changed");
            if let Ok(mut evts) = title_events.lock() {
                evts.push(WebViewEvent::TitleChanged {
                    pane_id: pid,
                    title,
                });
            }
        });

        // Navigation handler — allow all by default
        let nav_events = Arc::clone(&events);
        builder = builder.with_navigation_handler(move |url| {
            debug!(pane_id = pid, url = %url, "navigation requested");
            if let Ok(mut evts) = nav_events.lock() {
                evts.push(WebViewEvent::NavigationRequested { pane_id: pid, url });
            }
            true // allow navigation
        });

        // Custom protocol for bundled content
        if let Some(provider) = &self.content_provider {
            let cp = Arc::clone(provider);
            builder = builder.with_custom_protocol("jarvis".to_string(), move |_wv_id, request| {
                let uri = request.uri().to_string();
                let path = uri
                    .strip_prefix("jarvis://localhost/")
                    .or_else(|| uri.strip_prefix("jarvis://localhost"))
                    .or_else(|| uri.strip_prefix("jarvis:///"))
                    .or_else(|| uri.strip_prefix("jarvis://"))
                    .unwrap_or("");

                match cp.resolve(path) {
                    Some((mime, data)) => wry::http::Response::builder()
                        .status(200)
                        .header("Content-Type", mime.as_ref())
                        .header("Access-Control-Allow-Origin", "*")
                        .body(std::borrow::Cow::from(data.into_owned()))
                        .unwrap(),
                    None => {
                        warn!(path = %path, "custom protocol: asset not found");
                        wry::http::Response::builder()
                            .status(404)
                            .body(std::borrow::Cow::from(b"Not Found".to_vec()))
                            .unwrap()
                    }
                }
            });
        }

        // Set initial content
        let initial_url;
        if let Some(url) = &config.url {
            builder = builder.with_url(url);
            initial_url = url.clone();
        } else if let Some(html) = &config.html {
            builder = builder.with_html(html);
            initial_url = "about:blank".to_string();
        } else {
            builder = builder.with_html("<html><body></body></html>");
            initial_url = "about:blank".to_string();
        }

        // Build as child WebView
        let webview = builder.build_as_child(window)?;

        debug!(pane_id, url = %initial_url, "WebView created");

        Ok(WebViewHandle {
            webview,
            pane_id,
            current_url: initial_url,
            current_title: String::new(),
        })
    }
}

impl Default for WebViewManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A registry that maps pane IDs to WebView handles.
/// This is a higher-level convenience over `WebViewManager` for
/// managing the full lifecycle.
pub struct WebViewRegistry {
    manager: WebViewManager,
    handles: HashMap<u32, WebViewHandle>,
}

impl WebViewRegistry {
    pub fn new(manager: WebViewManager) -> Self {
        Self {
            manager,
            handles: HashMap::new(),
        }
    }

    /// Create a WebView for a pane and register it.
    pub fn create<W: raw_window_handle::HasWindowHandle>(
        &mut self,
        pane_id: u32,
        window: &W,
        bounds: wry::Rect,
        config: WebViewConfig,
    ) -> Result<(), wry::Error> {
        let handle = self.manager.create(pane_id, window, bounds, config)?;
        self.handles.insert(pane_id, handle);
        Ok(())
    }

    /// Get a handle to a WebView by pane ID.
    pub fn get(&self, pane_id: u32) -> Option<&WebViewHandle> {
        self.handles.get(&pane_id)
    }

    /// Get a mutable handle to a WebView by pane ID.
    pub fn get_mut(&mut self, pane_id: u32) -> Option<&mut WebViewHandle> {
        self.handles.get_mut(&pane_id)
    }

    /// Destroy a WebView by pane ID.
    pub fn destroy(&mut self, pane_id: u32) -> bool {
        if self.handles.remove(&pane_id).is_some() {
            debug!(pane_id, "WebView destroyed");
            if let Ok(mut evts) = self.manager.events.lock() {
                evts.push(WebViewEvent::Closed { pane_id });
            }
            true
        } else {
            false
        }
    }

    /// Get all active pane IDs with WebViews.
    pub fn active_panes(&self) -> Vec<u32> {
        self.handles.keys().copied().collect()
    }

    /// Drain all pending events from all WebViews.
    pub fn drain_events(&self) -> Vec<WebViewEvent> {
        self.manager.drain_events()
    }

    /// How many WebViews are active.
    pub fn count(&self) -> usize {
        self.handles.len()
    }
}
