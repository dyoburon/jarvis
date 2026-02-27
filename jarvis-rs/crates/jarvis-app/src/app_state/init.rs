//! Window creation and renderer initialization.

use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

use jarvis_renderer::RenderState;

use super::core::JarvisApp;

impl JarvisApp {
    /// Create the window and initialize the GPU renderer.
    /// Returns `false` if initialization failed and the event loop should exit.
    pub(super) fn initialize_window(&mut self, event_loop: &ActiveEventLoop) -> bool {
        let attrs = WindowAttributes::default()
            .with_title("Jarvis")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0));

        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                tracing::error!("Failed to create window: {e}");
                return false;
            }
        };

        let render_state = pollster::block_on(RenderState::new(window.clone()));

        match render_state {
            Ok(mut rs) => {
                if let Some(color) = jarvis_common::Color::from_hex(&self.config.colors.background)
                {
                    rs.set_clear_color(
                        color.r as f64 / 255.0,
                        color.g as f64 / 255.0,
                        color.b as f64 / 255.0,
                    );
                }
                self.render_state = Some(rs);
            }
            Err(e) => {
                tracing::error!("Failed to initialize renderer: {e}");
                return false;
            }
        }

        self.window = Some(window);
        tracing::info!("Window created and renderer initialized");
        true
    }
}
