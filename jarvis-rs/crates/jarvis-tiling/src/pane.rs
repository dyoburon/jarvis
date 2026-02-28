use jarvis_common::types::{PaneId, PaneKind};

#[derive(Debug)]
pub struct Pane {
    pub id: PaneId,
    pub kind: PaneKind,
    pub title: String,
}

impl Pane {
    pub fn new_terminal(id: PaneId, title: impl Into<String>) -> Self {
        Self {
            id,
            kind: PaneKind::Terminal,
            title: title.into(),
        }
    }

    pub fn new_webview(id: PaneId, title: impl Into<String>) -> Self {
        Self {
            id,
            kind: PaneKind::WebView,
            title: title.into(),
        }
    }

    pub fn new_assistant(id: PaneId, title: impl Into<String>) -> Self {
        Self {
            id,
            kind: PaneKind::Assistant,
            title: title.into(),
        }
    }

    pub fn new_chat(id: PaneId, title: impl Into<String>) -> Self {
        Self {
            id,
            kind: PaneKind::Chat,
            title: title.into(),
        }
    }

    pub fn new_external(id: PaneId, title: impl Into<String>) -> Self {
        Self {
            id,
            kind: PaneKind::ExternalApp,
            title: title.into(),
        }
    }
}
