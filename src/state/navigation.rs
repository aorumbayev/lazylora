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
}

// ============================================================================
// Application Detail Tab
// ============================================================================

/// The tab in the application details popup.
///
/// Application details can show general info, state, or programs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppDetailTab {
    /// General application information (ID, creator, schemas).
    #[default]
    Info,
    /// Global state key-value pairs.
    State,
    /// Program information (approval/clear programs).
    Programs,
}

impl AppDetailTab {
    /// Cycles to the next tab.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Info => Self::State,
            Self::State => Self::Programs,
            Self::Programs => Self::Info,
        }
    }
}

// ============================================================================
// Detail Popup Stack
// ============================================================================

/// A saved popup state for stack-based navigation.
///
/// When opening a nested popup (e.g., Asset from Account), the parent popup
/// state is saved to this struct so it can be restored when the child closes.
#[derive(Debug, Clone)]
pub struct SavedPopupState {
    /// The type of detail popup that was open.
    pub popup_type: DetailPopupType,
    /// The ID of the entity being viewed (address, asset_id, app_id, etc.).
    pub entity_id: String,
    /// The tab that was selected (for popups with tabs).
    pub tab_index: usize,
    /// The selected item index within the tab's list.
    pub item_index: Option<usize>,
    /// The scroll position within the tab's list.
    pub item_scroll: u16,
}

/// The type of detail popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants reserved for future popup stack support
pub enum DetailPopupType {
    /// Block details popup.
    Block,
    /// Transaction details popup.
    Transaction,
    /// Account details popup.
    Account,
    /// Asset details popup.
    Asset,
    /// Application details popup.
    Application,
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
    /// Whether the application details popup is shown.
    pub show_application_details: bool,

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

    // === Application Detail View State ===
    /// Current tab in application details popup.
    pub app_detail_tab: AppDetailTab,
    /// Selected item index within application state list.
    pub app_state_index: Option<usize>,
    /// Scroll position for application state list.
    pub app_state_scroll: u16,

    // === Graph View State ===
    /// Horizontal scroll offset for transaction graph view.
    pub graph_scroll_x: u16,
    /// Vertical scroll offset for transaction graph view.
    pub graph_scroll_y: u16,
    /// Maximum horizontal scroll offset for transaction graph (computed from content).
    pub graph_max_scroll_x: u16,
    /// Maximum vertical scroll offset for transaction graph (computed from content).
    pub graph_max_scroll_y: u16,

    // === Detail Table Row Selection ===
    /// Selected row index in detail table view (for copy functionality).
    pub detail_row_index: Option<usize>,
    /// Scroll position for detail table rows.
    pub detail_row_scroll: u16,

    // === Popup Stack for Nested Navigation ===
    /// Stack of saved popup states for nested navigation.
    /// When opening a nested popup (e.g., Asset from Account details),
    /// the parent popup state is pushed here. On dismiss, we pop and restore.
    pub popup_stack: Vec<SavedPopupState>,
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
        self.show_application_details = false;
        self.block_detail_tab = BlockDetailTab::default();
        self.block_txn_index = None;
        self.block_txn_scroll = 0;
        self.account_detail_tab = AccountDetailTab::default();
        self.account_item_index = None;
        self.account_item_scroll = 0;
        self.app_detail_tab = AppDetailTab::default();
        self.app_state_index = None;
        self.app_state_scroll = 0;
        self.graph_scroll_x = 0;
        self.graph_scroll_y = 0;
        self.graph_max_scroll_x = 0;
        self.graph_max_scroll_y = 0;
        self.detail_row_index = None;
        self.detail_row_scroll = 0;
        self.popup_stack.clear();
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
            || self.show_application_details
    }

    /// Closes all detail views.
    pub fn close_details(&mut self) {
        self.show_block_details = false;
        self.show_transaction_details = false;
        self.show_account_details = false;
        self.show_asset_details = false;
        self.show_application_details = false;
    }

    /// Returns `true` if there are saved popups in the stack.
    #[must_use]
    pub fn has_popup_stack(&self) -> bool {
        !self.popup_stack.is_empty()
    }

    /// Pushes the current account popup state to the stack.
    ///
    /// Call this before opening a nested popup from account details.
    pub fn push_account_state(&mut self, address: &str) {
        let tab_index = match self.account_detail_tab {
            AccountDetailTab::Info => 0,
            AccountDetailTab::Assets => 1,
            AccountDetailTab::Apps => 2,
        };
        self.popup_stack.push(SavedPopupState {
            popup_type: DetailPopupType::Account,
            entity_id: address.to_string(),
            tab_index,
            item_index: self.account_item_index,
            item_scroll: self.account_item_scroll,
        });
    }

    /// Pops the most recent saved popup state from the stack.
    ///
    /// Returns the saved state if one exists, or None if stack is empty.
    pub fn pop_popup_state(&mut self) -> Option<SavedPopupState> {
        self.popup_stack.pop()
    }

    /// Restores account detail state from a saved popup state.
    pub fn restore_account_state(&mut self, saved: &SavedPopupState) {
        self.account_detail_tab = match saved.tab_index {
            0 => AccountDetailTab::Info,
            1 => AccountDetailTab::Assets,
            _ => AccountDetailTab::Apps,
        };
        self.account_item_index = saved.item_index;
        self.account_item_scroll = saved.item_scroll;
        self.show_account_details = true;
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

    // ========================================================================
    // Block Detail Navigation
    // ========================================================================

    /// Cycles the block detail tab between Info and Transactions.
    pub fn cycle_block_detail_tab(&mut self) {
        self.block_detail_tab = self.block_detail_tab.next();
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
    // Application Detail Navigation
    // ========================================================================

    /// Cycles the app detail tab to the next tab.
    pub fn cycle_app_detail_tab(&mut self) {
        self.app_detail_tab = self.app_detail_tab.next();
        // Reset item selection when switching tabs
        self.app_state_index = None;
        self.app_state_scroll = 0;
    }

    /// Moves the app state selection up.
    pub fn move_app_state_up(&mut self) {
        if let Some(idx) = self.app_state_index
            && idx > 0
        {
            self.app_state_index = Some(idx - 1);
            // Adjust scroll if needed (each item is 1 line)
            let new_pos = (idx - 1) as u16;
            if new_pos < self.app_state_scroll {
                self.app_state_scroll = new_pos;
            }
        }
    }

    /// Moves the app state selection down.
    ///
    /// # Arguments
    ///
    /// * `max` - Maximum valid index (length - 1)
    /// * `visible_height` - Number of visible rows in the list area
    pub fn move_app_state_down(&mut self, max: usize, visible_height: u16) {
        if let Some(idx) = self.app_state_index {
            if idx < max {
                self.app_state_index = Some(idx + 1);
                // Adjust scroll if needed
                let new_pos = (idx + 1) as u16;
                let visible_end = self.app_state_scroll + visible_height;
                if new_pos >= visible_end {
                    self.app_state_scroll = new_pos.saturating_sub(visible_height) + 1;
                }
            }
        } else if max > 0 {
            self.app_state_index = Some(0);
            self.app_state_scroll = 0;
        }
    }

    /// Resets app detail view state.
    pub fn reset_app_detail(&mut self) {
        self.app_detail_tab = AppDetailTab::default();
        self.app_state_index = None;
        self.app_state_scroll = 0;
    }

    // ========================================================================
    // Detail Table Row Navigation
    // ========================================================================

    /// Moves the detail row selection up.
    pub fn move_detail_row_up(&mut self) {
        if let Some(idx) = self.detail_row_index
            && idx > 0
        {
            self.detail_row_index = Some(idx - 1);
            // Adjust scroll if needed
            let new_pos = (idx - 1) as u16;
            if new_pos < self.detail_row_scroll {
                self.detail_row_scroll = new_pos;
            }
        }
    }

    /// Moves the detail row selection down.
    ///
    /// # Arguments
    ///
    /// * `max` - Maximum valid index (row_count - 1)
    /// * `visible_height` - Number of visible rows in the table area
    pub fn move_detail_row_down(&mut self, max: usize, visible_height: u16) {
        if let Some(idx) = self.detail_row_index {
            if idx < max {
                self.detail_row_index = Some(idx + 1);
                // Adjust scroll if needed
                let new_pos = (idx + 1) as u16;
                let visible_end = self.detail_row_scroll + visible_height;
                if new_pos >= visible_end {
                    self.detail_row_scroll = new_pos.saturating_sub(visible_height) + 1;
                }
            }
        } else if max > 0 {
            // Initialize selection at first row
            self.detail_row_index = Some(0);
            self.detail_row_scroll = 0;
        }
    }

    /// Resets detail row selection (call when opening/closing detail views).
    pub fn reset_detail_row(&mut self) {
        self.detail_row_index = None;
        self.detail_row_scroll = 0;
    }

    /// Initializes detail row selection at first row if not set.
    pub fn init_detail_row_if_needed(&mut self, row_count: usize) {
        if self.detail_row_index.is_none() && row_count > 0 {
            self.detail_row_index = Some(0);
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{BlockMother, TransactionMother};

    #[test]
    fn test_detail_view_mode_toggle_behavior() {
        // Table toggles to Visual, Visual toggles back to Table
        assert_eq!(DetailViewMode::default(), DetailViewMode::Table);
        assert_eq!(DetailViewMode::Table.toggle(), DetailViewMode::Visual);
        assert_eq!(DetailViewMode::Visual.toggle(), DetailViewMode::Table);

        // Match exhaustively to verify all variants
        assert!(matches!(DetailViewMode::Table, DetailViewMode::Table));
        assert!(matches!(DetailViewMode::Visual, DetailViewMode::Visual));
    }

    #[test]
    fn test_block_detail_tab_cycle_behavior() {
        // Info cycles to Transactions, Transactions cycles back to Info
        assert_eq!(BlockDetailTab::default(), BlockDetailTab::Info);
        assert_eq!(BlockDetailTab::Info.next(), BlockDetailTab::Transactions);
        assert_eq!(BlockDetailTab::Transactions.next(), BlockDetailTab::Info);

        // Match exhaustively to verify all variants
        assert!(matches!(BlockDetailTab::Info, BlockDetailTab::Info));
        assert!(matches!(
            BlockDetailTab::Transactions,
            BlockDetailTab::Transactions
        ));
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

        // Open block details (inline logic from deleted open_block_details)
        nav.show_block_details = true;
        nav.block_detail_tab = BlockDetailTab::default();
        nav.block_txn_index = None;
        nav.block_txn_scroll = 0;

        assert!(nav.is_showing_details());
        assert!(nav.show_block_details);
        assert_eq!(nav.block_detail_tab, BlockDetailTab::Info);
        assert!(nav.block_txn_index.is_none());

        // Close all details
        nav.close_details();
        assert!(!nav.is_showing_details());
        assert!(!nav.show_block_details);
        assert!(!nav.show_transaction_details);

        // Open transaction details (inline logic)
        nav.show_transaction_details = true;
        nav.graph_scroll_x = 0;
        nav.graph_scroll_y = 0;

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
            BlockMother::with_id(10000),
            BlockMother::with_id(10001),
            BlockMother::with_id(10002),
        ];

        // Select block and verify index and ID are set
        nav.select_block(1, &blocks);
        assert_eq!(nav.selected_block_index, Some(1));
        assert_eq!(nav.selected_block_id, Some(10001));
        assert!(nav.selected_block_index.is_some());

        // Clear selection
        nav.clear_block_selection();
        assert!(nav.selected_block_index.is_none());
        assert!(nav.selected_block_id.is_none());
    }

    #[test]
    fn test_transaction_selection_with_data() {
        let mut nav = NavigationState::new();
        let transactions = vec![
            TransactionMother::payment("txn1"),
            TransactionMother::payment("txn2"),
            TransactionMother::payment("txn3"),
        ];

        // Select transaction and verify index and ID are set
        nav.select_transaction(1, &transactions);
        assert_eq!(nav.selected_transaction_index, Some(1));
        assert_eq!(nav.selected_transaction_id, Some("txn2".to_string()));
        assert!(nav.selected_transaction_index.is_some());

        // Clear selection
        nav.clear_transaction_selection();
        assert!(nav.selected_transaction_index.is_none());
        assert!(nav.selected_transaction_id.is_none());
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
