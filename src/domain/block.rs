//! Block types for Algorand blockchain.
//!
//! This module defines block-related types including basic block info
//! and detailed block information with transactions.

use std::collections::HashMap;

use super::transaction::{Transaction, TxnType};

// ============================================================================
// Helper Functions
// ============================================================================

/// Count the number of transactions in a block JSON.
///
/// # Arguments
///
/// * `block` - The JSON representation of the block
///
/// # Returns
///
/// The number of transactions in the block.
#[must_use]
pub fn count_transactions(block: &serde_json::Value) -> u16 {
    if let Some(txns) = block.get("txns") {
        if let Some(arr) = txns.as_array() {
            return arr.len() as u16;
        } else if let Some(obj) = txns.as_object()
            && let Some(transactions) = obj.get("transactions")
            && let Some(arr) = transactions.as_array()
        {
            return arr.len() as u16;
        }
    }
    0
}

// ============================================================================
// Block Types
// ============================================================================

/// Basic block information for list display.
///
/// Contains essential block metadata for display in block lists.
#[derive(Debug, Clone, PartialEq)]
pub struct AlgoBlock {
    /// Block number (round).
    pub id: u64,
    /// Number of transactions in the block.
    pub txn_count: u16,
    /// Human-readable timestamp.
    pub timestamp: String,
}

impl AlgoBlock {
    /// Create a new `AlgoBlock` with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `id` - Block number (round)
    /// * `txn_count` - Number of transactions in the block
    /// * `timestamp` - Human-readable timestamp string
    ///
    /// # Returns
    ///
    /// A new `AlgoBlock` instance.
    #[must_use]
    pub fn new(id: u64, txn_count: u16, timestamp: String) -> Self {
        Self {
            id,
            txn_count,
            timestamp,
        }
    }
}

// ============================================================================
// Block Info
// ============================================================================

/// Detailed block information for search results and popups.
///
/// Contains more detailed block metadata including proposer information.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockInfo {
    /// Block number (round).
    pub id: u64,
    /// Human-readable timestamp.
    pub timestamp: String,
    /// Number of transactions in the block.
    pub txn_count: u16,
    /// Block proposer address.
    pub proposer: String,
    /// Block seed for randomness.
    pub seed: String,
}

impl BlockInfo {
    /// Create a new `BlockInfo` with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `id` - Block number (round)
    /// * `timestamp` - Human-readable timestamp string
    /// * `txn_count` - Number of transactions in the block
    /// * `proposer` - Block proposer address
    /// * `seed` - Block seed
    ///
    /// # Returns
    ///
    /// A new `BlockInfo` instance.
    #[must_use]
    pub fn new(id: u64, timestamp: String, txn_count: u16, proposer: String, seed: String) -> Self {
        Self {
            id,
            timestamp,
            txn_count,
            proposer,
            seed,
        }
    }
}

// ============================================================================
// Block Details
// ============================================================================

/// Extended block details including transactions.
///
/// Contains complete block information with all transactions
/// and aggregated statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockDetails {
    /// Basic block info.
    pub info: BlockInfo,
    /// Transactions in this block.
    pub transactions: Vec<Transaction>,
    /// Count of transactions by type.
    pub txn_type_counts: HashMap<TxnType, usize>,
}

impl BlockDetails {
    /// Create a new `BlockDetails` with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `info` - Basic block information
    /// * `transactions` - List of transactions in the block
    ///
    /// # Returns
    ///
    /// A new `BlockDetails` instance with computed transaction type counts.
    #[must_use]
    pub fn new(info: BlockInfo, transactions: Vec<Transaction>) -> Self {
        // Compute transaction type counts
        let mut txn_type_counts = HashMap::new();
        for txn in &transactions {
            *txn_type_counts.entry(txn.txn_type).or_insert(0) += 1;
        }

        Self {
            info,
            transactions,
            txn_type_counts,
        }
    }

    /// Returns the total number of transactions in this block.
    ///
    /// # Returns
    ///
    /// The count of transactions.
    #[must_use]
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// Returns the count of a specific transaction type.
    ///
    /// # Arguments
    ///
    /// * `txn_type` - The transaction type to count
    ///
    /// # Returns
    ///
    /// The number of transactions of the specified type.
    #[must_use]
    pub fn count_by_type(&self, txn_type: TxnType) -> usize {
        self.txn_type_counts.get(&txn_type).copied().unwrap_or(0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algo_block_new() {
        let block = AlgoBlock::new(12345, 10, "2023-11-14".to_string());
        assert_eq!(block.id, 12345);
        assert_eq!(block.txn_count, 10);
        assert_eq!(block.timestamp, "2023-11-14");
    }

    #[test]
    fn test_block_info_new() {
        let info = BlockInfo::new(
            12345,
            "2023-11-14".to_string(),
            10,
            "PROPOSER".to_string(),
            "SEED".to_string(),
        );
        assert_eq!(info.id, 12345);
        assert_eq!(info.timestamp, "2023-11-14");
        assert_eq!(info.txn_count, 10);
        assert_eq!(info.proposer, "PROPOSER");
        assert_eq!(info.seed, "SEED");
    }

    #[test]
    fn test_block_details_new() {
        let info = BlockInfo::new(
            12345,
            "2023-11-14".to_string(),
            0,
            "PROPOSER".to_string(),
            "SEED".to_string(),
        );
        let details = BlockDetails::new(info, Vec::new());
        assert_eq!(details.transaction_count(), 0);
        assert!(details.txn_type_counts.is_empty());
    }

    #[test]
    fn test_count_transactions() {
        let block_with_txns = serde_json::json!({
            "txns": [
                {"id": "tx1"},
                {"id": "tx2"},
                {"id": "tx3"}
            ]
        });
        assert_eq!(count_transactions(&block_with_txns), 3);

        let empty_block = serde_json::json!({});
        assert_eq!(count_transactions(&empty_block), 0);
    }

    #[test]
    fn test_block_details_count_by_type() {
        let info = BlockInfo::new(
            12345,
            "2023-11-14".to_string(),
            0,
            "PROPOSER".to_string(),
            "SEED".to_string(),
        );
        let details = BlockDetails::new(info, Vec::new());
        assert_eq!(details.count_by_type(TxnType::Payment), 0);
        assert_eq!(details.count_by_type(TxnType::AppCall), 0);
    }
}
