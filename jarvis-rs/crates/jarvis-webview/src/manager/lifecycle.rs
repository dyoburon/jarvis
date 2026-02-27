use std::sync::Arc;

use tracing::{debug, warn};
use wry::raw_window_handle;
use wry::WebViewBuilder;

use crate::content::ContentProvider;
use crate::ipc::IPC_INIT_SCRIPT;

use super::handle::WebViewHandle;
use super::types::WebViewConfig;
use super::WebViewManager;

impl WebViewManager {
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
        builder = Self::attach_ipc_handler(builder, Arc::clone(&events), pid);

        // Page load handler
        builder = Self::attach_page_load_handler(builder, Arc::clone(&events), pid);

        // Title change handler
        builder = Self::attach_title_handler(builder, Arc::clone(&events), pid);

        // Navigation handler — allowlist: only https:// and jarvis:// schemes
        builder = Self::attach_navigation_handler(builder, Arc::clone(&events), pid);

        // Custom protocol for bundled content
        builder = self.attach_custom_protocol(builder);

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

    /// Set the content provider for serving bundled assets via `jarvis://`.
    pub fn set_content_provider(&mut self, provider: ContentProvider) {
        self.content_provider = Some(Arc::new(provider));
    }

    fn attach_custom_protocol<'a>(&self, mut builder: WebViewBuilder<'a>) -> WebViewBuilder<'a> {
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
                        .header("Access-Control-Allow-Origin", "jarvis://localhost")
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
        builder
    }
}
