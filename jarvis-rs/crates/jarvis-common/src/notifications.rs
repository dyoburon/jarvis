use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Severity level for in-app notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

/// An in-app notification for overlay rendering.
#[derive(Debug, Clone)]
pub struct Notification {
    pub level: NotificationLevel,
    pub title: String,
    pub body: String,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl Notification {
    /// Creates an info notification with a 5-second TTL.
    pub fn info(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            level: NotificationLevel::Info,
            title: title.into(),
            body: body.into(),
            created_at: Instant::now(),
            ttl: Duration::from_secs(5),
        }
    }

    /// Creates a warning notification with an 8-second TTL.
    pub fn warning(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            level: NotificationLevel::Warning,
            title: title.into(),
            body: body.into(),
            created_at: Instant::now(),
            ttl: Duration::from_secs(8),
        }
    }

    /// Creates an error notification with a 10-second TTL.
    pub fn error(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            level: NotificationLevel::Error,
            title: title.into(),
            body: body.into(),
            created_at: Instant::now(),
            ttl: Duration::from_secs(10),
        }
    }

    /// Returns `true` if this notification has exceeded its TTL.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.ttl
    }
}

/// A bounded queue of in-app notifications that auto-evicts expired entries.
#[derive(Debug)]
pub struct NotificationQueue {
    items: VecDeque<Notification>,
    capacity: usize,
}

impl NotificationQueue {
    /// Creates a new queue with the given maximum capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Pushes a notification, evicting expired entries first.
    /// If still at capacity after eviction, the oldest entry is removed.
    pub fn push(&mut self, notification: Notification) {
        self.evict_expired();
        if self.items.len() >= self.capacity {
            self.items.pop_front();
        }
        self.items.push_back(notification);
    }

    /// Returns all currently visible (non-expired) notifications.
    pub fn visible(&mut self) -> Vec<&Notification> {
        self.evict_expired();
        self.items.iter().collect()
    }

    /// Returns the number of notifications currently in the queue (including expired).
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn evict_expired(&mut self) {
        self.items.retain(|n| !n.is_expired());
    }
}

impl Default for NotificationQueue {
    fn default() -> Self {
        Self::new(16)
    }
}
