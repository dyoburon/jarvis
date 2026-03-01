use jarvis_common::types::Rect;

/// Default tab bar height in pixels.
pub(crate) const DEFAULT_TAB_BAR_HEIGHT: f32 = 32.0;

/// Default status bar height in pixels.
pub(crate) const DEFAULT_STATUS_BAR_HEIGHT: f32 = 24.0;

/// A border drawn around a terminal pane.
#[derive(Debug, Clone)]
pub struct PaneBorder {
    /// Bounding rectangle of the border.
    pub rect: Rect,
    /// Border color as RGBA (each component 0.0..=1.0).
    pub color: [f32; 4],
    /// Border line width in pixels.
    pub width: f32,
    /// Whether this pane currently has keyboard focus.
    pub is_focused: bool,
}

/// A single tab in the tab bar.
#[derive(Debug, Clone)]
pub struct Tab {
    /// The pane ID this tab represents (used for click-to-focus).
    pub pane_id: u32,
    /// Display title for the tab.
    pub title: String,
    /// Whether this tab is the currently active one.
    pub is_active: bool,
}

/// The tab bar shown at the top of the window.
#[derive(Debug, Clone)]
pub struct TabBar {
    /// All tabs in order.
    pub tabs: Vec<Tab>,
    /// Index of the active tab.
    pub active_tab: usize,
    /// Height of the tab bar in pixels.
    pub height: f32,
}

/// The status bar shown at the bottom of the window.
#[derive(Debug, Clone)]
pub struct StatusBar {
    /// Text aligned to the left.
    pub left_text: String,
    /// Text aligned to the center.
    pub center_text: String,
    /// Text aligned to the right.
    pub right_text: String,
    /// Height of the status bar in pixels.
    pub height: f32,
    /// Background color as RGBA.
    pub bg_color: [f32; 4],
    /// Foreground (text) color as RGBA.
    pub fg_color: [f32; 4],
}
