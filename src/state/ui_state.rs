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

#![allow(dead_code)]

use std::collections::HashSet;

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

    /// Returns the name of the focused panel.
    ///
    /// # Returns
    ///
    /// A static string describing the focused panel.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Blocks => "Blocks",
            Self::Transactions => "Transactions",
        }
    }
}

// ============================================================================
// Search Type
// ============================================================================

/// The type of search to perform.
///
/// This determines how search queries are interpreted and which
/// API endpoints are called.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchType {
    /// Search for transactions by ID.
    #[default]
    Transaction,
    /// Search for assets by ID.
    Asset,
    /// Search for accounts by address or NFD name.
    Account,
    /// Search for blocks by round number.
    Block,
}

impl SearchType {
    /// Returns the display string for this search type.
    ///
    /// # Returns
    ///
    /// A static string describing the search type.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Transaction => "Transaction",
            Self::Asset => "Asset",
            Self::Account => "Account",
            Self::Block => "Block",
        }
    }

    /// Cycles to the next search type.
    ///
    /// # Returns
    ///
    /// The next search type in the cycle.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Transaction => Self::Block,
            Self::Block => Self::Account,
            Self::Account => Self::Asset,
            Self::Asset => Self::Transaction,
        }
    }

    /// Returns all search types in order.
    ///
    /// # Returns
    ///
    /// An array of all search type variants.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [Self::Transaction, Self::Block, Self::Account, Self::Asset]
    }
}

// ============================================================================
// Popup State
// ============================================================================

/// Represents the current popup/modal state.
///
/// Only one popup can be active at a time. The popup state
/// determines which overlay is displayed and handles popup-specific data.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PopupState {
    /// No popup is displayed.
    #[default]
    None,
    /// Network selection popup with the currently highlighted index.
    NetworkSelect(usize),
    /// Search popup with query text and search type.
    SearchWithType(String, SearchType),
    /// Message/notification popup.
    Message(String),
    /// Search results display with indexed items.
    SearchResults(Vec<(usize, crate::algorand::SearchResultItem)>),
}

impl PopupState {
    /// Returns `true` if there is an active popup.
    ///
    /// # Returns
    ///
    /// `true` if any popup is displayed.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns the search query and type if in search mode.
    ///
    /// # Returns
    ///
    /// `Some` tuple of query string and search type if in search mode.
    #[must_use]
    pub fn as_search(&self) -> Option<(&str, SearchType)> {
        match self {
            Self::SearchWithType(query, search_type) => Some((query.as_str(), *search_type)),
            _ => None,
        }
    }

    /// Returns the search results if displaying results.
    ///
    /// # Returns
    ///
    /// `Some` slice of search results if in search results mode.
    #[must_use]
    pub fn as_search_results(&self) -> Option<&[(usize, crate::algorand::SearchResultItem)]> {
        match self {
            Self::SearchResults(results) => Some(results.as_slice()),
            _ => None,
        }
    }

    /// Returns the network select index if in network select mode.
    ///
    /// # Returns
    ///
    /// `Some` index if in network select mode.
    #[must_use]
    pub const fn as_network_select(&self) -> Option<usize> {
        match self {
            Self::NetworkSelect(index) => Some(*index),
            _ => None,
        }
    }

    /// Returns the message if displaying a message popup.
    ///
    /// # Returns
    ///
    /// `Some` message string if in message mode.
    #[must_use]
    pub fn as_message(&self) -> Option<&str> {
        match self {
            Self::Message(msg) => Some(msg.as_str()),
            _ => None,
        }
    }
}

// ============================================================================
// UI State
// ============================================================================

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

    /// Sets focus to a specific panel.
    ///
    /// # Arguments
    ///
    /// * `focus` - The panel to focus
    pub fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
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

    /// Opens the search popup with default settings.
    pub fn open_search(&mut self) {
        self.popup_state = PopupState::SearchWithType(String::new(), SearchType::Transaction);
    }

    /// Opens the network selection popup with the given current index.
    ///
    /// # Arguments
    ///
    /// * `current_index` - The index of the currently selected network
    pub fn open_network_select(&mut self, current_index: usize) {
        self.popup_state = PopupState::NetworkSelect(current_index);
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

    /// Sets search results popup.
    ///
    /// # Arguments
    ///
    /// * `results` - The search results to display
    pub fn show_search_results(
        &mut self,
        results: Vec<(usize, crate::algorand::SearchResultItem)>,
    ) {
        self.popup_state = PopupState::SearchResults(results);
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

    /// Returns `true` if a toast is currently displayed.
    #[must_use]
    pub fn has_toast(&self) -> bool {
        self.toast.is_some()
    }

    /// Gets the current toast message.
    ///
    /// # Returns
    ///
    /// The toast message if one is displayed.
    #[must_use]
    pub fn toast_message(&self) -> Option<&str> {
        self.toast.as_ref().map(|(msg, _)| msg.as_str())
    }

    // ========================================================================
    // Detail View Mode
    // ========================================================================

    /// Toggles the detail view mode between Visual and Table.
    pub fn toggle_detail_view_mode(&mut self) {
        self.detail_view_mode = self.detail_view_mode.toggle();
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

    /// Returns whether a section is expanded.
    ///
    /// # Arguments
    ///
    /// * `section_name` - The name of the section to check
    ///
    /// # Returns
    ///
    /// `true` if the section is expanded.
    #[must_use]
    pub fn is_section_expanded(&self, section_name: &str) -> bool {
        self.expanded_sections.contains(section_name)
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

    /// Moves the section selection up.
    ///
    /// # Arguments
    ///
    /// * `section_count` - Total number of expandable sections
    pub fn move_section_up(&mut self, section_count: usize) {
        if section_count == 0 {
            return;
        }

        if let Some(idx) = self.detail_section_index {
            if idx > 0 {
                self.detail_section_index = Some(idx - 1);
            }
        } else {
            // Start from the last section when pressing up with no selection
            self.detail_section_index = Some(section_count - 1);
        }
    }

    /// Moves the section selection down.
    ///
    /// # Arguments
    ///
    /// * `section_count` - Total number of expandable sections
    pub fn move_section_down(&mut self, section_count: usize) {
        if section_count == 0 {
            return;
        }

        if let Some(idx) = self.detail_section_index {
            if idx < section_count - 1 {
                self.detail_section_index = Some(idx + 1);
            }
        } else {
            self.detail_section_index = Some(0);
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod focus_tests {
        use super::*;

        #[test]
        fn test_default_is_blocks() {
            assert_eq!(Focus::default(), Focus::Blocks);
        }

        #[test]
        fn test_next_cycles() {
            assert_eq!(Focus::Blocks.next(), Focus::Transactions);
            assert_eq!(Focus::Transactions.next(), Focus::Blocks);
        }

        #[test]
        fn test_name() {
            assert_eq!(Focus::Blocks.name(), "Blocks");
            assert_eq!(Focus::Transactions.name(), "Transactions");
        }
    }

    mod search_type_tests {
        use super::*;

        #[test]
        fn test_default_is_transaction() {
            assert_eq!(SearchType::default(), SearchType::Transaction);
        }

        #[test]
        fn test_as_str() {
            assert_eq!(SearchType::Transaction.as_str(), "Transaction");
            assert_eq!(SearchType::Asset.as_str(), "Asset");
            assert_eq!(SearchType::Account.as_str(), "Account");
            assert_eq!(SearchType::Block.as_str(), "Block");
        }

        #[test]
        fn test_next_cycles() {
            assert_eq!(SearchType::Transaction.next(), SearchType::Block);
            assert_eq!(SearchType::Block.next(), SearchType::Account);
            assert_eq!(SearchType::Account.next(), SearchType::Asset);
            assert_eq!(SearchType::Asset.next(), SearchType::Transaction);
        }

        #[test]
        fn test_all() {
            let all = SearchType::all();
            assert_eq!(all.len(), 4);
            assert_eq!(all[0], SearchType::Transaction);
            assert_eq!(all[1], SearchType::Block);
            assert_eq!(all[2], SearchType::Account);
            assert_eq!(all[3], SearchType::Asset);
        }
    }

    mod popup_state_tests {
        use super::*;

        #[test]
        fn test_default_is_none() {
            assert_eq!(PopupState::default(), PopupState::None);
        }

        #[test]
        fn test_is_active() {
            assert!(!PopupState::None.is_active());
            assert!(PopupState::NetworkSelect(0).is_active());
            assert!(PopupState::SearchWithType(String::new(), SearchType::Transaction).is_active());
            assert!(PopupState::Message("test".to_string()).is_active());
        }

        #[test]
        fn test_as_search() {
            let popup = PopupState::SearchWithType("query".to_string(), SearchType::Account);
            let (query, search_type) = popup.as_search().unwrap();
            assert_eq!(query, "query");
            assert_eq!(search_type, SearchType::Account);

            assert!(PopupState::None.as_search().is_none());
        }

        #[test]
        fn test_as_network_select() {
            let popup = PopupState::NetworkSelect(2);
            assert_eq!(popup.as_network_select(), Some(2));
            assert!(PopupState::None.as_network_select().is_none());
        }

        #[test]
        fn test_as_message() {
            let popup = PopupState::Message("Hello".to_string());
            assert_eq!(popup.as_message(), Some("Hello"));
            assert!(PopupState::None.as_message().is_none());
        }
    }

    mod ui_state_tests {
        use super::*;

        #[test]
        fn test_new_creates_default() {
            let ui = UiState::new();
            assert_eq!(ui.focus, Focus::Blocks);
            assert!(!ui.has_active_popup());
            assert!(!ui.viewing_search_result);
            assert!(!ui.has_toast());
        }

        #[test]
        fn test_cycle_focus() {
            let mut ui = UiState::new();
            assert_eq!(ui.focus, Focus::Blocks);

            ui.cycle_focus();
            assert_eq!(ui.focus, Focus::Transactions);

            ui.cycle_focus();
            assert_eq!(ui.focus, Focus::Blocks);
        }

        #[test]
        fn test_show_and_dismiss_message() {
            let mut ui = UiState::new();
            ui.show_message("Test message");
            assert!(ui.has_active_popup());

            ui.dismiss_popup();
            assert!(!ui.has_active_popup());
        }

        #[test]
        fn test_toast_lifecycle() {
            let mut ui = UiState::new();
            assert!(!ui.has_toast());

            ui.show_toast("Hello", 3);
            assert!(ui.has_toast());
            assert_eq!(ui.toast_message(), Some("Hello"));

            // Tick twice (3 -> 2 -> 1)
            assert!(!ui.tick_toast()); // Still has time
            assert!(!ui.tick_toast()); // Still has time

            // Third tick removes the toast
            assert!(ui.tick_toast());
            assert!(!ui.has_toast());
        }

        #[test]
        fn test_open_search() {
            let mut ui = UiState::new();
            ui.open_search();

            let (query, search_type) = ui.popup_state.as_search().unwrap();
            assert_eq!(query, "");
            assert_eq!(search_type, SearchType::Transaction);
        }

        #[test]
        fn test_update_search_query() {
            let mut ui = UiState::new();
            ui.open_search();
            ui.update_search_query("test".to_string(), SearchType::Account);

            let (query, search_type) = ui.popup_state.as_search().unwrap();
            assert_eq!(query, "test");
            assert_eq!(search_type, SearchType::Account);
        }

        #[test]
        fn test_cycle_search_type() {
            let mut ui = UiState::new();
            ui.open_search();
            ui.update_search_query("query".to_string(), SearchType::Transaction);
            ui.cycle_search_type("query".to_string(), SearchType::Transaction);

            let (_, search_type) = ui.popup_state.as_search().unwrap();
            assert_eq!(search_type, SearchType::Block);
        }

        #[test]
        fn test_toggle_section() {
            let mut ui = UiState::new();
            assert!(!ui.is_section_expanded("test"));

            ui.toggle_section("test");
            assert!(ui.is_section_expanded("test"));

            ui.toggle_section("test");
            assert!(!ui.is_section_expanded("test"));
        }

        #[test]
        fn test_reset_expanded_sections() {
            let mut ui = UiState::new();
            ui.toggle_section("test1");
            ui.toggle_section("test2");
            ui.detail_section_index = Some(1);
            ui.detail_fullscreen = true;

            ui.reset_expanded_sections();

            assert!(!ui.is_section_expanded("test1"));
            assert!(!ui.is_section_expanded("test2"));
            assert!(ui.detail_section_index.is_none());
            assert!(!ui.detail_fullscreen);
        }

        #[test]
        fn test_move_section_up() {
            let mut ui = UiState::new();

            // Empty sections - no change
            ui.move_section_up(0);
            assert!(ui.detail_section_index.is_none());

            // Start from end when pressing up with no selection
            ui.move_section_up(3);
            assert_eq!(ui.detail_section_index, Some(2));

            // Move up from middle
            ui.move_section_up(3);
            assert_eq!(ui.detail_section_index, Some(1));

            // Stop at beginning
            ui.detail_section_index = Some(0);
            ui.move_section_up(3);
            assert_eq!(ui.detail_section_index, Some(0));
        }

        #[test]
        fn test_move_section_down() {
            let mut ui = UiState::new();

            // Empty sections - no change
            ui.move_section_down(0);
            assert!(ui.detail_section_index.is_none());

            // Start from beginning when pressing down with no selection
            ui.move_section_down(3);
            assert_eq!(ui.detail_section_index, Some(0));

            // Move down
            ui.move_section_down(3);
            assert_eq!(ui.detail_section_index, Some(1));

            // Stop at end
            ui.detail_section_index = Some(2);
            ui.move_section_down(3);
            assert_eq!(ui.detail_section_index, Some(2));
        }
    }
}
