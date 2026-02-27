/// Log the first frame presentation (once only).
pub(crate) fn log_first_frame(width: u32, height: u32, format: wgpu::TextureFormat) {
    static PRESENTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !PRESENTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        tracing::info!(
            "First frame presented ({}x{}, format={:?})",
            width,
            height,
            format,
        );
    }
}
