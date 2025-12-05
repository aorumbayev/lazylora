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
    AccountDetails, AlgoBlock, AssetDetails, BlockDetails, SearchResultItem, Transaction,
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
    }

    /// Clears viewed entity details (when closing detail popups).
    #[allow(dead_code)] // Part of data state API
    pub fn clear_viewed_details(&mut self) {
        self.viewed_transaction = None;
        self.viewed_account = None;
        self.viewed_asset = None;
    }

    // ========================================================================
    // Block Operations
    // ========================================================================

    /// Returns `true` if there are no blocks.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn has_no_blocks(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Returns the number of blocks.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

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

    /// Gets a block by index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index in the blocks list
    ///
    /// # Returns
    ///
    /// Reference to the block if the index is valid.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn get_block(&self, index: usize) -> Option<&AlgoBlock> {
        self.blocks.get(index)
    }

    /// Gets the ID of a block at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index in the blocks list
    ///
    /// # Returns
    ///
    /// The block ID if the index is valid.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn get_block_id(&self, index: usize) -> Option<u64> {
        self.blocks.get(index).map(|b| b.id)
    }

    // ========================================================================
    // Transaction Operations
    // ========================================================================

    /// Returns `true` if there are no transactions.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn has_no_transactions(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Returns the number of transactions.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

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

    /// Gets a transaction by index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index in the transactions list
    ///
    /// # Returns
    ///
    /// Reference to the transaction if the index is valid.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn get_transaction(&self, index: usize) -> Option<&Transaction> {
        self.transactions.get(index)
    }

    /// Gets the ID of a transaction at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index in the transactions list
    ///
    /// # Returns
    ///
    /// The transaction ID if the index is valid.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn get_transaction_id(&self, index: usize) -> Option<&str> {
        self.transactions.get(index).map(|t| t.id.as_str())
    }

    // ========================================================================
    // Search Results Operations
    // ========================================================================

    /// Returns `true` if there are no search results.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn has_no_search_results(&self) -> bool {
        self.filtered_search_results.is_empty()
    }

    /// Returns the number of search results.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn search_results_count(&self) -> usize {
        self.filtered_search_results.len()
    }

    /// Clears search results.
    #[allow(dead_code)] // Part of data state API
    pub fn clear_search_results(&mut self) {
        self.filtered_search_results.clear()
    }

    /// Sets search results from a list of items.
    ///
    /// # Arguments
    ///
    /// * `results` - The search result items to store
    #[allow(dead_code)] // Part of data state API
    pub fn set_search_results(&mut self, results: Vec<SearchResultItem>) {
        self.filtered_search_results = results.into_iter().enumerate().collect();
    }

    /// Gets a transaction by ID from search results.
    ///
    /// # Arguments
    ///
    /// * `txn_id` - The transaction ID to find
    ///
    /// # Returns
    ///
    /// Reference to the transaction if found in search results.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn find_search_result_transaction(&self, txn_id: &str) -> Option<&Transaction> {
        self.filtered_search_results
            .iter()
            .find_map(|(_, item)| match item {
                SearchResultItem::Transaction(t) if t.id == txn_id => Some(t.as_ref()),
                _ => None,
            })
    }

    /// Gets the first search result item.
    ///
    /// # Returns
    ///
    /// Reference to the first search result item, if any.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn first_search_result(&self) -> Option<&SearchResultItem> {
        self.filtered_search_results.first().map(|(_, item)| item)
    }

    // ========================================================================
    // Block Details Operations
    // ========================================================================

    /// Returns `true` if block details are loaded.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn has_block_details(&self) -> bool {
        self.block_details.is_some()
    }

    /// Gets the number of transactions in the loaded block details.
    ///
    /// # Returns
    ///
    /// The count of transactions if block details are loaded, otherwise 0.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn block_details_txn_count(&self) -> usize {
        self.block_details
            .as_ref()
            .map_or(0, |bd| bd.transactions.len())
    }

    /// Gets a transaction from block details by index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index in the block's transaction list
    ///
    /// # Returns
    ///
    /// Reference to the transaction if found.
    #[must_use]
    #[allow(dead_code)] // Part of data state API
    pub fn get_block_details_txn(&self, index: usize) -> Option<&Transaction> {
        self.block_details
            .as_ref()
            .and_then(|bd| bd.transactions.get(index))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TxnType;

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
            details: crate::domain::TransactionDetails::None,
            inner_transactions: Vec::new(),
        }
    }

    #[test]
    fn test_new_creates_empty_state() {
        let data = DataState::new();
        assert!(data.blocks.is_empty());
        assert!(data.transactions.is_empty());
        assert!(data.filtered_search_results.is_empty());
        assert!(data.block_details.is_none());
        assert!(data.viewed_transaction.is_none());
    }

    #[test]
    fn test_clear_removes_all_data() {
        let mut data = DataState::new();
        data.blocks.push(create_test_block(1));
        data.transactions.push(create_test_transaction("tx1"));
        data.viewed_transaction = Some(create_test_transaction("tx2"));

        data.clear();

        assert!(data.blocks.is_empty());
        assert!(data.transactions.is_empty());
        assert!(data.viewed_transaction.is_none());
    }

    #[test]
    fn test_find_block_index() {
        let mut data = DataState::new();
        data.blocks.push(create_test_block(100));
        data.blocks.push(create_test_block(200));
        data.blocks.push(create_test_block(300));

        assert_eq!(data.find_block_index(100), Some(0));
        assert_eq!(data.find_block_index(200), Some(1));
        assert_eq!(data.find_block_index(300), Some(2));
        assert_eq!(data.find_block_index(400), None);
    }

    #[test]
    fn test_find_transaction_index() {
        let mut data = DataState::new();
        data.transactions.push(create_test_transaction("tx1"));
        data.transactions.push(create_test_transaction("tx2"));
        data.transactions.push(create_test_transaction("tx3"));

        assert_eq!(data.find_transaction_index("tx1"), Some(0));
        assert_eq!(data.find_transaction_index("tx2"), Some(1));
        assert_eq!(data.find_transaction_index("tx3"), Some(2));
        assert_eq!(data.find_transaction_index("tx4"), None);
    }

    #[test]
    fn test_block_operations() {
        let mut data = DataState::new();
        assert!(data.has_no_blocks());
        assert_eq!(data.block_count(), 0);

        data.blocks.push(create_test_block(100));
        assert!(!data.has_no_blocks());
        assert_eq!(data.block_count(), 1);
        assert_eq!(data.get_block_id(0), Some(100));
        assert!(data.get_block(0).is_some());
        assert!(data.get_block(1).is_none());
    }

    #[test]
    fn test_transaction_operations() {
        let mut data = DataState::new();
        assert!(data.has_no_transactions());
        assert_eq!(data.transaction_count(), 0);

        data.transactions.push(create_test_transaction("tx1"));
        assert!(!data.has_no_transactions());
        assert_eq!(data.transaction_count(), 1);
        assert_eq!(data.get_transaction_id(0), Some("tx1"));
        assert!(data.get_transaction(0).is_some());
        assert!(data.get_transaction(1).is_none());
    }

    #[test]
    fn test_search_results_operations() {
        let mut data = DataState::new();
        assert!(data.has_no_search_results());
        assert_eq!(data.search_results_count(), 0);

        let results = vec![SearchResultItem::Transaction(Box::new(
            create_test_transaction("tx1"),
        ))];
        data.set_search_results(results);

        assert!(!data.has_no_search_results());
        assert_eq!(data.search_results_count(), 1);
        assert!(data.first_search_result().is_some());

        data.clear_search_results();
        assert!(data.has_no_search_results());
    }

    #[test]
    fn test_find_search_result_transaction() {
        let mut data = DataState::new();
        let results = vec![
            SearchResultItem::Transaction(Box::new(create_test_transaction("tx1"))),
            SearchResultItem::Transaction(Box::new(create_test_transaction("tx2"))),
        ];
        data.set_search_results(results);

        let found = data.find_search_result_transaction("tx2");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "tx2");

        assert!(data.find_search_result_transaction("tx3").is_none());
    }
}
