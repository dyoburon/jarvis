//! Command palette — fuzzy-searchable list of all application actions.
//!
//! The palette is opened via keybind, accepts text input for filtering,
//! and returns a selected [`Action`] when confirmed.

use jarvis_common::actions::Action;
use jarvis_platform::input::KeybindRegistry;

/// A single item in the command palette.
#[derive(Debug, Clone)]
pub struct PaletteItem {
    /// The action this item triggers.
    pub action: Action,
    /// Human-readable label.
    pub label: String,
    /// The keybind display string (e.g. "⌘T"), if one is bound.
    pub keybind_display: Option<String>,
}

/// Command palette state: query, filtered items, selection.
pub struct CommandPalette {
    query: String,
    items: Vec<PaletteItem>,
    filtered: Vec<usize>,
    selected: usize,
}

impl CommandPalette {
    /// Create a new command palette from the action registry.
    pub fn new(registry: &KeybindRegistry) -> Self {
        let items: Vec<PaletteItem> = Action::palette_actions()
            .into_iter()
            .map(|action| {
                let keybind_display = registry.keybind_for_action(&action);
                PaletteItem {
                    label: action.label().to_string(),
                    keybind_display,
                    action,
                }
            })
            .collect();

        let filtered = (0..items.len()).collect();

        Self {
            query: String::new(),
            items,
            filtered,
            selected: 0,
        }
    }

    /// Set the query and re-filter.
    pub fn set_query(&mut self, query: &str) {
        self.query = query.to_string();
        self.filter();
        self.selected = 0;
    }

    /// Append a character to the query.
    pub fn append_char(&mut self, c: char) {
        self.query.push(c);
        self.filter();
        self.selected = 0;
    }

    /// Remove the last character from the query.
    pub fn backspace(&mut self) {
        self.query.pop();
        self.filter();
        self.selected = 0;
    }

    /// Move selection down.
    pub fn select_next(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + 1) % self.filtered.len();
        }
    }

    /// Move selection up.
    pub fn select_prev(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + self.filtered.len() - 1) % self.filtered.len();
        }
    }

    /// Confirm the current selection, returning the action.
    pub fn confirm(&self) -> Option<Action> {
        self.filtered
            .get(self.selected)
            .map(|&idx| self.items[idx].action.clone())
    }

    /// The items currently visible after filtering.
    pub fn visible_items(&self) -> Vec<&PaletteItem> {
        self.filtered.iter().map(|&idx| &self.items[idx]).collect()
    }

    /// Index of the selected item within `visible_items()`.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// The current query string.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Re-filter items based on the current query (case-insensitive substring).
    fn filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.items.len()).collect();
            return;
        }

        let query_lower = self.query.to_lowercase();
        self.filtered = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.label.to_lowercase().contains(&query_lower))
            .map(|(i, _)| i)
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_config::schema::KeybindConfig;

    fn make_palette() -> CommandPalette {
        let registry = KeybindRegistry::from_config(&KeybindConfig::default());
        CommandPalette::new(&registry)
    }

    #[test]
    fn initial_state_shows_all() {
        let palette = make_palette();
        assert_eq!(palette.visible_items().len(), Action::palette_actions().len());
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
        assert_eq!(
            palette.selected_index(),
            palette.visible_items().len() - 1
        );
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
