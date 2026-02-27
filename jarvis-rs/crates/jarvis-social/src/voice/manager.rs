//! Voice room manager — create, join, leave, and audio state management.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

use crate::protocol::VoiceSignal;

use super::types::{VoiceConfig, VoiceEvent, VoiceParticipant, VoiceRoom, VoiceState};

// ---------------------------------------------------------------------------
// Voice Manager
// ---------------------------------------------------------------------------

/// Manages voice rooms and participant state.
pub struct VoiceManager {
    config: VoiceConfig,
    /// Combined rooms + user-room mapping under a single lock.
    state: Arc<RwLock<VoiceState>>,
    /// Event sender.
    event_tx: mpsc::Sender<VoiceEvent>,
}

impl VoiceManager {
    pub fn new(config: VoiceConfig) -> (Self, mpsc::Receiver<VoiceEvent>) {
        let (event_tx, event_rx) = mpsc::channel(256);
        let mgr = Self {
            config,
            state: Arc::new(RwLock::new(VoiceState {
                rooms: HashMap::new(),
                user_rooms: HashMap::new(),
            })),
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

        let mut state = self.state.write().await;
        if state.rooms.contains_key(room_id) {
            return Err(format!("Room {room_id} already exists"));
        }

        let mut room = VoiceRoom::new(room_id.to_string(), name.to_string(), user_id.to_string());
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
        state.rooms.insert(room_id.to_string(), room);
        state
            .user_rooms
            .insert(user_id.to_string(), room_id.to_string());
        drop(state);

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

        let mut state = self.state.write().await;

        // Leave current room first (inline to avoid deadlock from re-entrant lock)
        if let Some(old_room_id) = state.user_rooms.remove(user_id) {
            Self::remove_from_room_inner(&mut state, &old_room_id, user_id, &self.event_tx).await;
        }

        let room = state
            .rooms
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

        state
            .user_rooms
            .insert(user_id.to_string(), room_id.to_string());
        drop(state);

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
        let mut state = self.state.write().await;
        if let Some(room_id) = state.user_rooms.remove(user_id) {
            Self::remove_from_room_inner(&mut state, &room_id, user_id, &self.event_tx).await;
        }
    }

    /// Remove a user from a specific room, closing the room if empty.
    /// Operates on an already-acquired write guard to avoid deadlocks.
    async fn remove_from_room_inner(
        state: &mut VoiceState,
        room_id: &str,
        user_id: &str,
        event_tx: &mpsc::Sender<VoiceEvent>,
    ) {
        let should_close = if let Some(room) = state.rooms.get_mut(room_id) {
            let display_name = room
                .participants
                .get(user_id)
                .map(|p| p.display_name.clone())
                .unwrap_or_default();
            room.participants.remove(user_id);

            let _ = event_tx
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
            state.rooms.remove(room_id);
            let _ = event_tx
                .send(VoiceEvent::RoomClosed {
                    room_id: room_id.to_string(),
                })
                .await;
            info!(room_id, "Voice room closed (empty)");
        }
    }

    /// Toggle mute state for a user.
    pub async fn set_muted(&self, user_id: &str, muted: bool) {
        let mut state = self.state.write().await;
        if let Some(room_id) = state.user_rooms.get(user_id).cloned() {
            if let Some(room) = state.rooms.get_mut(&room_id) {
                if let Some(p) = room.participants.get_mut(user_id) {
                    p.muted = muted;
                }
            }
            drop(state);
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
        let mut state = self.state.write().await;
        if let Some(room_id) = state.user_rooms.get(user_id).cloned() {
            if let Some(room) = state.rooms.get_mut(&room_id) {
                if let Some(p) = room.participants.get_mut(user_id) {
                    p.deafened = deafened;
                    // Deafening also mutes
                    if deafened {
                        p.muted = true;
                    }
                }
            }
            drop(state);
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
        let mut state = self.state.write().await;
        if let Some(room_id) = state.user_rooms.get(user_id).cloned() {
            if let Some(room) = state.rooms.get_mut(&room_id) {
                if let Some(p) = room.participants.get_mut(user_id) {
                    p.speaking = speaking;
                }
            }
            drop(state);
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
        self.state.read().await.rooms.get(room_id).cloned()
    }

    /// List all active rooms.
    pub async fn list_rooms(&self) -> Vec<VoiceRoom> {
        self.state.read().await.rooms.values().cloned().collect()
    }

    /// Get which room a user is in.
    pub async fn user_room(&self, user_id: &str) -> Option<String> {
        self.state.read().await.user_rooms.get(user_id).cloned()
    }

    /// Clean up when a user goes offline.
    pub async fn handle_user_offline(&self, user_id: &str) {
        self.leave_current_room(user_id).await;
    }
}
