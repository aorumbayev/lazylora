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
    #[allow(dead_code)] // Part of UI state API
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
    #[allow(dead_code)] // Part of UI state API
    pub const fn all() -> [Self; 4] {
        [Self::Transaction, Self::Block, Self::Account, Self::Asset]
    }
}

// ============================================================================
// Search Type Auto-Detection
// ============================================================================

/// Auto-detect the search type based on input pattern.
///
/// Uses the following heuristics:
/// - 52-char uppercase alphanumeric → Transaction ID
/// - Pure digits → Block number (small) or Asset ID (large)
/// - 58-char uppercase alphanumeric → Account address
/// - Contains ".algo" or looks like NFD name → Account (NFD)
/// - Otherwise → None (unknown format)
#[must_use]
pub fn detect_search_type(query: &str) -> Option<SearchType> {
    let trimmed = query.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Check for transaction ID (52 chars, uppercase alphanumeric)
    if trimmed.len() == 52
        && trimmed
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Some(SearchType::Transaction);
    }

    // Check for valid Algorand address (58 chars, uppercase alphanumeric)
    if trimmed.len() == 58
        && trimmed
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Some(SearchType::Account);
    }

    // Check for NFD name (contains .algo or looks like name)
    if looks_like_nfd_name(trimmed) {
        return Some(SearchType::Account);
    }

    // Check for pure integer (block or asset)
    if let Ok(num) = trimmed.parse::<u64>() {
        // Use heuristic: blocks are typically < 100M, assets can be much larger
        // But really, both are valid - we'll default to Block for simplicity
        // since blocks are more commonly searched by number
        if num < 100_000_000 {
            return Some(SearchType::Block);
        }
        return Some(SearchType::Asset);
    }

    // Partial transaction ID (40-60 chars, mostly uppercase)
    if (40..=60).contains(&trimmed.len())
        && trimmed.chars().filter(|c| c.is_ascii_uppercase()).count() > trimmed.len() / 2
    {
        return Some(SearchType::Transaction);
    }

    None
}

/// Check if a query string looks like an NFD name.
#[must_use]
fn looks_like_nfd_name(query: &str) -> bool {
    let trimmed = query.trim().to_lowercase();

    if trimmed.is_empty() {
        return false;
    }

    // If it ends with .algo, it's definitely an NFD name
    if let Some(name_part) = trimmed.strip_suffix(".algo") {
        return !name_part.is_empty()
            && name_part
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
    }

    // Could be just the name without .algo suffix
    // It's likely an NFD if it's a short alphanumeric string that isn't a number
    trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && trimmed.parse::<u64>().is_err()
        && trimmed.len() < 30  // NFD names are typically short
        && trimmed.len() >= 2 // At least 2 chars for a name
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
    SearchResults(Vec<(usize, crate::domain::SearchResultItem)>),
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
    #[allow(dead_code)] // Part of UI state API
    pub fn as_search_results(&self) -> Option<&[(usize, crate::domain::SearchResultItem)]> {
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
    #[allow(dead_code)] // Part of UI state API
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
    #[allow(dead_code)] // Part of UI state API
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
    #[allow(dead_code)] // Part of UI state API
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
    pub fn show_search_results(&mut self, results: Vec<(usize, crate::domain::SearchResultItem)>) {
        self.popup_state = PopupState::SearchResults(results);
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

    /// Gets a hint text for the current search input.
    #[must_use]
    #[allow(dead_code)] // Part of UI state API
    pub fn search_hint(&self) -> &'static str {
        match self.detected_search_type {
            Some(SearchType::Transaction) => "Transaction ID",
            Some(SearchType::Block) => "Block number",
            Some(SearchType::Account) => "Account/NFD",
            Some(SearchType::Asset) => "Asset ID",
            None if self.search_input.is_empty() => "Search by ID or Address",
            None => "Unknown format",
        }
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

    /// Returns `true` if a toast is currently displayed.
    #[must_use]
    #[allow(dead_code)] // Part of UI state API
    pub fn has_toast(&self) -> bool {
        self.toast.is_some()
    }

    /// Gets the current toast message.
    ///
    /// # Returns
    ///
    /// The toast message if one is displayed.
    #[must_use]
    #[allow(dead_code)] // Part of UI state API
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
    #[allow(dead_code)] // Part of UI state API
    pub fn toggle_fullscreen(&mut self) {
        self.detail_fullscreen = !self.detail_fullscreen;
    }

    /// Moves the section selection up.
    ///
    /// # Arguments
    ///
    /// * `section_count` - Total number of expandable sections
    #[allow(dead_code)] // Part of UI state API
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
    #[allow(dead_code)] // Part of UI state API
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

    #[test]
    fn test_focus_cycle_behavior() {
        // Default is Blocks, cycles through both values
        assert_eq!(Focus::default(), Focus::Blocks);
        assert_eq!(Focus::Blocks.next(), Focus::Transactions);
        assert_eq!(Focus::Transactions.next(), Focus::Blocks);

        // Names are correct
        assert_eq!(Focus::Blocks.name(), "Blocks");
        assert_eq!(Focus::Transactions.name(), "Transactions");
    }

    #[test]
    fn test_search_type_cycle_behavior() {
        // Default is Transaction, cycles through all 4 types
        let mut current = SearchType::default();
        assert_eq!(current, SearchType::Transaction);

        let expected_cycle = [
            SearchType::Block,
            SearchType::Account,
            SearchType::Asset,
            SearchType::Transaction, // Back to start
        ];

        for expected in expected_cycle {
            current = current.next();
            assert_eq!(current, expected);
        }

        // All types have string representations
        for search_type in SearchType::all() {
            assert!(!search_type.as_str().is_empty());
        }
    }

    #[test]
    fn test_popup_state_variants() {
        // None is inactive, all others are active
        assert!(!PopupState::None.is_active());
        assert!(PopupState::NetworkSelect(0).is_active());
        assert!(PopupState::SearchWithType(String::new(), SearchType::Transaction).is_active());
        assert!(PopupState::Message("test".to_string()).is_active());

        // Accessors return correct values
        let search = PopupState::SearchWithType("query".to_string(), SearchType::Account);
        let (q, t) = search.as_search().unwrap();
        assert_eq!(q, "query");
        assert_eq!(t, SearchType::Account);

        assert_eq!(PopupState::NetworkSelect(5).as_network_select(), Some(5));
        assert_eq!(
            PopupState::Message("Hello".to_string()).as_message(),
            Some("Hello")
        );
    }

    #[test]
    fn test_ui_state_focus_management() {
        let mut ui = UiState::new();
        assert_eq!(ui.focus, Focus::Blocks);

        ui.cycle_focus();
        assert_eq!(ui.focus, Focus::Transactions);

        ui.set_focus(Focus::Blocks);
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
        ui.open_search();
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
        assert!(!ui.has_toast());

        ui.show_toast("Hello", 2);
        assert!(ui.has_toast());
        assert_eq!(ui.toast_message(), Some("Hello"));

        assert!(!ui.tick_toast()); // 2 -> 1
        assert!(ui.tick_toast()); // 1 -> removed
        assert!(!ui.has_toast());
    }

    #[test]
    fn test_ui_state_expandable_sections() {
        let mut ui = UiState::new();

        // Toggle section on and off
        assert!(!ui.is_section_expanded("test"));
        ui.toggle_section("test");
        assert!(ui.is_section_expanded("test"));
        ui.toggle_section("test");
        assert!(!ui.is_section_expanded("test"));

        // Reset clears everything
        ui.toggle_section("a");
        ui.toggle_section("b");
        ui.detail_section_index = Some(1);
        ui.detail_fullscreen = true;

        ui.reset_expanded_sections();
        assert!(!ui.is_section_expanded("a"));
        assert!(!ui.is_section_expanded("b"));
        assert!(ui.detail_section_index.is_none());
        assert!(!ui.detail_fullscreen);
    }

    #[test]
    fn test_ui_state_section_navigation() {
        let mut ui = UiState::new();

        // No sections - no change
        ui.move_section_down(0);
        assert!(ui.detail_section_index.is_none());

        // Down from no selection goes to 0
        ui.move_section_down(3);
        assert_eq!(ui.detail_section_index, Some(0));

        // Down increments
        ui.move_section_down(3);
        assert_eq!(ui.detail_section_index, Some(1));

        // Up decrements
        ui.move_section_up(3);
        assert_eq!(ui.detail_section_index, Some(0));

        // Can't go below 0
        ui.move_section_up(3);
        assert_eq!(ui.detail_section_index, Some(0));

        // Can't exceed max
        ui.detail_section_index = Some(2);
        ui.move_section_down(3);
        assert_eq!(ui.detail_section_index, Some(2));
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
    fn test_detect_search_type_all_cases() {
        // Empty string
        assert_eq!(detect_search_type(""), None);
        assert_eq!(detect_search_type("   "), None);

        // Transaction ID (52 chars)
        let txn_id = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        assert_eq!(txn_id.len(), 52);
        assert_eq!(detect_search_type(txn_id), Some(SearchType::Transaction));

        // Account address (58 chars)
        let address = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        assert_eq!(address.len(), 58);
        assert_eq!(detect_search_type(address), Some(SearchType::Account));

        // Block number (small integer)
        assert_eq!(detect_search_type("12345"), Some(SearchType::Block));
        assert_eq!(detect_search_type("1000000"), Some(SearchType::Block));

        // Asset ID (large integer)
        assert_eq!(detect_search_type("100000000"), Some(SearchType::Asset));
        assert_eq!(detect_search_type("999999999"), Some(SearchType::Asset));

        // NFD name
        assert_eq!(detect_search_type("alice.algo"), Some(SearchType::Account));
        assert_eq!(detect_search_type("bob"), Some(SearchType::Account));
        assert_eq!(detect_search_type("my-nfd"), Some(SearchType::Account));
    }

    #[test]
    fn test_search_hint_texts() {
        let mut ui = UiState::new();

        // Empty - show placeholder
        assert_eq!(ui.search_hint(), "Search by ID or Address");

        // Type a transaction ID
        for c in "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".chars() {
            ui.search_type_char(c);
        }
        assert_eq!(ui.search_hint(), "Transaction ID");

        // Clear and type block number
        ui.clear_search();
        ui.search_type_char('1');
        ui.search_type_char('2');
        ui.search_type_char('3');
        assert_eq!(ui.search_hint(), "Block number");

        // Clear and type NFD
        ui.clear_search();
        for c in "alice.algo".chars() {
            ui.search_type_char(c);
        }
        assert_eq!(ui.search_hint(), "Account/NFD");
    }
}
