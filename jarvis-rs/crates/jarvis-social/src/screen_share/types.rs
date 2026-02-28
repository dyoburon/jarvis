//! Types, configuration, and events for screen sharing sessions.

use std::collections::HashSet;

use crate::protocol::ScreenShareSignal;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Quality preset for screen sharing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ShareQuality {
    /// 720p, 10fps — low bandwidth.
    Low,
    /// 1080p, 15fps.
    #[default]
    Medium,
    /// 1080p, 30fps.
    High,
    /// Native resolution, 30fps.
    Ultra,
}

impl ShareQuality {
    pub fn max_width(&self) -> u32 {
        match self {
            Self::Low => 1280,
            Self::Medium => 1920,
            Self::High => 1920,
            Self::Ultra => 3840,
        }
    }

    pub fn max_height(&self) -> u32 {
        match self {
            Self::Low => 720,
            Self::Medium => 1080,
            Self::High => 1080,
            Self::Ultra => 2160,
        }
    }

    pub fn max_fps(&self) -> u32 {
        match self {
            Self::Low => 10,
            Self::Medium => 15,
            Self::High => 30,
            Self::Ultra => 30,
        }
    }
}

/// An active screen sharing session.
#[derive(Debug, Clone)]
pub struct ScreenShareSession {
    pub session_id: String,
    pub host_user_id: String,
    pub host_display_name: String,
    pub quality: ShareQuality,
    /// Users currently viewing the screen share.
    pub viewers: HashSet<String>,
    /// Whether the host is sharing a specific window vs full screen.
    pub window_title: Option<String>,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events emitted by the screen share system.
#[derive(Debug, Clone)]
pub enum ScreenShareEvent {
    SessionStarted {
        session_id: String,
        host_user_id: String,
        host_display_name: String,
    },
    SessionStopped {
        session_id: String,
        host_user_id: String,
    },
    ViewerJoined {
        session_id: String,
        viewer_user_id: String,
        viewer_display_name: String,
    },
    ViewerLeft {
        session_id: String,
        viewer_user_id: String,
    },
    QualityChanged {
        session_id: String,
        quality: ShareQuality,
    },
    /// WebRTC signaling message — forward to the transport layer.
    Signal {
        from_user: String,
        signal: ScreenShareSignal,
    },
    Error(String),
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for screen sharing.
#[derive(Debug, Clone)]
pub struct ScreenShareConfig {
    pub enabled: bool,
    pub default_quality: ShareQuality,
    /// Maximum concurrent viewers per session.
    pub max_viewers: usize,
}

impl Default for ScreenShareConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_quality: ShareQuality::Medium,
            max_viewers: 4,
        }
    }
}
