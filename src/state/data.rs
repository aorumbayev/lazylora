//! Data state management for the LazyLora TUI.
//!
//! This module manages all application data including:
//! - Blocks and transactions from the network
//! - Search results
//! - Viewed entity details (transaction, account, asset)
//!
//! # Design
//!
//! The data state is separate from navigation state, allowing the data
//! to be updated independently of what's currently selected or visible.

use crate::domain::{
    AccountDetails, AlgoBlock, ApplicationDetails, AssetDetails, BlockDetails, SearchResultItem,
    Transaction,
};

// ============================================================================
// Data State
// ============================================================================

/// Data state: blocks, transactions, and search results.
///
/// This struct manages all the actual data displayed in the application,
/// keeping it separate from navigation concerns.
///
/// # Example
///
/// ```ignore
/// use crate::state::DataState;
///
/// let mut data = DataState::new();
///
/// // Update blocks from network
/// data.blocks = fetched_blocks;
///
/// // Store viewed transaction
/// data.viewed_transaction = Some(selected_txn);
/// ```
#[derive(Debug, Default)]
pub struct DataState {
    // === List Data ===
    /// List of recent blocks.
    pub blocks: Vec<AlgoBlock>,
    /// List of recent transactions.
    pub transactions: Vec<Transaction>,

    // === Search Results ===
    /// Filtered search results with their original indices.
    pub filtered_search_results: Vec<(usize, SearchResultItem)>,

    // === Detail View Data ===
    /// Currently loaded block details (for block details popup).
    pub block_details: Option<BlockDetails>,
    /// Currently viewed transaction details (for transaction details popup).
    pub viewed_transaction: Option<Transaction>,
    /// Currently viewed account details (for account details popup).
    pub viewed_account: Option<AccountDetails>,
    /// Currently viewed asset details (for asset details popup).
    pub viewed_asset: Option<AssetDetails>,
    /// Currently viewed application details (for application details popup).
    pub viewed_application: Option<ApplicationDetails>,
}

impl DataState {
    /// Creates a new `DataState` with empty collections.
    ///
    /// # Returns
    ///
    /// A new data state with no data loaded.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all data (useful when switching networks).
    ///
    /// This removes all cached data to prepare for loading fresh data
    /// from a different network.
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.transactions.clear();
        self.filtered_search_results.clear();
        self.block_details = None;
        self.viewed_transaction = None;
        self.viewed_account = None;
        self.viewed_asset = None;
        self.viewed_application = None;
    }

    // ========================================================================
    // Block Operations
    // ========================================================================

    /// Finds a block index by its ID.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The block round number
    ///
    /// # Returns
    ///
    /// The index of the block in the list, if found.
    #[must_use]
    pub fn find_block_index(&self, block_id: u64) -> Option<usize> {
        self.blocks.iter().position(|b| b.id == block_id)
    }

    // ========================================================================
    // Transaction Operations
    // ========================================================================

    /// Finds a transaction index by its ID.
    ///
    /// # Arguments
    ///
    /// * `txn_id` - The transaction ID string
    ///
    /// # Returns
    ///
    /// The index of the transaction in the list, if found.
    #[must_use]
    pub fn find_transaction_index(&self, txn_id: &str) -> Option<usize> {
        self.transactions.iter().position(|t| t.id == txn_id)
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
    fn test_find_block_index() {
        let mut data = DataState::new();
        data.blocks.push(BlockMother::with_id(100));
        data.blocks.push(BlockMother::with_id(200));
        data.blocks.push(BlockMother::with_id(300));

        assert_eq!(data.find_block_index(100), Some(0));
        assert_eq!(data.find_block_index(200), Some(1));
        assert_eq!(data.find_block_index(300), Some(2));
        assert_eq!(data.find_block_index(400), None);
    }

    #[test]
    fn test_find_transaction_index() {
        let mut data = DataState::new();
        data.transactions.push(TransactionMother::payment("tx1"));
        data.transactions.push(TransactionMother::payment("tx2"));
        data.transactions.push(TransactionMother::payment("tx3"));

        assert_eq!(data.find_transaction_index("tx1"), Some(0));
        assert_eq!(data.find_transaction_index("tx2"), Some(1));
        assert_eq!(data.find_transaction_index("tx3"), Some(2));
        assert_eq!(data.find_transaction_index("tx4"), None);
    }
}
