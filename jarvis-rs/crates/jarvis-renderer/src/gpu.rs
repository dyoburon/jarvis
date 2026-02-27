use std::sync::Arc;
use winit::window::Window;

// ---------------------------------------------------------------------------
// RendererError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("surface error: {0}")]
    SurfaceError(String),

    #[error("no suitable GPU adapter found")]
    AdapterNotFound,

    #[error("device error: {0}")]
    DeviceError(String),

    #[error("text rendering error: {0}")]
    TextError(String),
}

impl From<wgpu::SurfaceError> for RendererError {
    fn from(e: wgpu::SurfaceError) -> Self {
        RendererError::SurfaceError(e.to_string())
    }
}

impl From<wgpu::RequestDeviceError> for RendererError {
    fn from(e: wgpu::RequestDeviceError) -> Self {
        RendererError::DeviceError(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// PhysicalSize
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

// ---------------------------------------------------------------------------
// GpuContext
// ---------------------------------------------------------------------------

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize,
    pub scale_factor: f64,
}

impl GpuContext {
    /// Initialize wgpu: create instance, surface, adapter, device, and configure
    /// the surface for rendering.
    pub async fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let inner_size = window.inner_size();
        let scale_factor = window.scale_factor();

        let width = inner_size.width.max(1);
        let height = inner_size.height.max(1);

        // 1. Create Instance with default backends
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        // 2. Create surface from window
        let surface = instance
            .create_surface(window)
            .map_err(|e| RendererError::SurfaceError(e.to_string()))?;

        // 3. Request adapter (prefer high-performance GPU, fallback to software)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await;

        let adapter = match adapter {
            Some(a) => a,
            None => {
                tracing::warn!("No hardware GPU adapter found, trying software fallback");
                instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::LowPower,
                        force_fallback_adapter: true,
                        compatible_surface: Some(&surface),
                    })
                    .await
                    .ok_or(RendererError::AdapterNotFound)?
            }
        };

        let adapter_info = adapter.get_info();
        tracing::info!(
            "GPU adapter: {} ({:?}, {:?})",
            adapter_info.name,
            adapter_info.device_type,
            adapter_info.backend,
        );

        // 4. Request device with default limits
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("jarvis-renderer device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await?;

        // 5. Use the surface's preferred format
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .first()
            .copied()
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);
        tracing::info!(
            "Surface format: {format:?} (available: {:?})",
            surface_caps.formats
        );

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            size: PhysicalSize { width, height },
            scale_factor,
        })
    }

    /// Reconfigure the surface after a window resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);

        self.size = PhysicalSize { width, height };
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Get the next frame's surface texture.
    pub fn current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    /// Return the surface texture format.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_error_adapter_not_found_display() {
        let err = RendererError::AdapterNotFound;
        assert_eq!(err.to_string(), "no suitable GPU adapter found");
    }

    #[test]
    fn renderer_error_surface_display() {
        let err = RendererError::SurfaceError("timeout".to_string());
        assert_eq!(err.to_string(), "surface error: timeout");
    }

    #[test]
    fn renderer_error_device_display() {
        let err = RendererError::DeviceError("out of memory".to_string());
        assert_eq!(err.to_string(), "device error: out of memory");
    }

    #[test]
    fn renderer_error_text_display() {
        let err = RendererError::TextError("atlas full".to_string());
        assert_eq!(err.to_string(), "text rendering error: atlas full");
    }

    #[test]
    fn physical_size_copy_and_eq() {
        let a = PhysicalSize {
            width: 800,
            height: 600,
        };
        let b = a;
        assert_eq!(a, b);
    }
}
