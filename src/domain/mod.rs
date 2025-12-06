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
pub mod application;
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
pub use network::{CustomNetwork, Network, NetworkConfig};

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

// Application types
pub use application::{AppStateValue, ApplicationDetails, ApplicationInfo};

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
    /// An application search result.
    Application(ApplicationInfo),
}
