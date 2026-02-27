mod operations;
mod traversal;
mod types;

pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_count() {
        assert_eq!(SplitNode::leaf(1).pane_count(), 1);
    }

    #[test]
    fn split_count() {
        let tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert_eq!(tree.pane_count(), 2);
    }

    #[test]
    fn contains() {
        let tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert!(tree.contains_pane(1));
        assert!(tree.contains_pane(3));
        assert!(!tree.contains_pane(99));
    }

    #[test]
    fn collect_pane_ids_single() {
        let tree = SplitNode::leaf(42);
        assert_eq!(tree.collect_pane_ids(), vec![42]);
    }

    #[test]
    fn collect_pane_ids_nested() {
        let tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert_eq!(tree.collect_pane_ids(), vec![1, 2, 3]);
    }

    #[test]
    fn split_at_leaf() {
        let mut tree = SplitNode::leaf(1);
        assert!(tree.split_at(1, 2, Direction::Horizontal));
        assert_eq!(tree.pane_count(), 2);
        assert!(tree.contains_pane(1));
        assert!(tree.contains_pane(2));
    }

    #[test]
    fn split_at_nested() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(tree.split_at(2, 3, Direction::Vertical));
        assert_eq!(tree.pane_count(), 3);
        assert_eq!(tree.collect_pane_ids(), vec![1, 2, 3]);
    }

    #[test]
    fn split_at_nonexistent() {
        let mut tree = SplitNode::leaf(1);
        assert!(!tree.split_at(99, 2, Direction::Horizontal));
        assert_eq!(tree.pane_count(), 1);
    }

    #[test]
    fn remove_pane_from_split() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(tree.remove_pane(1));
        assert_eq!(tree.pane_count(), 1);
        assert!(tree.contains_pane(2));
        assert!(!tree.contains_pane(1));
    }

    #[test]
    fn remove_pane_nested() {
        let mut tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert!(tree.remove_pane(2));
        assert_eq!(tree.pane_count(), 2);
        assert!(tree.contains_pane(1));
        assert!(tree.contains_pane(3));
    }

    #[test]
    fn remove_last_pane_fails() {
        let mut tree = SplitNode::leaf(1);
        assert!(!tree.remove_pane(1));
    }

    #[test]
    fn swap_panes_basic() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(tree.swap_panes(1, 2));
        assert_eq!(tree.collect_pane_ids(), vec![2, 1]);
    }

    #[test]
    fn swap_panes_nested() {
        let mut tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert!(tree.swap_panes(1, 3));
        assert_eq!(tree.collect_pane_ids(), vec![3, 2, 1]);
    }

    #[test]
    fn swap_nonexistent_fails() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(!tree.swap_panes(1, 99));
    }

    #[test]
    fn adjust_ratio_grow_first() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(tree.adjust_ratio(1, 0.1));
        if let SplitNode::Split { ratio, .. } = &tree {
            assert!((*ratio - 0.6).abs() < 0.001);
        } else {
            panic!("expected split");
        }
    }

    #[test]
    fn adjust_ratio_grow_second() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(tree.adjust_ratio(2, 0.1));
        if let SplitNode::Split { ratio, .. } = &tree {
            assert!((*ratio - 0.4).abs() < 0.001);
        } else {
            panic!("expected split");
        }
    }

    #[test]
    fn adjust_ratio_clamps() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        // Try to grow way beyond limit
        assert!(tree.adjust_ratio(1, 0.9));
        if let SplitNode::Split { ratio, .. } = &tree {
            assert!((*ratio - 0.9).abs() < 0.001);
        } else {
            panic!("expected split");
        }
    }

    #[test]
    fn next_pane_wraps() {
        let tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert_eq!(tree.next_pane(1), Some(2));
        assert_eq!(tree.next_pane(2), Some(3));
        assert_eq!(tree.next_pane(3), Some(1)); // wraps
    }

    #[test]
    fn prev_pane_wraps() {
        let tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert_eq!(tree.prev_pane(1), Some(3)); // wraps
        assert_eq!(tree.prev_pane(2), Some(1));
        assert_eq!(tree.prev_pane(3), Some(2));
    }

    #[test]
    fn next_prev_single_pane_returns_none() {
        let tree = SplitNode::leaf(1);
        assert_eq!(tree.next_pane(1), None);
        assert_eq!(tree.prev_pane(1), None);
    }

    #[test]
    fn find_neighbor_horizontal() {
        let tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert_eq!(tree.find_neighbor(1, Direction::Horizontal), Some(2));
        assert_eq!(tree.find_neighbor(2, Direction::Horizontal), None);
    }

    #[test]
    fn find_neighbor_vertical() {
        let tree = SplitNode::split_v(SplitNode::leaf(1), SplitNode::leaf(2));
        assert_eq!(tree.find_neighbor(2, Direction::Vertical), Some(1));
        assert_eq!(tree.find_neighbor(1, Direction::Vertical), None);
    }
}
