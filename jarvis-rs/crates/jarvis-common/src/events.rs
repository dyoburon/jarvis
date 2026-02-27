use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::types::PaneId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    ConfigReloaded,
    PaneOpened(PaneId),
    PaneClosed(PaneId),
    PaneFocused(PaneId),
    PresenceUpdate { user_id: String, status: String },
    ChatMessage { from: String, text: String },
    Notification(String),
    Shutdown,
    #[serde(other)]
    Unknown,
}

pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: Event) -> usize {
        self.sender.send(event).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_and_receive() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.publish(Event::ConfigReloaded);

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, Event::ConfigReloaded));
    }

    #[tokio::test]
    async fn multiple_subscribers() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(Event::Shutdown);

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        assert!(matches!(e1, Event::Shutdown));
        assert!(matches!(e2, Event::Shutdown));
    }

    #[tokio::test]
    async fn pane_events() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        let pane = PaneId(1);

        bus.publish(Event::PaneOpened(pane));
        bus.publish(Event::PaneFocused(pane));
        bus.publish(Event::PaneClosed(pane));

        let e1 = rx.recv().await.unwrap();
        assert!(matches!(e1, Event::PaneOpened(id) if id == PaneId(1)));

        let e2 = rx.recv().await.unwrap();
        assert!(matches!(e2, Event::PaneFocused(id) if id == PaneId(1)));

        let e3 = rx.recv().await.unwrap();
        assert!(matches!(e3, Event::PaneClosed(id) if id == PaneId(1)));
    }

    #[tokio::test]
    async fn chat_and_presence_events() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.publish(Event::PresenceUpdate {
            user_id: "alice".into(),
            status: "online".into(),
        });
        bus.publish(Event::ChatMessage {
            from: "bob".into(),
            text: "hello".into(),
        });
        bus.publish(Event::Notification("test".into()));

        let e1 = rx.recv().await.unwrap();
        assert!(matches!(e1, Event::PresenceUpdate { ref user_id, .. } if user_id == "alice"));

        let e2 = rx.recv().await.unwrap();
        assert!(
            matches!(e2, Event::ChatMessage { ref from, ref text, .. } if from == "bob" && text == "hello")
        );

        let e3 = rx.recv().await.unwrap();
        assert!(matches!(e3, Event::Notification(ref msg) if msg == "test"));
    }

    #[test]
    fn publish_returns_zero_with_no_subscribers() {
        let bus = EventBus::new(16);
        let count = bus.publish(Event::Shutdown);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn publish_returns_subscriber_count() {
        let bus = EventBus::new(16);
        let _rx1 = bus.subscribe();
        let _rx2 = bus.subscribe();
        let _rx3 = bus.subscribe();

        let count = bus.publish(Event::ConfigReloaded);
        assert_eq!(count, 3);
    }

    #[test]
    fn unknown_event_deserializes() {
        let json = r#"{"type":"SomeNewEventWeNeverHeardOf","data":null}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert!(matches!(event, Event::Unknown));
    }
}
