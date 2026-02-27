//! Adaptive polling logic for PTY output, presence, and assistant events.

use std::time::{Duration, Instant};

use winit::event_loop::ActiveEventLoop;

use super::core::JarvisApp;
use super::types::POLL_INTERVAL;

impl JarvisApp {
    /// Run adaptive polling and schedule the next wake-up.
    ///
    /// Uses a shorter interval (1ms) right after a keystroke for snappy
    /// feedback, falling back to ~120Hz (8ms) when idle.
    pub(super) fn poll_and_schedule(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        let adaptive_interval =
            if now.duration_since(self.last_pty_write) < Duration::from_millis(100) {
                Duration::from_millis(1)
            } else {
                POLL_INTERVAL
            };

        if now.duration_since(self.last_poll) >= adaptive_interval {
            self.last_poll = now;
            if self.poll_pty_output() {
                self.needs_redraw = true;
            }
            self.poll_presence();
            self.poll_assistant();
        }

        if self.needs_redraw {
            self.request_redraw();
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                Instant::now() + adaptive_interval,
            ));
        }
    }
}
