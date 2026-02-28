//! Broadcast channel bridging the sync main thread to async WS clients.

use tokio::sync::broadcast;

use super::protocol::ServerMessage;

/// Broadcast capacity. If a mobile client falls behind by more than this
/// many messages, it will skip (lagged).
const BROADCAST_CAPACITY: usize = 256;

/// Owned by `JarvisApp` on the main thread. Provides a non-blocking `send()`
/// for `poll_pty_output()` and `subscribe()` for new WS client tasks.
pub struct MobileBroadcaster {
    tx: broadcast::Sender<ServerMessage>,
}

impl MobileBroadcaster {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self { tx }
    }

    /// Push a message to all connected mobile clients.
    /// Non-blocking. No-op if no clients are connected.
    pub fn send(&self, msg: ServerMessage) {
        let _ = self.tx.send(msg);
    }

    /// Create a new receiver for an incoming WS client.
    pub fn subscribe(&self) -> broadcast::Receiver<ServerMessage> {
        self.tx.subscribe()
    }
}
