//! Screen sharing session management.
//!
//! Tracks active screen share sessions, viewer lists, and quality
//! settings. Like voice, the actual media transport is WebRTC P2P —
//! this module handles coordination and signaling relay.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

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

// ---------------------------------------------------------------------------
// Screen Share Manager
// ---------------------------------------------------------------------------

/// Manages screen sharing sessions.
pub struct ScreenShareManager {
    config: ScreenShareConfig,
    /// Active sessions keyed by session_id.
    sessions: Arc<RwLock<HashMap<String, ScreenShareSession>>>,
    /// host_user_id → session_id (a user can only host one session).
    host_sessions: Arc<RwLock<HashMap<String, String>>>,
    event_tx: mpsc::Sender<ScreenShareEvent>,
}

impl ScreenShareManager {
    pub fn new(config: ScreenShareConfig) -> (Self, mpsc::Receiver<ScreenShareEvent>) {
        let (event_tx, event_rx) = mpsc::channel(256);
        let mgr = Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            host_sessions: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        };
        (mgr, event_rx)
    }

    /// Start sharing your screen.
    pub async fn start_sharing(
        &self,
        session_id: &str,
        user_id: &str,
        display_name: &str,
        window_title: Option<String>,
    ) -> Result<(), String> {
        if !self.config.enabled {
            return Err("Screen sharing is disabled".into());
        }

        // Stop any existing session by this user
        self.stop_sharing(user_id).await;

        let session = ScreenShareSession {
            session_id: session_id.to_string(),
            host_user_id: user_id.to_string(),
            host_display_name: display_name.to_string(),
            quality: self.config.default_quality,
            viewers: HashSet::new(),
            window_title,
        };

        self.sessions
            .write()
            .await
            .insert(session_id.to_string(), session);
        self.host_sessions
            .write()
            .await
            .insert(user_id.to_string(), session_id.to_string());

        let _ = self
            .event_tx
            .send(ScreenShareEvent::SessionStarted {
                session_id: session_id.to_string(),
                host_user_id: user_id.to_string(),
                host_display_name: display_name.to_string(),
            })
            .await;

        info!(session_id, user_id, "Screen share started");
        Ok(())
    }

    /// Stop sharing (as the host).
    pub async fn stop_sharing(&self, user_id: &str) {
        let session_id = self.host_sessions.write().await.remove(user_id);
        if let Some(session_id) = session_id {
            self.sessions.write().await.remove(&session_id);
            let _ = self
                .event_tx
                .send(ScreenShareEvent::SessionStopped {
                    session_id,
                    host_user_id: user_id.to_string(),
                })
                .await;
            info!(user_id, "Screen share stopped");
        }
    }

    /// Join as a viewer of someone's screen share.
    pub async fn join_session(
        &self,
        session_id: &str,
        viewer_id: &str,
        viewer_display_name: &str,
    ) -> Result<String, String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session {session_id} not found"))?;

        if session.viewers.len() >= self.config.max_viewers {
            return Err("Session is full".into());
        }

        session.viewers.insert(viewer_id.to_string());
        let host_id = session.host_user_id.clone();
        drop(sessions);

        let _ = self
            .event_tx
            .send(ScreenShareEvent::ViewerJoined {
                session_id: session_id.to_string(),
                viewer_user_id: viewer_id.to_string(),
                viewer_display_name: viewer_display_name.to_string(),
            })
            .await;

        info!(session_id, viewer_id, "Viewer joined screen share");
        // Return the host user_id so caller can initiate WebRTC connection
        Ok(host_id)
    }

    /// Leave a screen share session (as a viewer).
    pub async fn leave_session(&self, session_id: &str, viewer_id: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.viewers.remove(viewer_id);
        }
        drop(sessions);

        let _ = self
            .event_tx
            .send(ScreenShareEvent::ViewerLeft {
                session_id: session_id.to_string(),
                viewer_user_id: viewer_id.to_string(),
            })
            .await;
    }

    /// Change quality for a session (host only).
    pub async fn set_quality(
        &self,
        session_id: &str,
        user_id: &str,
        quality: ShareQuality,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session {session_id} not found"))?;

        if session.host_user_id != user_id {
            return Err("Only the host can change quality".into());
        }

        session.quality = quality;
        drop(sessions);

        let _ = self
            .event_tx
            .send(ScreenShareEvent::QualityChanged {
                session_id: session_id.to_string(),
                quality,
            })
            .await;

        Ok(())
    }

    /// Handle an incoming WebRTC signaling message.
    pub async fn handle_signal(&self, from_user: &str, signal: ScreenShareSignal) {
        debug!(from = from_user, ?signal, "Received screen share signal");
        let _ = self
            .event_tx
            .send(ScreenShareEvent::Signal {
                from_user: from_user.to_string(),
                signal,
            })
            .await;
    }

    /// Get a session by ID.
    pub async fn get_session(&self, session_id: &str) -> Option<ScreenShareSession> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// List all active sessions.
    pub async fn list_sessions(&self) -> Vec<ScreenShareSession> {
        self.sessions.read().await.values().cloned().collect()
    }

    /// Clean up when a user goes offline.
    pub async fn handle_user_offline(&self, user_id: &str) {
        // Stop their session if hosting
        self.stop_sharing(user_id).await;

        // Remove them from any sessions they're viewing
        let sessions = self.sessions.read().await;
        let session_ids: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| s.viewers.contains(user_id))
            .map(|(id, _)| id.clone())
            .collect();
        drop(sessions);

        for sid in session_ids {
            self.leave_session(&sid, user_id).await;
        }
    }
}
