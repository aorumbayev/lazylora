//! State management for list widgets.
//!
//! Provides state types for tracking selection and scroll position in list widgets.

#![allow(dead_code)] // Transitional phase - items will be used after integration

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols::scrollbar,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};

// ============================================================================
// BlockListState
// ============================================================================

/// State for the block list widget.
///
/// This state tracks the currently selected block index and scroll position,
/// allowing the widget to maintain its state across renders.
///
/// # Example
///
/// ```ignore
/// use crate::widgets::list::BlockListState;
///
/// let mut state = BlockListState::new();
/// state.select(Some(0)); // Select first block
/// state.scroll_position = 3; // Scroll down by 3 rows
/// ```
#[derive(Debug, Default, Clone)]
pub struct BlockListState {
    /// Currently selected block index in the list.
    pub selected_index: Option<usize>,
    /// Scroll position (in pixels/rows).
    pub scroll_position: u16,
}

impl BlockListState {
    /// Creates a new `BlockListState` with no selection.
    ///
    /// # Returns
    ///
    /// A new `BlockListState` with default values
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected_index: None,
            scroll_position: 0,
        }
    }

    /// Creates a new `BlockListState` with the given selection.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to select initially
    ///
    /// # Returns
    ///
    /// A new `BlockListState` with the specified selection
    #[must_use]
    pub const fn with_selection(index: usize) -> Self {
        Self {
            selected_index: Some(index),
            scroll_position: 0,
        }
    }

    /// Sets the selected index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to select, or `None` to clear selection
    pub fn select(&mut self, index: Option<usize>) {
        self.selected_index = index;
    }

    /// Returns the currently selected index.
    ///
    /// # Returns
    ///
    /// The currently selected index, or `None` if nothing is selected
    #[must_use]
    pub const fn selected(&self) -> Option<usize> {
        self.selected_index
    }
}

// ============================================================================
// TransactionListState
// ============================================================================

/// State for the transaction list widget.
///
/// This state tracks the currently selected transaction index and scroll position,
/// allowing the widget to maintain its state across renders.
///
/// # Example
///
/// ```ignore
/// use crate::widgets::list::TransactionListState;
///
/// let mut state = TransactionListState::new();
/// state.select(Some(2)); // Select third transaction
/// state.scroll_position = 8; // Scroll down by 8 rows
/// ```
#[derive(Debug, Default, Clone)]
pub struct TransactionListState {
    /// Currently selected transaction index in the list.
    pub selected_index: Option<usize>,
    /// Scroll position (in pixels/rows).
    pub scroll_position: u16,
}

impl TransactionListState {
    /// Creates a new `TransactionListState` with no selection.
    ///
    /// # Returns
    ///
    /// A new `TransactionListState` with default values
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected_index: None,
            scroll_position: 0,
        }
    }

    /// Creates a new `TransactionListState` with the given selection.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to select initially
    ///
    /// # Returns
    ///
    /// A new `TransactionListState` with the specified selection
    #[must_use]
    pub const fn with_selection(index: usize) -> Self {
        Self {
            selected_index: Some(index),
            scroll_position: 0,
        }
    }

    /// Sets the selected index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to select, or `None` to clear selection
    pub fn select(&mut self, index: Option<usize>) {
        self.selected_index = index;
    }

    /// Returns the currently selected index.
    ///
    /// # Returns
    ///
    /// The currently selected index, or `None` if nothing is selected
    #[must_use]
    pub const fn selected(&self) -> Option<usize> {
        self.selected_index
    }
}

// ============================================================================
// Scrollbar Helper
// ============================================================================

/// Renders a scrollbar for a list widget.
///
/// This helper function renders a vertical scrollbar on the right side of the
/// given area, properly sized based on the content length and viewport.
///
/// # Arguments
///
/// * `area` - The area to render the scrollbar in
/// * `buf` - The buffer to render to
/// * `total_items` - Total number of items in the list
/// * `item_height` - Height of each item in rows
/// * `items_per_page` - Number of items visible per page
/// * `scroll_position` - Current scroll position
pub fn render_list_scrollbar(
    area: Rect,
    buf: &mut Buffer,
    total_items: usize,
    item_height: usize,
    items_per_page: usize,
    scroll_position: usize,
) {
    if total_items <= items_per_page {
        return;
    }

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(scrollbar::VERTICAL)
        .track_symbol(None)
        .begin_symbol(None)
        .end_symbol(None)
        .style(Style::default().fg(Color::Gray))
        .track_style(Style::default().fg(Color::DarkGray));

    let content_length = total_items * item_height;

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(content_length)
        .viewport_content_length(items_per_page * item_height)
        .position(scroll_position);

    scrollbar.render(area, buf, &mut scrollbar_state);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // BlockListState Tests
    #[test]
    fn test_block_list_state_new() {
        let state = BlockListState::new();
        assert!(state.selected_index.is_none());
        assert_eq!(state.scroll_position, 0);
    }

    #[test]
    fn test_block_list_state_with_selection() {
        let state = BlockListState::with_selection(2);
        assert_eq!(state.selected_index, Some(2));
        assert_eq!(state.scroll_position, 0);
    }

    #[test]
    fn test_block_list_state_select() {
        let mut state = BlockListState::new();
        state.select(Some(5));
        assert_eq!(state.selected(), Some(5));

        state.select(None);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn test_block_list_state_updates() {
        let mut state = BlockListState::new();

        // Initial state
        assert!(state.selected().is_none());

        // Select first item
        state.select(Some(0));
        assert_eq!(state.selected(), Some(0));

        // Update scroll position
        state.scroll_position = 6;
        assert_eq!(state.scroll_position, 6);

        // Select different item
        state.select(Some(2));
        assert_eq!(state.selected(), Some(2));

        // Clear selection
        state.select(None);
        assert!(state.selected().is_none());
    }

    // TransactionListState Tests
    #[test]
    fn test_transaction_list_state_new() {
        let state = TransactionListState::new();
        assert!(state.selected_index.is_none());
        assert_eq!(state.scroll_position, 0);
    }

    #[test]
    fn test_transaction_list_state_with_selection() {
        let state = TransactionListState::with_selection(1);
        assert_eq!(state.selected_index, Some(1));
        assert_eq!(state.scroll_position, 0);
    }

    #[test]
    fn test_transaction_list_state_select() {
        let mut state = TransactionListState::new();
        state.select(Some(3));
        assert_eq!(state.selected(), Some(3));

        state.select(None);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn test_transaction_list_state_updates() {
        let mut state = TransactionListState::new();

        // Initial state
        assert!(state.selected().is_none());

        // Select first item
        state.select(Some(0));
        assert_eq!(state.selected(), Some(0));

        // Update scroll position
        state.scroll_position = 8;
        assert_eq!(state.scroll_position, 8);

        // Select different item
        state.select(Some(5));
        assert_eq!(state.selected(), Some(5));

        // Clear selection
        state.select(None);
        assert!(state.selected().is_none());
    }
}
