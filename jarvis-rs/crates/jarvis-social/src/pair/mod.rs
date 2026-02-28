//! Pair programming session management.
//!
//! Enables collaborative terminal sessions where one user hosts a
//! shared terminal and others can view or take the driver seat.
//! Terminal output is streamed from host → guests through the
//! presence WebSocket, and guest keystrokes (when driving) are
//! relayed back to the host's PTY.

mod manager;
mod types;

pub use manager::PairManager;
pub use types::{
    PairConfig, PairEvent, PairParticipant, PairRole, PairSession, PairState,
};
