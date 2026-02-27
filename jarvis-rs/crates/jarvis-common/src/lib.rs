pub mod actions;
pub mod errors;
pub mod events;
pub mod id;
pub mod notifications;
pub mod types;

pub use actions::{Action, ResizeDirection};
pub use errors::{ConfigError, JarvisError, PlatformError};
pub use events::{Event, EventBus};
pub use id::{new_correlation_id, new_id, SessionId};
pub use notifications::{Notification, NotificationLevel, NotificationQueue};
pub use types::{AppState, Color, PaneId, PaneKind, Rect};

pub type Result<T> = std::result::Result<T, JarvisError>;
