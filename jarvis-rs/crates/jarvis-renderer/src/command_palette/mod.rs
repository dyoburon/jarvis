//! Command palette — fuzzy-searchable list of all application actions.
//!
//! The palette is opened via keybind, accepts text input for filtering,
//! and returns a selected [`Action`] when confirmed.

mod palette;
mod types;

pub use palette::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_common::actions::Action;
    use jarvis_config::schema::KeybindConfig;
    use jarvis_platform::input::KeybindRegistry;

    fn make_palette() -> CommandPalette {
        let registry = KeybindRegistry::from_config(&KeybindConfig::default());
        CommandPalette::new(&registry)
    }

    #[test]
    fn initial_state_shows_all() {
        let palette = make_palette();
        assert_eq!(
            palette.visible_items().len(),
            Action::palette_actions().len()
        );
        assert_eq!(palette.selected_index(), 0);
        assert_eq!(palette.query(), "");
    }

    #[test]
    fn filter_narrows_results() {
        let mut palette = make_palette();
        palette.set_query("split");
        let visible = palette.visible_items();
        assert!(visible.len() < Action::palette_actions().len());
        for item in &visible {
            assert!(item.label.to_lowercase().contains("split"));
        }
    }

    #[test]
    fn filter_no_results() {
        let mut palette = make_palette();
        palette.set_query("xyznonexistent");
        assert!(palette.visible_items().is_empty());
        assert_eq!(palette.confirm(), None);
    }

    #[test]
    fn append_and_backspace() {
        let mut palette = make_palette();
        let initial_count = palette.visible_items().len();

        palette.append_char('q');
        let filtered_count = palette.visible_items().len();
        assert!(filtered_count <= initial_count);

        palette.backspace();
        assert_eq!(palette.visible_items().len(), initial_count);
    }

    #[test]
    fn select_next_wraps() {
        let mut palette = make_palette();
        let count = palette.visible_items().len();

        for _ in 0..count {
            palette.select_next();
        }
        // Should wrap back to 0
        assert_eq!(palette.selected_index(), 0);
    }

    #[test]
    fn select_prev_wraps() {
        let mut palette = make_palette();
        palette.select_prev();
        // Should wrap to last item
        assert_eq!(palette.selected_index(), palette.visible_items().len() - 1);
    }

    #[test]
    fn confirm_returns_action() {
        let palette = make_palette();
        let action = palette.confirm();
        assert!(action.is_some());
        assert_eq!(action.unwrap(), Action::palette_actions()[0]);
    }

    #[test]
    fn keybind_display_populated() {
        let palette = make_palette();
        // NewPane should have a keybind (Cmd+T)
        let new_pane = palette
            .visible_items()
            .into_iter()
            .find(|item| item.action == Action::NewPane);
        assert!(new_pane.is_some());
        assert!(new_pane.unwrap().keybind_display.is_some());
    }
}
