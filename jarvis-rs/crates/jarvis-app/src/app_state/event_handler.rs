//! `ApplicationHandler` implementation for the winit event loop.

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::WindowId;

use jarvis_platform::input_processor::{InputResult, Modifiers};
use jarvis_platform::winit_keys::normalize_winit_key;

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

        self.start_presence();
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
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(ref mut rs) = self.render_state {
                        rs.resize(size.width, size.height);
                    }
                    self.needs_redraw = true;
                }
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

    /// Render a single frame (background only — panels are webviews).
    fn render_frame(&mut self) {
        if let Some(ref mut rs) = self.render_state {
            if let Err(e) = rs.render_background() {
                tracing::error!("Render error: {e}");
            }
        }
    }
}
