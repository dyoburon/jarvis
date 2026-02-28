//! Pair programming session manager.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::info;

use super::types::{
    PairConfig, PairEvent, PairParticipant, PairRole, PairSession, PairState,
};

// ---------------------------------------------------------------------------
// Pair Manager
// ---------------------------------------------------------------------------

/// Manages pair programming sessions.
pub struct PairManager {
    config: PairConfig,
    /// Combined sessions + user-session mapping under a single lock.
    state: Arc<RwLock<PairState>>,
    event_tx: mpsc::Sender<PairEvent>,
}

impl PairManager {
    pub fn new(config: PairConfig) -> (Self, mpsc::Receiver<PairEvent>) {
        let (event_tx, event_rx) = mpsc::channel(512);
        let mgr = Self {
            config,
            state: Arc::new(RwLock::new(PairState {
                sessions: HashMap::new(),
                user_sessions: HashMap::new(),
            })),
            event_tx,
        };
        (mgr, event_rx)
    }

    /// Create a new pair session. The creator is the host and initial driver.
    pub async fn create_session(
        &self,
        session_id: &str,
        user_id: &str,
        display_name: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), String> {
        if !self.config.enabled {
            return Err("Pair programming is disabled".into());
        }

        // Leave any existing session
        self.leave_session(user_id).await;

        let mut participants = HashMap::new();
        participants.insert(
            user_id.to_string(),
            PairParticipant {
                user_id: user_id.to_string(),
                display_name: display_name.to_string(),
                role: PairRole::Driver,
                cursor_row: 0,
                cursor_col: 0,
            },
        );

        let session = PairSession {
            session_id: session_id.to_string(),
            host_user_id: user_id.to_string(),
            host_display_name: display_name.to_string(),
            cols,
            rows,
            participants,
            driver_user_id: user_id.to_string(),
            allow_takeover: self.config.allow_takeover,
        };

        {
            let mut state = self.state.write().await;
            state.sessions.insert(session_id.to_string(), session);
            state
                .user_sessions
                .insert(user_id.to_string(), session_id.to_string());
        }

        let _ = self
            .event_tx
            .send(PairEvent::SessionCreated {
                session_id: session_id.to_string(),
                host_user_id: user_id.to_string(),
                host_display_name: display_name.to_string(),
                cols,
                rows,
            })
            .await;

        info!(session_id, user_id, "Pair session created");
        Ok(())
    }

    /// Join an existing pair session as a navigator.
    pub async fn join_session(
        &self,
        session_id: &str,
        user_id: &str,
        display_name: &str,
    ) -> Result<(), String> {
        if !self.config.enabled {
            return Err("Pair programming is disabled".into());
        }

        // Leave any existing session
        self.leave_session(user_id).await;

        {
            let mut state = self.state.write().await;
            let session = state
                .sessions
                .get_mut(session_id)
                .ok_or_else(|| format!("Session {session_id} not found"))?;

            if session.participants.len() >= self.config.max_participants {
                return Err("Session is full".into());
            }

            session.participants.insert(
                user_id.to_string(),
                PairParticipant {
                    user_id: user_id.to_string(),
                    display_name: display_name.to_string(),
                    role: PairRole::Navigator,
                    cursor_row: 0,
                    cursor_col: 0,
                },
            );

            state
                .user_sessions
                .insert(user_id.to_string(), session_id.to_string());
        }

        let _ = self
            .event_tx
            .send(PairEvent::UserJoined {
                session_id: session_id.to_string(),
                user_id: user_id.to_string(),
                display_name: display_name.to_string(),
                role: PairRole::Navigator,
            })
            .await;

        info!(session_id, user_id, "User joined pair session");
        Ok(())
    }

    /// Leave the current pair session. If the host leaves, the session ends.
    pub async fn leave_session(&self, user_id: &str) {
        let mut state = self.state.write().await;

        let session_id = state.user_sessions.remove(user_id);
        let Some(session_id) = session_id else {
            return;
        };

        let should_end = if let Some(session) = state.sessions.get_mut(&session_id) {
            session.participants.remove(user_id);

            // If the host left, end the session
            if session.host_user_id == user_id || session.participants.is_empty() {
                true
            } else {
                // If the driver left, pass control to host
                if session.driver_user_id == user_id {
                    let new_driver = session.host_user_id.clone();
                    let old_driver = user_id.to_string();
                    session.driver_user_id = new_driver.clone();
                    if let Some(p) = session.participants.get_mut(&new_driver) {
                        p.role = PairRole::Driver;
                    }
                    // Drop state before awaiting the channel send
                    drop(state);
                    let _ = self
                        .event_tx
                        .send(PairEvent::DriverChanged {
                            session_id: session_id.clone(),
                            new_driver,
                            old_driver,
                        })
                        .await;
                    // Re-acquire for the UserLeft event below — but we already
                    // mutated everything we needed, so just send UserLeft and return.
                    let _ = self
                        .event_tx
                        .send(PairEvent::UserLeft {
                            session_id: session_id.clone(),
                            user_id: user_id.to_string(),
                        })
                        .await;
                    return;
                }

                // Drop state before awaiting the channel send
                drop(state);
                let _ = self
                    .event_tx
                    .send(PairEvent::UserLeft {
                        session_id: session_id.clone(),
                        user_id: user_id.to_string(),
                    })
                    .await;

                return;
            }
        } else {
            false
        };

        if should_end {
            let session = state.sessions.remove(&session_id);
            // Clean up all participants' user_sessions entries
            if let Some(session) = session {
                for pid in session.participants.keys() {
                    state.user_sessions.remove(pid);
                }
            }
            drop(state);

            let _ = self
                .event_tx
                .send(PairEvent::SessionEnded {
                    session_id: session_id.clone(),
                })
                .await;
            info!(session_id, "Pair session ended");
        }
    }

    /// Transfer driver role to another participant.
    pub async fn set_driver(
        &self,
        session_id: &str,
        requester_id: &str,
        new_driver_id: &str,
    ) -> Result<(), String> {
        let old_driver = {
            let mut state = self.state.write().await;
            let session = state
                .sessions
                .get_mut(session_id)
                .ok_or_else(|| format!("Session {session_id} not found"))?;

            // Only the host or current driver can reassign
            if requester_id != session.host_user_id && requester_id != session.driver_user_id {
                if !session.allow_takeover {
                    return Err("Takeover not allowed in this session".into());
                }
                // Navigator requesting takeover — only allowed if allow_takeover
                if requester_id != new_driver_id {
                    return Err("Navigators can only request control for themselves".into());
                }
            }

            if !session.participants.contains_key(new_driver_id) {
                return Err("Target user not in session".into());
            }

            let old_driver = session.driver_user_id.clone();

            // Demote old driver
            if let Some(p) = session.participants.get_mut(&old_driver) {
                p.role = PairRole::Navigator;
            }

            // Promote new driver
            session.driver_user_id = new_driver_id.to_string();
            if let Some(p) = session.participants.get_mut(new_driver_id) {
                p.role = PairRole::Driver;
            }

            old_driver
        };

        let _ = self
            .event_tx
            .send(PairEvent::DriverChanged {
                session_id: session_id.to_string(),
                new_driver: new_driver_id.to_string(),
                old_driver,
            })
            .await;

        info!(session_id, new_driver = new_driver_id, "Driver changed");
        Ok(())
    }

    /// Forward terminal output from the host to all guests.
    pub async fn broadcast_output(&self, session_id: &str, data: Vec<u8>) {
        let _ = self
            .event_tx
            .send(PairEvent::TerminalOutput {
                session_id: session_id.to_string(),
                data,
            })
            .await;
    }

    /// Forward keystroke input from the current driver.
    /// Only accepted if `from_user` is the current driver.
    pub async fn relay_input(
        &self,
        session_id: &str,
        from_user: &str,
        data: Vec<u8>,
    ) -> Result<(), String> {
        {
            let state = self.state.read().await;
            let session = state
                .sessions
                .get(session_id)
                .ok_or_else(|| format!("Session {session_id} not found"))?;

            if session.driver_user_id != from_user {
                return Err("Only the driver can send input".into());
            }
        }

        let _ = self
            .event_tx
            .send(PairEvent::TerminalInput {
                session_id: session_id.to_string(),
                from_user: from_user.to_string(),
                data,
            })
            .await;

        Ok(())
    }

    /// Update a participant's cursor position.
    pub async fn update_cursor(&self, session_id: &str, user_id: &str, row: u16, col: u16) {
        {
            let mut state = self.state.write().await;
            if let Some(session) = state.sessions.get_mut(session_id) {
                if let Some(p) = session.participants.get_mut(user_id) {
                    p.cursor_row = row;
                    p.cursor_col = col;
                }
            }
        }

        let _ = self
            .event_tx
            .send(PairEvent::CursorMoved {
                session_id: session_id.to_string(),
                user_id: user_id.to_string(),
                row,
                col,
            })
            .await;
    }

    /// Notify that the host resized the terminal.
    pub async fn resize(&self, session_id: &str, cols: u16, rows: u16) {
        {
            let mut state = self.state.write().await;
            if let Some(session) = state.sessions.get_mut(session_id) {
                session.cols = cols;
                session.rows = rows;
            }
        }

        let _ = self
            .event_tx
            .send(PairEvent::Resized {
                session_id: session_id.to_string(),
                cols,
                rows,
            })
            .await;
    }

    /// Get a session snapshot.
    pub async fn get_session(&self, session_id: &str) -> Option<PairSession> {
        self.state.read().await.sessions.get(session_id).cloned()
    }

    /// List all active sessions.
    pub async fn list_sessions(&self) -> Vec<PairSession> {
        self.state.read().await.sessions.values().cloned().collect()
    }

    /// Get which session a user is in.
    pub async fn user_session(&self, user_id: &str) -> Option<String> {
        self.state.read().await.user_sessions.get(user_id).cloned()
    }

    /// Clean up when a user goes offline.
    pub async fn handle_user_offline(&self, user_id: &str) {
        self.leave_session(user_id).await;
    }
}
