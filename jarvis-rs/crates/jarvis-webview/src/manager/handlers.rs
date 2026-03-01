use std::sync::{Arc, Mutex};

use tracing::{debug, warn};
use wry::WebViewBuilder;

use crate::events::{PageLoadState, WebViewEvent};

use super::WebViewManager;

// =============================================================================
// NAVIGATION ALLOWLIST
// =============================================================================

/// Allowed URL prefixes for webview navigation.
///
/// Only these origins are permitted. Everything else is blocked.
/// - `jarvis://` — custom protocol for bundled panel assets
/// - `about:blank` — default empty page
/// - Supabase — chat backend (Realtime, REST)
/// - CDN origins — xterm.js, other panel dependencies
pub const ALLOWED_NAV_PREFIXES: &[&str] = &[
    "jarvis://",
    // On Windows, WebView2 rewrites custom protocols: jarvis://localhost/… → http://jarvis.localhost/…
    "http://jarvis.localhost",
    "about:blank",
    "https://ojmqzagktzkualzgpcbq.supabase.co",
    "https://cdn.jsdelivr.net/",
    "https://unpkg.com/",
];

/// Check whether a URL is allowed by the navigation allowlist.
pub fn is_navigation_allowed(url: &str) -> bool {
    ALLOWED_NAV_PREFIXES
        .iter()
        .any(|prefix| url.starts_with(prefix))
}

// =============================================================================
// HANDLER ATTACHMENTS
// =============================================================================

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
            if !is_navigation_allowed(&url) {
                warn!(
                    pane_id = pid,
                    url = %url,
                    "navigation blocked: URL not in allowlist"
                );
                return false;
            }

            debug!(pane_id = pid, url = %url, "navigation allowed");
            if let Ok(mut evts) = events.lock() {
                evts.push(WebViewEvent::NavigationRequested { pane_id: pid, url });
            }
            true
        })
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Allowed URLs --

    #[test]
    fn allows_jarvis_protocol() {
        assert!(is_navigation_allowed(
            "jarvis://localhost/terminal/index.html"
        ));
        assert!(is_navigation_allowed("jarvis://localhost/chat/index.html"));
        assert!(is_navigation_allowed(
            "jarvis://localhost/games/tetris.html"
        ));
    }

    #[test]
    fn allows_about_blank() {
        assert!(is_navigation_allowed("about:blank"));
    }

    #[test]
    fn allows_supabase_origin() {
        assert!(is_navigation_allowed(
            "https://ojmqzagktzkualzgpcbq.supabase.co/rest/v1/channels"
        ));
        assert!(is_navigation_allowed(
            "https://ojmqzagktzkualzgpcbq.supabase.co/realtime/v1/websocket"
        ));
    }

    #[test]
    fn allows_cdn_origins() {
        assert!(is_navigation_allowed(
            "https://cdn.jsdelivr.net/npm/xterm@5.5.0/css/xterm.css"
        ));
        assert!(is_navigation_allowed(
            "https://unpkg.com/some-package@1.0.0/dist/index.js"
        ));
    }

    // -- Blocked URLs --

    #[test]
    fn blocks_arbitrary_https() {
        assert!(!is_navigation_allowed("https://evil.com"));
        assert!(!is_navigation_allowed("https://google.com"));
        assert!(!is_navigation_allowed("https://example.com/phishing"));
    }

    #[test]
    fn blocks_file_protocol() {
        assert!(!is_navigation_allowed("file:///etc/passwd"));
        assert!(!is_navigation_allowed("file:///Users/cw/.ssh/id_rsa"));
        assert!(!is_navigation_allowed("file://localhost/etc/hosts"));
    }

    #[test]
    fn allows_webview2_rewritten_custom_protocol() {
        // WebView2 on Windows rewrites jarvis://localhost/… → http://jarvis.localhost/…
        assert!(is_navigation_allowed(
            "http://jarvis.localhost/boot/index.html"
        ));
        assert!(is_navigation_allowed(
            "http://jarvis.localhost/terminal/index.html"
        ));
    }

    #[test]
    fn blocks_http_unencrypted() {
        assert!(!is_navigation_allowed("http://evil.com"));
        assert!(!is_navigation_allowed("http://localhost:8080"));
    }

    #[test]
    fn blocks_javascript_protocol() {
        assert!(!is_navigation_allowed("javascript:alert(1)"));
        assert!(!is_navigation_allowed("javascript:void(0)"));
    }

    #[test]
    fn blocks_data_protocol() {
        assert!(!is_navigation_allowed("data:text/html,<h1>XSS</h1>"));
        assert!(!is_navigation_allowed(
            "data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg=="
        ));
    }

    #[test]
    fn blocks_empty_and_garbage() {
        assert!(!is_navigation_allowed(""));
        assert!(!is_navigation_allowed("   "));
        assert!(!is_navigation_allowed("not-a-url"));
        assert!(!is_navigation_allowed("ftp://files.example.com"));
    }

    #[test]
    fn blocks_similar_but_wrong_supabase() {
        // Different project ID — not our Supabase
        assert!(!is_navigation_allowed(
            "https://xyzabc123.supabase.co/rest/v1/data"
        ));
    }

    // -- Allowlist structure --

    #[test]
    fn allowlist_has_expected_entries() {
        assert_eq!(ALLOWED_NAV_PREFIXES.len(), 6);
        assert!(ALLOWED_NAV_PREFIXES.contains(&"jarvis://"));
        assert!(ALLOWED_NAV_PREFIXES.contains(&"about:blank"));
    }
}
