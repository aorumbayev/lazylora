//! Domain types for the LazyLora Algorand explorer.
//!
//! This module contains all the core domain types used throughout the application,
//! including network configuration, transactions, blocks, accounts, assets, and NFD.
//!
//! # Module Organization
//!
//! - [`error`] - Custom error types for Algorand operations
//! - [`network`] - Network configuration (MainNet, TestNet, LocalNet)
//! - [`transaction`] - Transaction types and details
//! - [`block`] - Block types and information
//! - [`account`] - Account types and details
//! - [`asset`] - Asset types and details
//! - [`nfd`] - NFD (Non-Fungible Domain) types

// ============================================================================
// Module Declarations
// ============================================================================

pub mod account;
pub mod asset;
pub mod block;
pub mod error;
pub mod network;
pub mod nfd;
pub mod transaction;

// ============================================================================
// Re-exports
// ============================================================================

// Error types
pub use error::AlgoError;

// Network types
pub use network::Network;

// Transaction types
#[allow(unused_imports)] // OnComplete used by tests in client/algo.rs
pub use transaction::{
    AppCallDetails, AssetConfigDetails, AssetFreezeDetails, AssetTransferDetails, HeartbeatDetails,
    KeyRegDetails, OnComplete, PaymentDetails, StateProofDetails, Transaction, TransactionDetails,
    TxnType, format_timestamp,
};

// Block types
pub use block::{AlgoBlock, BlockDetails, BlockInfo, count_transactions};

// Account types
pub use account::{
    AccountAssetHolding, AccountDetails, AccountInfo, AppLocalState, CreatedAppInfo,
    CreatedAssetInfo, ParticipationInfo,
};

// Asset types
pub use asset::{AssetDetails, AssetInfo};

// NFD types
pub use nfd::NfdInfo;

// ============================================================================
// Search Result Types
// ============================================================================

/// Search result item that can hold any searchable entity.
///
/// Used to return heterogeneous search results from the client.
#[derive(Debug, Clone, PartialEq)]
pub enum SearchResultItem {
    /// A transaction search result.
    Transaction(Box<Transaction>),
    /// A block search result.
    Block(BlockInfo),
    /// An account search result.
    Account(AccountInfo),
    /// An asset search result.
    Asset(AssetInfo),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_item_variants() {
        let txn_result = SearchResultItem::Transaction(Box::new(Transaction {
            id: "test".to_string(),
            txn_type: TxnType::Payment,
            from: "from".to_string(),
            to: "to".to_string(),
            timestamp: "now".to_string(),
            block: 1,
            fee: 1000,
            note: String::new(),
            amount: 0,
            asset_id: None,
            rekey_to: None,
            details: TransactionDetails::None,
            inner_transactions: Vec::new(),
        }));
        assert!(matches!(txn_result, SearchResultItem::Transaction(_)));
        assert!(!matches!(txn_result, SearchResultItem::Block(_)));

        let block_result = SearchResultItem::Block(BlockInfo::new(
            1,
            "now".to_string(),
            0,
            "proposer".to_string(),
            "seed".to_string(),
        ));
        assert!(matches!(block_result, SearchResultItem::Block(_)));

        let account_result = SearchResultItem::Account(AccountInfo::new(
            "addr".to_string(),
            0,
            0,
            0,
            "Offline".to_string(),
            0,
            0,
        ));
        assert!(matches!(account_result, SearchResultItem::Account(_)));

        let asset_result = SearchResultItem::Asset(AssetInfo::new(
            1,
            "name".to_string(),
            "unit".to_string(),
            "creator".to_string(),
            100,
            0,
            String::new(),
        ));
        assert!(matches!(asset_result, SearchResultItem::Asset(_)));
    }

    #[test]
    fn test_search_result_item_pattern_matching() {
        let block_result = SearchResultItem::Block(BlockInfo::new(
            12345,
            "now".to_string(),
            5,
            "proposer".to_string(),
            "seed".to_string(),
        ));

        // Test pattern matching to extract data
        if let SearchResultItem::Block(block) = &block_result {
            assert_eq!(block.id, 12345);
        } else {
            panic!("Expected Block variant");
        }

        // Test that other variants don't match
        assert!(!matches!(block_result, SearchResultItem::Transaction(_)));
        assert!(!matches!(block_result, SearchResultItem::Account(_)));
        assert!(!matches!(block_result, SearchResultItem::Asset(_)));
    }
}
