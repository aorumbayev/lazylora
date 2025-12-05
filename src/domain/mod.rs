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
#[allow(unused_imports)] // OnComplete used by tests in algorand.rs
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

#[allow(dead_code)] // Methods used by tests and as public API for future use
impl SearchResultItem {
    /// Returns the type name of this search result item.
    ///
    /// # Returns
    ///
    /// A static string describing the item type.
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Transaction(_) => "Transaction",
            Self::Block(_) => "Block",
            Self::Account(_) => "Account",
            Self::Asset(_) => "Asset",
        }
    }

    /// Returns `true` if this is a transaction result.
    #[must_use]
    pub const fn is_transaction(&self) -> bool {
        matches!(self, Self::Transaction(_))
    }

    /// Returns `true` if this is a block result.
    #[must_use]
    pub const fn is_block(&self) -> bool {
        matches!(self, Self::Block(_))
    }

    /// Returns `true` if this is an account result.
    #[must_use]
    pub const fn is_account(&self) -> bool {
        matches!(self, Self::Account(_))
    }

    /// Returns `true` if this is an asset result.
    #[must_use]
    pub const fn is_asset(&self) -> bool {
        matches!(self, Self::Asset(_))
    }

    /// Attempts to get the transaction from this result.
    ///
    /// # Returns
    ///
    /// `Some` reference to the transaction if this is a transaction result.
    #[must_use]
    pub fn as_transaction(&self) -> Option<&Transaction> {
        match self {
            Self::Transaction(txn) => Some(txn),
            _ => None,
        }
    }

    /// Attempts to get the block info from this result.
    ///
    /// # Returns
    ///
    /// `Some` reference to the block info if this is a block result.
    #[must_use]
    pub fn as_block(&self) -> Option<&BlockInfo> {
        match self {
            Self::Block(block) => Some(block),
            _ => None,
        }
    }

    /// Attempts to get the account info from this result.
    ///
    /// # Returns
    ///
    /// `Some` reference to the account info if this is an account result.
    #[must_use]
    pub fn as_account(&self) -> Option<&AccountInfo> {
        match self {
            Self::Account(account) => Some(account),
            _ => None,
        }
    }

    /// Attempts to get the asset info from this result.
    ///
    /// # Returns
    ///
    /// `Some` reference to the asset info if this is an asset result.
    #[must_use]
    pub fn as_asset(&self) -> Option<&AssetInfo> {
        match self {
            Self::Asset(asset) => Some(asset),
            _ => None,
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
    fn test_search_result_item_type_name() {
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
        assert_eq!(txn_result.type_name(), "Transaction");
        assert!(txn_result.is_transaction());
        assert!(!txn_result.is_block());

        let block_result = SearchResultItem::Block(BlockInfo::new(
            1,
            "now".to_string(),
            0,
            "proposer".to_string(),
            "seed".to_string(),
        ));
        assert_eq!(block_result.type_name(), "Block");
        assert!(block_result.is_block());

        let account_result = SearchResultItem::Account(AccountInfo::new(
            "addr".to_string(),
            0,
            0,
            0,
            "Offline".to_string(),
            0,
            0,
        ));
        assert_eq!(account_result.type_name(), "Account");
        assert!(account_result.is_account());

        let asset_result = SearchResultItem::Asset(AssetInfo::new(
            1,
            "name".to_string(),
            "unit".to_string(),
            "creator".to_string(),
            100,
            0,
            String::new(),
        ));
        assert_eq!(asset_result.type_name(), "Asset");
        assert!(asset_result.is_asset());
    }

    #[test]
    fn test_search_result_item_as_methods() {
        let block_result = SearchResultItem::Block(BlockInfo::new(
            12345,
            "now".to_string(),
            5,
            "proposer".to_string(),
            "seed".to_string(),
        ));

        assert!(block_result.as_block().is_some());
        assert_eq!(block_result.as_block().unwrap().id, 12345);
        assert!(block_result.as_transaction().is_none());
        assert!(block_result.as_account().is_none());
        assert!(block_result.as_asset().is_none());
    }
}
