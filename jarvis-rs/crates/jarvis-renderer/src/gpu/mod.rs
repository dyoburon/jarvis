mod context;
mod types;
mod uniforms;

pub use context::*;
pub use types::*;
pub use uniforms::*;

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
