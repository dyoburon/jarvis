//! Adaptive polling logic for presence and assistant events.

use std::time::Instant;

use winit::event_loop::ActiveEventLoop;

use super::core::JarvisApp;
use super::types::POLL_INTERVAL;

impl JarvisApp {
    /// Run polling and schedule the next wake-up.
    pub(super) fn poll_and_schedule(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if now.duration_since(self.last_poll) >= POLL_INTERVAL {
            self.last_poll = now;
            self.poll_presence();
            self.poll_assistant();
            self.poll_webview_events();
            self.poll_pty_output();
            self.poll_mobile_commands();
            self.poll_relay_events();
        }

        if self.needs_redraw {
            self.request_redraw();
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                Instant::now() + POLL_INTERVAL,
            ));
        }
    }
}
