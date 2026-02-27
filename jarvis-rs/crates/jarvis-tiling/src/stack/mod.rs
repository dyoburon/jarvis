//! Pane stacking — multiple panes occupying the same leaf position (tabs).

mod operations;
mod types;

pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stack_has_one_pane() {
        let stack = PaneStack::new(1);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.active(), 1);
    }

    #[test]
    fn push_makes_new_pane_active() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.active(), 2);
    }

    #[test]
    fn remove_adjusts_active() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        // Active is 3 (index 2)
        assert!(stack.remove(3));
        assert_eq!(stack.active(), 2); // Falls back to previous
    }

    #[test]
    fn remove_earlier_pane_adjusts_index() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        // Active is 3 (index 2), remove pane 1 (index 0)
        assert!(stack.remove(1));
        assert_eq!(stack.active(), 3); // Index shifted but active stays same
        assert_eq!(stack.active_index(), 1);
    }

    #[test]
    fn remove_last_pane_fails() {
        let mut stack = PaneStack::new(1);
        assert!(!stack.remove(1));
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn remove_nonexistent_fails() {
        let mut stack = PaneStack::new(1);
        assert!(!stack.remove(99));
    }

    #[test]
    fn cycle_next_wraps() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        stack.set_active(1);
        assert_eq!(stack.active(), 1);
        stack.cycle_next();
        assert_eq!(stack.active(), 2);
        stack.cycle_next();
        assert_eq!(stack.active(), 3);
        stack.cycle_next();
        assert_eq!(stack.active(), 1); // wrapped
    }

    #[test]
    fn cycle_prev_wraps() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        stack.set_active(1);
        stack.cycle_prev();
        assert_eq!(stack.active(), 3); // wrapped
        stack.cycle_prev();
        assert_eq!(stack.active(), 2);
    }

    #[test]
    fn cycle_single_pane_no_change() {
        let mut stack = PaneStack::new(1);
        stack.cycle_next();
        assert_eq!(stack.active(), 1);
        stack.cycle_prev();
        assert_eq!(stack.active(), 1);
    }

    #[test]
    fn set_active_by_id() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        assert!(stack.set_active(1));
        assert_eq!(stack.active(), 1);
        assert!(!stack.set_active(99));
    }

    #[test]
    fn contains_works() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        assert!(stack.contains(1));
        assert!(stack.contains(2));
        assert!(!stack.contains(3));
    }

    #[test]
    fn pane_ids_returns_ordered() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.pane_ids(), &[1, 2, 3]);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        stack.set_active(2);
        let json = serde_json::to_string(&stack).unwrap();
        let deserialized: PaneStack = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.active(), 2);
        assert_eq!(deserialized.len(), 3);
    }
}
