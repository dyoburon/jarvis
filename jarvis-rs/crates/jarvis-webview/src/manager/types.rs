/// Configuration for creating a new WebView instance.
#[derive(Debug, Clone)]
pub struct WebViewConfig {
    /// Initial URL to load (mutually exclusive with `html`).
    pub url: Option<String>,
    /// Initial HTML content to render (mutually exclusive with `url`).
    pub html: Option<String>,
    /// Whether the WebView background should be transparent.
    pub transparent: bool,
    /// Whether to enable dev tools (always on in debug builds).
    pub devtools: bool,
    /// Custom user agent string.
    pub user_agent: Option<String>,
    /// Whether to enable clipboard access.
    pub clipboard: bool,
    /// Whether to enable autoplay for media.
    pub autoplay: bool,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            url: None,
            html: None,
            transparent: false,
            devtools: cfg!(debug_assertions),
            user_agent: Some("Jarvis/0.1".to_string()),
            clipboard: true,
            autoplay: true,
        }
    }
}

impl WebViewConfig {
    /// Create a config that loads a URL.
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            ..Default::default()
        }
    }

    /// Create a config that renders inline HTML.
    pub fn with_html(html: impl Into<String>) -> Self {
        Self {
            html: Some(html.into()),
            ..Default::default()
        }
    }
}
