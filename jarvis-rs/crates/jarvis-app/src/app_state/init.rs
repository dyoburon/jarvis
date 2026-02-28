//! Window creation, renderer initialization, and webview setup.

use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

use jarvis_renderer::RenderState;
use jarvis_webview::{ContentProvider, WebViewManager, WebViewRegistry};

use crate::boot::BootSequence;

use super::core::JarvisApp;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Relative path from the binary to the bundled panel assets.
const PANELS_DIR: &str = "assets/panels";

// =============================================================================
// INITIALIZATION
// =============================================================================

impl JarvisApp {
    /// Create the window and initialize the GPU renderer.
    /// Returns `false` if initialization failed and the event loop should exit.
    pub(super) fn initialize_window(&mut self, event_loop: &ActiveEventLoop) -> bool {
        let attrs = WindowAttributes::default()
            .with_title("Jarvis")
            .with_transparent(true)
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0));

        // macOS: transparent titlebar with content extending behind traffic lights
        #[cfg(target_os = "macos")]
        let attrs = {
            use winit::platform::macos::WindowAttributesExtMacOS;
            if self.config.window.titlebar_height > 0 {
                attrs
                    .with_titlebar_transparent(true)
                    .with_title_hidden(true)
                    .with_fullsize_content_view(true)
            } else {
                attrs
            }
        };

        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                tracing::error!("Failed to create window: {e}");
                return false;
            }
        };

        let render_state = pollster::block_on(RenderState::new(window.clone(), &self.config));

        match render_state {
            Ok(mut rs) => {
                if let Some(color) = jarvis_common::Color::from_hex(&self.config.colors.background)
                {
                    let alpha = self.config.opacity.background;
                    rs.set_clear_color_alpha(
                        srgb_to_linear(color.r as f64 / 255.0),
                        srgb_to_linear(color.g as f64 / 255.0),
                        srgb_to_linear(color.b as f64 / 255.0),
                        alpha,
                    );
                }

                self.boot = Some(BootSequence::new(&self.config));
                self.render_state = Some(rs);
            }
            Err(e) => {
                tracing::error!("Failed to initialize renderer: {e}");
                return false;
            }
        }

        // Initialize webview subsystem
        self.initialize_webviews();

        self.window = Some(window);
        tracing::info!("Window created and renderer initialized");
        true
    }

    /// Set up the WebView registry with the content provider for `jarvis://`.
    fn initialize_webviews(&mut self) {
        let panels_path = std::env::current_dir().unwrap_or_default().join(PANELS_DIR);

        if !panels_path.is_dir() {
            tracing::warn!(
                path = %panels_path.display(),
                "Panels directory not found — webviews will have no bundled content"
            );
        }

        let content_provider = ContentProvider::new(&panels_path);
        let mut manager = WebViewManager::new();
        manager.set_content_provider(content_provider);

        self.webviews = Some(WebViewRegistry::new(manager));
        tracing::info!(
            panels_dir = %panels_path.display(),
            "WebView registry initialized"
        );
    }
}

/// sRGB → linear conversion for wgpu clear color on sRGB surfaces.
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
