use serde::{Deserialize, Serialize};

mod action_enum;
mod dispatch;

pub use action_enum::*;

/// Direction for pane resizing and swapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResizeDirection {
    Left,
    Right,
    Up,
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_palette_actions_have_labels() {
        for action in Action::palette_actions() {
            let label = action.label();
            assert!(!label.is_empty(), "action {:?} has empty label", action);
        }
    }

    #[test]
    fn palette_actions_not_empty() {
        assert!(!Action::palette_actions().is_empty());
    }

    #[test]
    fn focus_pane_labels() {
        assert_eq!(Action::FocusPane(1).label(), "Focus Pane 1");
        assert_eq!(Action::FocusPane(5).label(), "Focus Pane 5");
        assert_eq!(Action::FocusPane(99).label(), "Focus Pane");
    }

    #[test]
    fn action_serde_roundtrip() {
        let actions = vec![
            Action::NewPane,
            Action::FocusPane(3),
            Action::ResizePane {
                direction: ResizeDirection::Left,
                delta: 10,
            },
            Action::ScrollUp(5),
        ];

        for action in &actions {
            let json = serde_json::to_string(action).unwrap();
            let deserialized: Action = serde_json::from_str(&json).unwrap();
            assert_eq!(*action, deserialized);
        }
    }

    #[test]
    fn resize_direction_serde() {
        let dir = ResizeDirection::Up;
        let json = serde_json::to_string(&dir).unwrap();
        let back: ResizeDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(dir, back);
    }
}
