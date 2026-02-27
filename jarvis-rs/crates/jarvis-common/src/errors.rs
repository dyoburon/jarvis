use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("config parse error: {0}")]
    ParseError(String),

    #[error("config validation error: {0}")]
    ValidationError(String),

    #[error("config watch error: {0}")]
    WatchError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("clipboard error: {0}")]
    ClipboardError(String),

    #[error("path error: {0}")]
    PathError(String),

    #[error("audio error: {0}")]
    AudioError(String),

    #[error("window manager error: {0}")]
    WindowManagerError(String),

    #[error("notification error: {0}")]
    NotificationError(String),

    #[error("not supported: {0}")]
    NotSupported(String),
}

#[derive(Debug, thiserror::Error)]
pub enum JarvisError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Platform(#[from] PlatformError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("network error: {0}")]
    Network(String),

    #[error("ai error: {0}")]
    Ai(String),

    #[error("social error: {0}")]
    Social(String),

    #[error("terminal error: {0}")]
    Terminal(String),

    #[error("renderer error: {0}")]
    Renderer(String),

    #[error("webview error: {0}")]
    WebView(String),

    #[error("{0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_display() {
        let err = ConfigError::FileNotFound(PathBuf::from("/tmp/missing.toml"));
        assert_eq!(err.to_string(), "config file not found: /tmp/missing.toml");

        let err = ConfigError::ParseError("unexpected token".into());
        assert_eq!(err.to_string(), "config parse error: unexpected token");

        let err = ConfigError::ValidationError("missing field 'name'".into());
        assert_eq!(
            err.to_string(),
            "config validation error: missing field 'name'"
        );

        let err = ConfigError::WatchError("inotify limit reached".into());
        assert_eq!(err.to_string(), "config watch error: inotify limit reached");
    }

    #[test]
    fn platform_error_display() {
        let err = PlatformError::ClipboardError("access denied".into());
        assert_eq!(err.to_string(), "clipboard error: access denied");

        let err = PlatformError::NotSupported("linux wayland".into());
        assert_eq!(err.to_string(), "not supported: linux wayland");
    }

    #[test]
    fn jarvis_error_from_config() {
        let config_err = ConfigError::ParseError("bad toml".into());
        let jarvis_err: JarvisError = config_err.into();
        assert!(matches!(jarvis_err, JarvisError::Config(_)));
        assert!(jarvis_err.to_string().contains("bad toml"));
    }

    #[test]
    fn jarvis_error_from_platform() {
        let platform_err = PlatformError::AudioError("device not found".into());
        let jarvis_err: JarvisError = platform_err.into();
        assert!(matches!(jarvis_err, JarvisError::Platform(_)));
        assert!(jarvis_err.to_string().contains("device not found"));
    }

    #[test]
    fn jarvis_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let jarvis_err: JarvisError = io_err.into();
        assert!(matches!(jarvis_err, JarvisError::Io(_)));
        assert!(jarvis_err.to_string().contains("file missing"));
    }

    #[test]
    fn jarvis_error_other_variants() {
        let err = JarvisError::Network("timeout".into());
        assert_eq!(err.to_string(), "network error: timeout");

        let err = JarvisError::Ai("model unavailable".into());
        assert_eq!(err.to_string(), "ai error: model unavailable");

        let err = JarvisError::Social("connection refused".into());
        assert_eq!(err.to_string(), "social error: connection refused");

        let err = JarvisError::Terminal("pty allocation failed".into());
        assert_eq!(err.to_string(), "terminal error: pty allocation failed");

        let err = JarvisError::Renderer("gpu not found".into());
        assert_eq!(err.to_string(), "renderer error: gpu not found");

        let err = JarvisError::WebView("js error".into());
        assert_eq!(err.to_string(), "webview error: js error");

        let err = JarvisError::Other("something went wrong".into());
        assert_eq!(err.to_string(), "something went wrong");
    }
}
