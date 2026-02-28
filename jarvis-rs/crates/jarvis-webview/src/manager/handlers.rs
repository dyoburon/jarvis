use std::sync::{Arc, Mutex};

use tracing::{debug, warn};
use wry::WebViewBuilder;

use crate::events::{PageLoadState, WebViewEvent};

use super::WebViewManager;

impl WebViewManager {
    pub(super) fn attach_ipc_handler<'a>(
        builder: WebViewBuilder<'a>,
        events: Arc<Mutex<Vec<WebViewEvent>>>,
        pid: u32,
    ) -> WebViewBuilder<'a> {
        builder.with_ipc_handler(move |request| {
            let body = request.body().to_string();

            // Validate that the IPC body is valid JSON before forwarding
            if serde_json::from_str::<serde_json::Value>(&body).is_err() {
                warn!(
                    pane_id = pid,
                    body_len = body.len(),
                    "IPC message rejected: invalid JSON"
                );
                return;
            }

            debug!(pane_id = pid, body_len = body.len(), "IPC message from JS");
            if let Ok(mut evts) = events.lock() {
                evts.push(WebViewEvent::IpcMessage { pane_id: pid, body });
            }
        })
    }

    pub(super) fn attach_page_load_handler<'a>(
        builder: WebViewBuilder<'a>,
        events: Arc<Mutex<Vec<WebViewEvent>>>,
        pid: u32,
    ) -> WebViewBuilder<'a> {
        builder.with_on_page_load_handler(move |event, url| {
            let state = PageLoadState::from(event);
            debug!(pane_id = pid, ?state, url = %url, "page load");
            if let Ok(mut evts) = events.lock() {
                evts.push(WebViewEvent::PageLoad {
                    pane_id: pid,
                    state,
                    url,
                });
            }
        })
    }

    pub(super) fn attach_title_handler<'a>(
        builder: WebViewBuilder<'a>,
        events: Arc<Mutex<Vec<WebViewEvent>>>,
        pid: u32,
    ) -> WebViewBuilder<'a> {
        builder.with_document_title_changed_handler(move |title| {
            debug!(pane_id = pid, title = %title, "title changed");
            if let Ok(mut evts) = events.lock() {
                evts.push(WebViewEvent::TitleChanged {
                    pane_id: pid,
                    title,
                });
            }
        })
    }

    pub(super) fn attach_navigation_handler<'a>(
        builder: WebViewBuilder<'a>,
        events: Arc<Mutex<Vec<WebViewEvent>>>,
        pid: u32,
    ) -> WebViewBuilder<'a> {
        builder.with_navigation_handler(move |url| {
            let allowed = url.starts_with("https://") || url.starts_with("jarvis://");
            if !allowed {
                warn!(pane_id = pid, url = %url, "navigation blocked: scheme not in allowlist");
                return false;
            }

            debug!(pane_id = pid, url = %url, "navigation requested");
            if let Ok(mut evts) = events.lock() {
                evts.push(WebViewEvent::NavigationRequested { pane_id: pid, url });
            }
            true
        })
    }
}
