use super::AlgoClient;
use crate::domain::{Network, OnComplete, Transaction, TransactionDetails, TxnType};
use crate::state::SearchType;
use crate::test_utils::JsonMother;
use rstest::rstest;

#[test]
fn test_search_suggestions() {
    assert!(
        AlgoClient::get_search_suggestions("", SearchType::Account)
            .contains("Enter an Algorand address")
    );

    // "ABC" looks like an NFD name, so it shows NFD suggestion
    assert!(
        AlgoClient::get_search_suggestions("ABC", SearchType::Account).contains("NFD")
            || AlgoClient::get_search_suggestions("ABC", SearchType::Account).contains("NFD name")
    );

    // Test with clearly invalid input that's too short and not an NFD
    assert!(AlgoClient::get_search_suggestions("A1!", SearchType::Account).contains("too short"));

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
        AlgoClient::get_search_suggestions("ABC", SearchType::Transaction).contains("too short")
    );

    assert!(
        AlgoClient::get_search_suggestions("", SearchType::Block).contains("Enter a block number")
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

// ========================================================================
// Transaction Parsing Tests (consolidated with rstest)
// ========================================================================

/// Consolidated test for transaction type detection from JSON.
/// Uses JsonMother for test data and rstest for parametrization.
#[rstest]
#[case::payment(JsonMother::payment(), TxnType::Payment, "test-payment-id")]
#[case::payment_close(JsonMother::payment_with_close(), TxnType::Payment, "close-txn-id")]
#[case::asset_transfer(JsonMother::asset_transfer(), TxnType::AssetTransfer, "asset-txn-id")]
#[case::asset_clawback(
    JsonMother::asset_transfer_clawback(),
    TxnType::AssetTransfer,
    "clawback-txn-id"
)]
#[case::asset_config_create(
    JsonMother::asset_config_create(),
    TxnType::AssetConfig,
    "asset-create-id"
)]
#[case::asset_config_modify(
    JsonMother::asset_config_modify(),
    TxnType::AssetConfig,
    "asset-modify-id"
)]
#[case::asset_freeze(JsonMother::asset_freeze(), TxnType::AssetFreeze, "freeze-txn-id")]
#[case::app_call_create(JsonMother::app_call_create(), TxnType::AppCall, "app-create-id")]
#[case::app_call_boxes(
    JsonMother::app_call_with_boxes(),
    TxnType::AppCall,
    "app-call-boxes-id"
)]
#[case::keyreg_online(JsonMother::keyreg_online(), TxnType::KeyReg, "keyreg-online-id")]
#[case::keyreg_offline(JsonMother::keyreg_offline(), TxnType::KeyReg, "keyreg-offline-id")]
#[case::state_proof(JsonMother::state_proof(), TxnType::StateProof, "state-proof-id")]
#[case::heartbeat(JsonMother::heartbeat(), TxnType::Heartbeat, "heartbeat-id")]
#[case::empty_defaults(JsonMother::empty(), TxnType::Unknown, "unknown")]
fn test_transaction_type_parsing(
    #[case] json: serde_json::Value,
    #[case] expected_type: TxnType,
    #[case] expected_id: &str,
) {
    let txn = Transaction::from_json(&json).unwrap();
    assert_eq!(txn.txn_type, expected_type);
    assert_eq!(txn.id, expected_id);
}

/// Test payment transaction details parsing.
#[test]
fn test_payment_details_parsing() {
    // Basic payment
    let txn = Transaction::from_json(&JsonMother::payment()).unwrap();
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

    // Payment with close
    let txn_close = Transaction::from_json(&JsonMother::payment_with_close()).unwrap();
    match txn_close.details {
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

/// Test asset transfer details parsing.
#[test]
fn test_asset_transfer_details_parsing() {
    // Basic transfer
    let txn = Transaction::from_json(&JsonMother::asset_transfer()).unwrap();
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

    // Clawback
    let txn_clawback = Transaction::from_json(&JsonMother::asset_transfer_clawback()).unwrap();
    match txn_clawback.details {
        TransactionDetails::AssetTransfer(details) => {
            assert_eq!(details.asset_sender, Some("CLAWBACK_TARGET".to_string()));
            assert_eq!(details.close_to, Some("CLOSE_ADDRESS".to_string()));
            assert_eq!(details.close_amount, Some(25));
        }
        _ => panic!("Expected AssetTransfer details"),
    }
}

/// Test asset config details parsing.
#[test]
fn test_asset_config_details_parsing() {
    // Create
    let txn = Transaction::from_json(&JsonMother::asset_config_create()).unwrap();
    match &txn.details {
        TransactionDetails::AssetConfig(details) => {
            assert!(details.asset_id.is_none());
            assert_eq!(details.created_asset_id, Some(123_456_789));
            assert_eq!(details.total, Some(1_000_000));
            assert_eq!(details.decimals, Some(6));
            assert_eq!(details.default_frozen, Some(false));
            assert_eq!(details.asset_name, Some("Test Token".to_string()));
            assert_eq!(details.unit_name, Some("TEST".to_string()));
            assert!(txn.details.is_creation());
            assert_eq!(txn.details.created_id(), Some(123_456_789));
        }
        _ => panic!("Expected AssetConfig details"),
    }

    // Modify
    let txn_modify = Transaction::from_json(&JsonMother::asset_config_modify()).unwrap();
    match &txn_modify.details {
        TransactionDetails::AssetConfig(details) => {
            assert_eq!(details.asset_id, Some(123_456_789));
            assert!(details.created_asset_id.is_none());
            assert!(!txn_modify.details.is_creation());
        }
        _ => panic!("Expected AssetConfig details"),
    }
}

/// Test asset freeze details parsing.
#[test]
fn test_asset_freeze_details_parsing() {
    let txn = Transaction::from_json(&JsonMother::asset_freeze()).unwrap();
    assert_eq!(txn.to, "TARGET_ADDRESS");

    match txn.details {
        TransactionDetails::AssetFreeze(details) => {
            assert!(details.frozen);
            assert_eq!(details.freeze_target, "TARGET_ADDRESS");
        }
        _ => panic!("Expected AssetFreeze details"),
    }
}

/// Test app call details parsing.
#[test]
fn test_app_call_details_parsing() {
    // Create
    let txn = Transaction::from_json(&JsonMother::app_call_create()).unwrap();
    match &txn.details {
        TransactionDetails::AppCall(details) => {
            assert_eq!(details.app_id, 0);
            assert_eq!(details.created_app_id, Some(987_654_321));
            assert_eq!(details.on_complete, OnComplete::NoOp);
            assert_eq!(details.approval_program, Some("BIAKBQAKAI==".to_string()));
            assert_eq!(details.app_args, vec!["YXJnMQ==", "YXJnMg=="]);
            assert_eq!(details.foreign_apps, vec![111, 222]);
            assert!(txn.details.is_creation());
        }
        _ => panic!("Expected AppCall details"),
    }

    // With boxes
    let txn_boxes = Transaction::from_json(&JsonMother::app_call_with_boxes()).unwrap();
    match &txn_boxes.details {
        TransactionDetails::AppCall(details) => {
            assert_eq!(details.app_id, 123_456);
            assert_eq!(details.boxes.len(), 2);
            assert_eq!(details.boxes[0].app_id, 0);
            assert_eq!(details.boxes[0].name, "Ym94MQ==");
            assert!(!txn_boxes.details.is_creation());
        }
        _ => panic!("Expected AppCall details"),
    }
}

/// Test on-complete variants using rstest parametrization.
#[rstest]
#[case::noop("noop", OnComplete::NoOp)]
#[case::optin("optin", OnComplete::OptIn)]
#[case::closeout("closeout", OnComplete::CloseOut)]
#[case::clearstate("clearstate", OnComplete::ClearState)]
#[case::update_full("updateapplication", OnComplete::UpdateApplication)]
#[case::update_short("update", OnComplete::UpdateApplication)]
#[case::delete_full("deleteapplication", OnComplete::DeleteApplication)]
#[case::delete_short("delete", OnComplete::DeleteApplication)]
fn test_on_complete_parsing(#[case] input: &str, #[case] expected: OnComplete) {
    let json = JsonMother::app_call_on_complete(input);
    let txn = Transaction::from_json(&json).unwrap();

    match txn.details {
        TransactionDetails::AppCall(details) => {
            assert_eq!(details.on_complete, expected);
        }
        _ => panic!("Expected AppCall details"),
    }
}

/// Test keyreg details parsing.
#[test]
fn test_keyreg_details_parsing() {
    // Online
    let txn = Transaction::from_json(&JsonMother::keyreg_online()).unwrap();
    match txn.details {
        TransactionDetails::KeyReg(details) => {
            assert_eq!(details.vote_key, Some("dm90ZUtleQ==".to_string()));
            assert_eq!(details.selection_key, Some("c2VsS2V5".to_string()));
            assert_eq!(details.vote_first, Some(1000));
            assert_eq!(details.vote_last, Some(2_000_000));
            assert!(!details.non_participation);
        }
        _ => panic!("Expected KeyReg details"),
    }

    // Offline
    let txn_offline = Transaction::from_json(&JsonMother::keyreg_offline()).unwrap();
    match txn_offline.details {
        TransactionDetails::KeyReg(details) => {
            assert!(details.vote_key.is_none());
            assert!(details.non_participation);
        }
        _ => panic!("Expected KeyReg details"),
    }
}

/// Test state proof details parsing.
#[test]
fn test_state_proof_details_parsing() {
    let txn = Transaction::from_json(&JsonMother::state_proof()).unwrap();
    match txn.details {
        TransactionDetails::StateProof(details) => {
            assert_eq!(details.state_proof_type, Some(0));
            assert_eq!(details.message, Some("deadbeef".to_string()));
        }
        _ => panic!("Expected StateProof details"),
    }
}

/// Test heartbeat details parsing.
#[test]
fn test_heartbeat_details_parsing() {
    let txn = Transaction::from_json(&JsonMother::heartbeat()).unwrap();
    match txn.details {
        TransactionDetails::Heartbeat(details) => {
            assert_eq!(details.hb_address, Some("HEARTBEAT_TARGET".to_string()));
            assert_eq!(details.hb_key_dilution, Some(10_000));
        }
        _ => panic!("Expected Heartbeat details"),
    }
}

/// Test empty JSON defaults.
#[test]
fn test_empty_json_defaults() {
    let txn = Transaction::from_json(&JsonMother::empty()).unwrap();
    assert_eq!(txn.id, "unknown");
    assert_eq!(txn.from, "unknown");
    assert_eq!(txn.txn_type, TxnType::Unknown);
    assert_eq!(txn.amount, 0);
    assert_eq!(txn.fee, 0);
    assert_eq!(txn.details, TransactionDetails::None);
}
