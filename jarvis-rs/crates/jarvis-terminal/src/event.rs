//! Event bridge between alacritty_terminal and jarvis.
//!
//! @module terminal/event

use std::sync::mpsc;

use alacritty_terminal::event::{Event as AlacrittyEvent, EventListener};

// =============================================================================
// TYPES
// =============================================================================

/// Terminal events forwarded from alacritty_terminal to jarvis.
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// Terminal bell triggered.
    Bell,
    /// Window title changed.
    Title(String),
    /// Title reset to default.
    ResetTitle,
    /// Terminal exited.
    Exit,
    /// Child process exited with a status code.
    ChildExit(i32),
    /// Request to store text in clipboard.
    ClipboardStore(String),
    /// Terminal content changed — renderer should redraw.
    Wakeup,
    /// Cursor blinking state changed.
    CursorBlinkingChange,
}

// =============================================================================
// EVENT PROXY
// =============================================================================

/// Bridges `alacritty_terminal::Event` to jarvis's event system via an mpsc
/// channel. Created alongside each `Term` instance.
pub struct JarvisEventProxy {
    sender: mpsc::Sender<TerminalEvent>,
}

impl JarvisEventProxy {
    /// Create a new event proxy and its corresponding receiver.
    pub fn new() -> (Self, mpsc::Receiver<TerminalEvent>) {
        let (sender, receiver) = mpsc::channel();
        (Self { sender }, receiver)
    }
}

impl EventListener for JarvisEventProxy {
    fn send_event(&self, event: AlacrittyEvent) {
        let mapped = match event {
            AlacrittyEvent::Bell => TerminalEvent::Bell,
            AlacrittyEvent::Title(title) => TerminalEvent::Title(title),
            AlacrittyEvent::ResetTitle => TerminalEvent::ResetTitle,
            AlacrittyEvent::Exit => TerminalEvent::Exit,
            AlacrittyEvent::ChildExit(code) => TerminalEvent::ChildExit(code),
            AlacrittyEvent::ClipboardStore(_clipboard_type, text) => {
                TerminalEvent::ClipboardStore(text)
            }
            AlacrittyEvent::Wakeup => TerminalEvent::Wakeup,
            AlacrittyEvent::CursorBlinkingChange => TerminalEvent::CursorBlinkingChange,
            // Events that need PTY write-back are handled by the caller, not
            // forwarded through the channel.
            AlacrittyEvent::PtyWrite(_)
            | AlacrittyEvent::ClipboardLoad(..)
            | AlacrittyEvent::ColorRequest(..)
            | AlacrittyEvent::TextAreaSizeRequest(..)
            | AlacrittyEvent::MouseCursorDirty => return,
        };

        // Non-blocking send — if the receiver is dropped, we silently discard.
        let _ = self.sender.send(mapped);
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alacritty_terminal::event::Event as AlacrittyEvent;

    #[test]
    fn event_proxy_forwards_bell() {
        let (proxy, rx) = JarvisEventProxy::new();
        proxy.send_event(AlacrittyEvent::Bell);

        let event = rx.recv().expect("should receive Bell event");
        assert!(matches!(event, TerminalEvent::Bell));
    }

    #[test]
    fn event_proxy_forwards_title() {
        let (proxy, rx) = JarvisEventProxy::new();
        proxy.send_event(AlacrittyEvent::Title("Hello Jarvis".into()));

        let event = rx.recv().expect("should receive Title event");
        match event {
            TerminalEvent::Title(title) => assert_eq!(title, "Hello Jarvis"),
            other => panic!("expected Title, got {other:?}"),
        }
    }

    #[test]
    fn event_proxy_forwards_child_exit() {
        let (proxy, rx) = JarvisEventProxy::new();
        proxy.send_event(AlacrittyEvent::ChildExit(42));

        let event = rx.recv().expect("should receive ChildExit event");
        match event {
            TerminalEvent::ChildExit(code) => assert_eq!(code, 42),
            other => panic!("expected ChildExit, got {other:?}"),
        }
    }

    #[test]
    fn event_proxy_discards_pty_write() {
        let (proxy, rx) = JarvisEventProxy::new();
        proxy.send_event(AlacrittyEvent::PtyWrite("test".into()));

        // PtyWrite should not be forwarded
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn event_proxy_survives_dropped_receiver() {
        let (proxy, rx) = JarvisEventProxy::new();
        drop(rx);

        // Should not panic even with dropped receiver
        proxy.send_event(AlacrittyEvent::Bell);
    }
}
