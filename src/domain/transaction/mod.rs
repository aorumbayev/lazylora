//! Transaction types for Algorand blockchain.
//!
//! This module defines all transaction-related types including the main
//! `Transaction` struct and its type-specific details for payments,
//! asset transfers, application calls, and more.
//!
//! # Module Organization
//!
//! - [`types`] - Type-specific detail structs (PaymentDetails, AppCallDetails, etc.)
//! - [`parsing`] - JSON parsing logic for transactions

use ratatui::style::Color;
use serde_json::Value;

pub mod parsing;
pub mod types;

// Re-export all public types for convenient access
pub use types::{
    AppCallDetails, AssetConfigDetails, AssetFreezeDetails, AssetTransferDetails, HeartbeatDetails,
    KeyRegDetails, OnComplete, PaymentDetails, StateProofDetails, TransactionDetails,
};

// ============================================================================
// Helper Functions
// ============================================================================

/// Format a Unix timestamp into a human-readable string.
///
/// # Arguments
///
/// * `timestamp_secs` - Unix timestamp in seconds
///
/// # Returns
///
/// A formatted date string, or "Timestamp not available" if the timestamp is 0.
#[must_use]
pub fn format_timestamp(timestamp_secs: u64) -> String {
    if timestamp_secs == 0 {
        return "Timestamp not available".to_string();
    }

    let datetime =
        chrono::DateTime::from_timestamp(timestamp_secs as i64, 0).unwrap_or_else(chrono::Utc::now);

    datetime.format("%a, %d %b %Y %H:%M:%S").to_string()
}

// ============================================================================
// Transaction Type
// ============================================================================

/// Algorand transaction types.
///
/// Each variant represents a different category of transaction
/// that can be performed on the Algorand blockchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TxnType {
    /// Payment transaction - transfers Algos between accounts.
    Payment,
    /// Application call - interacts with smart contracts.
    AppCall,
    /// Asset transfer - transfers ASAs between accounts.
    AssetTransfer,
    /// Asset configuration - creates, modifies, or destroys ASAs.
    AssetConfig,
    /// Asset freeze - freezes or unfreezes asset holdings.
    AssetFreeze,
    /// Key registration - registers participation keys.
    KeyReg,
    /// State proof - cryptographic proof of blockchain state.
    StateProof,
    /// Heartbeat - node liveness indicator.
    Heartbeat,
    /// Unknown transaction type.
    #[default]
    Unknown,
}

impl TxnType {
    /// Returns the human-readable name of the transaction type.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Payment => "Payment",
            Self::AppCall => "App Call",
            Self::AssetTransfer => "Asset Transfer",
            Self::AssetConfig => "Asset Config",
            Self::AssetFreeze => "Asset Freeze",
            Self::KeyReg => "Key Registration",
            Self::StateProof => "State Proof",
            Self::Heartbeat => "Heartbeat",
            Self::Unknown => "Unknown",
        }
    }

    /// Returns the display color for this transaction type.
    ///
    /// Used for visual differentiation in the TUI.
    #[must_use]
    pub const fn color(&self) -> Color {
        match self {
            Self::Payment => Color::Green,
            Self::AppCall => Color::Blue,
            Self::AssetTransfer => Color::Yellow,
            Self::AssetConfig => Color::Cyan,
            Self::AssetFreeze => Color::Magenta,
            Self::KeyReg => Color::Red,
            Self::StateProof => Color::Gray,
            Self::Heartbeat => Color::White,
            Self::Unknown => Color::DarkGray,
        }
    }

    /// Determine transaction type from JSON data.
    #[must_use]
    pub fn from_json(txn_json: &Value) -> Self {
        if txn_json["payment-transaction"].is_object() {
            Self::Payment
        } else if txn_json["application-transaction"].is_object() {
            Self::AppCall
        } else if txn_json["asset-transfer-transaction"].is_object() {
            Self::AssetTransfer
        } else if txn_json["asset-config-transaction"].is_object() {
            Self::AssetConfig
        } else if txn_json["asset-freeze-transaction"].is_object() {
            Self::AssetFreeze
        } else if txn_json["keyreg-transaction"].is_object() {
            Self::KeyReg
        } else if txn_json["state-proof-transaction"].is_object() {
            Self::StateProof
        } else if txn_json["heartbeat-transaction"].is_object() {
            Self::Heartbeat
        } else {
            Self::Unknown
        }
    }
}

impl std::fmt::Display for TxnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Transaction
// ============================================================================

/// Represents an Algorand transaction with all its metadata.
///
/// This is the main transaction type that contains all common fields
/// plus type-specific details for different transaction categories.
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    /// Transaction ID (52-character base32 string).
    pub id: String,
    /// Transaction type.
    pub txn_type: TxnType,
    /// Sender address.
    pub from: String,
    /// Receiver address (or app ID for app calls).
    pub to: String,
    /// Human-readable timestamp.
    pub timestamp: String,
    /// Block number where the transaction was confirmed.
    pub block: u64,
    /// Transaction fee in microAlgos.
    pub fee: u64,
    /// Transaction note (may be Base64 encoded).
    pub note: String,
    /// Amount transferred (in microAlgos or asset units).
    pub amount: u64,
    /// Asset ID for asset-related transactions.
    pub asset_id: Option<u64>,
    /// Rekey-to address (if this transaction rekeys the sender).
    pub rekey_to: Option<String>,
    /// Group ID for atomic transaction groups (Base64 encoded).
    pub group: Option<String>,
    /// Type-specific transaction details.
    pub details: TransactionDetails,
    /// Inner transactions (for app calls).
    pub inner_transactions: Vec<Transaction>,
}

// Note: Transaction::from_json is implemented in parsing.rs

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests TxnType string representation and color mapping.
    #[test]
    fn test_txn_type_properties() {
        let test_cases = [
            (TxnType::Payment, "Payment", Color::Green),
            (TxnType::AppCall, "App Call", Color::Blue),
            (TxnType::AssetTransfer, "Asset Transfer", Color::Yellow),
            (TxnType::AssetConfig, "Asset Config", Color::Cyan),
            (TxnType::AssetFreeze, "Asset Freeze", Color::Magenta),
            (TxnType::KeyReg, "Key Registration", Color::Red),
            (TxnType::StateProof, "State Proof", Color::Gray),
            (TxnType::Heartbeat, "Heartbeat", Color::White),
            (TxnType::Unknown, "Unknown", Color::DarkGray),
        ];

        for (txn_type, expected_str, expected_color) in test_cases {
            assert_eq!(
                txn_type.as_str(),
                expected_str,
                "{:?}.as_str() mismatch",
                txn_type
            );
            assert_eq!(
                txn_type.color(),
                expected_color,
                "{:?}.color() mismatch",
                txn_type
            );
        }
    }

    /// Tests timestamp formatting for edge cases.
    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "Timestamp not available");
        // Non-zero timestamp should produce a formatted string
        let result = format_timestamp(1_700_000_000);
        assert!(result.contains("2023")); // Should be a date in 2023
    }

    // Note: Transaction::from_json() parsing tests are in client/algo.rs
    // to avoid duplication and keep all JSON parsing tests in one place.
}
