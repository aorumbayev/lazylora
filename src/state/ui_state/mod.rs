//! UI state management for the LazyLora TUI.
//!
//! This module manages UI presentation concerns including:
//! - Panel focus (which panel is active)
//! - Popup/modal state
//! - Toast notifications
//! - Expanded sections in detail views
//!
//! # Design
//!
//! The UI state is separate from navigation and data state,
//! focusing purely on presentation layer concerns.

mod popups;
mod search;

use std::collections::HashSet;

pub use popups::{NetworkFormField, NetworkFormState, PopupState};
pub use search::{SearchType, detect_search_type};

// ============================================================================
// Focus
// ============================================================================

/// Represents which UI panel currently has focus.
///
/// Focus determines which panel receives keyboard navigation input
/// and is visually highlighted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    /// The blocks panel has focus.
    #[default]
    Blocks,
    /// The transactions panel has focus.
    Transactions,
}

impl Focus {
    /// Cycles to the next focus target.
    ///
    /// # Returns
    ///
    /// The next focus target in the cycle.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Blocks => Self::Transactions,
            Self::Transactions => Self::Blocks,
        }
    }
}

// ============================================================================
// UI State
// ============================================================================

/// Maximum number of recent searches to remember.
const MAX_SEARCH_HISTORY: usize = 10;

/// UI state: focus, popup state, and viewing flags.
///
/// This struct manages all UI presentation concerns, keeping them
/// separate from navigation and data.
///
/// # Example
///
/// ```ignore
/// use crate::state::UiState;
///
/// let mut ui = UiState::new();
///
/// // Cycle focus between panels
/// ui.cycle_focus();
///
/// // Show a toast notification
/// ui.show_toast("Operation completed!", 20);
/// ```
#[derive(Debug, Default)]
pub struct UiState {
    // === Focus ===
    /// Which panel currently has focus.
    pub focus: Focus,

    // === Popup State ===
    /// Current popup/modal state.
    pub popup_state: PopupState,

    // === Inline Search ===
    /// Current search input text (inline search bar in header).
    pub search_input: String,
    /// Cursor position within search input (byte offset).
    pub search_cursor: usize,
    /// Whether the inline search bar has focus.
    pub search_focused: bool,
    /// Auto-detected search type based on input pattern.
    pub detected_search_type: Option<SearchType>,
    /// User override for search type (Tab cycles through types).
    pub search_type_override: Option<SearchType>,
    /// Whether a search is currently in progress.
    pub search_loading: bool,
    /// Recent search history (most recent first).
    pub search_history: Vec<String>,
    /// Current position in search history (-1 = current input, 0+ = history index).
    pub search_history_index: Option<usize>,
    /// Saved current input when navigating history.
    pub search_input_saved: Option<String>,

    // === View Flags ===
    /// Whether we're currently viewing a search result (affects transaction details display).
    pub viewing_search_result: bool,
    /// The view mode for detail popups (Visual or Table).
    pub detail_view_mode: crate::state::navigation::DetailViewMode,

    // === Expandable Sections ===
    /// Set of expanded section names in transaction details (e.g., "app_args", "accounts").
    pub expanded_sections: HashSet<String>,
    /// Currently focused expandable section index in transaction details.
    pub detail_section_index: Option<usize>,
    /// Whether the detail popup is in fullscreen mode.
    pub detail_fullscreen: bool,

    // === Toast Notifications ===
    /// Toast notification message and remaining ticks (non-blocking overlay).
    pub toast: Option<(String, u8)>,

    // === Help Popup ===
    /// Whether the help popup is visible.
    pub show_help: bool,
    /// Scroll offset for help popup content.
    pub help_scroll_offset: u16,

    // === Detail Table Row Data ===
    /// Current detail table rows (label, value) for copy functionality.
    /// Updated each time a detail popup is rendered in Table mode.
    pub detail_table_rows: Vec<(String, String)>,
}

impl UiState {
    /// Creates a new `UiState` with default values.
    ///
    /// # Returns
    ///
    /// A new UI state with default focus and no popups.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // ========================================================================
    // Focus Management
    // ========================================================================

    /// Cycles focus between Blocks and Transactions panels.
    pub fn cycle_focus(&mut self) {
        self.focus = self.focus.next();
    }

    // ========================================================================
    // Popup Management
    // ========================================================================

    /// Returns `true` if the popup is active.
    #[must_use]
    pub fn has_active_popup(&self) -> bool {
        self.popup_state.is_active()
    }

    /// Dismisses the current popup.
    pub fn dismiss_popup(&mut self) {
        self.popup_state = PopupState::None;
    }

    /// Shows a message popup.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display
    pub fn show_message(&mut self, message: impl Into<String>) {
        self.popup_state = PopupState::Message(message.into());
    }

    /// Opens the network selection popup with the given current index.
    ///
    /// # Arguments
    ///
    /// * `current_index` - The index of the currently selected network
    pub fn open_network_select(&mut self, current_index: usize) {
        self.popup_state = PopupState::NetworkSelect(current_index);
    }

    /// Opens the add custom network form, remembering the originating selection.
    pub fn open_network_form(&mut self, return_to_index: usize) {
        self.popup_state = PopupState::NetworkForm(NetworkFormState::new(return_to_index));
    }

    /// Opens the quit confirmation popup.
    pub fn open_confirm_quit(&mut self) {
        self.popup_state = PopupState::ConfirmQuit;
    }

    /// Updates the search query text while preserving the search type.
    ///
    /// # Arguments
    ///
    /// * `new_query` - The updated query string
    /// * `search_type` - The current search type
    pub fn update_search_query(&mut self, new_query: String, search_type: SearchType) {
        self.popup_state = PopupState::SearchWithType(new_query, search_type);
    }

    /// Switches to the next search type while preserving the query.
    ///
    /// # Arguments
    ///
    /// * `query` - The current query string
    /// * `current_type` - The current search type
    pub fn cycle_search_type(&mut self, query: String, current_type: SearchType) {
        self.popup_state = PopupState::SearchWithType(query, current_type.next());
    }

    /// Updates the network selection index.
    ///
    /// # Arguments
    ///
    /// * `index` - The new selected index
    pub fn update_network_selection(&mut self, index: usize) {
        self.popup_state = PopupState::NetworkSelect(index);
    }

    /// Moves to the next field in the network form.
    pub fn network_form_next_field(&mut self) {
        if let PopupState::NetworkForm(form) = &mut self.popup_state {
            form.next_field();
        }
    }

    /// Moves to the previous field in the network form.
    pub fn network_form_prev_field(&mut self) {
        if let PopupState::NetworkForm(form) = &mut self.popup_state {
            form.prev_field();
        }
    }

    /// Types a character into the active network form field.
    pub fn network_form_type_char(&mut self, c: char) {
        if let PopupState::NetworkForm(form) = &mut self.popup_state {
            form.push_char(c);
        }
    }

    /// Deletes a character from the active network form field.
    pub fn network_form_backspace(&mut self) {
        if let PopupState::NetworkForm(form) = &mut self.popup_state {
            form.backspace();
        }
    }

    /// Sets search results popup.
    ///
    /// # Arguments
    ///
    /// * `results` - The search results to display
    pub fn show_search_results(&mut self, results: Vec<(usize, crate::domain::SearchResultItem)>) {
        self.popup_state = PopupState::SearchResults(results);
    }

    /// Rotates search results forward (first item moves to end).
    ///
    /// This is used for the "previous result" action, bringing the next
    /// result to the front by rotating left.
    pub fn rotate_search_results_forward(&mut self) {
        if let PopupState::SearchResults(results) = &mut self.popup_state
            && results.len() > 1
        {
            results.rotate_left(1);
        }
    }

    /// Rotates search results backward (last item moves to front).
    ///
    /// This is used for the "next result" action, bringing the previous
    /// result to the front by rotating right.
    pub fn rotate_search_results_backward(&mut self) {
        if let PopupState::SearchResults(results) = &mut self.popup_state
            && results.len() > 1
        {
            results.rotate_right(1);
        }
    }

    // ========================================================================
    // Inline Search
    // ========================================================================

    /// Focus the inline search bar.
    pub fn focus_search(&mut self) {
        self.search_focused = true;
        self.search_cursor = self.search_input.len();
        self.search_history_index = None;
        self.search_input_saved = None;
    }

    /// Unfocus the inline search bar.
    pub fn unfocus_search(&mut self) {
        self.search_focused = false;
        self.search_cursor = 0;
        self.search_history_index = None;
        self.search_input_saved = None;
    }

    /// Returns whether the inline search bar is focused.
    #[must_use]
    pub fn is_search_focused(&self) -> bool {
        self.search_focused
    }

    /// Appends a character at cursor position.
    pub fn search_type_char(&mut self, c: char) {
        // Reset history navigation when typing
        self.search_history_index = None;
        self.search_input_saved = None;
        self.search_input.insert(self.search_cursor, c);
        self.search_cursor += c.len_utf8();
        self.update_detected_search_type();
        // Clear override when input changes
        self.search_type_override = None;
    }

    /// Removes character before cursor.
    pub fn search_backspace(&mut self) {
        // Reset history navigation when typing
        self.search_history_index = None;
        self.search_input_saved = None;
        if self.search_cursor > 0 {
            // Find the previous char boundary
            let prev = self.search_input[..self.search_cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.search_input.remove(prev);
            self.search_cursor = prev;
        }
        self.update_detected_search_type();
        // Clear override when input changes
        self.search_type_override = None;
    }

    /// Clears the search input.
    pub fn clear_search(&mut self) {
        self.search_input.clear();
        self.search_cursor = 0;
        self.detected_search_type = None;
        self.search_type_override = None;
        self.search_history_index = None;
        self.search_input_saved = None;
    }

    /// Gets the current search input text.
    #[must_use]
    pub fn search_query(&self) -> &str {
        &self.search_input
    }

    /// Gets the current cursor position (byte offset).
    #[must_use]
    pub fn cursor_position(&self) -> usize {
        self.search_cursor
    }

    /// Move cursor left by one character.
    pub fn search_cursor_left(&mut self) {
        if self.search_cursor > 0 {
            self.search_cursor = self.search_input[..self.search_cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right by one character.
    pub fn search_cursor_right(&mut self) {
        if self.search_cursor < self.search_input.len() {
            self.search_cursor = self.search_input[self.search_cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.search_cursor + i)
                .unwrap_or(self.search_input.len());
        }
    }

    /// Updates the detected search type based on the current input.
    fn update_detected_search_type(&mut self) {
        self.detected_search_type = detect_search_type(&self.search_input);
    }

    /// Cycle to the next search type (Tab key).
    pub fn cycle_inline_search_type(&mut self) {
        let current = self.get_effective_search_type();
        let next = match current {
            Some(t) => t.next(),
            None => SearchType::Transaction,
        };
        self.search_type_override = Some(next);
    }

    /// Get the effective search type (override or auto-detected).
    #[must_use]
    pub fn get_effective_search_type(&self) -> Option<SearchType> {
        self.search_type_override.or(self.detected_search_type)
    }

    /// Set search loading state.
    pub fn set_search_loading(&mut self, loading: bool) {
        self.search_loading = loading;
    }

    /// Add a search query to history.
    pub fn add_to_search_history(&mut self, query: &str) {
        if query.is_empty() {
            return;
        }
        // Remove if already exists
        self.search_history.retain(|q| q != query);
        // Add to front
        self.search_history.insert(0, query.to_string());
        // Trim to max size
        self.search_history.truncate(MAX_SEARCH_HISTORY);
    }

    /// Navigate to previous search in history (Up arrow).
    pub fn search_history_prev(&mut self) {
        if self.search_history.is_empty() {
            return;
        }

        match self.search_history_index {
            None => {
                // Save current input and go to first history item
                self.search_input_saved = Some(self.search_input.clone());
                self.search_history_index = Some(0);
                if let Some(query) = self.search_history.first() {
                    self.search_input = query.clone();
                    self.search_cursor = self.search_input.len();
                    self.update_detected_search_type();
                    self.search_type_override = None;
                }
            }
            Some(idx) if idx + 1 < self.search_history.len() => {
                // Go to next older item
                self.search_history_index = Some(idx + 1);
                if let Some(query) = self.search_history.get(idx + 1) {
                    self.search_input = query.clone();
                    self.search_cursor = self.search_input.len();
                    self.update_detected_search_type();
                    self.search_type_override = None;
                }
            }
            _ => {} // At oldest item, do nothing
        }
    }

    /// Navigate to next search in history (Down arrow).
    pub fn search_history_next(&mut self) {
        match self.search_history_index {
            Some(0) => {
                // At newest history item, restore saved input
                self.search_history_index = None;
                if let Some(saved) = self.search_input_saved.take() {
                    self.search_input = saved;
                    self.search_cursor = self.search_input.len();
                    self.update_detected_search_type();
                    self.search_type_override = None;
                }
            }
            Some(idx) => {
                // Go to next newer item
                self.search_history_index = Some(idx - 1);
                if let Some(query) = self.search_history.get(idx - 1) {
                    self.search_input = query.clone();
                    self.search_cursor = self.search_input.len();
                    self.update_detected_search_type();
                    self.search_type_override = None;
                }
            }
            None => {} // Not in history navigation, do nothing
        }
    }

    // ========================================================================
    // Toast Notifications
    // ========================================================================

    /// Shows a toast notification (non-blocking overlay that auto-dismisses).
    ///
    /// Duration is in ticks (each tick is ~100ms in the main loop).
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display
    /// * `ticks` - Number of ticks before auto-dismiss
    pub fn show_toast(&mut self, message: impl Into<String>, ticks: u8) {
        self.toast = Some((message.into(), ticks));
    }

    /// Decrements the toast countdown.
    ///
    /// # Returns
    ///
    /// `true` if the toast was removed (countdown reached zero).
    pub fn tick_toast(&mut self) -> bool {
        if let Some((_, ref mut ticks)) = self.toast {
            if *ticks > 1 {
                *ticks -= 1;
                false
            } else {
                self.toast = None;
                true
            }
        } else {
            false
        }
    }

    // ========================================================================
    // Detail View Mode
    // ========================================================================

    /// Toggles the detail view mode between Visual and Table.
    pub fn toggle_detail_view_mode(&mut self) {
        self.detail_view_mode = self.detail_view_mode.toggle();
    }

    // ========================================================================
    // Help Popup
    // ========================================================================

    /// Toggles the help popup visibility.
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if !self.show_help {
            self.help_scroll_offset = 0;
        }
    }

    /// Scrolls help popup up by one line.
    pub fn scroll_help_up(&mut self) {
        self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
    }

    /// Scrolls help popup down by one line.
    pub fn scroll_help_down(&mut self) {
        self.help_scroll_offset = self.help_scroll_offset.saturating_add(1);
    }

    // ========================================================================
    // Expandable Sections
    // ========================================================================

    /// Toggles whether a section is expanded in transaction details.
    ///
    /// # Arguments
    ///
    /// * `section_name` - The name of the section to toggle
    pub fn toggle_section(&mut self, section_name: &str) {
        if self.expanded_sections.contains(section_name) {
            self.expanded_sections.remove(section_name);
        } else {
            self.expanded_sections.insert(section_name.to_string());
        }
    }

    /// Resets expanded sections and fullscreen state (call when closing details).
    pub fn reset_expanded_sections(&mut self) {
        self.expanded_sections.clear();
        self.detail_section_index = None;
        self.detail_fullscreen = false;
    }

    /// Toggles fullscreen mode for detail popups.
    pub fn toggle_fullscreen(&mut self) {
        self.detail_fullscreen = !self.detail_fullscreen;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_cycle_behavior() {
        // Default is Blocks, cycles through both values
        assert_eq!(Focus::default(), Focus::Blocks);
        assert_eq!(Focus::Blocks.next(), Focus::Transactions);
        assert_eq!(Focus::Transactions.next(), Focus::Blocks);
    }

    #[test]
    fn test_network_form_input_flow() {
        let mut ui = UiState::new();
        ui.open_network_form(2);

        ui.network_form_type_char('A');
        ui.network_form_next_field();
        ui.network_form_type_char('h');
        ui.network_form_backspace();
        ui.network_form_type_char('t');

        if let PopupState::NetworkForm(form) = &ui.popup_state {
            assert_eq!(form.name, "A");
            assert_eq!(form.algod_url, "t");
            assert_eq!(form.active_field, NetworkFormField::AlgodUrl);
        } else {
            panic!("Expected network form popup");
        }
    }

    #[test]
    fn test_ui_state_focus_management() {
        let mut ui = UiState::new();
        assert_eq!(ui.focus, Focus::Blocks);

        ui.cycle_focus();
        assert_eq!(ui.focus, Focus::Transactions);

        ui.cycle_focus();
        assert_eq!(ui.focus, Focus::Blocks);
    }

    #[test]
    fn test_ui_state_popup_lifecycle() {
        let mut ui = UiState::new();
        assert!(!ui.has_active_popup());

        // Show message
        ui.show_message("Test");
        assert!(ui.has_active_popup());

        // Dismiss
        ui.dismiss_popup();
        assert!(!ui.has_active_popup());

        // Open search and manipulate
        ui.popup_state = PopupState::SearchWithType(String::new(), SearchType::Transaction);
        let (query, search_type) = ui.popup_state.as_search().unwrap();
        assert_eq!(query, "");
        assert_eq!(search_type, SearchType::Transaction);

        ui.update_search_query("test".to_string(), SearchType::Account);
        ui.cycle_search_type("test".to_string(), SearchType::Account);
        let (_, new_type) = ui.popup_state.as_search().unwrap();
        assert_eq!(new_type, SearchType::Asset);
    }

    #[test]
    fn test_ui_state_toast_lifecycle() {
        let mut ui = UiState::new();
        assert!(ui.toast.is_none());

        ui.show_toast("Hello", 2);
        assert!(ui.toast.is_some());
        assert_eq!(
            ui.toast.as_ref().map(|(msg, _)| msg.as_str()),
            Some("Hello")
        );

        assert!(!ui.tick_toast()); // 2 -> 1
        assert!(ui.tick_toast()); // 1 -> removed
        assert!(ui.toast.is_none());
    }

    #[test]
    fn test_ui_state_expandable_sections() {
        let mut ui = UiState::new();

        // Toggle section on and off
        assert!(!ui.expanded_sections.contains("test"));
        ui.toggle_section("test");
        assert!(ui.expanded_sections.contains("test"));
        ui.toggle_section("test");
        assert!(!ui.expanded_sections.contains("test"));

        // Reset clears everything
        ui.toggle_section("a");
        ui.toggle_section("b");
        ui.detail_section_index = Some(1);
        ui.detail_fullscreen = true;

        ui.reset_expanded_sections();
        assert!(!ui.expanded_sections.contains("a"));
        assert!(!ui.expanded_sections.contains("b"));
        assert!(ui.detail_section_index.is_none());
        assert!(!ui.detail_fullscreen);
    }

    #[test]
    fn test_help_popup_lifecycle() {
        let mut ui = UiState::new();

        // Initially not shown
        assert!(!ui.show_help);
        assert_eq!(ui.help_scroll_offset, 0);

        // Toggle on
        ui.toggle_help();
        assert!(ui.show_help);

        // Scroll down
        ui.scroll_help_down();
        assert_eq!(ui.help_scroll_offset, 1);
        ui.scroll_help_down();
        assert_eq!(ui.help_scroll_offset, 2);

        // Scroll up
        ui.scroll_help_up();
        assert_eq!(ui.help_scroll_offset, 1);
        ui.scroll_help_up();
        assert_eq!(ui.help_scroll_offset, 0);

        // Can't go below zero
        ui.scroll_help_up();
        assert_eq!(ui.help_scroll_offset, 0);

        // Toggle off resets scroll
        ui.scroll_help_down();
        ui.scroll_help_down();
        assert_eq!(ui.help_scroll_offset, 2);
        ui.toggle_help();
        assert!(!ui.show_help);
        assert_eq!(ui.help_scroll_offset, 0);
    }

    #[test]
    fn test_inline_search_lifecycle() {
        let mut ui = UiState::new();

        // Initially not focused
        assert!(!ui.is_search_focused());
        assert!(ui.search_query().is_empty());

        // Focus the search bar
        ui.focus_search();
        assert!(ui.is_search_focused());

        // Type characters
        ui.search_type_char('a');
        ui.search_type_char('b');
        ui.search_type_char('c');
        assert_eq!(ui.search_query(), "abc");

        // Backspace
        ui.search_backspace();
        assert_eq!(ui.search_query(), "ab");

        // Clear search
        ui.clear_search();
        assert!(ui.search_query().is_empty());
        assert!(ui.detected_search_type.is_none());

        // Unfocus
        ui.unfocus_search();
        assert!(!ui.is_search_focused());
    }

    #[test]
    fn test_search_results_rotation() {
        use crate::domain::{AssetInfo, SearchResultItem};

        let mut ui = UiState::new();

        // Create test results using AssetInfo for simplicity
        let make_asset = |id: u64| {
            SearchResultItem::Asset(AssetInfo::new(
                id,
                format!("Asset{id}"),
                format!("A{id}"),
                "CREATOR".to_string(),
                1000,
                6,
                String::new(),
            ))
        };

        let results = vec![
            (0, make_asset(100)),
            (1, make_asset(200)),
            (2, make_asset(300)),
        ];
        ui.show_search_results(results);

        // Rotate forward: first goes to end -> indices become [1, 2, 0]
        ui.rotate_search_results_forward();
        if let PopupState::SearchResults(r) = &ui.popup_state {
            assert_eq!(r[0].0, 1);
            assert_eq!(r[1].0, 2);
            assert_eq!(r[2].0, 0);
        } else {
            panic!("Expected SearchResults popup");
        }

        // Rotate backward: last goes to front -> indices become [0, 1, 2]
        ui.rotate_search_results_backward();
        if let PopupState::SearchResults(r) = &ui.popup_state {
            assert_eq!(r[0].0, 0);
            assert_eq!(r[1].0, 1);
            assert_eq!(r[2].0, 2);
        } else {
            panic!("Expected SearchResults popup");
        }
    }

    #[test]
    fn test_search_results_rotation_edge_cases() {
        use crate::domain::{AssetInfo, SearchResultItem};

        let mut ui = UiState::new();

        let make_asset = |id: u64| {
            SearchResultItem::Asset(AssetInfo::new(
                id,
                format!("Asset{id}"),
                format!("A{id}"),
                "CREATOR".to_string(),
                1000,
                6,
                String::new(),
            ))
        };

        // Single element: rotation should be no-op
        let single = vec![(0, make_asset(100))];
        ui.show_search_results(single);
        ui.rotate_search_results_forward();
        if let PopupState::SearchResults(r) = &ui.popup_state {
            assert_eq!(r.len(), 1);
            assert_eq!(r[0].0, 0);
        }

        // Empty: should handle gracefully (though unlikely in practice)
        ui.show_search_results(vec![]);
        ui.rotate_search_results_forward();
        ui.rotate_search_results_backward();
        if let PopupState::SearchResults(r) = &ui.popup_state {
            assert!(r.is_empty());
        }

        // Wrong popup state: should be no-op
        ui.popup_state = PopupState::None;
        ui.rotate_search_results_forward(); // Should not panic
        ui.rotate_search_results_backward(); // Should not panic
    }
}
