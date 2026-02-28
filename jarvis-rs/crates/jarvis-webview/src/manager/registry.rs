use std::collections::HashMap;

use tracing::debug;
use wry::raw_window_handle;

use crate::events::WebViewEvent;

use super::handle::WebViewHandle;
use super::types::WebViewConfig;
use super::WebViewManager;

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

    /// Destroy all active WebViews. Used during graceful shutdown.
    pub fn destroy_all(&mut self) {
        let pane_ids = self.active_panes();
        for pane_id in pane_ids {
            self.destroy(pane_id);
        }
    }

    /// How many WebViews are active.
    pub fn count(&self) -> usize {
        self.handles.len()
    }
}
