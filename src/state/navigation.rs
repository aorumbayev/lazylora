//! Navigation state management for the LazyLora TUI.
//!
//! This module handles all UI navigation concerns including:
//! - Selection indices for blocks and transactions
//! - Scroll positions for scrollable lists
//! - Detail view state (which popup is shown)
//! - Graph view scroll positions
//!
//! # Design
//!
//! The navigation state is decoupled from the actual data it navigates.
//! It maintains indices and IDs that can be synchronized with the data state.

use crate::domain::{AlgoBlock, Transaction};

// ============================================================================
// Detail View Mode
// ============================================================================

/// The view mode for transaction/block details popup.
///
/// Controls how transaction details are displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailViewMode {
    /// Table view showing structured key-value pairs.
    #[default]
    Table,
    /// Visual/graph view showing transaction flow.
    Visual,
}

impl DetailViewMode {
    /// Toggles between Visual and Table modes.
    ///
    /// # Returns
    ///
    /// The opposite view mode.
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Table => Self::Visual,
            Self::Visual => Self::Table,
        }
    }

    /// Returns `true` if in visual/graph mode.
    #[must_use]
    #[allow(dead_code)] // Part of navigation API
    pub const fn is_visual(self) -> bool {
        matches!(self, Self::Visual)
    }

    /// Returns `true` if in table mode.
    #[must_use]
    #[allow(dead_code)] // Part of navigation API
    pub const fn is_table(self) -> bool {
        matches!(self, Self::Table)
    }
}

// ============================================================================
// Block Detail Tab
// ============================================================================

/// The tab in the block details popup.
///
/// Block details can show either general information or a list of transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockDetailTab {
    /// General block information (timestamp, proposer, etc.).
    #[default]
    Info,
    /// List of transactions in the block.
    Transactions,
}

impl BlockDetailTab {
    /// Cycles to the next tab.
    ///
    /// # Returns
    ///
    /// The next tab in the cycle.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Info => Self::Transactions,
            Self::Transactions => Self::Info,
        }
    }

    /// Returns `true` if showing the info tab.
    #[must_use]
    #[allow(dead_code)] // Part of navigation API
    pub const fn is_info(self) -> bool {
        matches!(self, Self::Info)
    }

    /// Returns `true` if showing the transactions tab.
    #[must_use]
    #[allow(dead_code)] // Part of navigation API
    pub const fn is_transactions(self) -> bool {
        matches!(self, Self::Transactions)
    }
}

// ============================================================================
// Account Detail Tab
// ============================================================================

/// The tab in the account details popup.
///
/// Account details can show general info, asset holdings, or application opt-ins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccountDetailTab {
    /// General account information (balance, status, etc.).
    #[default]
    Info,
    /// Asset holdings and created assets.
    Assets,
    /// Application opt-ins and created apps.
    Apps,
}

impl AccountDetailTab {
    /// Cycles to the next tab.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Info => Self::Assets,
            Self::Assets => Self::Apps,
            Self::Apps => Self::Info,
        }
    }

    /// Cycles to the previous tab.
    #[must_use]
    pub const fn prev(self) -> Self {
        match self {
            Self::Info => Self::Apps,
            Self::Assets => Self::Info,
            Self::Apps => Self::Assets,
        }
    }

    /// Returns `true` if showing the info tab.
    #[must_use]
    #[allow(dead_code)] // Part of navigation API
    pub const fn is_info(self) -> bool {
        matches!(self, Self::Info)
    }

    /// Returns `true` if showing the assets tab.
    #[must_use]
    #[allow(dead_code)] // Part of navigation API
    pub const fn is_assets(self) -> bool {
        matches!(self, Self::Assets)
    }

    /// Returns `true` if showing the apps tab.
    #[must_use]
    #[allow(dead_code)] // Part of navigation API
    pub const fn is_apps(self) -> bool {
        matches!(self, Self::Apps)
    }
}

// ============================================================================
// Navigation State
// ============================================================================

/// Navigation state: selection indices, scroll positions, and detail view flags.
///
/// This struct manages all UI navigation concerns, keeping track of what's
/// selected, how far lists are scrolled, and which detail views are open.
///
/// # Example
///
/// ```ignore
/// use crate::state::NavigationState;
///
/// let mut nav = NavigationState::new();
///
/// // Select a block
/// nav.selected_block_index = Some(0);
/// nav.selected_block_id = Some(12345);
///
/// // Open block details
/// nav.show_block_details = true;
/// ```
#[derive(Debug, Default)]
pub struct NavigationState {
    // === List Scroll Positions ===
    /// Scroll position for blocks list (in rows).
    pub block_scroll: u16,
    /// Scroll position for transactions list (in rows).
    pub transaction_scroll: u16,

    // === Selection State ===
    /// Currently selected block index in the blocks list.
    pub selected_block_index: Option<usize>,
    /// Currently selected transaction index in the transactions list.
    pub selected_transaction_index: Option<usize>,
    /// The block ID of the currently selected block (for stable selection across updates).
    pub selected_block_id: Option<u64>,
    /// The transaction ID of the currently selected transaction (for stable selection).
    pub selected_transaction_id: Option<String>,

    // === Detail View Flags ===
    /// Whether the block details popup is shown.
    pub show_block_details: bool,
    /// Whether the transaction details popup is shown.
    pub show_transaction_details: bool,
    /// Whether the account details popup is shown.
    pub show_account_details: bool,
    /// Whether the asset details popup is shown.
    pub show_asset_details: bool,

    // === Block Detail View State ===
    /// Current tab in block details popup.
    pub block_detail_tab: BlockDetailTab,
    /// Selected transaction index within block details.
    pub block_txn_index: Option<usize>,
    /// Scroll position for block transactions list.
    pub block_txn_scroll: u16,

    // === Account Detail View State ===
    /// Current tab in account details popup.
    pub account_detail_tab: AccountDetailTab,
    /// Selected item index within account details list (assets or apps).
    pub account_item_index: Option<usize>,
    /// Scroll position for account details list.
    pub account_item_scroll: u16,

    // === Graph View State ===
    /// Horizontal scroll offset for transaction graph view.
    pub graph_scroll_x: u16,
    /// Vertical scroll offset for transaction graph view.
    pub graph_scroll_y: u16,
    /// Maximum horizontal scroll offset for transaction graph (computed from content).
    pub graph_max_scroll_x: u16,
    /// Maximum vertical scroll offset for transaction graph (computed from content).
    pub graph_max_scroll_y: u16,
}

impl NavigationState {
    /// Creates a new `NavigationState` with default values.
    ///
    /// # Returns
    ///
    /// A new navigation state with no selections and zero scroll positions.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets all navigation state (useful when switching networks).
    ///
    /// This clears all selections, scroll positions, and closes all detail views.
    pub fn reset(&mut self) {
        self.block_scroll = 0;
        self.transaction_scroll = 0;
        self.selected_block_index = None;
        self.selected_transaction_index = None;
        self.selected_block_id = None;
        self.selected_transaction_id = None;
        self.show_block_details = false;
        self.show_transaction_details = false;
        self.show_account_details = false;
        self.show_asset_details = false;
        self.block_detail_tab = BlockDetailTab::default();
        self.block_txn_index = None;
        self.block_txn_scroll = 0;
        self.account_detail_tab = AccountDetailTab::default();
        self.account_item_index = None;
        self.account_item_scroll = 0;
        self.graph_scroll_x = 0;
        self.graph_scroll_y = 0;
        self.graph_max_scroll_x = 0;
        self.graph_max_scroll_y = 0;
    }

    // ========================================================================
    // Detail View Management
    // ========================================================================

    /// Returns `true` if any detail view is currently shown.
    ///
    /// # Returns
    ///
    /// `true` if any of the detail popups are open.
    #[must_use]
    pub const fn is_showing_details(&self) -> bool {
        self.show_block_details
            || self.show_transaction_details
            || self.show_account_details
            || self.show_asset_details
    }

    /// Closes all detail views.
    pub fn close_details(&mut self) {
        self.show_block_details = false;
        self.show_transaction_details = false;
        self.show_account_details = false;
        self.show_asset_details = false;
    }

    /// Opens the block details view.
    #[allow(dead_code)] // Part of navigation API
    pub fn open_block_details(&mut self) {
        self.show_block_details = true;
        self.block_detail_tab = BlockDetailTab::default();
        self.block_txn_index = None;
        self.block_txn_scroll = 0;
    }

    /// Opens the transaction details view.
    #[allow(dead_code)] // Part of navigation API
    pub fn open_transaction_details(&mut self) {
        self.show_transaction_details = true;
        self.reset_graph_scroll();
    }

    /// Resets graph scroll position and bounds.
    #[allow(dead_code)] // Part of navigation API
    pub fn reset_graph_scroll(&mut self) {
        self.graph_scroll_x = 0;
        self.graph_scroll_y = 0;
        self.graph_max_scroll_x = 0;
        self.graph_max_scroll_y = 0;
    }

    // ========================================================================
    // Block Selection
    // ========================================================================

    /// Selects a block by index and updates the stable ID.
    ///
    /// # Arguments
    ///
    /// * `index` - The index in the blocks list
    /// * `blocks` - The blocks slice to extract the ID from
    pub fn select_block(&mut self, index: usize, blocks: &[AlgoBlock]) {
        self.selected_block_index = Some(index);
        self.selected_block_id = blocks.get(index).map(|b| b.id);
    }

    /// Clears block selection.
    pub fn clear_block_selection(&mut self) {
        self.selected_block_index = None;
        self.selected_block_id = None;
    }

    /// Returns `true` if a block is selected.
    #[must_use]
    #[allow(dead_code)] // Part of navigation API
    pub const fn has_block_selection(&self) -> bool {
        self.selected_block_index.is_some()
    }

    // ========================================================================
    // Transaction Selection
    // ========================================================================

    /// Selects a transaction by index and updates the stable ID.
    ///
    /// # Arguments
    ///
    /// * `index` - The index in the transactions list
    /// * `transactions` - The transactions slice to extract the ID from
    pub fn select_transaction(&mut self, index: usize, transactions: &[Transaction]) {
        self.selected_transaction_index = Some(index);
        self.selected_transaction_id = transactions.get(index).map(|t| t.id.clone());
    }

    /// Clears transaction selection.
    pub fn clear_transaction_selection(&mut self) {
        self.selected_transaction_index = None;
        self.selected_transaction_id = None;
    }

    /// Returns `true` if a transaction is selected.
    #[must_use]
    #[allow(dead_code)] // Part of navigation API
    pub const fn has_transaction_selection(&self) -> bool {
        self.selected_transaction_index.is_some()
    }

    // ========================================================================
    // Block Detail Navigation
    // ========================================================================

    /// Cycles the block detail tab between Info and Transactions.
    pub fn cycle_block_detail_tab(&mut self) {
        self.block_detail_tab = self.block_detail_tab.next();
    }

    /// Selects a transaction within the block details view.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the transaction in the block's transaction list
    #[allow(dead_code)] // Part of navigation API
    pub fn select_block_txn(&mut self, index: usize) {
        self.block_txn_index = Some(index);
    }

    /// Moves the block transaction selection up.
    pub fn move_block_txn_up(&mut self) {
        if let Some(idx) = self.block_txn_index
            && idx > 0
        {
            self.block_txn_index = Some(idx - 1);
            // Adjust scroll if needed (each txn item is 2 lines)
            let item_height: u16 = 2;
            let new_pos = (idx - 1) as u16 * item_height;
            if new_pos < self.block_txn_scroll {
                self.block_txn_scroll = new_pos;
            }
        }
    }

    /// Moves the block transaction selection down.
    ///
    /// # Arguments
    ///
    /// * `max` - Maximum valid index (length - 1)
    /// * `visible_height` - Number of visible rows in the list area
    pub fn move_block_txn_down(&mut self, max: usize, visible_height: u16) {
        let item_height: u16 = 2;
        if let Some(idx) = self.block_txn_index {
            if idx < max {
                self.block_txn_index = Some(idx + 1);
                // Adjust scroll if needed
                let new_pos = (idx + 1) as u16 * item_height;
                let visible_end = self.block_txn_scroll + visible_height;
                if new_pos + item_height > visible_end {
                    self.block_txn_scroll = (new_pos + item_height).saturating_sub(visible_height);
                }
            }
        } else if max > 0 {
            self.block_txn_index = Some(0);
            self.block_txn_scroll = 0;
        }
    }

    // ========================================================================
    // Account Detail Navigation
    // ========================================================================

    /// Cycles the account detail tab to the next tab.
    pub fn cycle_account_detail_tab(&mut self) {
        self.account_detail_tab = self.account_detail_tab.next();
        // Reset item selection when switching tabs
        self.account_item_index = None;
        self.account_item_scroll = 0;
    }

    /// Cycles the account detail tab to the previous tab.
    #[allow(dead_code)] // Part of navigation API
    pub fn cycle_account_detail_tab_prev(&mut self) {
        self.account_detail_tab = self.account_detail_tab.prev();
        // Reset item selection when switching tabs
        self.account_item_index = None;
        self.account_item_scroll = 0;
    }

    /// Moves the account item selection up.
    pub fn move_account_item_up(&mut self) {
        if let Some(idx) = self.account_item_index
            && idx > 0
        {
            self.account_item_index = Some(idx - 1);
            // Adjust scroll if needed (each item is 1 line)
            let new_pos = (idx - 1) as u16;
            if new_pos < self.account_item_scroll {
                self.account_item_scroll = new_pos;
            }
        }
    }

    /// Moves the account item selection down.
    ///
    /// # Arguments
    ///
    /// * `max` - Maximum valid index (length - 1)
    /// * `visible_height` - Number of visible rows in the list area
    pub fn move_account_item_down(&mut self, max: usize, visible_height: u16) {
        if let Some(idx) = self.account_item_index {
            if idx < max {
                self.account_item_index = Some(idx + 1);
                // Adjust scroll if needed
                let new_pos = (idx + 1) as u16;
                let visible_end = self.account_item_scroll + visible_height;
                if new_pos >= visible_end {
                    self.account_item_scroll = new_pos.saturating_sub(visible_height) + 1;
                }
            }
        } else if max > 0 {
            self.account_item_index = Some(0);
            self.account_item_scroll = 0;
        }
    }

    /// Resets account detail view state.
    pub fn reset_account_detail(&mut self) {
        self.account_detail_tab = AccountDetailTab::default();
        self.account_item_index = None;
        self.account_item_scroll = 0;
    }

    // ========================================================================
    // Graph Scrolling
    // ========================================================================

    /// Scrolls the graph view left by the specified amount.
    ///
    /// # Arguments
    ///
    /// * `amount` - Number of columns to scroll
    #[allow(dead_code)] // Part of navigation API
    pub fn scroll_graph_left(&mut self, amount: u16) {
        self.graph_scroll_x = self.graph_scroll_x.saturating_sub(amount);
    }

    /// Scrolls the graph view right by the specified amount.
    ///
    /// # Arguments
    ///
    /// * `amount` - Number of columns to scroll
    #[allow(dead_code)] // Part of navigation API
    pub fn scroll_graph_right(&mut self, amount: u16) {
        self.graph_scroll_x = self
            .graph_scroll_x
            .saturating_add(amount)
            .min(self.graph_max_scroll_x);
    }

    /// Scrolls the graph view up by the specified amount.
    ///
    /// # Arguments
    ///
    /// * `amount` - Number of rows to scroll
    #[allow(dead_code)] // Part of navigation API
    pub fn scroll_graph_up(&mut self, amount: u16) {
        self.graph_scroll_y = self.graph_scroll_y.saturating_sub(amount);
    }

    /// Scrolls the graph view down by the specified amount.
    ///
    /// # Arguments
    ///
    /// * `amount` - Number of rows to scroll
    #[allow(dead_code)] // Part of navigation API
    pub fn scroll_graph_down(&mut self, amount: u16) {
        self.graph_scroll_y = self
            .graph_scroll_y
            .saturating_add(amount)
            .min(self.graph_max_scroll_y);
    }

    /// Updates the maximum scroll bounds for the graph view.
    ///
    /// # Arguments
    ///
    /// * `max_x` - Maximum horizontal scroll offset
    /// * `max_y` - Maximum vertical scroll offset
    #[allow(dead_code)] // Part of navigation API
    pub fn set_graph_bounds(&mut self, max_x: u16, max_y: u16) {
        self.graph_max_scroll_x = max_x;
        self.graph_max_scroll_y = max_y;
        // Clamp current scroll to new bounds
        self.graph_scroll_x = self.graph_scroll_x.min(max_x);
        self.graph_scroll_y = self.graph_scroll_y.min(max_y);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TxnType;

    // ========================================================================
    // Helper Functions
    // ========================================================================

    fn create_test_block(id: u64) -> AlgoBlock {
        AlgoBlock {
            id,
            txn_count: 5,
            timestamp: "2024-01-01 12:00:00".to_string(),
        }
    }

    fn create_test_transaction(id: &str) -> Transaction {
        Transaction {
            id: id.to_string(),
            txn_type: TxnType::Payment,
            from: "sender".to_string(),
            to: "receiver".to_string(),
            timestamp: "2024-01-01 12:00:00".to_string(),
            block: 12345,
            fee: 1000,
            note: String::new(),
            amount: 1_000_000,
            asset_id: None,
            rekey_to: None,
            group: None,
            details: crate::domain::TransactionDetails::None,
            inner_transactions: Vec::new(),
        }
    }

    // ========================================================================
    // Behavior Tests
    // ========================================================================

    #[test]
    fn test_detail_view_mode_toggle_behavior() {
        // Table toggles to Visual, Visual toggles back to Table
        assert_eq!(DetailViewMode::default(), DetailViewMode::Table);
        assert_eq!(DetailViewMode::Table.toggle(), DetailViewMode::Visual);
        assert_eq!(DetailViewMode::Visual.toggle(), DetailViewMode::Table);

        // Predicates work correctly
        assert!(DetailViewMode::Table.is_table());
        assert!(!DetailViewMode::Table.is_visual());
        assert!(DetailViewMode::Visual.is_visual());
        assert!(!DetailViewMode::Visual.is_table());
    }

    #[test]
    fn test_block_detail_tab_cycle_behavior() {
        // Info cycles to Transactions, Transactions cycles back to Info
        assert_eq!(BlockDetailTab::default(), BlockDetailTab::Info);
        assert_eq!(BlockDetailTab::Info.next(), BlockDetailTab::Transactions);
        assert_eq!(BlockDetailTab::Transactions.next(), BlockDetailTab::Info);

        // Predicates work correctly
        assert!(BlockDetailTab::Info.is_info());
        assert!(!BlockDetailTab::Info.is_transactions());
        assert!(BlockDetailTab::Transactions.is_transactions());
        assert!(!BlockDetailTab::Transactions.is_info());
    }

    #[test]
    fn test_navigation_reset_clears_all_state() {
        let mut nav = NavigationState::new();

        // Set up some state
        nav.block_scroll = 10;
        nav.selected_block_index = Some(5);
        nav.selected_block_id = Some(12345);
        nav.show_block_details = true;
        nav.graph_scroll_x = 50;

        nav.reset();

        // Verify everything is cleared
        assert_eq!(nav.block_scroll, 0);
        assert!(nav.selected_block_index.is_none());
        assert!(nav.selected_block_id.is_none());
        assert!(!nav.show_block_details);
        assert_eq!(nav.graph_scroll_x, 0);
    }

    #[test]
    fn test_detail_view_open_close_cycle() {
        let mut nav = NavigationState::new();

        // Initially no details shown
        assert!(!nav.is_showing_details());

        // Open block details
        nav.open_block_details();
        assert!(nav.is_showing_details());
        assert!(nav.show_block_details);
        assert_eq!(nav.block_detail_tab, BlockDetailTab::Info);
        assert!(nav.block_txn_index.is_none());

        // Close all details
        nav.close_details();
        assert!(!nav.is_showing_details());
        assert!(!nav.show_block_details);
        assert!(!nav.show_transaction_details);

        // Open transaction details
        nav.open_transaction_details();
        assert!(nav.is_showing_details());
        assert!(nav.show_transaction_details);
        assert_eq!(nav.graph_scroll_x, 0);
        assert_eq!(nav.graph_scroll_y, 0);

        // Verify all detail types work with is_showing_details
        nav.close_details();
        nav.show_account_details = true;
        assert!(nav.is_showing_details());

        nav.close_details();
        nav.show_asset_details = true;
        assert!(nav.is_showing_details());
    }

    #[test]
    fn test_block_selection_with_data() {
        let mut nav = NavigationState::new();
        let blocks = vec![
            create_test_block(10000),
            create_test_block(10001),
            create_test_block(10002),
        ];

        // Select block and verify index and ID are set
        nav.select_block(1, &blocks);
        assert_eq!(nav.selected_block_index, Some(1));
        assert_eq!(nav.selected_block_id, Some(10001));
        assert!(nav.has_block_selection());

        // Clear selection
        nav.clear_block_selection();
        assert!(!nav.has_block_selection());
        assert!(nav.selected_block_index.is_none());
        assert!(nav.selected_block_id.is_none());
    }

    #[test]
    fn test_transaction_selection_with_data() {
        let mut nav = NavigationState::new();
        let transactions = vec![
            create_test_transaction("txn1"),
            create_test_transaction("txn2"),
            create_test_transaction("txn3"),
        ];

        // Select transaction and verify index and ID are set
        nav.select_transaction(1, &transactions);
        assert_eq!(nav.selected_transaction_index, Some(1));
        assert_eq!(nav.selected_transaction_id, Some("txn2".to_string()));
        assert!(nav.has_transaction_selection());

        // Clear selection
        nav.clear_transaction_selection();
        assert!(!nav.has_transaction_selection());
        assert!(nav.selected_transaction_index.is_none());
        assert!(nav.selected_transaction_id.is_none());
    }

    #[test]
    fn test_graph_scroll_respects_bounds() {
        let mut nav = NavigationState::new();
        nav.set_graph_bounds(100, 50);

        // Scroll right and verify clamping at max
        nav.scroll_graph_right(200);
        assert_eq!(nav.graph_scroll_x, 100);

        // Scroll left and verify clamping at 0
        nav.scroll_graph_left(200);
        assert_eq!(nav.graph_scroll_x, 0);

        // Same for vertical
        nav.scroll_graph_down(200);
        assert_eq!(nav.graph_scroll_y, 50);
        nav.scroll_graph_up(200);
        assert_eq!(nav.graph_scroll_y, 0);

        // Test set_graph_bounds clamps existing scroll position
        nav.graph_scroll_x = 200;
        nav.graph_scroll_y = 150;
        nav.set_graph_bounds(80, 40);
        assert_eq!(nav.graph_scroll_x, 80);
        assert_eq!(nav.graph_scroll_y, 40);
    }

    #[test]
    fn test_block_txn_navigation() {
        let mut nav = NavigationState::new();

        // Initialize at first item when moving down with no selection
        nav.move_block_txn_down(5, 10);
        assert_eq!(nav.block_txn_index, Some(0));

        // Move down
        nav.move_block_txn_down(5, 10);
        assert_eq!(nav.block_txn_index, Some(1));

        // Move up
        nav.move_block_txn_up();
        assert_eq!(nav.block_txn_index, Some(0));

        // Can't go below 0
        nav.move_block_txn_up();
        assert_eq!(nav.block_txn_index, Some(0));

        // Can't go above max
        nav.block_txn_index = Some(5);
        nav.move_block_txn_down(5, 10);
        assert_eq!(nav.block_txn_index, Some(5));
    }
}
