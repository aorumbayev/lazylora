//! Type-specific transaction detail structs.
//!
//! This module contains all the detail structs for different transaction types,
//! including payment, asset transfer, asset config, asset freeze, application call,
//! key registration, state proof, and heartbeat transactions.

// ============================================================================
// Transaction Details - Type-specific metadata
// ============================================================================

/// Type-specific transaction details.
///
/// Contains additional information specific to each transaction type,
/// providing access to fields that are only relevant for certain operations.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TransactionDetails {
    /// No additional details available.
    #[default]
    None,
    /// Payment transaction details.
    Payment(PaymentDetails),
    /// Asset transfer transaction details.
    AssetTransfer(AssetTransferDetails),
    /// Asset configuration transaction details.
    AssetConfig(AssetConfigDetails),
    /// Asset freeze transaction details.
    AssetFreeze(AssetFreezeDetails),
    /// Application call transaction details.
    AppCall(AppCallDetails),
    /// Key registration transaction details.
    KeyReg(KeyRegDetails),
    /// State proof transaction details.
    StateProof(StateProofDetails),
    /// Heartbeat transaction details.
    Heartbeat(HeartbeatDetails),
}

impl TransactionDetails {
    /// Returns true if this transaction creates something (app, asset).
    #[must_use]
    #[allow(dead_code)] // Public API
    pub fn is_creation(&self) -> bool {
        match self {
            Self::AssetConfig(details) => details.asset_id.is_none() && details.total.is_some(),
            Self::AppCall(details) => details.app_id == 0,
            _ => false,
        }
    }

    /// Returns the created entity ID if this was a creation transaction.
    #[must_use]
    #[allow(dead_code)] // Public API
    pub fn created_id(&self) -> Option<u64> {
        match self {
            Self::AssetConfig(details) => details.created_asset_id,
            Self::AppCall(details) => details.created_app_id,
            _ => None,
        }
    }
}

// ============================================================================
// Payment Details
// ============================================================================

/// Payment transaction details.
///
/// Contains additional information specific to payment transactions,
/// particularly for close-out operations.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PaymentDetails {
    /// Address to receive remaining funds when closing out.
    pub close_remainder_to: Option<String>,
    /// Amount sent to close-to address.
    pub close_amount: Option<u64>,
}

// ============================================================================
// Asset Transfer Details
// ============================================================================

/// Asset transfer transaction details.
///
/// Contains additional information for asset transfer operations,
/// including clawback and close-out information.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AssetTransferDetails {
    /// For clawback transactions, the address being clawed back from.
    pub asset_sender: Option<String>,
    /// Address to receive remaining asset holdings when closing out.
    pub close_to: Option<String>,
    /// Amount of asset sent to close-to address.
    pub close_amount: Option<u64>,
}

// ============================================================================
// Asset Config Details
// ============================================================================

/// Asset configuration transaction details.
///
/// Contains all parameters for asset creation, modification, or destruction.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AssetConfigDetails {
    /// Asset ID for modify/destroy (None for create).
    pub asset_id: Option<u64>,
    /// Set after creation - the ID of the created asset.
    pub created_asset_id: Option<u64>,
    /// Total number of units of this asset.
    pub total: Option<u64>,
    /// Number of decimal places for asset display.
    pub decimals: Option<u64>,
    /// Whether asset holdings are frozen by default.
    pub default_frozen: Option<bool>,
    /// Asset name.
    pub asset_name: Option<String>,
    /// Asset unit name.
    pub unit_name: Option<String>,
    /// URL with asset metadata.
    pub url: Option<String>,
    /// Hash of metadata (32 bytes).
    pub metadata_hash: Option<String>,
    /// Manager address - can change asset config.
    pub manager: Option<String>,
    /// Reserve address - holds non-minted units.
    pub reserve: Option<String>,
    /// Freeze address - can freeze/unfreeze holdings.
    pub freeze: Option<String>,
    /// Clawback address - can revoke holdings.
    pub clawback: Option<String>,
}

// ============================================================================
// Asset Freeze Details
// ============================================================================

/// Asset freeze transaction details.
///
/// Contains information about freeze/unfreeze operations on asset holdings.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AssetFreezeDetails {
    /// Whether the target is being frozen or unfrozen.
    pub frozen: bool,
    /// Address whose asset holdings are being frozen/unfrozen.
    pub freeze_target: String,
}

// ============================================================================
// Application Call Details
// ============================================================================

/// Application call transaction details.
///
/// Contains all parameters for smart contract interactions including
/// creation, calls, updates, and deletions.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppCallDetails {
    /// Application ID (0 for creation).
    pub app_id: u64,
    /// Set after creation - the ID of the created application.
    pub created_app_id: Option<u64>,
    /// Type of application call.
    pub on_complete: OnComplete,
    /// Approval program (Base64 encoded).
    pub approval_program: Option<String>,
    /// Clear state program (Base64 encoded).
    pub clear_state_program: Option<String>,
    /// Application arguments (Base64 encoded).
    pub app_args: Vec<String>,
    /// Referenced accounts.
    pub accounts: Vec<String>,
    /// Referenced applications.
    pub foreign_apps: Vec<u64>,
    /// Referenced assets.
    pub foreign_assets: Vec<u64>,
    /// Box references.
    pub boxes: Vec<BoxRef>,
    /// Global state schema for app creation.
    pub global_state_schema: Option<StateSchema>,
    /// Local state schema for app creation.
    pub local_state_schema: Option<StateSchema>,
    /// Extra program pages for large programs.
    pub extra_program_pages: Option<u64>,
}

// ============================================================================
// On Complete Type
// ============================================================================

/// Application call on-completion type.
///
/// Specifies what action to take after the application call completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnComplete {
    /// No additional action (default).
    #[default]
    NoOp,
    /// Opt the sender into the application.
    OptIn,
    /// Close out the sender's local state.
    CloseOut,
    /// Clear the sender's local state (cannot be rejected).
    ClearState,
    /// Update the application's programs.
    UpdateApplication,
    /// Delete the application.
    DeleteApplication,
}

impl OnComplete {
    /// Returns the string representation of the on-complete type.
    #[must_use]
    #[allow(dead_code)] // Public API
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NoOp => "NoOp",
            Self::OptIn => "OptIn",
            Self::CloseOut => "CloseOut",
            Self::ClearState => "ClearState",
            Self::UpdateApplication => "Update",
            Self::DeleteApplication => "Delete",
        }
    }

    /// Parse on-complete type from string.
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "noop" => Self::NoOp,
            "optin" => Self::OptIn,
            "closeout" => Self::CloseOut,
            "clearstate" => Self::ClearState,
            "updateapplication" | "update" => Self::UpdateApplication,
            "deleteapplication" | "delete" => Self::DeleteApplication,
            _ => Self::NoOp,
        }
    }
}

impl std::fmt::Display for OnComplete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Box Reference
// ============================================================================

/// Box reference for application calls.
///
/// References a box in an application's box storage.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BoxRef {
    /// Application ID (0 means current app).
    pub app_id: u64,
    /// Box name (Base64 encoded).
    pub name: String,
}

// ============================================================================
// State Schema
// ============================================================================

/// State schema for application storage.
///
/// Defines the storage requirements for application state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StateSchema {
    /// Number of uint64 values.
    pub num_uint: u64,
    /// Number of byte slice values.
    pub num_byte_slice: u64,
}

// ============================================================================
// Key Registration Details
// ============================================================================

/// Key registration transaction details.
///
/// Contains participation key information for consensus participation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KeyRegDetails {
    /// Voting public key (Base64 encoded).
    pub vote_key: Option<String>,
    /// VRF selection public key (Base64 encoded).
    pub selection_key: Option<String>,
    /// State proof public key (Base64 encoded).
    pub state_proof_key: Option<String>,
    /// First round for which this key is valid.
    pub vote_first: Option<u64>,
    /// Last round for which this key is valid.
    pub vote_last: Option<u64>,
    /// Key dilution for voting key.
    pub vote_key_dilution: Option<u64>,
    /// Whether this marks the account as non-participating.
    pub non_participation: bool,
}

// ============================================================================
// State Proof Details
// ============================================================================

/// State proof transaction details.
///
/// Contains cryptographic proof information for blockchain state verification.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StateProofDetails {
    /// Type of state proof.
    pub state_proof_type: Option<u64>,
    /// State proof message (hex encoded).
    pub message: Option<String>,
}

// ============================================================================
// Heartbeat Details
// ============================================================================

/// Heartbeat transaction details.
///
/// Contains node liveness indicator information.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HeartbeatDetails {
    /// Heartbeat address.
    pub hb_address: Option<String>,
    /// Key dilution for heartbeat.
    pub hb_key_dilution: Option<u64>,
    /// Heartbeat proof (Base64 encoded).
    pub hb_proof: Option<String>,
    /// Heartbeat seed (Base64 encoded).
    pub hb_seed: Option<String>,
    /// Heartbeat vote ID (Base64 encoded).
    pub hb_vote_id: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests OnComplete string conversion in both directions.
    #[test]
    fn test_on_complete_conversions() {
        // Test as_str
        let as_str_cases = [
            (OnComplete::NoOp, "NoOp"),
            (OnComplete::OptIn, "OptIn"),
            (OnComplete::CloseOut, "CloseOut"),
            (OnComplete::ClearState, "ClearState"),
            (OnComplete::UpdateApplication, "Update"),
            (OnComplete::DeleteApplication, "Delete"),
        ];

        for (variant, expected) in as_str_cases {
            assert_eq!(
                variant.as_str(),
                expected,
                "{:?}.as_str() mismatch",
                variant
            );
        }

        // Test from_str (including aliases and unknown)
        let from_str_cases = [
            ("noop", OnComplete::NoOp),
            ("optin", OnComplete::OptIn),
            ("closeout", OnComplete::CloseOut),
            ("clearstate", OnComplete::ClearState),
            ("updateapplication", OnComplete::UpdateApplication),
            ("update", OnComplete::UpdateApplication),
            ("deleteapplication", OnComplete::DeleteApplication),
            ("delete", OnComplete::DeleteApplication),
            ("unknown", OnComplete::NoOp), // Unknown defaults to NoOp
            ("NOOP", OnComplete::NoOp),    // Case insensitive
        ];

        for (input, expected) in from_str_cases {
            assert_eq!(
                OnComplete::from_str(input),
                expected,
                "OnComplete::from_str({:?}) mismatch",
                input
            );
        }
    }

    /// Tests TransactionDetails default and predicates.
    #[test]
    fn test_transaction_details_behavior() {
        let details = TransactionDetails::default();
        assert_eq!(details, TransactionDetails::None);
        assert!(!details.is_creation());
        assert!(details.created_id().is_none());
    }
}
