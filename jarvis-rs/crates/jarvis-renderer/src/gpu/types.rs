/// Errors that can occur during GPU rendering operations.
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

/// Physical pixel dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}
