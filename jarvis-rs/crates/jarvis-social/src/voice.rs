//! Voice chat room management and signaling.
//!
//! Manages voice rooms, participant state (muted, deafened, speaking),
//! and relays WebRTC signaling messages through the presence WebSocket.
//! Actual media transport is peer-to-peer via WebRTC — this module only
//! handles the coordination layer.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

use crate::protocol::VoiceSignal;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Per-participant state inside a voice room.
#[derive(Debug, Clone)]
pub struct VoiceParticipant {
    pub user_id: String,
    pub display_name: String,
    pub muted: bool,
    pub deafened: bool,
    pub speaking: bool,
}

/// A voice room that users can join for real-time audio.
#[derive(Debug, Clone)]
pub struct VoiceRoom {
    pub room_id: String,
    pub name: String,
    pub created_by: String,
    pub participants: HashMap<String, VoiceParticipant>,
    pub max_participants: usize,
}

impl VoiceRoom {
    pub fn new(room_id: String, name: String, created_by: String) -> Self {
        Self {
            room_id,
            name,
            created_by,
            participants: HashMap::new(),
            max_participants: 8,
        }
    }

    pub fn is_full(&self) -> bool {
        self.participants.len() >= self.max_participants
    }

    pub fn participant_ids(&self) -> HashSet<String> {
        self.participants.keys().cloned().collect()
    }
}

/// Events emitted by the voice system for the UI.
#[derive(Debug, Clone)]
pub enum VoiceEvent {
    RoomCreated { room_id: String, name: String },
    RoomClosed { room_id: String },
    UserJoined { room_id: String, user_id: String, display_name: String },
    UserLeft { room_id: String, user_id: String, display_name: String },
    MuteChanged { room_id: String, user_id: String, muted: bool },
    DeafenChanged { room_id: String, user_id: String, deafened: bool },
    SpeakingChanged { room_id: String, user_id: String, speaking: bool },
    /// WebRTC signaling message received — forward to the WebRTC layer.
    Signal { from_user: String, signal: VoiceSignal },
    Error(String),
}

/// Configuration for voice chat.
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// Maximum participants per room.
    pub max_participants: usize,
    /// Whether voice is enabled.
    pub enabled: bool,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            max_participants: 8,
            enabled: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Voice Manager
// ---------------------------------------------------------------------------

/// Manages voice rooms and participant state.
pub struct VoiceManager {
    config: VoiceConfig,
    /// All active rooms keyed by room_id.
    rooms: Arc<RwLock<HashMap<String, VoiceRoom>>>,
    /// Which room each user is in (user_id → room_id).
    user_rooms: Arc<RwLock<HashMap<String, String>>>,
    /// Event sender.
    event_tx: mpsc::Sender<VoiceEvent>,
}

impl VoiceManager {
    pub fn new(config: VoiceConfig) -> (Self, mpsc::Receiver<VoiceEvent>) {
        let (event_tx, event_rx) = mpsc::channel(256);
        let mgr = Self {
            config,
            rooms: Arc::new(RwLock::new(HashMap::new())),
            user_rooms: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        };
        (mgr, event_rx)
    }

    /// Create a new voice room. The creator automatically joins.
    pub async fn create_room(
        &self,
        room_id: &str,
        name: &str,
        user_id: &str,
        display_name: &str,
    ) -> Result<(), String> {
        if !self.config.enabled {
            return Err("Voice chat is disabled".into());
        }

        let mut rooms = self.rooms.write().await;
        if rooms.contains_key(room_id) {
            return Err(format!("Room {room_id} already exists"));
        }

        let mut room = VoiceRoom::new(
            room_id.to_string(),
            name.to_string(),
            user_id.to_string(),
        );
        room.max_participants = self.config.max_participants;
        room.participants.insert(
            user_id.to_string(),
            VoiceParticipant {
                user_id: user_id.to_string(),
                display_name: display_name.to_string(),
                muted: false,
                deafened: false,
                speaking: false,
            },
        );
        rooms.insert(room_id.to_string(), room);
        drop(rooms);

        self.user_rooms
            .write()
            .await
            .insert(user_id.to_string(), room_id.to_string());

        let _ = self
            .event_tx
            .send(VoiceEvent::RoomCreated {
                room_id: room_id.to_string(),
                name: name.to_string(),
            })
            .await;

        info!(room_id, user_id, "Voice room created");
        Ok(())
    }

    /// Join an existing voice room.
    pub async fn join_room(
        &self,
        room_id: &str,
        user_id: &str,
        display_name: &str,
    ) -> Result<Vec<String>, String> {
        if !self.config.enabled {
            return Err("Voice chat is disabled".into());
        }

        // Leave current room first
        self.leave_current_room(user_id).await;

        let mut rooms = self.rooms.write().await;
        let room = rooms
            .get_mut(room_id)
            .ok_or_else(|| format!("Room {room_id} not found"))?;

        if room.is_full() {
            return Err("Room is full".into());
        }

        let existing_ids: Vec<String> = room.participants.keys().cloned().collect();

        room.participants.insert(
            user_id.to_string(),
            VoiceParticipant {
                user_id: user_id.to_string(),
                display_name: display_name.to_string(),
                muted: false,
                deafened: false,
                speaking: false,
            },
        );
        drop(rooms);

        self.user_rooms
            .write()
            .await
            .insert(user_id.to_string(), room_id.to_string());

        let _ = self
            .event_tx
            .send(VoiceEvent::UserJoined {
                room_id: room_id.to_string(),
                user_id: user_id.to_string(),
                display_name: display_name.to_string(),
            })
            .await;

        info!(room_id, user_id, "User joined voice room");
        // Return list of existing participants so caller can initiate WebRTC offers
        Ok(existing_ids)
    }

    /// Leave the current voice room.
    pub async fn leave_current_room(&self, user_id: &str) {
        let room_id = self.user_rooms.write().await.remove(user_id);
        if let Some(room_id) = room_id {
            self.remove_from_room(&room_id, user_id).await;
        }
    }

    /// Remove a user from a specific room, closing the room if empty.
    async fn remove_from_room(&self, room_id: &str, user_id: &str) {
        let mut rooms = self.rooms.write().await;
        let should_close = if let Some(room) = rooms.get_mut(room_id) {
            let display_name = room
                .participants
                .get(user_id)
                .map(|p| p.display_name.clone())
                .unwrap_or_default();
            room.participants.remove(user_id);

            let _ = self
                .event_tx
                .send(VoiceEvent::UserLeft {
                    room_id: room_id.to_string(),
                    user_id: user_id.to_string(),
                    display_name,
                })
                .await;

            room.participants.is_empty()
        } else {
            false
        };

        if should_close {
            rooms.remove(room_id);
            drop(rooms);
            let _ = self
                .event_tx
                .send(VoiceEvent::RoomClosed {
                    room_id: room_id.to_string(),
                })
                .await;
            info!(room_id, "Voice room closed (empty)");
        }
    }

    /// Toggle mute state for a user.
    pub async fn set_muted(&self, user_id: &str, muted: bool) {
        let user_rooms = self.user_rooms.read().await;
        if let Some(room_id) = user_rooms.get(user_id) {
            let room_id = room_id.clone();
            drop(user_rooms);
            let mut rooms = self.rooms.write().await;
            if let Some(room) = rooms.get_mut(&room_id)
                && let Some(p) = room.participants.get_mut(user_id)
            {
                p.muted = muted;
            }
            let _ = self
                .event_tx
                .send(VoiceEvent::MuteChanged {
                    room_id,
                    user_id: user_id.to_string(),
                    muted,
                })
                .await;
        }
    }

    /// Toggle deafen state for a user.
    pub async fn set_deafened(&self, user_id: &str, deafened: bool) {
        let user_rooms = self.user_rooms.read().await;
        if let Some(room_id) = user_rooms.get(user_id) {
            let room_id = room_id.clone();
            drop(user_rooms);
            let mut rooms = self.rooms.write().await;
            if let Some(room) = rooms.get_mut(&room_id)
                && let Some(p) = room.participants.get_mut(user_id)
            {
                p.deafened = deafened;
                // Deafening also mutes
                if deafened {
                    p.muted = true;
                }
            }
            let _ = self
                .event_tx
                .send(VoiceEvent::DeafenChanged {
                    room_id,
                    user_id: user_id.to_string(),
                    deafened,
                })
                .await;
        }
    }

    /// Update speaking indicator for a user (driven by voice activity detection).
    pub async fn set_speaking(&self, user_id: &str, speaking: bool) {
        let user_rooms = self.user_rooms.read().await;
        if let Some(room_id) = user_rooms.get(user_id) {
            let room_id = room_id.clone();
            drop(user_rooms);
            let mut rooms = self.rooms.write().await;
            if let Some(room) = rooms.get_mut(&room_id)
                && let Some(p) = room.participants.get_mut(user_id)
            {
                p.speaking = speaking;
            }
            let _ = self
                .event_tx
                .send(VoiceEvent::SpeakingChanged {
                    room_id,
                    user_id: user_id.to_string(),
                    speaking,
                })
                .await;
        }
    }

    /// Handle an incoming WebRTC signaling message.
    pub async fn handle_signal(&self, from_user: &str, signal: VoiceSignal) {
        debug!(from = from_user, ?signal, "Received voice signal");
        let _ = self
            .event_tx
            .send(VoiceEvent::Signal {
                from_user: from_user.to_string(),
                signal,
            })
            .await;
    }

    /// Get a snapshot of a room.
    pub async fn get_room(&self, room_id: &str) -> Option<VoiceRoom> {
        self.rooms.read().await.get(room_id).cloned()
    }

    /// List all active rooms.
    pub async fn list_rooms(&self) -> Vec<VoiceRoom> {
        self.rooms.read().await.values().cloned().collect()
    }

    /// Get which room a user is in.
    pub async fn user_room(&self, user_id: &str) -> Option<String> {
        self.user_rooms.read().await.get(user_id).cloned()
    }

    /// Clean up when a user goes offline.
    pub async fn handle_user_offline(&self, user_id: &str) {
        self.leave_current_room(user_id).await;
    }
}
