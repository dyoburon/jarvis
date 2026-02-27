//! WebView lifecycle management: create, destroy, sync bounds, poll events.

use jarvis_common::types::Rect;
use jarvis_webview::{WebViewConfig, WebViewEvent};

use crate::app_state::core::JarvisApp;

use super::bounds::tiling_rect_to_wry;

// =============================================================================
// WEBVIEW LIFECYCLE
// =============================================================================

impl JarvisApp {
    /// Create a webview for a pane, loading the default terminal panel.
    pub(in crate::app_state) fn create_webview_for_pane(&mut self, pane_id: u32) {
        let window = match &self.window {
            Some(w) => w,
            None => {
                tracing::warn!(pane_id, "Cannot create webview: no window");
                return;
            }
        };

        let registry = match &mut self.webviews {
            Some(r) => r,
            None => {
                tracing::warn!(pane_id, "Cannot create webview: registry not initialized");
                return;
            }
        };

        // Compute the bounds for this pane from the tiling layout
        let window_size = window.inner_size();
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: window_size.width as f64,
            height: window_size.height as f64,
        };
        let layout = self.tiling.compute_layout(viewport);

        let bounds = layout
            .iter()
            .find(|(id, _)| *id == pane_id)
            .map(|(_, r)| tiling_rect_to_wry(r))
            .unwrap_or_default();

        let config = WebViewConfig::with_url("jarvis://localhost/terminal/index.html");

        if let Err(e) = registry.create(pane_id, window.as_ref(), bounds, config) {
            tracing::error!(pane_id, error = %e, "Failed to create webview");
        } else {
            tracing::info!(pane_id, "WebView created for pane");
            // Inject current theme into the new webview
            self.inject_theme_into_all_webviews();
        }
    }

    /// Destroy the webview and PTY for a pane.
    pub(in crate::app_state) fn destroy_webview_for_pane(&mut self, pane_id: u32) {
        // Kill PTY first (if any)
        if self.ptys.contains(pane_id) {
            let exit_code = self.ptys.kill_and_remove(pane_id);
            tracing::info!(pane_id, ?exit_code, "PTY killed for pane");
        }

        // Then destroy the webview
        if let Some(ref mut registry) = self.webviews {
            if registry.destroy(pane_id) {
                tracing::info!(pane_id, "WebView destroyed for pane");
            }
        }
    }

    /// Sync all webview bounds to match the current tiling layout.
    pub(in crate::app_state) fn sync_webview_bounds(&mut self) {
        let window = match &self.window {
            Some(w) => w,
            None => return,
        };
        let registry = match &mut self.webviews {
            Some(r) => r,
            None => return,
        };

        let window_size = window.inner_size();
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: window_size.width as f64,
            height: window_size.height as f64,
        };
        let layout = self.tiling.compute_layout(viewport);

        for (pane_id, rect) in &layout {
            if let Some(handle) = registry.get(*pane_id) {
                let wry_rect = tiling_rect_to_wry(rect);
                if let Err(e) = handle.set_bounds(wry_rect) {
                    tracing::warn!(
                        pane_id,
                        error = %e,
                        "Failed to update webview bounds"
                    );
                }
            }
        }
    }

    /// Process pending webview events (IPC messages, page loads, etc.).
    pub(in crate::app_state) fn poll_webview_events(&mut self) {
        let events: Vec<WebViewEvent> = match &self.webviews {
            Some(registry) => registry.drain_events(),
            None => return,
        };

        for event in events {
            match event {
                WebViewEvent::IpcMessage { pane_id, body } => {
                    self.handle_ipc_message(pane_id, &body);
                }
                WebViewEvent::PageLoad {
                    pane_id,
                    state,
                    url,
                } => {
                    tracing::debug!(
                        pane_id,
                        ?state,
                        url = %url,
                        "WebView page load event"
                    );
                }
                WebViewEvent::TitleChanged { pane_id, title } => {
                    tracing::debug!(pane_id, title = %title, "WebView title changed");
                }
                WebViewEvent::NavigationRequested { pane_id, url } => {
                    tracing::debug!(pane_id, url = %url, "WebView navigation");
                }
                WebViewEvent::Closed { pane_id } => {
                    tracing::debug!(pane_id, "WebView closed event");
                }
            }
        }
    }
}
