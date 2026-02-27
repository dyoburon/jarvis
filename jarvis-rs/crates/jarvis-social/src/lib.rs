pub mod channels;
pub mod chat;
pub mod identity;
pub mod presence;
pub mod protocol;
pub mod realtime;

#[cfg(feature = "experimental-collab")]
pub mod pair;
#[cfg(feature = "experimental-collab")]
pub mod screen_share;
#[cfg(feature = "experimental-collab")]
pub mod voice;

pub use channels::{Channel, ChannelManager};
pub use chat::{ChatHistory, ChatHistoryConfig, ChatMessage};
pub use identity::Identity;
pub use presence::{PresenceClient, PresenceConfig, PresenceEvent};
pub use protocol::{
    ActivityUpdatePayload, ChatMessagePayload, GameInvitePayload, OnlineUser, PokePayload,
    PresencePayload, UserStatus,
};
pub use realtime::{RealtimeClient, RealtimeConfig};

#[cfg(feature = "experimental-collab")]
pub use pair::{PairConfig, PairEvent, PairManager, PairRole, PairSession};
#[cfg(feature = "experimental-collab")]
pub use protocol::{ScreenShareSignal, VoiceSignal};
#[cfg(feature = "experimental-collab")]
pub use screen_share::{ScreenShareConfig, ScreenShareEvent, ScreenShareManager, ShareQuality};
#[cfg(feature = "experimental-collab")]
pub use voice::{VoiceConfig, VoiceEvent, VoiceManager, VoiceRoom};
