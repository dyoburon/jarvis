//! Session store: maps session IDs to paired desktop/mobile channels.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{mpsc, RwLock};

/// Role of a connected client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Desktop,
    Mobile,
}

/// A session pairs exactly one desktop and one mobile connection.
pub struct Session {
    pub desktop_tx: Option<mpsc::Sender<String>>,
    pub mobile_tx: Option<mpsc::Sender<String>>,
    pub created_at: Instant,
}

/// Thread-safe session store.
#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a session (desktop calls this). Returns true if created, false if already exists.
    pub async fn create_session(&self, session_id: &str) -> bool {
        let mut map = self.sessions.write().await;
        if map.contains_key(session_id) {
            return false;
        }
        map.insert(
            session_id.to_string(),
            Session {
                desktop_tx: None,
                mobile_tx: None,
                created_at: Instant::now(),
            },
        );
        true
    }

    /// Register a client's sender. Returns the peer's sender if already connected.
    pub async fn register(
        &self,
        session_id: &str,
        role: Role,
        tx: mpsc::Sender<String>,
    ) -> Result<Option<mpsc::Sender<String>>, &'static str> {
        let mut map = self.sessions.write().await;
        let session = map.get_mut(session_id).ok_or("session not found")?;

        let peer_tx = match role {
            Role::Desktop => {
                if session.desktop_tx.is_some() {
                    return Err("desktop already connected");
                }
                session.desktop_tx = Some(tx);
                session.mobile_tx.clone()
            }
            Role::Mobile => {
                if session.mobile_tx.is_some() {
                    return Err("mobile already connected");
                }
                session.mobile_tx = Some(tx);
                session.desktop_tx.clone()
            }
        };

        Ok(peer_tx)
    }

    /// Get the peer's sender for forwarding.
    pub async fn get_peer_tx(
        &self,
        session_id: &str,
        role: Role,
    ) -> Option<mpsc::Sender<String>> {
        let map = self.sessions.read().await;
        let session = map.get(session_id)?;
        match role {
            Role::Desktop => session.mobile_tx.clone(),
            Role::Mobile => session.desktop_tx.clone(),
        }
    }

    /// Unregister a client. Returns true if session was removed (both gone).
    pub async fn unregister(&self, session_id: &str, role: Role) -> bool {
        let mut map = self.sessions.write().await;
        if let Some(session) = map.get_mut(session_id) {
            match role {
                Role::Desktop => session.desktop_tx = None,
                Role::Mobile => session.mobile_tx = None,
            }
            // Remove session if both sides are gone
            if session.desktop_tx.is_none() && session.mobile_tx.is_none() {
                map.remove(session_id);
                return true;
            }
        }
        false
    }

    /// Reap sessions older than `max_age` with no mobile peer.
    pub async fn reap_stale(&self, max_age: std::time::Duration) {
        let mut map = self.sessions.write().await;
        let now = Instant::now();
        map.retain(|id, session| {
            let stale = session.mobile_tx.is_none() && now.duration_since(session.created_at) > max_age;
            if stale {
                tracing::info!(session_id = %id, "Reaping stale session");
            }
            !stale
        });
    }

    /// Check if a session exists.
    pub async fn exists(&self, session_id: &str) -> bool {
        self.sessions.read().await.contains_key(session_id)
    }

    /// Number of active sessions.
    pub async fn count(&self) -> usize {
        self.sessions.read().await.len()
    }
}
