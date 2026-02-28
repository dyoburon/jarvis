//! Scrollback ring buffer for terminal history.
//!
//! Stores lines that have scrolled off the top of the visible grid, up to a
//! configurable maximum. Oldest lines are dropped when the capacity is exceeded.

mod buffer;

pub use buffer::*;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Cell;
    use crate::grid::CellAttributes;

    /// Helper: build a line of `Cell`s from a `&str`.
    fn make_line(s: &str) -> Vec<Cell> {
        s.chars()
            .map(|c| Cell {
                c,
                attrs: CellAttributes::default(),
                width: 1,
            })
            .collect()
    }

    #[test]
    fn push_and_retrieve() {
        let mut buf = ScrollbackBuffer::new(100);
        buf.push(make_line("hello"));
        buf.push(make_line("world"));

        assert_eq!(buf.len(), 2);
        assert_eq!(buf.line_to_string(0), Some("hello".to_string()));
        assert_eq!(buf.line_to_string(1), Some("world".to_string()));
    }

    #[test]
    fn capacity_limit_drops_oldest() {
        let mut buf = ScrollbackBuffer::new(2);
        buf.push(make_line("aaa"));
        buf.push(make_line("bbb"));
        buf.push(make_line("ccc"));

        assert_eq!(buf.len(), 2);
        // "aaa" should have been evicted.
        assert_eq!(buf.line_to_string(0), Some("bbb".to_string()));
        assert_eq!(buf.line_to_string(1), Some("ccc".to_string()));
    }

    #[test]
    fn push_many_respects_capacity() {
        let mut buf = ScrollbackBuffer::new(3);
        buf.push_many(vec![
            make_line("1"),
            make_line("2"),
            make_line("3"),
            make_line("4"),
            make_line("5"),
        ]);

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.line_to_string(0), Some("3".to_string()));
        assert_eq!(buf.line_to_string(1), Some("4".to_string()));
        assert_eq!(buf.line_to_string(2), Some("5".to_string()));
    }

    #[test]
    fn line_to_string_trims_trailing_spaces() {
        let mut buf = ScrollbackBuffer::new(10);
        buf.push(make_line("hello   "));

        assert_eq!(buf.line_to_string(0), Some("hello".to_string()));
    }

    #[test]
    fn clear_removes_all() {
        let mut buf = ScrollbackBuffer::new(10);
        buf.push(make_line("one"));
        buf.push(make_line("two"));
        buf.clear();

        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn search_finds_matches() {
        let mut buf = ScrollbackBuffer::new(100);
        buf.push(make_line("hello world"));
        buf.push(make_line("foo bar"));
        buf.push(make_line("hello again"));

        let matches = buf.search("hello");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (0, 0));
        assert_eq!(matches[1], (2, 0));
    }

    #[test]
    fn search_returns_empty_for_no_match() {
        let mut buf = ScrollbackBuffer::new(100);
        buf.push(make_line("hello world"));

        let matches = buf.search("xyz");
        assert!(matches.is_empty());
    }

    #[test]
    fn empty_buffer() {
        let buf = ScrollbackBuffer::new(100);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.get(0), None);
        assert_eq!(buf.line_to_string(0), None);
        assert!(buf.search("anything").is_empty());
    }
}
