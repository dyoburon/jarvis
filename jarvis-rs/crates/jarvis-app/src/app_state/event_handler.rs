//! `ApplicationHandler` implementation for the winit event loop.

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::{CursorIcon, WindowId};

use jarvis_common::types::{PaneKind, Rect};
use jarvis_config::schema::PanelKind;
use jarvis_platform::input_processor::{InputResult, Modifiers};
use jarvis_platform::winit_keys::normalize_winit_key;
use jarvis_tiling::layout::borders::compute_borders;
use jarvis_tiling::tree::Direction;

use super::resize_drag::{
    cursor_zone, drag_ratio_delta, find_hovered_border, CursorZone, DragState,
};

use crate::boot::BootPhase;

use super::core::JarvisApp;

impl ApplicationHandler for JarvisApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        if !self.initialize_window(event_loop) {
            event_loop.exit();
            return;
        }

        // If boot animation is active, show fullscreen boot webview.
        // Otherwise set up panels immediately.
        let booting = self
            .boot
            .as_ref()
            .is_some_and(|b| b.phase() == BootPhase::Splash);

        if booting {
            self.show_boot_webview();
        } else {
            self.setup_default_layout();
        }

        self.start_presence();
        self.start_relay_client();
        self.update_window_title();
        self.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Window close requested");
                self.shutdown();
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(ref mut rs) = self.render_state {
                        rs.resize(size.width, size.height);
                    }
                    self.sync_webview_bounds();
                    self.needs_redraw = true;
                }
            }

            WindowEvent::ScaleFactorChanged { .. } => {
                // Monitor changed (e.g. dragged to a different DPI display).
                // winit will follow up with a Resized event using the new
                // physical size, but we also need to re-sync webview bounds
                // and redraw immediately to avoid visual glitches.
                if let Some(ref w) = self.window {
                    let size = w.inner_size();
                    if size.width > 0 && size.height > 0 {
                        if let Some(ref mut rs) = self.render_state {
                            rs.resize(size.width, size.height);
                        }
                        self.sync_webview_bounds();
                        self.needs_redraw = true;
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_moved(position.x, position.y);
            }

            WindowEvent::MouseInput { state, button, .. } => {
                tracing::info!(
                    ?state, ?button,
                    x = self.cursor_pos.0, y = self.cursor_pos.1,
                    "[winit] MouseInput"
                );
                self.handle_mouse_input(state, button);
            }

            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers.state();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                tracing::info!(
                    key = ?event.logical_key,
                    state = ?event.state,
                    "[winit] KeyboardInput"
                );
                self.handle_keyboard_input(event);
            }

            WindowEvent::RedrawRequested => {
                if self.should_exit {
                    event_loop.exit();
                    return;
                }
                self.update_chrome();
                self.render_frame();
                self.needs_redraw = false;
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.should_exit {
            event_loop.exit();
            return;
        }

        // Boot webview sends 'boot:complete' IPC when done — handled in
        // ipc_dispatch. Nothing to drive here.

        self.poll_and_schedule(event_loop);
    }
}

impl JarvisApp {
    /// Process a keyboard input event: route to overlays or dispatch actions.
    fn handle_keyboard_input(&mut self, event: KeyEvent) {
        let KeyEvent {
            logical_key, state, ..
        } = event;
        let is_press = state == ElementState::Pressed;

        let key_name = match &logical_key {
            Key::Named(named) => format!("{named:?}"),
            Key::Character(c) => c.to_string(),
            _ => return,
        };

        let normalized = normalize_winit_key(&key_name);

        // If command palette is open, route keys there first
        if self.command_palette_open && is_press && self.handle_palette_key(&normalized, is_press) {
            self.needs_redraw = true;
            return;
        }

        // If assistant is open, route keys there
        if self.assistant_open && is_press && self.handle_assistant_key(&normalized, is_press) {
            self.needs_redraw = true;
            return;
        }

        let mods = Modifiers {
            ctrl: self.modifiers.control_key(),
            alt: self.modifiers.alt_key(),
            shift: self.modifiers.shift_key(),
            super_key: self.modifiers.super_key(),
        };
        let result = self
            .input
            .process_key(&self.registry, &normalized, mods, is_press);

        match result {
            InputResult::Action(action) => {
                self.dispatch(action);
            }
            InputResult::TerminalInput(_bytes) => {
                // Will be forwarded to xterm.js webview in future phases
            }
            InputResult::Consumed => {}
        }
    }

    /// Opens panels from `config.auto_open.panels`.
    /// Falls back to a single terminal if the list is empty.
    pub(super) fn setup_default_layout(&mut self) {
        let panels = self.config.auto_open.panels.clone();

        let first = panels.first();
        let first_kind = first
            .map(|p| config_panel_to_pane_kind(&p.kind))
            .unwrap_or(PaneKind::Terminal);
        let first_title = first.and_then(|p| p.title.as_deref()).unwrap_or("Terminal");
        let first_url = first
            .map(|p| panel_kind_to_url(&p.kind))
            .unwrap_or("jarvis://localhost/terminal/index.html");

        let pane1 = self.tiling.focused_id();
        if let Some(pane) = self.tiling.pane_mut(pane1) {
            pane.kind = first_kind;
            pane.title = first_title.into();
        }
        self.create_webview_for_pane_with_url(pane1, first_url);

        for panel in panels.iter().skip(1) {
            let kind = config_panel_to_pane_kind(&panel.kind);
            let title = panel
                .title
                .as_deref()
                .unwrap_or(panel_kind_default_title(&panel.kind));
            let url = panel_kind_to_url(&panel.kind);
            if let Some(new_id) = self.tiling.split_with(Direction::Horizontal, kind, title) {
                self.create_webview_for_pane_with_url(new_id, url);
            }
        }

        self.tiling.focus_pane(pane1);
        self.notify_focus_changed();
        self.sync_webview_bounds();
    }

    /// Render a single frame (background only — panels are webviews).
    fn render_frame(&mut self) {
        if let Some(ref mut rs) = self.render_state {
            if let Err(e) = rs.render_background() {
                tracing::error!("Render error: {e}");
            }
        }
    }

    /// Compute the current viewport rect from the window.
    fn viewport(&self) -> Rect {
        match &self.window {
            Some(w) => {
                let size = w.inner_size();
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: size.width as f64,
                    height: size.height as f64,
                }
            }
            None => Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        }
    }

    /// Handle cursor movement: update cursor icon near borders and
    /// adjust split ratios during active drag.
    fn handle_cursor_moved(&mut self, x: f64, y: f64) {
        self.cursor_pos = (x, y);

        // If actively dragging, update the split ratio
        if let Some(ref drag) = self.drag_state {
            let current_pos = match drag.border.direction {
                Direction::Horizontal => x,
                Direction::Vertical => y,
            };
            let ratio_delta = drag_ratio_delta(drag, current_pos);
            let pane_id = drag.border.first_pane;
            self.tiling.tree_mut().adjust_ratio(pane_id, ratio_delta);

            // Update start position for incremental dragging
            if let Some(ref mut drag) = self.drag_state {
                drag.start_pos = current_pos;
            }

            self.sync_webview_bounds();
            self.needs_redraw = true;
            return;
        }

        // Not dragging — update cursor icon based on proximity to borders
        let viewport = self.viewport();
        let gap = self.tiling.gap() as f64;
        let borders = compute_borders(self.tiling.tree(), viewport, gap);
        let hovered = find_hovered_border(&borders, x, y);

        let zone = cursor_zone(hovered);
        let icon = match zone {
            CursorZone::ColResize => CursorIcon::ColResize,
            CursorZone::RowResize => CursorIcon::RowResize,
            CursorZone::None => CursorIcon::Default,
        };

        if let Some(ref w) = self.window {
            w.set_cursor(icon);
        }
    }

    /// Handle mouse button press/release: start or stop drag resize.
    fn handle_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        if button != MouseButton::Left {
            return;
        }

        match state {
            ElementState::Pressed => {
                let (x, y) = self.cursor_pos;
                let viewport = self.viewport();
                let gap = self.tiling.gap() as f64;
                let borders = compute_borders(self.tiling.tree(), viewport, gap);

                if let Some(border) = find_hovered_border(&borders, x, y) {
                    let start_pos = match border.direction {
                        Direction::Horizontal => x,
                        Direction::Vertical => y,
                    };
                    self.drag_state = Some(DragState {
                        border: border.clone(),
                        start_pos,
                    });
                } else {
                    // Click is not on a resize border — check if it's in a
                    // draggable zone and initiate window drag.
                    let is_booting = self
                        .boot
                        .as_ref()
                        .is_some_and(|b| b.phase() == crate::boot::BootPhase::Splash);

                    let should_drag = if is_booting {
                        // During boot: anywhere drags the window
                        true
                    } else {
                        // After loaded: the titlebar area (top 38px) or
                        // chrome gaps between/around panels are drag zones
                        let titlebar_h = self.config.window.titlebar_height as f64;
                        if y < titlebar_h {
                            true
                        } else {
                            let layout = self.tiling.compute_layout(viewport);
                            !layout.iter().any(|(_, rect)| {
                                x >= rect.x
                                    && x < rect.x + rect.width
                                    && y >= rect.y
                                    && y < rect.y + rect.height
                            })
                        }
                    };

                    if should_drag {
                        if let Some(ref w) = self.window {
                            let _ = w.drag_window();
                        }
                    } else {
                        // Click is inside a pane — focus it and give native
                        // focus to its webview so it receives keyboard input.
                        let layout = self.tiling.compute_layout(viewport);
                        if let Some((pane_id, _)) = layout.iter().find(|(_, rect)| {
                            x >= rect.x
                                && x < rect.x + rect.width
                                && y >= rect.y
                                && y < rect.y + rect.height
                        }) {
                            let pid = *pane_id;
                            if pid != self.tiling.focused_id() {
                                self.tiling.focus_pane(pid);
                                self.notify_focus_changed();
                                self.needs_redraw = true;
                            } else if let Some(ref registry) = self.webviews {
                                if let Some(handle) = registry.get(pid) {
                                    let _ = handle.focus();
                                }
                            }
                        }
                    }
                }
            }
            ElementState::Released => {
                if self.drag_state.is_some() {
                    self.drag_state = None;
                    // Reset cursor
                    if let Some(ref w) = self.window {
                        w.set_cursor(CursorIcon::Default);
                    }
                }
            }
        }
    }
}

// =============================================================================
// AUTO-OPEN HELPERS
// =============================================================================

/// Map config `PanelKind` to runtime `PaneKind`.
fn config_panel_to_pane_kind(kind: &PanelKind) -> PaneKind {
    match kind {
        PanelKind::Terminal => PaneKind::Terminal,
        PanelKind::Assistant => PaneKind::Assistant,
        PanelKind::Chat => PaneKind::Chat,
        PanelKind::Settings | PanelKind::Presence => PaneKind::WebView,
    }
}

/// Map config `PanelKind` to `jarvis://` URL.
fn panel_kind_to_url(kind: &PanelKind) -> &'static str {
    match kind {
        PanelKind::Terminal => "jarvis://localhost/terminal/index.html",
        PanelKind::Assistant => "jarvis://localhost/assistant/index.html",
        PanelKind::Chat => "jarvis://localhost/chat/index.html",
        PanelKind::Settings => "jarvis://localhost/settings/index.html",
        PanelKind::Presence => "jarvis://localhost/presence/index.html",
    }
}

/// Default display title for a `PanelKind`.
fn panel_kind_default_title(kind: &PanelKind) -> &'static str {
    match kind {
        PanelKind::Terminal => "Terminal",
        PanelKind::Assistant => "Assistant",
        PanelKind::Chat => "Chat",
        PanelKind::Settings => "Settings",
        PanelKind::Presence => "Presence",
    }
}
