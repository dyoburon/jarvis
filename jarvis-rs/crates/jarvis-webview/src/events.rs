//! WebView event types.

use serde::{Deserialize, Serialize};

/// State of a page load lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageLoadState {
    /// Navigation has started.
    Started,
    /// Page has fully loaded (DOMContentLoaded + resources).
    Finished,
}

impl From<wry::PageLoadEvent> for PageLoadState {
    fn from(e: wry::PageLoadEvent) -> Self {
        match e {
            wry::PageLoadEvent::Started => Self::Started,
            wry::PageLoadEvent::Finished => Self::Finished,
        }
    }
}

/// Events emitted by a WebView instance.
#[derive(Debug, Clone)]
pub enum WebViewEvent {
    /// Page load state changed. Carries the URL.
    PageLoad {
        pane_id: u32,
        state: PageLoadState,
        url: String,
    },
    /// Document title changed.
    TitleChanged {
        pane_id: u32,
        title: String,
    },
    /// An IPC message was received from JavaScript.
    IpcMessage {
        pane_id: u32,
        body: String,
    },
    /// A navigation was requested. If `allowed` is false, it was blocked.
    NavigationRequested {
        pane_id: u32,
        url: String,
    },
    /// WebView was closed / destroyed.
    Closed {
        pane_id: u32,
    },
}
