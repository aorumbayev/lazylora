//! Transaction parsing from JSON.
//!
//! This module contains all the JSON parsing logic for converting
//! Algorand indexer API responses into `Transaction` structs.

use serde_json::Value;

use crate::domain::error::AlgoError;

use super::types::{
    AppCallDetails, AssetConfigDetails, AssetFreezeDetails, AssetTransferDetails, BoxRef,
    HeartbeatDetails, KeyRegDetails, OnComplete, PaymentDetails, StateProofDetails, StateSchema,
    TransactionDetails,
};
use super::{Transaction, TxnType, format_timestamp};

// ============================================================================
// Transaction Parsing
// ============================================================================

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

        let to = extract_receiver(txn_json, &txn_type);

        let timestamp = txn_json["round-time"]
            .as_u64()
            .map(format_timestamp)
            .unwrap_or_else(|| "Unknown".to_string());

        let block = txn_json["confirmed-round"].as_u64().unwrap_or(0);
        let fee = txn_json["fee"].as_u64().unwrap_or(0);

        let note = extract_note(txn_json);
        let (amount, asset_id) = extract_amount_and_asset(txn_json, &txn_type);
        let rekey_to = txn_json["rekey-to"].as_str().map(String::from);
        let group = txn_json["group"].as_str().map(String::from);
        let details = extract_details(txn_json, &txn_type);

        // Parse inner transactions recursively
        let inner_transactions = parse_inner_transactions(txn_json)?;

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
            group,
            details,
            inner_transactions,
        })
    }
}

// ============================================================================
// Extraction Functions
// ============================================================================

/// Parse inner transactions from the JSON data.
fn parse_inner_transactions(txn_json: &Value) -> std::result::Result<Vec<Transaction>, AlgoError> {
    let inner_txns_json = txn_json.get("inner-txns");

    match inner_txns_json {
        Some(Value::Array(arr)) => {
            let mut inner_txns = Vec::with_capacity(arr.len());
            for inner_json in arr {
                // Recursively parse inner transaction
                let inner_txn = Transaction::from_json(inner_json)?;
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
        TxnType::Payment => extract_payment_details(txn_json),
        TxnType::AssetTransfer => extract_asset_transfer_details(txn_json),
        TxnType::AssetConfig => extract_asset_config_details(txn_json),
        TxnType::AssetFreeze => extract_asset_freeze_details(txn_json),
        TxnType::AppCall => extract_app_call_details(txn_json),
        TxnType::KeyReg => extract_keyreg_details(txn_json),
        TxnType::StateProof => extract_state_proof_details(txn_json),
        TxnType::Heartbeat => extract_heartbeat_details(txn_json),
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
