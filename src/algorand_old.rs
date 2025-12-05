use crate::state::SearchType;
use color_eyre::Result;
use ratatui::style::Color;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Custom error type for Algorand client operations
#[derive(Debug, Error)]
pub enum AlgoError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Parse error: {message}")]
    Parse { message: String },

    #[error("{entity} '{id}' not found")]
    NotFound { entity: &'static str, id: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl AlgoError {
    /// Create a new parse error with the given message
    #[must_use]
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }

    /// Create a new not found error
    #[must_use]
    pub fn not_found(entity: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity,
            id: id.into(),
        }
    }

    /// Create a new invalid input error
    #[must_use]
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Convert to a color_eyre::Report for API compatibility
    #[must_use = "this converts the error into a Report for display"]
    pub fn into_report(self) -> color_eyre::Report {
        color_eyre::eyre::eyre!("{}", self)
    }
}

// ============================================================================
// Network Configuration
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum Network {
    MainNet,
    TestNet,
    LocalNet,
}

impl Network {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::MainNet => "MainNet",
            Self::TestNet => "TestNet",
            Self::LocalNet => "LocalNet",
        }
    }

    #[must_use]
    pub const fn indexer_url(&self) -> &str {
        match self {
            Self::MainNet => "https://mainnet-idx.algonode.cloud",
            Self::TestNet => "https://testnet-idx.algonode.cloud",
            Self::LocalNet => "http://localhost:8980",
        }
    }

    #[must_use]
    pub const fn algod_url(&self) -> &str {
        match self {
            Self::MainNet => "https://mainnet-api.algonode.cloud",
            Self::TestNet => "https://testnet-api.algonode.cloud",
            Self::LocalNet => "http://localhost:4001",
        }
    }

    /// Returns the NFD API base URL for the network.
    /// NFD is only available on MainNet and TestNet.
    #[must_use]
    pub const fn nfd_api_url(&self) -> Option<&str> {
        match self {
            Self::MainNet => Some("https://api.nf.domains"),
            Self::TestNet => Some("https://api.testnet.nf.domains"),
            Self::LocalNet => None, // NFD not available on LocalNet
        }
    }

    /// Returns whether NFD lookups are supported on this network.
    #[must_use]
    pub const fn supports_nfd(&self) -> bool {
        matches!(self, Self::MainNet | Self::TestNet)
    }
}

// ============================================================================
// Transaction Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TxnType {
    Payment,
    AppCall,
    AssetTransfer,
    AssetConfig,
    AssetFreeze,
    KeyReg,
    StateProof,
    Heartbeat,
    Unknown,
}

impl TxnType {
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

    /// Determine transaction type from JSON data
    #[must_use]
    fn from_json(txn_json: &Value) -> Self {
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

// ============================================================================
// Transaction Details - Type-specific metadata
// ============================================================================

/// Type-specific transaction details
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TransactionDetails {
    #[default]
    None,
    Payment(PaymentDetails),
    AssetTransfer(AssetTransferDetails),
    AssetConfig(AssetConfigDetails),
    AssetFreeze(AssetFreezeDetails),
    AppCall(AppCallDetails),
    KeyReg(KeyRegDetails),
    StateProof(StateProofDetails),
    Heartbeat(HeartbeatDetails),
}

impl TransactionDetails {
    /// Returns true if this transaction creates something (app, asset)
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn is_creation(&self) -> bool {
        match self {
            Self::AssetConfig(details) => details.asset_id.is_none() && details.total.is_some(),
            Self::AppCall(details) => details.app_id == 0,
            _ => false,
        }
    }

    /// Returns the created entity ID if this was a creation transaction
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn created_id(&self) -> Option<u64> {
        match self {
            Self::AssetConfig(details) => details.created_asset_id,
            Self::AppCall(details) => details.created_app_id,
            _ => None,
        }
    }
}

/// Payment transaction details
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PaymentDetails {
    /// Address to receive remaining funds when closing out
    pub close_remainder_to: Option<String>,
    /// Amount sent to close-to address
    pub close_amount: Option<u64>,
}

/// Asset transfer transaction details
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AssetTransferDetails {
    /// For clawback transactions, the address being clawed back from
    pub asset_sender: Option<String>,
    /// Address to receive remaining asset holdings when closing out
    pub close_to: Option<String>,
    /// Amount of asset sent to close-to address
    pub close_amount: Option<u64>,
}

/// Asset configuration transaction details
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AssetConfigDetails {
    /// Asset ID for modify/destroy (None for create)
    pub asset_id: Option<u64>,
    /// Set after creation - the ID of the created asset
    pub created_asset_id: Option<u64>,
    /// Total number of units of this asset
    pub total: Option<u64>,
    /// Number of decimal places for asset display
    pub decimals: Option<u64>,
    /// Whether asset holdings are frozen by default
    pub default_frozen: Option<bool>,
    /// Asset name
    pub asset_name: Option<String>,
    /// Asset unit name
    pub unit_name: Option<String>,
    /// URL with asset metadata
    pub url: Option<String>,
    /// Hash of metadata (32 bytes)
    pub metadata_hash: Option<String>,
    /// Manager address - can change asset config
    pub manager: Option<String>,
    /// Reserve address - holds non-minted units
    pub reserve: Option<String>,
    /// Freeze address - can freeze/unfreeze holdings
    pub freeze: Option<String>,
    /// Clawback address - can revoke holdings
    pub clawback: Option<String>,
}

/// Asset freeze transaction details
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AssetFreezeDetails {
    /// Whether the target is being frozen or unfrozen
    pub frozen: bool,
    /// Address whose asset holdings are being frozen/unfrozen
    pub freeze_target: String,
}

/// Application call transaction details
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppCallDetails {
    /// Application ID (0 for creation)
    pub app_id: u64,
    /// Set after creation - the ID of the created application
    pub created_app_id: Option<u64>,
    /// Type of application call
    pub on_complete: OnComplete,
    /// Approval program (Base64 encoded)
    pub approval_program: Option<String>,
    /// Clear state program (Base64 encoded)
    pub clear_state_program: Option<String>,
    /// Application arguments (Base64 encoded)
    pub app_args: Vec<String>,
    /// Referenced accounts
    pub accounts: Vec<String>,
    /// Referenced applications
    pub foreign_apps: Vec<u64>,
    /// Referenced assets
    pub foreign_assets: Vec<u64>,
    /// Box references
    pub boxes: Vec<BoxRef>,
    /// Global state schema for app creation
    pub global_state_schema: Option<StateSchema>,
    /// Local state schema for app creation
    pub local_state_schema: Option<StateSchema>,
    /// Extra program pages for large programs
    pub extra_program_pages: Option<u64>,
}

/// Application call on-completion type
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum OnComplete {
    #[default]
    NoOp,
    OptIn,
    CloseOut,
    ClearState,
    UpdateApplication,
    DeleteApplication,
}

impl OnComplete {
    /// Returns the string representation of the on-complete type
    #[must_use]
    #[allow(dead_code)] // Public API for external use
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

    /// Parse on-complete type from string
    #[must_use]
    fn from_str(s: &str) -> Self {
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

/// Box reference for application calls
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BoxRef {
    /// Application ID (0 means current app)
    pub app_id: u64,
    /// Box name (Base64 encoded)
    pub name: String,
}

/// State schema for application storage
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StateSchema {
    /// Number of uint64 values
    pub num_uint: u64,
    /// Number of byte slice values
    pub num_byte_slice: u64,
}

/// Key registration transaction details
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KeyRegDetails {
    /// Voting public key (Base64 encoded)
    pub vote_key: Option<String>,
    /// VRF selection public key (Base64 encoded)
    pub selection_key: Option<String>,
    /// State proof public key (Base64 encoded)
    pub state_proof_key: Option<String>,
    /// First round for which this key is valid
    pub vote_first: Option<u64>,
    /// Last round for which this key is valid
    pub vote_last: Option<u64>,
    /// Key dilution for voting key
    pub vote_key_dilution: Option<u64>,
    /// Whether this marks the account as non-participating
    pub non_participation: bool,
}

/// State proof transaction details
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StateProofDetails {
    /// Type of state proof
    pub state_proof_type: Option<u64>,
    /// State proof message (hex encoded)
    pub message: Option<String>,
}

/// Heartbeat transaction details
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HeartbeatDetails {
    /// Heartbeat address
    pub hb_address: Option<String>,
    /// Key dilution for heartbeat
    pub hb_key_dilution: Option<u64>,
    /// Heartbeat proof (Base64 encoded)
    pub hb_proof: Option<String>,
    /// Heartbeat seed (Base64 encoded)
    pub hb_seed: Option<String>,
    /// Heartbeat vote ID (Base64 encoded)
    pub hb_vote_id: Option<String>,
}

// ============================================================================
// Transaction
// ============================================================================

/// Represents an Algorand transaction with all its metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    /// Transaction ID (52-character base32 string)
    pub id: String,
    /// Transaction type
    pub txn_type: TxnType,
    /// Sender address
    pub from: String,
    /// Receiver address (or app ID for app calls)
    pub to: String,
    /// Human-readable timestamp
    pub timestamp: String,
    /// Block number where the transaction was confirmed
    pub block: u64,
    /// Transaction fee in microAlgos
    pub fee: u64,
    /// Transaction note (may be Base64 encoded)
    pub note: String,
    /// Amount transferred (in microAlgos or asset units)
    pub amount: u64,
    /// Asset ID for asset-related transactions
    pub asset_id: Option<u64>,
    /// Rekey-to address (if this transaction rekeys the sender)
    pub rekey_to: Option<String>,
    /// Type-specific transaction details
    pub details: TransactionDetails,
    /// Inner transactions (for app calls)
    pub inner_transactions: Vec<Transaction>,
}

impl Transaction {
    /// Parse a Transaction from JSON data.
    ///
    /// This is the single source of truth for transaction parsing,
    /// consolidating logic that was previously duplicated across multiple methods.
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

    /// Parse inner transactions from the JSON data
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

    /// Extract the receiver address based on transaction type
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

    /// Extract note from transaction JSON
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

    /// Extract amount and asset ID based on transaction type
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

    /// Extract type-specific transaction details
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

    /// Extract payment transaction details
    #[must_use]
    fn extract_payment_details(txn_json: &Value) -> TransactionDetails {
        let pay = &txn_json["payment-transaction"];
        TransactionDetails::Payment(PaymentDetails {
            close_remainder_to: pay["close-remainder-to"].as_str().map(String::from),
            close_amount: pay["close-amount"].as_u64(),
        })
    }

    /// Extract asset transfer transaction details
    #[must_use]
    fn extract_asset_transfer_details(txn_json: &Value) -> TransactionDetails {
        let axfer = &txn_json["asset-transfer-transaction"];
        TransactionDetails::AssetTransfer(AssetTransferDetails {
            asset_sender: axfer["sender"].as_str().map(String::from),
            close_to: axfer["close-to"].as_str().map(String::from),
            close_amount: axfer["close-amount"].as_u64(),
        })
    }

    /// Extract asset configuration transaction details
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

    /// Extract asset freeze transaction details
    #[must_use]
    fn extract_asset_freeze_details(txn_json: &Value) -> TransactionDetails {
        let afrz = &txn_json["asset-freeze-transaction"];
        TransactionDetails::AssetFreeze(AssetFreezeDetails {
            frozen: afrz["new-freeze-status"].as_bool().unwrap_or(false),
            freeze_target: afrz["address"].as_str().unwrap_or("unknown").to_string(),
        })
    }

    /// Extract application call transaction details
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

    /// Extract key registration transaction details
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

    /// Extract state proof transaction details
    #[must_use]
    fn extract_state_proof_details(txn_json: &Value) -> TransactionDetails {
        let sp = &txn_json["state-proof-transaction"];
        TransactionDetails::StateProof(StateProofDetails {
            state_proof_type: sp["state-proof-type"].as_u64(),
            message: sp["message"].as_str().map(String::from),
        })
    }

    /// Extract heartbeat transaction details
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
// Block Types
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct AlgoBlock {
    pub id: u64,
    pub txn_count: u16,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockInfo {
    pub id: u64,
    pub timestamp: String,
    pub txn_count: u16,
    pub proposer: String,
    pub seed: String,
}

/// Extended block details including transactions
#[derive(Debug, Clone, PartialEq)]
pub struct BlockDetails {
    /// Basic block info
    pub info: BlockInfo,
    /// Transactions in this block
    pub transactions: Vec<Transaction>,
    /// Count of transactions by type
    pub txn_type_counts: std::collections::HashMap<TxnType, usize>,
}

// ============================================================================
// Account & Asset Types
// ============================================================================

/// Basic account info for search results display
#[derive(Debug, Clone, PartialEq)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u64,         // In microAlgos
    pub pending_rewards: u64, // In microAlgos
    pub reward_base: u64,
    pub status: String,              // e.g., "Offline", "Online"
    pub assets_count: usize,         // Number of assets the account holds
    pub created_assets_count: usize, // Number of assets created by the account
}

/// Detailed account information for popup display
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AccountDetails {
    pub address: String,
    pub balance: u64,         // In microAlgos
    pub min_balance: u64,     // Minimum balance required
    pub pending_rewards: u64, // In microAlgos
    pub rewards: u64,         // Total rewards earned
    pub reward_base: u64,
    pub status: String,                           // e.g., "Offline", "Online"
    pub total_apps_opted_in: usize,               // Number of apps opted into
    pub total_assets_opted_in: usize,             // Number of assets opted into
    pub total_created_apps: usize,                // Number of apps created
    pub total_created_assets: usize,              // Number of assets created
    pub total_boxes: usize,                       // Number of boxes
    pub auth_addr: Option<String>,                // Authorized address (rekeyed)
    pub participation: Option<ParticipationInfo>, // Online participation info
    pub assets: Vec<AccountAssetHolding>,         // Asset holdings (limited)
    pub created_assets: Vec<CreatedAssetInfo>,    // Created assets (limited)
    pub apps_local_state: Vec<AppLocalState>,     // App local states (limited)
    pub created_apps: Vec<CreatedAppInfo>,        // Created apps (limited)
    pub nfd: Option<NfdInfo>,                     // NFD name if available (MainNet/TestNet only)
}

/// Participation key info for online accounts
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParticipationInfo {
    pub vote_first: u64,
    pub vote_last: u64,
    pub vote_key_dilution: u64,
    pub selection_key: String,
    pub vote_key: String,
    pub state_proof_key: Option<String>,
}

/// Asset holding info for an account
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AccountAssetHolding {
    pub asset_id: u64,
    pub amount: u64,
    pub is_frozen: bool,
}

/// Created asset summary
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CreatedAssetInfo {
    pub asset_id: u64,
    pub name: String,
    pub unit_name: String,
}

/// App local state summary
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppLocalState {
    pub app_id: u64,
    pub schema_num_uint: u64,
    pub schema_num_byte_slice: u64,
}

/// Created app summary
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CreatedAppInfo {
    pub app_id: u64,
}

/// Basic asset info for search results display
#[derive(Debug, Clone, PartialEq)]
pub struct AssetInfo {
    pub id: u64,
    pub name: String,
    pub unit_name: String,
    pub creator: String,
    pub total: u64,    // Total supply
    pub decimals: u64, // For display formatting
    pub url: String,   // Optional URL for metadata
}

/// Detailed asset information for popup display
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AssetDetails {
    pub id: u64,
    pub name: String,
    pub unit_name: String,
    pub creator: String,
    pub total: u64,                    // Total supply
    pub decimals: u64,                 // For display formatting
    pub url: String,                   // Optional URL for metadata
    pub metadata_hash: Option<String>, // Metadata hash (base64)
    pub default_frozen: bool,
    pub manager: Option<String>,
    pub reserve: Option<String>,
    pub freeze: Option<String>,
    pub clawback: Option<String>,
    pub deleted: bool,
    pub created_at_round: Option<u64>,
}

// ============================================================================
// NFD (NFDomains) Types
// ============================================================================

/// NFD (Non-Fungible Domain) information from the NFD API.
/// This is a simplified view of the NFD data for display purposes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NfdInfo {
    /// The NFD name (e.g., "alice.algo")
    pub name: String,
    /// The deposit account address linked to this NFD
    pub deposit_account: Option<String>,
    /// The owner address of this NFD
    pub owner: Option<String>,
    /// Avatar URL if available
    pub avatar_url: Option<String>,
    /// Whether this is a verified NFD
    pub is_verified: bool,
}

impl NfdInfo {
    /// Create a new NFD info from API response JSON
    #[must_use]
    fn from_json(json: &Value) -> Self {
        let name = json["name"].as_str().unwrap_or("").to_string();
        let deposit_account = json["depositAccount"].as_str().map(String::from);
        let owner = json["owner"].as_str().map(String::from);

        // Avatar can be in properties.userDefined.avatar or properties.verified.avatar
        let avatar_url = json["properties"]["verified"]["avatar"]
            .as_str()
            .or_else(|| json["properties"]["userDefined"]["avatar"].as_str())
            .map(String::from);

        // Check if there are verified caAlgo addresses (indicates verification)
        let is_verified = json["caAlgo"].as_array().is_some_and(|arr| !arr.is_empty());

        Self {
            name,
            deposit_account,
            owner,
            avatar_url,
            is_verified,
        }
    }
}

// ============================================================================
// Search Results
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum SearchResultItem {
    Transaction(Box<Transaction>),
    Block(BlockInfo),
    Account(AccountInfo),
    Asset(AssetInfo),
}

// ============================================================================
// Algorand Client
// ============================================================================

#[derive(Debug, Clone)]
pub struct AlgoClient {
    network: Network,
    client: Client,
}

impl AlgoClient {
    #[must_use]
    pub fn new(network: Network) -> Self {
        Self {
            network,
            client: Client::new(),
        }
    }

    fn build_algod_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.get(url).header("accept", "application/json");

        if self.network == Network::LocalNet {
            request = request.header(
                "X-Algo-API-Token",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            );
        }

        request
    }

    fn build_indexer_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.get(url).header("accept", "application/json");

        if self.network == Network::LocalNet {
            request = request.header(
                "X-Indexer-API-Token",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            );
        }

        request
    }

    pub async fn get_network_status(&self) -> std::result::Result<(), String> {
        let algod_url = format!("{}/health", self.network.algod_url());
        let indexer_url = format!("{}/health", self.network.indexer_url());

        let algod_result = self
            .build_algod_request(&algod_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;

        let indexer_result = self
            .build_indexer_request(&indexer_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;

        if let Err(e) = algod_result {
            return Err(format!(
                "Unable to connect to algod at {}. Error: {}",
                self.network.algod_url(),
                e
            ));
        }

        if self.network == Network::LocalNet && indexer_result.is_err() {
            return Err(format!(
                "Unable to connect to indexer at {}. Algod is running but indexer is not available.",
                self.network.indexer_url()
            ));
        }

        Ok(())
    }

    /// Fetch a single transaction by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or JSON parsing fails.
    #[allow(dead_code)]
    pub async fn get_transaction_by_id(&self, txid: &str) -> Result<Option<Transaction>> {
        let url = format!("{}/v2/transactions/{}", self.network.indexer_url(), txid);
        let response = self
            .build_indexer_request(&url)
            .send()
            .await
            .map_err(AlgoError::Network)?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let json: Value = response
            .json()
            .await
            .map_err(|_| AlgoError::parse("Failed to parse transaction JSON").into_report())?;

        let txn_json = match json.get("transaction") {
            Some(txn) => txn,
            None => return Ok(None),
        };

        Transaction::from_json(txn_json)
            .map(Some)
            .map_err(AlgoError::into_report)
    }

    /// Fetch the latest blocks from the network
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails.
    pub async fn get_latest_blocks(&self, limit: usize) -> Result<Vec<AlgoBlock>> {
        let status_url = format!("{}/v2/status", self.network.algod_url());
        let status_response = self.build_algod_request(&status_url).send().await?;

        let status: Value = status_response.json().await?;
        let latest_round = status["last-round"].as_u64().unwrap_or(0);

        if latest_round == 0 {
            return Ok(Vec::new());
        }

        let mut blocks = Vec::with_capacity(limit);

        for i in 0..limit {
            if i >= latest_round as usize {
                break;
            }

            let round = latest_round - i as u64;
            let block_url = format!("{}/v2/blocks/{}", self.network.algod_url(), round);

            let response = match self.build_algod_request(&block_url).send().await {
                Ok(resp) if resp.status().is_success() => resp,
                _ => continue,
            };

            let block_data: Value = match response.json().await {
                Ok(data) => data,
                Err(_) => continue,
            };

            let block = block_data.get("block").unwrap_or(&block_data);
            let timestamp_secs = block["ts"].as_u64().unwrap_or(0);
            let formatted_time = format_timestamp(timestamp_secs);
            let txn_count = count_transactions(block);

            blocks.push(AlgoBlock {
                id: round,
                txn_count,
                timestamp: formatted_time,
            });
        }

        Ok(blocks)
    }

    /// Fetch the latest transactions from the network
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails.
    pub async fn get_latest_transactions(&self, limit: usize) -> Result<Vec<Transaction>> {
        let status_url = format!("{}/v2/status", self.network.algod_url());
        let status_response = match self.build_algod_request(&status_url).send().await {
            Ok(resp) if resp.status().is_success() => resp,
            _ => return Ok(Vec::new()),
        };

        let status: Value = match status_response.json().await {
            Ok(data) => data,
            Err(_) => return Ok(Vec::new()),
        };

        let latest_round = status["last-round"].as_u64().unwrap_or(0);
        if latest_round == 0 {
            return Ok(Vec::new());
        }

        let min_round = latest_round.saturating_sub(20);
        let url = format!(
            "{}/v2/transactions?limit={}&min-round={}&max-round={}&order=desc",
            self.network.indexer_url(),
            limit,
            min_round,
            latest_round
        );

        let response = match self.build_indexer_request(&url).send().await {
            Ok(resp) if resp.status().is_success() => resp,
            _ => return Ok(Vec::new()),
        };

        let json: Value = match response.json().await {
            Ok(data) => data,
            Err(_) => return Ok(Vec::new()),
        };

        let mut transactions = parse_transactions_array(&json)?;
        transactions.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(transactions)
    }

    /// Search by query with specified search type
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails or entity is not found.
    pub async fn search_by_query(
        &self,
        query: &str,
        search_type: SearchType,
    ) -> Result<Vec<SearchResultItem>> {
        let results = match search_type {
            SearchType::Transaction => {
                let txns = self.search_transaction(query).await?;
                txns.into_iter()
                    .map(|t| SearchResultItem::Transaction(Box::new(t)))
                    .collect()
            }
            SearchType::Account => match self.search_address(query).await {
                Ok(Some(account)) => {
                    vec![SearchResultItem::Account(account)]
                }
                Ok(None) => {
                    vec![]
                }
                Err(e) => {
                    return Err(e);
                }
            },
            SearchType::Block => match self.search_block(query).await? {
                Some(block) => vec![SearchResultItem::Block(block)],
                None => {
                    return Err(color_eyre::eyre::eyre!(
                        "Block '{}' not found. Please enter a valid block number.",
                        query
                    ));
                }
            },
            SearchType::Asset => match self.search_asset(query).await? {
                Some(asset) => vec![SearchResultItem::Asset(asset)],
                None => {
                    return Err(color_eyre::eyre::eyre!(
                        "Asset '{}' not found. Please enter a valid asset ID.",
                        query
                    ));
                }
            },
        };

        Ok(results)
    }

    async fn search_transaction(&self, txid: &str) -> Result<Vec<Transaction>> {
        if txid.is_empty() {
            return Err(AlgoError::invalid_input("Transaction ID cannot be empty").into_report());
        }

        if txid.len() < 40 || txid.len() > 60 {
            return Err(AlgoError::invalid_input(
                "Invalid transaction ID format. Transaction IDs are typically 52 characters long.",
            )
            .into_report());
        }

        let url = format!("{}/v2/transactions/{}", self.network.indexer_url(), txid);

        let response = self.build_indexer_request(&url).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(json) = resp.json::<Value>().await
                    && let Some(txn_json) = json.get("transaction")
                {
                    let transaction =
                        Transaction::from_json(txn_json).map_err(AlgoError::into_report)?;
                    return Ok(vec![transaction]);
                }
            }
            Ok(resp) => {
                let status = resp.status();
                if status.as_u16() != 404 {}
            }
            Err(_) => {}
        }

        let search_url = format!(
            "{}/v2/transactions?txid={}&limit=10",
            self.network.indexer_url(),
            txid
        );

        let search_results = self.fetch_transactions_from_url(&search_url).await?;

        if search_results.is_empty() {
            return Err(AlgoError::not_found("transaction", txid).into_report());
        }

        Ok(search_results)
    }

    async fn search_block(&self, round_str: &str) -> Result<Option<BlockInfo>> {
        let round = round_str.parse::<u64>().map_err(|_| {
            AlgoError::invalid_input(format!(
                "Invalid block number '{}'. Please enter a valid positive integer.",
                round_str
            ))
            .into_report()
        })?;

        let block_url = format!("{}/v2/blocks/{}", self.network.algod_url(), round);

        let response = self.build_algod_request(&block_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 404 {
                return Ok(None);
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "Failed to fetch block #{}: HTTP {} - {}",
                    round,
                    status,
                    error_text
                ));
            }
        }

        let block_data: Value = response.json().await?;
        let block_val = block_data.get("block").unwrap_or(&block_data);

        let txn_count = count_transactions(block_val);
        let timestamp_secs = block_val["ts"].as_u64().unwrap_or(0);
        let formatted_time = format_timestamp(timestamp_secs);

        let proposer = block_val["cert"]["prop"]["addr"]
            .as_str()
            .or_else(|| block_val["proposer"].as_str())
            .unwrap_or("unknown")
            .to_string();

        let seed = block_val["seed"].as_str().unwrap_or("unknown").to_string();

        Ok(Some(BlockInfo {
            id: round,
            timestamp: formatted_time,
            txn_count,
            proposer,
            seed,
        }))
    }

    /// Get detailed block information including all transactions
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or parsing fails.
    pub async fn get_block_details(&self, round: u64) -> Result<Option<BlockDetails>> {
        // First, get the basic block info
        let block_info = match self.search_block(&round.to_string()).await? {
            Some(info) => info,
            None => return Ok(None),
        };

        // Fetch transactions for this round using the indexer
        let txns_url = format!(
            "{}/v2/transactions?round={}",
            self.network.indexer_url(),
            round
        );

        let response = self.build_indexer_request(&txns_url).send().await?;

        let transactions = if response.status().is_success() {
            let json: Value = response.json().await?;
            parse_transactions_array(&json)?
        } else {
            // If we can't get transactions, return empty list
            Vec::new()
        };

        // Compute transaction type counts
        let mut txn_type_counts = std::collections::HashMap::new();
        for txn in &transactions {
            *txn_type_counts.entry(txn.txn_type).or_insert(0) += 1;
        }

        Ok(Some(BlockDetails {
            info: block_info,
            transactions,
            txn_type_counts,
        }))
    }

    async fn search_asset(&self, asset_id_str: &str) -> Result<Option<AssetInfo>> {
        let asset_id = asset_id_str.parse::<u64>().map_err(|_| {
            AlgoError::invalid_input(format!(
                "Invalid asset ID '{}'. Please enter a valid positive integer.",
                asset_id_str
            ))
            .into_report()
        })?;

        let asset_url = format!("{}/v2/assets/{}", self.network.indexer_url(), asset_id);

        let response = self.build_indexer_request(&asset_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 404 {
                return Ok(None);
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "Failed to fetch asset #{}: HTTP {} - {}",
                    asset_id,
                    status,
                    error_text
                ));
            }
        }

        let asset_data: Value = response.json().await?;
        let params = &asset_data["asset"]["params"];

        let name = params["name"].as_str().unwrap_or("").to_string();
        let unit_name = params["unit-name"].as_str().unwrap_or("").to_string();
        let creator = params["creator"].as_str().unwrap_or("unknown").to_string();
        let total = params["total"].as_u64().unwrap_or(0);
        let decimals = params["decimals"].as_u64().unwrap_or(0);
        let url = params["url"].as_str().unwrap_or("").to_string();

        Ok(Some(AssetInfo {
            id: asset_id,
            name,
            unit_name,
            creator,
            total,
            decimals,
            url,
        }))
    }

    async fn search_address(&self, query: &str) -> Result<Option<AccountInfo>> {
        let trimmed = query.trim();

        // First, check if it's a valid Algorand address
        if trimmed.len() == 58
            && trimmed
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            // It's a valid address format, search directly
            return self.search_address_direct(trimmed).await;
        }

        // Check if NFD is supported and the query looks like an NFD name
        if self.network.supports_nfd() && Self::looks_like_nfd_name(trimmed) {
            // Try to resolve as NFD name
            if let Ok(Some(nfd_info)) = self.get_nfd_by_name(trimmed).await {
                // Get the deposit account from NFD
                let address = nfd_info
                    .deposit_account
                    .as_ref()
                    .or(nfd_info.owner.as_ref());

                if let Some(addr) = address {
                    // Search for the resolved address
                    return self.search_address_direct(addr).await;
                }
            }

            // NFD not found
            return Err(AlgoError::not_found("NFD", trimmed).into_report());
        }

        // Not a valid address and not an NFD name
        Err(AlgoError::invalid_input(
            "Invalid input. Enter a 58-character Algorand address or an NFD name (e.g., alice.algo)."
        ).into_report())
    }

    /// Search for an address directly (after validation or NFD resolution)
    async fn search_address_direct(&self, address: &str) -> Result<Option<AccountInfo>> {
        let indexer_result = self.search_address_via_indexer(address).await;

        match indexer_result {
            Ok(Some(account)) => {
                return Ok(Some(account));
            }
            Ok(None) => {}
            Err(_) => {}
        }

        let algod_result = self.search_address_via_algod(address).await;

        match algod_result {
            Ok(Some(account)) => Ok(Some(account)),
            Ok(None) => Err(AlgoError::not_found("account", address).into_report()),
            Err(e) => Err(color_eyre::eyre::eyre!(
                "Failed to fetch account information for '{}': {}",
                address,
                e
            )),
        }
    }

    async fn search_address_via_indexer(&self, address: &str) -> Result<Option<AccountInfo>> {
        let account_url = format!("{}/v2/accounts/{}", self.network.indexer_url(), address);

        let response = self.build_indexer_request(&account_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 404 {
                return Ok(None);
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "Indexer request failed with status {}: {}",
                    status,
                    error_text
                ));
            }
        }

        let account_data: Value = response.json().await?;

        if let Some(account) = account_data.get("account") {
            Ok(Some(Self::parse_account_info(account, address)))
        } else {
            Err(AlgoError::parse("Invalid indexer response format").into_report())
        }
    }

    async fn search_address_via_algod(&self, address: &str) -> Result<Option<AccountInfo>> {
        let account_url = format!("{}/v2/accounts/{}", self.network.algod_url(), address);

        let response = self.build_algod_request(&account_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 404 {
                return Ok(None);
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "Algod request failed with status {}: {}",
                    status,
                    error_text
                ));
            }
        }

        let account_data: Value = response.json().await?;

        Ok(Some(Self::parse_account_info(&account_data, address)))
    }

    #[must_use]
    fn parse_account_info(account: &Value, address: &str) -> AccountInfo {
        let balance = account["amount"].as_u64().unwrap_or(0);
        let pending_rewards = account["pending-rewards"].as_u64().unwrap_or(0);
        let reward_base = account["reward-base"].as_u64().unwrap_or(0);
        let status = account["status"].as_str().unwrap_or("unknown").to_string();

        let assets_count = account["assets"]
            .as_array()
            .map_or(0, |assets| assets.len());

        let created_assets_count = account["created-assets"]
            .as_array()
            .map_or(0, |assets| assets.len());

        AccountInfo {
            address: address.to_string(),
            balance,
            pending_rewards,
            reward_base,
            status,
            assets_count,
            created_assets_count,
        }
    }

    async fn fetch_transactions_from_url(&self, url: &str) -> Result<Vec<Transaction>> {
        let response = match self.build_indexer_request(url).send().await {
            Ok(resp) if resp.status().is_success() => resp,
            _ => return Ok(Vec::new()),
        };

        let json: Value = match response.json().await {
            Ok(data) => data,
            Err(_) => return Ok(Vec::new()),
        };

        parse_transactions_array(&json)
    }

    /// Get detailed account information from algod.
    /// Also fetches NFD info for the address on MainNet/TestNet.
    pub async fn get_account_details(&self, address: &str) -> Result<AccountDetails> {
        // Validate address format
        if address.len() != 58
            || !address
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return Err(AlgoError::invalid_input("Invalid Algorand address format").into_report());
        }

        let account_url = format!("{}/v2/accounts/{}", self.network.algod_url(), address);
        let response = self.build_algod_request(&account_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(AlgoError::not_found("account", address).into_report());
            }
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(color_eyre::eyre::eyre!(
                "Failed to fetch account details: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let account_data: Value = response.json().await?;
        let mut account_details = Self::parse_account_details(&account_data, address);

        // Fetch NFD info if supported on this network
        if self.network.supports_nfd() {
            account_details.nfd = self.get_nfd_for_address(address).await.unwrap_or(None);
        }

        Ok(account_details)
    }

    #[must_use]
    fn parse_account_details(account: &Value, address: &str) -> AccountDetails {
        let balance = account["amount"].as_u64().unwrap_or(0);
        let min_balance = account["min-balance"].as_u64().unwrap_or(0);
        let pending_rewards = account["pending-rewards"].as_u64().unwrap_or(0);
        let rewards = account["rewards"].as_u64().unwrap_or(0);
        let reward_base = account["reward-base"].as_u64().unwrap_or(0);
        let status = account["status"].as_str().unwrap_or("unknown").to_string();

        let total_apps_opted_in = account["total-apps-opted-in"].as_u64().unwrap_or(0) as usize;
        let total_assets_opted_in = account["total-assets-opted-in"].as_u64().unwrap_or(0) as usize;
        let total_created_apps = account["total-created-apps"].as_u64().unwrap_or(0) as usize;
        let total_created_assets = account["total-created-assets"].as_u64().unwrap_or(0) as usize;
        let total_boxes = account["total-boxes"].as_u64().unwrap_or(0) as usize;

        let auth_addr = account["auth-addr"].as_str().map(String::from);

        // Parse participation info if online
        let participation = account.get("participation").map(|part| ParticipationInfo {
            vote_first: part["vote-first-valid"].as_u64().unwrap_or(0),
            vote_last: part["vote-last-valid"].as_u64().unwrap_or(0),
            vote_key_dilution: part["vote-key-dilution"].as_u64().unwrap_or(0),
            selection_key: part["selection-participation-key"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            vote_key: part["vote-participation-key"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            state_proof_key: part["state-proof-key"].as_str().map(String::from),
        });

        // Parse asset holdings (limited to first 10)
        let assets = account["assets"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .map(|a| AccountAssetHolding {
                        asset_id: a["asset-id"].as_u64().unwrap_or(0),
                        amount: a["amount"].as_u64().unwrap_or(0),
                        is_frozen: a["is-frozen"].as_bool().unwrap_or(false),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse created assets (limited to first 10)
        let created_assets = account["created-assets"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .map(|a| {
                        let params = &a["params"];
                        CreatedAssetInfo {
                            asset_id: a["index"].as_u64().unwrap_or(0),
                            name: params["name"].as_str().unwrap_or("").to_string(),
                            unit_name: params["unit-name"].as_str().unwrap_or("").to_string(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse app local states (limited to first 10)
        let apps_local_state = account["apps-local-state"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .map(|a| AppLocalState {
                        app_id: a["id"].as_u64().unwrap_or(0),
                        schema_num_uint: a["schema"]["num-uint"].as_u64().unwrap_or(0),
                        schema_num_byte_slice: a["schema"]["num-byte-slice"].as_u64().unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse created apps (limited to first 10)
        let created_apps = account["created-apps"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .map(|a| CreatedAppInfo {
                        app_id: a["id"].as_u64().unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();

        AccountDetails {
            address: address.to_string(),
            balance,
            min_balance,
            pending_rewards,
            rewards,
            reward_base,
            status,
            total_apps_opted_in,
            total_assets_opted_in,
            total_created_apps,
            total_created_assets,
            total_boxes,
            auth_addr,
            participation,
            assets,
            created_assets,
            apps_local_state,
            created_apps,
            nfd: None, // NFD is set separately after fetching
        }
    }

    /// Get detailed asset information from indexer
    pub async fn get_asset_details(&self, asset_id: u64) -> Result<AssetDetails> {
        let asset_url = format!("{}/v2/assets/{}", self.network.indexer_url(), asset_id);
        let response = self.build_indexer_request(&asset_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(AlgoError::not_found("asset", asset_id.to_string()).into_report());
            }
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(color_eyre::eyre::eyre!(
                "Failed to fetch asset details: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let asset_data: Value = response.json().await?;
        Ok(Self::parse_asset_details(&asset_data, asset_id))
    }

    #[must_use]
    fn parse_asset_details(data: &Value, asset_id: u64) -> AssetDetails {
        let asset = &data["asset"];
        let params = &asset["params"];

        AssetDetails {
            id: asset_id,
            name: params["name"].as_str().unwrap_or("").to_string(),
            unit_name: params["unit-name"].as_str().unwrap_or("").to_string(),
            creator: params["creator"].as_str().unwrap_or("").to_string(),
            total: params["total"].as_u64().unwrap_or(0),
            decimals: params["decimals"].as_u64().unwrap_or(0),
            url: params["url"].as_str().unwrap_or("").to_string(),
            metadata_hash: params["metadata-hash"].as_str().map(String::from),
            default_frozen: params["default-frozen"].as_bool().unwrap_or(false),
            manager: params["manager"].as_str().map(String::from),
            reserve: params["reserve"].as_str().map(String::from),
            freeze: params["freeze"].as_str().map(String::from),
            clawback: params["clawback"].as_str().map(String::from),
            deleted: asset["deleted"].as_bool().unwrap_or(false),
            created_at_round: asset["created-at-round"].as_u64(),
        }
    }

    /// Get search suggestions based on the current query and search type
    #[must_use]
    pub fn get_search_suggestions(query: &str, search_type: SearchType) -> String {
        let trimmed = query.trim();

        match search_type {
            SearchType::Account => {
                if trimmed.is_empty() {
                    "Enter an Algorand address or NFD name (e.g., alice.algo)".to_string()
                } else if Self::looks_like_nfd_name(trimmed) {
                    // Could be an NFD name
                    if trimmed.ends_with(".algo") {
                        format!("NFD name '{}'. Press Enter to search.", trimmed)
                    } else {
                        format!(
                            "Looks like NFD name '{}'. Press Enter to search (will try {}.algo).",
                            trimmed, trimmed
                        )
                    }
                } else if trimmed.len() < 58 {
                    format!(
                        "Address too short ({} chars). Try an NFD name or 58-char address.",
                        trimmed.len()
                    )
                } else if trimmed.len() > 58 {
                    format!(
                        "Address too long ({} chars). Algorand addresses are 58 characters long.",
                        trimmed.len()
                    )
                } else if !trimmed
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                {
                    "Address contains invalid characters. Use only uppercase letters and numbers."
                        .to_string()
                } else {
                    "Valid address format. Press Enter to search.".to_string()
                }
            }
            SearchType::Transaction => {
                if trimmed.is_empty() {
                    "Enter a transaction ID (typically 52 characters)".to_string()
                } else if trimmed.len() < 40 {
                    format!(
                        "Transaction ID too short ({} chars). Most transaction IDs are 52 characters.",
                        trimmed.len()
                    )
                } else if trimmed.len() > 60 {
                    format!(
                        "Transaction ID too long ({} chars). Most transaction IDs are 52 characters.",
                        trimmed.len()
                    )
                } else {
                    "Valid transaction ID format. Press Enter to search.".to_string()
                }
            }
            SearchType::Block => {
                if trimmed.is_empty() {
                    "Enter a block number (positive integer)".to_string()
                } else if trimmed.parse::<u64>().is_err() {
                    "Block number must be a positive integer".to_string()
                } else {
                    "Valid block number. Press Enter to search.".to_string()
                }
            }
            SearchType::Asset => {
                if trimmed.is_empty() {
                    "Enter an asset ID (positive integer)".to_string()
                } else if trimmed.parse::<u64>().is_err() {
                    "Asset ID must be a positive integer".to_string()
                } else {
                    "Valid asset ID. Press Enter to search.".to_string()
                }
            }
        }
    }

    // ========================================================================
    // NFD (NFDomains) API Methods
    // ========================================================================

    /// Look up an NFD by name (e.g., "alice.algo").
    /// Returns None if the NFD doesn't exist or if NFD is not supported on this network.
    pub async fn get_nfd_by_name(&self, name: &str) -> Result<Option<NfdInfo>> {
        let Some(nfd_url) = self.network.nfd_api_url() else {
            return Ok(None); // NFD not supported on this network
        };

        // Normalize the name - ensure it ends with .algo
        let normalized_name = if name.ends_with(".algo") {
            name.to_string()
        } else {
            format!("{}.algo", name)
        };

        let url = format!("{}/nfd/{}?view=brief", nfd_url, normalized_name);

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let json: Value = resp.json().await?;
                    Ok(Some(NfdInfo::from_json(&json)))
                } else {
                    Ok(None) // NFD not found or other errors
                }
            }
            Err(_) => Ok(None), // Network errors, treat as not found
        }
    }

    /// Reverse lookup - get the primary NFD for an Algorand address.
    /// Returns None if no NFD is linked to this address or if NFD is not supported.
    pub async fn get_nfd_for_address(&self, address: &str) -> Result<Option<NfdInfo>> {
        let Some(nfd_url) = self.network.nfd_api_url() else {
            return Ok(None); // NFD not supported on this network
        };

        // Validate address format first
        if address.len() != 58
            || !address
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return Ok(None);
        }

        let url = format!(
            "{}/nfd/lookup?address={}&view=brief&allowUnverified=true",
            nfd_url, address
        );

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let json: Value = resp.json().await?;
                    // The response is a map of address -> NFD info
                    if let Some(nfd_data) = json.get(address) {
                        Ok(Some(NfdInfo::from_json(nfd_data)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None) // 404 or other errors
                }
            }
            Err(_) => Ok(None), // Network errors
        }
    }

    /// Check if a query string looks like an NFD name.
    /// NFD names end with .algo or could be just the name part.
    #[must_use]
    pub fn looks_like_nfd_name(query: &str) -> bool {
        let trimmed = query.trim().to_lowercase();

        // Must have at least 1 character before .algo or be a simple name
        if trimmed.is_empty() {
            return false;
        }

        // If it ends with .algo, check the part before it
        if let Some(name_part) = trimmed.strip_suffix(".algo") {
            // NFD names must be at least 1 char and contain only valid chars
            !name_part.is_empty()
                && name_part
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        } else {
            // Could be just the name without .algo suffix
            // It's likely an NFD if it contains alphanumeric chars and isn't a valid address/number
            trimmed
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                && trimmed.parse::<u64>().is_err()
                && trimmed.len() < 58 // Not an Algorand address
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse an array of transactions from JSON response
///
/// This helper function extracts the "transactions" array from the JSON
/// and parses each transaction using `Transaction::from_json()`.
fn parse_transactions_array(json: &Value) -> Result<Vec<Transaction>> {
    let empty_vec = Vec::new();
    let transactions_array = json["transactions"].as_array().unwrap_or(&empty_vec);
    let mut transactions = Vec::with_capacity(transactions_array.len());

    for txn_json in transactions_array {
        match Transaction::from_json(txn_json) {
            Ok(txn) => transactions.push(txn),
            Err(_) => {
                // Skip malformed transactions but continue processing
                continue;
            }
        }
    }

    Ok(transactions)
}

/// Format a Unix timestamp into a human-readable string
#[must_use]
fn format_timestamp(timestamp_secs: u64) -> String {
    if timestamp_secs == 0 {
        return "Timestamp not available".to_string();
    }

    let datetime =
        chrono::DateTime::from_timestamp(timestamp_secs as i64, 0).unwrap_or_else(chrono::Utc::now);

    datetime.format("%a, %d %b %Y %H:%M:%S").to_string()
}

/// Count the number of transactions in a block
#[must_use]
fn count_transactions(block: &Value) -> u16 {
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_suggestions() {
        assert!(
            AlgoClient::get_search_suggestions("", SearchType::Account)
                .contains("Enter an Algorand address")
        );

        // "ABC" looks like an NFD name, so it shows NFD suggestion
        assert!(
            AlgoClient::get_search_suggestions("ABC", SearchType::Account).contains("NFD")
                || AlgoClient::get_search_suggestions("ABC", SearchType::Account)
                    .contains("NFD name")
        );

        // Test with clearly invalid input that's too short and not an NFD
        assert!(
            AlgoClient::get_search_suggestions("A1!", SearchType::Account).contains("too short")
        );

        assert!(
            AlgoClient::get_search_suggestions(
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                SearchType::Account
            )
            .contains("Valid address format")
        );

        assert!(
            AlgoClient::get_search_suggestions("", SearchType::Transaction)
                .contains("Enter a transaction ID")
        );

        assert!(
            AlgoClient::get_search_suggestions("ABC", SearchType::Transaction)
                .contains("too short")
        );

        assert!(
            AlgoClient::get_search_suggestions("", SearchType::Block)
                .contains("Enter a block number")
        );

        assert!(
            AlgoClient::get_search_suggestions("123456", SearchType::Block)
                .contains("Valid block number")
        );

        assert!(
            AlgoClient::get_search_suggestions("abc", SearchType::Block)
                .contains("must be a positive integer")
        );

        assert!(
            AlgoClient::get_search_suggestions("", SearchType::Asset).contains("Enter an asset ID")
        );

        assert!(
            AlgoClient::get_search_suggestions("123", SearchType::Asset).contains("Valid asset ID")
        );
    }

    #[test]
    fn test_looks_like_nfd_name() {
        // Valid NFD names
        assert!(AlgoClient::looks_like_nfd_name("alice.algo"));
        assert!(AlgoClient::looks_like_nfd_name("alice"));
        assert!(AlgoClient::looks_like_nfd_name("nfdomains.algo"));
        assert!(AlgoClient::looks_like_nfd_name("test-name.algo"));
        assert!(AlgoClient::looks_like_nfd_name("ABC"));

        // Invalid - empty
        assert!(!AlgoClient::looks_like_nfd_name(""));
        assert!(!AlgoClient::looks_like_nfd_name("   "));

        // Invalid - just numbers (could be asset ID or block)
        assert!(!AlgoClient::looks_like_nfd_name("123456"));

        // Invalid - 58 char address-like string
        assert!(!AlgoClient::looks_like_nfd_name(
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        ));
    }

    #[test]
    fn test_nfd_api_url() {
        assert!(Network::MainNet.nfd_api_url().is_some());
        assert!(Network::TestNet.nfd_api_url().is_some());
        assert!(Network::LocalNet.nfd_api_url().is_none());

        assert!(Network::MainNet.supports_nfd());
        assert!(Network::TestNet.supports_nfd());
        assert!(!Network::LocalNet.supports_nfd());
    }

    #[test]
    fn test_txn_type_as_str() {
        assert_eq!(TxnType::Payment.as_str(), "Payment");
        assert_eq!(TxnType::AppCall.as_str(), "App Call");
        assert_eq!(TxnType::AssetTransfer.as_str(), "Asset Transfer");
        assert_eq!(TxnType::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_network_as_str() {
        assert_eq!(Network::MainNet.as_str(), "MainNet");
        assert_eq!(Network::TestNet.as_str(), "TestNet");
        assert_eq!(Network::LocalNet.as_str(), "LocalNet");
    }

    #[test]
    fn test_network_urls() {
        assert!(Network::MainNet.indexer_url().contains("mainnet"));
        assert!(Network::TestNet.algod_url().contains("testnet"));
        assert!(Network::LocalNet.algod_url().contains("localhost"));
    }

    #[test]
    fn test_transaction_from_json_payment() {
        let json = serde_json::json!({
            "id": "test-txn-id",
            "sender": "SENDER_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12345_u64,
            "fee": 1000_u64,
            "payment-transaction": {
                "amount": 5000000_u64,
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

        // Verify payment details
        match txn.details {
            TransactionDetails::Payment(details) => {
                assert!(details.close_remainder_to.is_none());
                assert!(details.close_amount.is_none());
            }
            _ => panic!("Expected Payment details"),
        }
    }

    #[test]
    fn test_transaction_from_json_payment_with_close() {
        let json = serde_json::json!({
            "id": "close-txn-id",
            "sender": "SENDER_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12345_u64,
            "fee": 1000_u64,
            "payment-transaction": {
                "amount": 5000000_u64,
                "receiver": "RECEIVER_ADDRESS",
                "close-remainder-to": "CLOSE_TO_ADDRESS",
                "close-amount": 1000000_u64
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::Payment);

        match txn.details {
            TransactionDetails::Payment(details) => {
                assert_eq!(
                    details.close_remainder_to,
                    Some("CLOSE_TO_ADDRESS".to_string())
                );
                assert_eq!(details.close_amount, Some(1_000_000));
            }
            _ => panic!("Expected Payment details"),
        }
    }

    #[test]
    fn test_transaction_from_json_asset_transfer() {
        let json = serde_json::json!({
            "id": "asset-txn-id",
            "sender": "SENDER_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12346_u64,
            "fee": 1000_u64,
            "asset-transfer-transaction": {
                "amount": 100_u64,
                "receiver": "RECEIVER_ADDRESS",
                "asset-id": 31566704_u64
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::AssetTransfer);
        assert_eq!(txn.amount, 100);
        assert_eq!(txn.asset_id, Some(31_566_704));

        match txn.details {
            TransactionDetails::AssetTransfer(details) => {
                assert!(details.asset_sender.is_none());
                assert!(details.close_to.is_none());
                assert!(details.close_amount.is_none());
            }
            _ => panic!("Expected AssetTransfer details"),
        }
    }

    #[test]
    fn test_transaction_from_json_asset_transfer_clawback() {
        let json = serde_json::json!({
            "id": "clawback-txn-id",
            "sender": "CLAWBACK_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12347_u64,
            "fee": 1000_u64,
            "asset-transfer-transaction": {
                "amount": 50_u64,
                "receiver": "RECEIVER_ADDRESS",
                "asset-id": 31566704_u64,
                "sender": "CLAWBACK_TARGET",
                "close-to": "CLOSE_ADDRESS",
                "close-amount": 25_u64
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::AssetTransfer);

        match txn.details {
            TransactionDetails::AssetTransfer(details) => {
                assert_eq!(details.asset_sender, Some("CLAWBACK_TARGET".to_string()));
                assert_eq!(details.close_to, Some("CLOSE_ADDRESS".to_string()));
                assert_eq!(details.close_amount, Some(25));
            }
            _ => panic!("Expected AssetTransfer details"),
        }
    }

    #[test]
    fn test_transaction_from_json_asset_config_create() {
        let json = serde_json::json!({
            "id": "asset-create-id",
            "sender": "CREATOR_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12348_u64,
            "fee": 1000_u64,
            "created-asset-index": 123456789_u64,
            "asset-config-transaction": {
                "params": {
                    "total": 1000000_u64,
                    "decimals": 6_u64,
                    "default-frozen": false,
                    "name": "Test Token",
                    "unit-name": "TEST",
                    "url": "https://test.com",
                    "metadata-hash": "abc123",
                    "manager": "MANAGER_ADDRESS",
                    "reserve": "RESERVE_ADDRESS",
                    "freeze": "FREEZE_ADDRESS",
                    "clawback": "CLAWBACK_ADDRESS"
                }
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::AssetConfig);

        match &txn.details {
            TransactionDetails::AssetConfig(details) => {
                assert!(details.asset_id.is_none()); // Creation doesn't have asset_id
                assert_eq!(details.created_asset_id, Some(123_456_789));
                assert_eq!(details.total, Some(1_000_000));
                assert_eq!(details.decimals, Some(6));
                assert_eq!(details.default_frozen, Some(false));
                assert_eq!(details.asset_name, Some("Test Token".to_string()));
                assert_eq!(details.unit_name, Some("TEST".to_string()));
                assert_eq!(details.url, Some("https://test.com".to_string()));
                assert_eq!(details.metadata_hash, Some("abc123".to_string()));
                assert_eq!(details.manager, Some("MANAGER_ADDRESS".to_string()));
                assert_eq!(details.reserve, Some("RESERVE_ADDRESS".to_string()));
                assert_eq!(details.freeze, Some("FREEZE_ADDRESS".to_string()));
                assert_eq!(details.clawback, Some("CLAWBACK_ADDRESS".to_string()));

                // Test is_creation helper
                assert!(txn.details.is_creation());
                assert_eq!(txn.details.created_id(), Some(123_456_789));
            }
            _ => panic!("Expected AssetConfig details"),
        }
    }

    #[test]
    fn test_transaction_from_json_asset_config_modify() {
        let json = serde_json::json!({
            "id": "asset-modify-id",
            "sender": "MANAGER_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12349_u64,
            "fee": 1000_u64,
            "asset-config-transaction": {
                "asset-id": 123456789_u64,
                "params": {
                    "manager": "NEW_MANAGER_ADDRESS"
                }
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::AssetConfig);

        match &txn.details {
            TransactionDetails::AssetConfig(details) => {
                assert_eq!(details.asset_id, Some(123_456_789));
                assert!(details.created_asset_id.is_none());
                assert_eq!(details.manager, Some("NEW_MANAGER_ADDRESS".to_string()));

                // Modify is not a creation
                assert!(!txn.details.is_creation());
                assert!(txn.details.created_id().is_none());
            }
            _ => panic!("Expected AssetConfig details"),
        }
    }

    #[test]
    fn test_transaction_from_json_asset_freeze() {
        let json = serde_json::json!({
            "id": "freeze-txn-id",
            "sender": "FREEZE_MANAGER",
            "round-time": 1700000000_u64,
            "confirmed-round": 12350_u64,
            "fee": 1000_u64,
            "asset-freeze-transaction": {
                "address": "TARGET_ADDRESS",
                "asset-id": 31566704_u64,
                "new-freeze-status": true
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::AssetFreeze);
        assert_eq!(txn.to, "TARGET_ADDRESS");

        match txn.details {
            TransactionDetails::AssetFreeze(details) => {
                assert!(details.frozen);
                assert_eq!(details.freeze_target, "TARGET_ADDRESS");
            }
            _ => panic!("Expected AssetFreeze details"),
        }
    }

    #[test]
    fn test_transaction_from_json_app_call_create() {
        let json = serde_json::json!({
            "id": "app-create-id",
            "sender": "CREATOR_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12351_u64,
            "fee": 1000_u64,
            "created-application-index": 987654321_u64,
            "application-transaction": {
                "application-id": 0_u64,
                "on-completion": "noop",
                "approval-program": "BIAKBQAKAI==",
                "clear-state-program": "BIA=",
                "application-args": ["YXJnMQ==", "YXJnMg=="],
                "accounts": ["ACCOUNT1", "ACCOUNT2"],
                "foreign-apps": [111_u64, 222_u64],
                "foreign-assets": [333_u64, 444_u64],
                "global-state-schema": {
                    "num-uint": 10_u64,
                    "num-byte-slice": 5_u64
                },
                "local-state-schema": {
                    "num-uint": 3_u64,
                    "num-byte-slice": 2_u64
                },
                "extra-program-pages": 1_u64
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::AppCall);

        match &txn.details {
            TransactionDetails::AppCall(details) => {
                assert_eq!(details.app_id, 0);
                assert_eq!(details.created_app_id, Some(987_654_321));
                assert_eq!(details.on_complete, OnComplete::NoOp);
                assert_eq!(details.approval_program, Some("BIAKBQAKAI==".to_string()));
                assert_eq!(details.clear_state_program, Some("BIA=".to_string()));
                assert_eq!(details.app_args, vec!["YXJnMQ==", "YXJnMg=="]);
                assert_eq!(details.accounts, vec!["ACCOUNT1", "ACCOUNT2"]);
                assert_eq!(details.foreign_apps, vec![111, 222]);
                assert_eq!(details.foreign_assets, vec![333, 444]);
                assert_eq!(details.global_state_schema.as_ref().unwrap().num_uint, 10);
                assert_eq!(
                    details.global_state_schema.as_ref().unwrap().num_byte_slice,
                    5
                );
                assert_eq!(details.local_state_schema.as_ref().unwrap().num_uint, 3);
                assert_eq!(
                    details.local_state_schema.as_ref().unwrap().num_byte_slice,
                    2
                );
                assert_eq!(details.extra_program_pages, Some(1));

                // Test is_creation helper
                assert!(txn.details.is_creation());
                assert_eq!(txn.details.created_id(), Some(987_654_321));
            }
            _ => panic!("Expected AppCall details"),
        }
    }

    #[test]
    fn test_transaction_from_json_app_call_with_boxes() {
        let json = serde_json::json!({
            "id": "app-call-boxes-id",
            "sender": "CALLER_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12352_u64,
            "fee": 1000_u64,
            "application-transaction": {
                "application-id": 123456_u64,
                "on-completion": "noop",
                "boxes": [
                    {"i": 0_u64, "n": "Ym94MQ=="},
                    {"i": 789_u64, "n": "Ym94Mg=="}
                ]
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::AppCall);

        match &txn.details {
            TransactionDetails::AppCall(details) => {
                assert_eq!(details.app_id, 123_456);
                assert_eq!(details.boxes.len(), 2);
                assert_eq!(details.boxes[0].app_id, 0);
                assert_eq!(details.boxes[0].name, "Ym94MQ==");
                assert_eq!(details.boxes[1].app_id, 789);
                assert_eq!(details.boxes[1].name, "Ym94Mg==");

                // Not a creation
                assert!(!txn.details.is_creation());
            }
            _ => panic!("Expected AppCall details"),
        }
    }

    #[test]
    fn test_transaction_from_json_app_call_on_complete_variants() {
        let test_cases = vec![
            ("noop", OnComplete::NoOp),
            ("optin", OnComplete::OptIn),
            ("closeout", OnComplete::CloseOut),
            ("clearstate", OnComplete::ClearState),
            ("updateapplication", OnComplete::UpdateApplication),
            ("update", OnComplete::UpdateApplication),
            ("deleteapplication", OnComplete::DeleteApplication),
            ("delete", OnComplete::DeleteApplication),
        ];

        for (input, expected) in test_cases {
            let json = serde_json::json!({
                "id": format!("app-{}-id", input),
                "sender": "SENDER",
                "round-time": 1700000000_u64,
                "confirmed-round": 12353_u64,
                "fee": 1000_u64,
                "application-transaction": {
                    "application-id": 123_u64,
                    "on-completion": input
                }
            });

            let txn = Transaction::from_json(&json).unwrap();
            match txn.details {
                TransactionDetails::AppCall(details) => {
                    assert_eq!(details.on_complete, expected, "Failed for input: {}", input);
                }
                _ => panic!("Expected AppCall details for input: {}", input),
            }
        }
    }

    #[test]
    fn test_transaction_from_json_keyreg_online() {
        let json = serde_json::json!({
            "id": "keyreg-online-id",
            "sender": "REGISTERING_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12354_u64,
            "fee": 1000_u64,
            "keyreg-transaction": {
                "vote-participation-key": "dm90ZUtleQ==",
                "selection-participation-key": "c2VsS2V5",
                "state-proof-key": "c3BLZXk=",
                "vote-first-valid": 1000_u64,
                "vote-last-valid": 2000000_u64,
                "vote-key-dilution": 10000_u64,
                "non-participation": false
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::KeyReg);

        match txn.details {
            TransactionDetails::KeyReg(details) => {
                assert_eq!(details.vote_key, Some("dm90ZUtleQ==".to_string()));
                assert_eq!(details.selection_key, Some("c2VsS2V5".to_string()));
                assert_eq!(details.state_proof_key, Some("c3BLZXk=".to_string()));
                assert_eq!(details.vote_first, Some(1000));
                assert_eq!(details.vote_last, Some(2_000_000));
                assert_eq!(details.vote_key_dilution, Some(10000));
                assert!(!details.non_participation);
            }
            _ => panic!("Expected KeyReg details"),
        }
    }

    #[test]
    fn test_transaction_from_json_keyreg_offline() {
        let json = serde_json::json!({
            "id": "keyreg-offline-id",
            "sender": "REGISTERING_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12355_u64,
            "fee": 1000_u64,
            "keyreg-transaction": {
                "non-participation": true
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::KeyReg);

        match txn.details {
            TransactionDetails::KeyReg(details) => {
                assert!(details.vote_key.is_none());
                assert!(details.selection_key.is_none());
                assert!(details.non_participation);
            }
            _ => panic!("Expected KeyReg details"),
        }
    }

    #[test]
    fn test_transaction_from_json_state_proof() {
        let json = serde_json::json!({
            "id": "state-proof-id",
            "sender": "SENDER_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12356_u64,
            "fee": 0_u64,
            "state-proof-transaction": {
                "state-proof-type": 0_u64,
                "message": "deadbeef"
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::StateProof);

        match txn.details {
            TransactionDetails::StateProof(details) => {
                assert_eq!(details.state_proof_type, Some(0));
                assert_eq!(details.message, Some("deadbeef".to_string()));
            }
            _ => panic!("Expected StateProof details"),
        }
    }

    #[test]
    fn test_transaction_from_json_heartbeat() {
        let json = serde_json::json!({
            "id": "heartbeat-id",
            "sender": "HEARTBEAT_ADDRESS",
            "round-time": 1700000000_u64,
            "confirmed-round": 12357_u64,
            "fee": 0_u64,
            "heartbeat-transaction": {
                "hb-address": "HEARTBEAT_TARGET",
                "hb-key-dilution": 10000_u64,
                "hb-proof": "cHJvb2Y=",
                "hb-seed": "c2VlZA==",
                "hb-vote-id": "dm90ZUlk"
            }
        });

        let txn = Transaction::from_json(&json).unwrap();
        assert_eq!(txn.txn_type, TxnType::Heartbeat);

        match txn.details {
            TransactionDetails::Heartbeat(details) => {
                assert_eq!(details.hb_address, Some("HEARTBEAT_TARGET".to_string()));
                assert_eq!(details.hb_key_dilution, Some(10000));
                assert_eq!(details.hb_proof, Some("cHJvb2Y=".to_string()));
                assert_eq!(details.hb_seed, Some("c2VlZA==".to_string()));
                assert_eq!(details.hb_vote_id, Some("dm90ZUlk".to_string()));
            }
            _ => panic!("Expected Heartbeat details"),
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
    fn test_transaction_details_default() {
        let details = TransactionDetails::default();
        assert_eq!(details, TransactionDetails::None);
        assert!(!details.is_creation());
        assert!(details.created_id().is_none());
    }

    #[test]
    fn test_algo_error_display() {
        let parse_err = AlgoError::parse("test error");
        assert_eq!(format!("{}", parse_err), "Parse error: test error");

        let not_found_err = AlgoError::not_found("transaction", "abc123");
        assert_eq!(
            format!("{}", not_found_err),
            "transaction 'abc123' not found"
        );

        let invalid_err = AlgoError::invalid_input("bad input");
        assert_eq!(format!("{}", invalid_err), "Invalid input: bad input");
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "Timestamp not available");
        // Non-zero timestamp should produce a formatted string
        let result = format_timestamp(1700000000);
        assert!(result.contains("2023")); // Should be a date in 2023
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
}
