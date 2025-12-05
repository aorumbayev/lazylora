//! Transaction types for Algorand blockchain.
//!
//! This module defines all transaction-related types including the main
//! `Transaction` struct and its type-specific details for payments,
//! asset transfers, application calls, and more.

use ratatui::style::Color;
use serde_json::Value;

use super::error::AlgoError;

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
    ///
    /// # Returns
    ///
    /// A static string slice describing the transaction type.
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
    ///
    /// # Returns
    ///
    /// A `ratatui::style::Color` appropriate for this transaction type.
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
    ///
    /// # Arguments
    ///
    /// * `txn_json` - The JSON representation of the transaction
    ///
    /// # Returns
    ///
    /// The appropriate `TxnType` variant based on the JSON structure.
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
    ///
    /// # Returns
    ///
    /// `true` if this is a creation transaction, `false` otherwise.
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
    ///
    /// # Returns
    ///
    /// `Some(id)` if an entity was created, `None` otherwise.
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
    ///
    /// # Returns
    ///
    /// A static string describing the on-complete action.
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
    ///
    /// # Arguments
    ///
    /// * `s` - The string representation of the on-complete type
    ///
    /// # Returns
    ///
    /// The matching `OnComplete` variant, defaulting to `NoOp` for unknown values.
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
    /// Type-specific transaction details.
    pub details: TransactionDetails,
    /// Inner transactions (for app calls).
    pub inner_transactions: Vec<Transaction>,
}

impl Transaction {
    /// Parse a Transaction from JSON data.
    ///
    /// This is the single source of truth for transaction parsing,
    /// consolidating logic that was previously duplicated across multiple methods.
    ///
    /// # Arguments
    ///
    /// * `txn_json` - The JSON representation of the transaction
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Transaction` or an `AlgoError`.
    ///
    /// # Errors
    ///
    /// Returns `AlgoError::Parse` if the JSON structure is invalid.
    pub fn from_json(txn_json: &Value) -> std::result::Result<Self, AlgoError> {
        let id = txn_json["id"].as_str().unwrap_or("unknown").to_string();

        let txn_type = TxnType::from_json(txn_json);

        let from = txn_json["sender"].as_str().unwrap_or("unknown").to_string();

        let to = Self::extract_receiver(txn_json, &txn_type);

        let timestamp = txn_json["round-time"]
            .as_u64()
            .map(format_timestamp)
            .unwrap_or_else(|| "Unknown".to_string());

        let block = txn_json["confirmed-round"].as_u64().unwrap_or(0);
        let fee = txn_json["fee"].as_u64().unwrap_or(0);

        let note = Self::extract_note(txn_json);
        let (amount, asset_id) = Self::extract_amount_and_asset(txn_json, &txn_type);
        let rekey_to = txn_json["rekey-to"].as_str().map(String::from);
        let details = Self::extract_details(txn_json, &txn_type);

        // Parse inner transactions recursively
        let inner_transactions = Self::parse_inner_transactions(txn_json)?;

        Ok(Self {
            id,
            txn_type,
            from,
            to,
            timestamp,
            block,
            fee,
            note,
            amount,
            asset_id,
            rekey_to,
            details,
            inner_transactions,
        })
    }

    /// Parse inner transactions from the JSON data.
    fn parse_inner_transactions(
        txn_json: &Value,
    ) -> std::result::Result<Vec<Transaction>, AlgoError> {
        let inner_txns_json = txn_json.get("inner-txns");

        match inner_txns_json {
            Some(Value::Array(arr)) => {
                let mut inner_txns = Vec::with_capacity(arr.len());
                for inner_json in arr {
                    // Recursively parse inner transaction
                    let inner_txn = Self::from_json(inner_json)?;
                    inner_txns.push(inner_txn);
                }
                Ok(inner_txns)
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Extract the receiver address based on transaction type.
    #[must_use]
    fn extract_receiver(txn_json: &Value, txn_type: &TxnType) -> String {
        match txn_type {
            TxnType::Payment => txn_json["payment-transaction"]["receiver"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            TxnType::AssetTransfer => txn_json["asset-transfer-transaction"]["receiver"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            TxnType::AssetConfig => {
                if txn_json["asset-config-transaction"]["params"].is_object() {
                    txn_json["asset-config-transaction"]["params"]["manager"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string()
                } else {
                    "unknown".to_string()
                }
            }
            TxnType::AssetFreeze => txn_json["asset-freeze-transaction"]["address"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            TxnType::AppCall => {
                let app_id = txn_json["application-transaction"]["application-id"]
                    .as_u64()
                    .unwrap_or(0);
                if app_id > 0 {
                    txn_json["application-transaction"]["application-id"].to_string()
                } else {
                    txn_json["created-application-index"].to_string()
                }
            }
            _ => "unknown".to_string(),
        }
    }

    /// Extract note from transaction JSON.
    #[must_use]
    fn extract_note(txn_json: &Value) -> String {
        txn_json["note"]
            .as_str()
            .map(|n| n.to_string())
            .unwrap_or_else(|| {
                txn_json["note"]
                    .as_array()
                    .map(|bytes| format!("{:?}", bytes))
                    .unwrap_or_else(|| "None".to_string())
            })
    }

    /// Extract amount and asset ID based on transaction type.
    #[must_use]
    fn extract_amount_and_asset(txn_json: &Value, txn_type: &TxnType) -> (u64, Option<u64>) {
        match txn_type {
            TxnType::Payment => {
                let amount = txn_json["payment-transaction"]["amount"]
                    .as_u64()
                    .unwrap_or(0);
                (amount, None)
            }
            TxnType::AssetTransfer => {
                let amount = txn_json["asset-transfer-transaction"]["amount"]
                    .as_u64()
                    .unwrap_or(0);
                let asset_id = txn_json["asset-transfer-transaction"]["asset-id"].as_u64();
                (amount, asset_id)
            }
            _ => (0, None),
        }
    }

    /// Extract type-specific transaction details.
    #[must_use]
    fn extract_details(txn_json: &Value, txn_type: &TxnType) -> TransactionDetails {
        match txn_type {
            TxnType::Payment => Self::extract_payment_details(txn_json),
            TxnType::AssetTransfer => Self::extract_asset_transfer_details(txn_json),
            TxnType::AssetConfig => Self::extract_asset_config_details(txn_json),
            TxnType::AssetFreeze => Self::extract_asset_freeze_details(txn_json),
            TxnType::AppCall => Self::extract_app_call_details(txn_json),
            TxnType::KeyReg => Self::extract_keyreg_details(txn_json),
            TxnType::StateProof => Self::extract_state_proof_details(txn_json),
            TxnType::Heartbeat => Self::extract_heartbeat_details(txn_json),
            TxnType::Unknown => TransactionDetails::None,
        }
    }

    /// Extract payment transaction details.
    #[must_use]
    fn extract_payment_details(txn_json: &Value) -> TransactionDetails {
        let pay = &txn_json["payment-transaction"];
        TransactionDetails::Payment(PaymentDetails {
            close_remainder_to: pay["close-remainder-to"].as_str().map(String::from),
            close_amount: pay["close-amount"].as_u64(),
        })
    }

    /// Extract asset transfer transaction details.
    #[must_use]
    fn extract_asset_transfer_details(txn_json: &Value) -> TransactionDetails {
        let axfer = &txn_json["asset-transfer-transaction"];
        TransactionDetails::AssetTransfer(AssetTransferDetails {
            asset_sender: axfer["sender"].as_str().map(String::from),
            close_to: axfer["close-to"].as_str().map(String::from),
            close_amount: axfer["close-amount"].as_u64(),
        })
    }

    /// Extract asset configuration transaction details.
    #[must_use]
    fn extract_asset_config_details(txn_json: &Value) -> TransactionDetails {
        let acfg = &txn_json["asset-config-transaction"];
        let params = &acfg["params"];

        TransactionDetails::AssetConfig(AssetConfigDetails {
            asset_id: acfg["asset-id"].as_u64(),
            created_asset_id: txn_json["created-asset-index"].as_u64(),
            total: params["total"].as_u64(),
            decimals: params["decimals"].as_u64(),
            default_frozen: params["default-frozen"].as_bool(),
            asset_name: params["name"].as_str().map(String::from),
            unit_name: params["unit-name"].as_str().map(String::from),
            url: params["url"].as_str().map(String::from),
            metadata_hash: params["metadata-hash"].as_str().map(String::from),
            manager: params["manager"].as_str().map(String::from),
            reserve: params["reserve"].as_str().map(String::from),
            freeze: params["freeze"].as_str().map(String::from),
            clawback: params["clawback"].as_str().map(String::from),
        })
    }

    /// Extract asset freeze transaction details.
    #[must_use]
    fn extract_asset_freeze_details(txn_json: &Value) -> TransactionDetails {
        let afrz = &txn_json["asset-freeze-transaction"];
        TransactionDetails::AssetFreeze(AssetFreezeDetails {
            frozen: afrz["new-freeze-status"].as_bool().unwrap_or(false),
            freeze_target: afrz["address"].as_str().unwrap_or("unknown").to_string(),
        })
    }

    /// Extract application call transaction details.
    #[must_use]
    fn extract_app_call_details(txn_json: &Value) -> TransactionDetails {
        let appl = &txn_json["application-transaction"];

        let on_complete = appl["on-completion"]
            .as_str()
            .map(OnComplete::from_str)
            .unwrap_or_default();

        let app_args = appl["application-args"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let accounts = appl["accounts"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let foreign_apps = appl["foreign-apps"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_u64()).collect())
            .unwrap_or_default();

        let foreign_assets = appl["foreign-assets"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_u64()).collect())
            .unwrap_or_default();

        let boxes = appl["boxes"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|b| BoxRef {
                        app_id: b["i"].as_u64().unwrap_or(0),
                        name: b["n"].as_str().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let global_state_schema = if appl["global-state-schema"].is_object() {
            Some(StateSchema {
                num_uint: appl["global-state-schema"]["num-uint"]
                    .as_u64()
                    .unwrap_or(0),
                num_byte_slice: appl["global-state-schema"]["num-byte-slice"]
                    .as_u64()
                    .unwrap_or(0),
            })
        } else {
            None
        };

        let local_state_schema = if appl["local-state-schema"].is_object() {
            Some(StateSchema {
                num_uint: appl["local-state-schema"]["num-uint"].as_u64().unwrap_or(0),
                num_byte_slice: appl["local-state-schema"]["num-byte-slice"]
                    .as_u64()
                    .unwrap_or(0),
            })
        } else {
            None
        };

        TransactionDetails::AppCall(AppCallDetails {
            app_id: appl["application-id"].as_u64().unwrap_or(0),
            created_app_id: txn_json["created-application-index"].as_u64(),
            on_complete,
            approval_program: appl["approval-program"].as_str().map(String::from),
            clear_state_program: appl["clear-state-program"].as_str().map(String::from),
            app_args,
            accounts,
            foreign_apps,
            foreign_assets,
            boxes,
            global_state_schema,
            local_state_schema,
            extra_program_pages: appl["extra-program-pages"].as_u64(),
        })
    }

    /// Extract key registration transaction details.
    #[must_use]
    fn extract_keyreg_details(txn_json: &Value) -> TransactionDetails {
        let keyreg = &txn_json["keyreg-transaction"];
        TransactionDetails::KeyReg(KeyRegDetails {
            vote_key: keyreg["vote-participation-key"].as_str().map(String::from),
            selection_key: keyreg["selection-participation-key"]
                .as_str()
                .map(String::from),
            state_proof_key: keyreg["state-proof-key"].as_str().map(String::from),
            vote_first: keyreg["vote-first-valid"].as_u64(),
            vote_last: keyreg["vote-last-valid"].as_u64(),
            vote_key_dilution: keyreg["vote-key-dilution"].as_u64(),
            non_participation: keyreg["non-participation"].as_bool().unwrap_or(false),
        })
    }

    /// Extract state proof transaction details.
    #[must_use]
    fn extract_state_proof_details(txn_json: &Value) -> TransactionDetails {
        let sp = &txn_json["state-proof-transaction"];
        TransactionDetails::StateProof(StateProofDetails {
            state_proof_type: sp["state-proof-type"].as_u64(),
            message: sp["message"].as_str().map(String::from),
        })
    }

    /// Extract heartbeat transaction details.
    #[must_use]
    fn extract_heartbeat_details(txn_json: &Value) -> TransactionDetails {
        let hb = &txn_json["heartbeat-transaction"];
        TransactionDetails::Heartbeat(HeartbeatDetails {
            hb_address: hb["hb-address"].as_str().map(String::from),
            hb_key_dilution: hb["hb-key-dilution"].as_u64(),
            hb_proof: hb["hb-proof"].as_str().map(String::from),
            hb_seed: hb["hb-seed"].as_str().map(String::from),
            hb_vote_id: hb["hb-vote-id"].as_str().map(String::from),
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_txn_type_as_str() {
        assert_eq!(TxnType::Payment.as_str(), "Payment");
        assert_eq!(TxnType::AppCall.as_str(), "App Call");
        assert_eq!(TxnType::AssetTransfer.as_str(), "Asset Transfer");
        assert_eq!(TxnType::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_txn_type_color() {
        assert_eq!(TxnType::Payment.color(), Color::Green);
        assert_eq!(TxnType::AppCall.color(), Color::Blue);
        assert_eq!(TxnType::Unknown.color(), Color::DarkGray);
    }

    #[test]
    fn test_on_complete_as_str() {
        assert_eq!(OnComplete::NoOp.as_str(), "NoOp");
        assert_eq!(OnComplete::OptIn.as_str(), "OptIn");
        assert_eq!(OnComplete::CloseOut.as_str(), "CloseOut");
        assert_eq!(OnComplete::ClearState.as_str(), "ClearState");
        assert_eq!(OnComplete::UpdateApplication.as_str(), "Update");
        assert_eq!(OnComplete::DeleteApplication.as_str(), "Delete");
    }

    #[test]
    fn test_on_complete_from_str() {
        assert_eq!(OnComplete::from_str("noop"), OnComplete::NoOp);
        assert_eq!(OnComplete::from_str("optin"), OnComplete::OptIn);
        assert_eq!(OnComplete::from_str("closeout"), OnComplete::CloseOut);
        assert_eq!(OnComplete::from_str("clearstate"), OnComplete::ClearState);
        assert_eq!(
            OnComplete::from_str("updateapplication"),
            OnComplete::UpdateApplication
        );
        assert_eq!(
            OnComplete::from_str("update"),
            OnComplete::UpdateApplication
        );
        assert_eq!(
            OnComplete::from_str("deleteapplication"),
            OnComplete::DeleteApplication
        );
        assert_eq!(
            OnComplete::from_str("delete"),
            OnComplete::DeleteApplication
        );
        assert_eq!(OnComplete::from_str("unknown"), OnComplete::NoOp);
    }

    #[test]
    fn test_transaction_details_default() {
        let details = TransactionDetails::default();
        assert_eq!(details, TransactionDetails::None);
        assert!(!details.is_creation());
        assert!(details.created_id().is_none());
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "Timestamp not available");
        // Non-zero timestamp should produce a formatted string
        let result = format_timestamp(1_700_000_000);
        assert!(result.contains("2023")); // Should be a date in 2023
    }

    #[test]
    fn test_transaction_from_json_payment() {
        let json = serde_json::json!({
            "id": "test-txn-id",
            "sender": "SENDER_ADDRESS",
            "round-time": 1_700_000_000_u64,
            "confirmed-round": 12345_u64,
            "fee": 1000_u64,
            "payment-transaction": {
                "amount": 5_000_000_u64,
                "receiver": "RECEIVER_ADDRESS"
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.id, "test-txn-id");
        assert_eq!(txn.txn_type, TxnType::Payment);
        assert_eq!(txn.from, "SENDER_ADDRESS");
        assert_eq!(txn.to, "RECEIVER_ADDRESS");
        assert_eq!(txn.amount, 5_000_000);
        assert_eq!(txn.fee, 1000);
        assert!(txn.asset_id.is_none());

        match txn.details {
            TransactionDetails::Payment(details) => {
                assert!(details.close_remainder_to.is_none());
                assert!(details.close_amount.is_none());
            }
            _ => panic!("Expected Payment details"),
        }
    }

    #[test]
    fn test_transaction_from_json_defaults() {
        let json = serde_json::json!({});

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.id, "unknown");
        assert_eq!(txn.from, "unknown");
        assert_eq!(txn.txn_type, TxnType::Unknown);
        assert_eq!(txn.amount, 0);
        assert_eq!(txn.fee, 0);
        assert_eq!(txn.details, TransactionDetails::None);
    }
}
