//! `ApplicationHandler` implementation for the winit event loop.

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::{CursorIcon, WindowId};

use jarvis_common::types::{PaneKind, Rect};
use jarvis_platform::input_processor::{InputResult, Modifiers};
use jarvis_platform::winit_keys::normalize_winit_key;
use jarvis_tiling::layout::borders::compute_borders;
use jarvis_tiling::tree::Direction;

use super::resize_drag::{
    cursor_zone, drag_ratio_delta, find_hovered_border, CursorZone, DragState,
};

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

        // Set up default 3-pane layout: 2 assistant + 1 chat
        self.setup_default_layout();

        self.start_presence();
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

            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_moved(position.x, position.y);
            }

            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_input(state, button);
            }

            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers.state();
            }

            WindowEvent::KeyboardInput { event, .. } => {
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

    /// Set up the default 3-pane layout: 2 assistant panels + 1 chat panel.
    ///
    /// Layout: `[Assistant | Assistant / Chat]`
    /// - Left half: primary assistant panel
    /// - Right top: secondary assistant panel
    /// - Right bottom: chat panel
    fn setup_default_layout(&mut self) {
        // Pane 1 already exists from TilingManager::new() as Terminal.
        // Re-label it as Assistant.
        let pane1 = self.tiling.focused_id();
        if let Some(pane) = self.tiling.pane_mut(pane1) {
            pane.kind = PaneKind::Assistant;
            pane.title = "Assistant".into();
        }
        self.create_webview_for_pane_with_kind(pane1, PaneKind::Assistant);

        // Split horizontally → pane 2 (right side) as Assistant
        if let Some(pane2) =
            self.tiling
                .split_with(Direction::Horizontal, PaneKind::Assistant, "Assistant")
        {
            self.create_webview_for_pane_with_kind(pane2, PaneKind::Assistant);

            // Split pane 2 vertically → pane 3 (bottom-right) as Chat
            if let Some(pane3) = self
                .tiling
                .split_with(Direction::Vertical, PaneKind::Chat, "Chat")
            {
                self.create_webview_for_pane_with_kind(pane3, PaneKind::Chat);
            }
        }

        // Focus the primary assistant (pane 1)
        self.tiling.focus_pane(pane1);
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
