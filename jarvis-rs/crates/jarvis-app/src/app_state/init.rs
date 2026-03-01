//! Window creation, renderer initialization, and webview setup.

use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::{Icon, WindowAttributes};

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
        let mut attrs = WindowAttributes::default()
            .with_title("Jarvis")
            .with_transparent(true)
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0));

        // Load window icon from embedded PNG
        if let Some(icon) = load_window_icon() {
            attrs = attrs.with_window_icon(Some(icon));
        }

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

        // Initialize crypto identity (load or generate)
        match jarvis_platform::identity_file() {
            Ok(path) => match jarvis_platform::CryptoService::load_or_generate(&path) {
                Ok(svc) => {
                    tracing::info!(fingerprint = %svc.fingerprint, "Crypto identity loaded");
                    self.crypto = Some(svc);
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to initialize crypto service");
                }
            },
            Err(e) => {
                tracing::error!(error = %e, "Failed to resolve identity file path");
            }
        }

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

/// Load the application icon from the bundled PNG asset.
fn load_window_icon() -> Option<Icon> {
    let icon_bytes = include_bytes!("../../../../assets/jarvis-icon.png");
    let decoder = png::Decoder::new(icon_bytes.as_slice());
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());

    // Convert RGB to RGBA if needed
    let rgba = if info.color_type == png::ColorType::Rgb {
        let mut rgba = Vec::with_capacity(buf.len() / 3 * 4);
        for chunk in buf.chunks(3) {
            rgba.extend_from_slice(chunk);
            rgba.push(255);
        }
        rgba
    } else {
        buf
    };

    Icon::from_rgba(rgba, info.width, info.height).ok()
}

/// sRGB → linear conversion for wgpu clear color on sRGB surfaces.
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
