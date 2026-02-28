use jarvis_common::actions::Action;
use jarvis_platform::input::KeybindRegistry;

use super::types::PaletteItem;

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
