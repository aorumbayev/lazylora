//! Tests for transaction graph visualization.
//!
//! This module contains all tests for the `TxnGraph` struct and related
//! widget rendering, including:
//! - Unit tests for graph construction and configuration
//! - Snapshot tests for visual validation
//! - Static fixture tests for offline validation

use crate::domain::{Transaction, TransactionDetails, TxnType};
use crate::widgets::TxnGraphWidget;

use super::txn_graph::TxnGraph;
use super::types::{GraphEntityType, GraphRepresentation};

use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};

// ============================================================================
// Test Data Factory (Mother pattern)
// ============================================================================

fn create_test_payment() -> Transaction {
    Transaction {
        id: "TEST123".to_string(),
        txn_type: TxnType::Payment,
        from: "SENDER_ADDRESS".to_string(),
        to: "RECEIVER_ADDRESS".to_string(),
        timestamp: "2024-01-01".to_string(),
        block: 12345,
        fee: 1000,
        note: "".to_string(),
        amount: 5_000_000,
        asset_id: None,
        rekey_to: None,
        group: None,
        details: TransactionDetails::default(),
        inner_transactions: Vec::new(),
    }
}

fn create_test_app_call() -> Transaction {
    Transaction {
        id: "APP123".to_string(),
        txn_type: TxnType::AppCall,
        from: "CALLER_ADDRESS".to_string(),
        to: "12345".to_string(), // App ID
        timestamp: "2024-01-01".to_string(),
        block: 12345,
        fee: 1000,
        note: "".to_string(),
        amount: 0,
        asset_id: None,
        rekey_to: None,
        group: None,
        details: TransactionDetails::default(),
        inner_transactions: Vec::new(),
    }
}

// ============================================================================
// Unit Tests: Graph Construction and Configuration
// ============================================================================

#[test]
fn test_txn_graph_construction_and_config() {
    struct TestCase {
        name: &'static str,
        graph: TxnGraph,
        expected_empty: bool,
        expected_width: usize,
        expected_spacing: usize,
    }

    let cases = [
        TestCase {
            name: "new graph is empty with defaults",
            graph: TxnGraph::new(),
            expected_empty: true,
            expected_width: TxnGraph::DEFAULT_COLUMN_WIDTH,
            expected_spacing: TxnGraph::DEFAULT_COLUMN_SPACING,
        },
        TestCase {
            name: "default() matches new()",
            graph: TxnGraph::default(),
            expected_empty: true,
            expected_width: TxnGraph::DEFAULT_COLUMN_WIDTH,
            expected_spacing: TxnGraph::DEFAULT_COLUMN_SPACING,
        },
        TestCase {
            name: "custom column width",
            graph: TxnGraph::new().with_column_width(12),
            expected_empty: true,
            expected_width: 12,
            expected_spacing: TxnGraph::DEFAULT_COLUMN_SPACING,
        },
        TestCase {
            name: "custom column spacing",
            graph: TxnGraph::new().with_column_spacing(5),
            expected_empty: true,
            expected_width: TxnGraph::DEFAULT_COLUMN_WIDTH,
            expected_spacing: 5,
        },
    ];

    for case in &cases {
        assert_eq!(
            case.graph.is_empty(),
            case.expected_empty,
            "{}: empty check failed",
            case.name
        );
        assert_eq!(
            case.graph.column_width, case.expected_width,
            "{}: column width failed",
            case.name
        );
        assert_eq!(
            case.graph.column_spacing, case.expected_spacing,
            "{}: column spacing failed",
            case.name
        );
    }
}

#[test]
fn test_txn_graph_transaction_types() {
    struct TestCase {
        name: &'static str,
        create_txn: fn() -> Transaction,
        expected_columns: usize,
        expected_rows: usize,
        expected_representation: GraphRepresentation,
        check_column_types: bool,
        expected_col_0_type: GraphEntityType,
        expected_col_1_type: Option<GraphEntityType>,
    }

    let cases = [
        TestCase {
            name: "payment",
            create_txn: create_test_payment,
            expected_columns: 2,
            expected_rows: 1,
            expected_representation: GraphRepresentation::Vector,
            check_column_types: true,
            expected_col_0_type: GraphEntityType::Account,
            expected_col_1_type: Some(GraphEntityType::Account),
        },
        TestCase {
            name: "app call",
            create_txn: create_test_app_call,
            expected_columns: 2,
            expected_rows: 1,
            expected_representation: GraphRepresentation::Vector,
            check_column_types: true,
            expected_col_0_type: GraphEntityType::Account,
            expected_col_1_type: Some(GraphEntityType::Application),
        },
        TestCase {
            name: "self transfer",
            create_txn: || {
                let mut txn = create_test_payment();
                txn.to = txn.from.clone();
                txn
            },
            expected_columns: 1,
            expected_rows: 1,
            expected_representation: GraphRepresentation::SelfLoop,
            check_column_types: false,
            expected_col_0_type: GraphEntityType::Account,
            expected_col_1_type: None,
        },
        TestCase {
            name: "keyreg",
            create_txn: || Transaction {
                id: "KEYREG123".to_string(),
                txn_type: TxnType::KeyReg,
                from: "ACCOUNT_ADDRESS".to_string(),
                to: "".to_string(),
                timestamp: "2024-01-01".to_string(),
                block: 12345,
                fee: 1000,
                note: "".to_string(),
                amount: 0,
                asset_id: None,
                rekey_to: None,
                group: None,
                details: TransactionDetails::default(),
                inner_transactions: Vec::new(),
            },
            expected_columns: 1,
            expected_rows: 1,
            expected_representation: GraphRepresentation::Point,
            check_column_types: false,
            expected_col_0_type: GraphEntityType::Account,
            expected_col_1_type: None,
        },
    ];

    for case in &cases {
        let txn = (case.create_txn)();
        let graph = TxnGraph::from_transaction(&txn);

        assert_eq!(
            graph.columns.len(),
            case.expected_columns,
            "{}: columns",
            case.name
        );
        assert_eq!(graph.rows.len(), case.expected_rows, "{}: rows", case.name);
        assert_eq!(
            graph.rows[0].representation, case.expected_representation,
            "{}: representation",
            case.name
        );

        if case.check_column_types {
            assert_eq!(
                graph.columns[0].entity_type, case.expected_col_0_type,
                "{}: col 0 type",
                case.name
            );
            if let Some(ref expected_type) = case.expected_col_1_type {
                assert_eq!(
                    graph.columns[1].entity_type, *expected_type,
                    "{}: col 1 type",
                    case.name
                );
            }
        }
    }
}

#[test]
fn test_txn_graph_layout_calculations() {
    struct TestCase {
        name: &'static str,
        width: usize,
        spacing: usize,
        col_index: usize,
        expected_center_x: usize,
        expected_start_x: usize,
    }

    let cases = [
        TestCase {
            name: "column 0",
            width: 10,
            spacing: 5,
            col_index: 0,
            expected_center_x: 5,
            expected_start_x: 0,
        },
        TestCase {
            name: "column 1",
            width: 10,
            spacing: 5,
            col_index: 1,
            expected_center_x: 20,
            expected_start_x: 15,
        },
        TestCase {
            name: "column 2",
            width: 10,
            spacing: 5,
            col_index: 2,
            expected_center_x: 35,
            expected_start_x: 30,
        },
    ];

    for case in &cases {
        let graph = TxnGraph::new()
            .with_column_width(case.width)
            .with_column_spacing(case.spacing);

        assert_eq!(
            graph.column_center_x(case.col_index),
            case.expected_center_x,
            "{}: center_x",
            case.name
        );
        assert_eq!(
            graph.column_start_x(case.col_index),
            case.expected_start_x,
            "{}: start_x",
            case.name
        );
    }

    // Test total_width separately
    let empty_graph = TxnGraph::new();
    assert_eq!(empty_graph.total_width(), 0, "empty graph width");

    let graph_with_data = TxnGraph::from_transaction(&create_test_payment());
    let expected_width = 2 * graph_with_data.column_width + graph_with_data.column_spacing;
    assert_eq!(
        graph_with_data.total_width(),
        expected_width,
        "payment graph width"
    );
}

#[test]
fn test_txn_graph_special_features() {
    // Test inner transactions
    let inner_txn = create_test_payment();
    let mut outer_txn = create_test_app_call();
    outer_txn.inner_transactions = vec![inner_txn];
    let graph = TxnGraph::from_transaction(&outer_txn);

    assert_eq!(graph.rows.len(), 2, "inner txn: rows");
    assert_eq!(graph.rows[1].depth, 1, "inner txn: depth");
    assert_eq!(graph.rows[1].parent_index, Some(0), "inner txn: parent");

    // Test rekey
    let mut txn_with_rekey = create_test_payment();
    txn_with_rekey.rekey_to = Some("NEW_AUTH_ADDRESS".to_string());
    let graph_rekey = TxnGraph::from_transaction(&txn_with_rekey);

    assert_eq!(graph_rekey.columns.len(), 3, "rekey: columns");
    assert!(
        graph_rekey.rows[0].rekey_col.is_some(),
        "rekey: rekey_col set"
    );

    // Test multiple transactions
    let transactions = vec![create_test_payment(), create_test_app_call()];
    let graph_multi = TxnGraph::from_transactions(&transactions);

    assert_eq!(graph_multi.rows.len(), 2, "multi: rows");
}

// ============================================================================
// Edge Case Snapshot Tests (Mock Data)
// ============================================================================

/// Helper to create a mock transaction for snapshot testing
fn create_mock_txn(
    txn_type: TxnType,
    from: &str,
    to: &str,
    amount: u64,
    asset_id: Option<u64>,
    rekey_to: Option<&str>,
    details: TransactionDetails,
) -> Transaction {
    Transaction {
        id: "MOCK_TXN_ID".to_string(),
        txn_type,
        from: from.to_string(),
        to: to.to_string(),
        timestamp: "2024-01-01 12:00:00".to_string(),
        block: 12345,
        fee: 1000,
        note: "".to_string(),
        amount,
        asset_id,
        rekey_to: rekey_to.map(String::from),
        group: None,
        details,
        inner_transactions: Vec::new(),
    }
}

/// Snapshot test: Payment with rekey
#[test]
fn test_snapshot_payment_with_rekey() {
    let txn = create_mock_txn(
        TxnType::Payment,
        "SENDER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
        "RECEIVER5BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
        10_000_000, // 10 ALGO
        None,
        Some("NEWAUTH3CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCA"),
        TransactionDetails::default(),
    );

    let graph = TxnGraph::from_transaction(&txn);
    let widget = TxnGraphWidget::new(&graph);

    let mut terminal = Terminal::new(TestBackend::new(70, 10)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(widget, frame.area());
        })
        .unwrap();

    assert_snapshot!("payment_with_rekey", terminal.backend());
}

/// Snapshot test: Asset opt-in (self-transfer with 0 amount)
#[test]
fn test_snapshot_asset_opt_in() {
    let txn = create_mock_txn(
        TxnType::AssetTransfer,
        "ACCOUNT7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
        "ACCOUNT7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A", // Same as sender
        0,
        Some(31566704), // USDC asset ID
        None,
        TransactionDetails::default(),
    );

    let graph = TxnGraph::from_transaction(&txn);
    let widget = TxnGraphWidget::new(&graph);

    let mut terminal = Terminal::new(TestBackend::new(50, 10)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(widget, frame.area());
        })
        .unwrap();

    assert_snapshot!("asset_opt_in", terminal.backend());
}

/// Snapshot test: Regular asset transfer
#[test]
fn test_snapshot_asset_transfer() {
    let txn = create_mock_txn(
        TxnType::AssetTransfer,
        "SENDER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
        "RECEIVER5BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
        1000000, // 1 USDC (6 decimals)
        Some(31566704),
        None,
        TransactionDetails::default(),
    );

    let graph = TxnGraph::from_transaction(&txn);
    let widget = TxnGraphWidget::new(&graph);

    let mut terminal = Terminal::new(TestBackend::new(50, 10)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(widget, frame.area());
        })
        .unwrap();

    assert_snapshot!("asset_transfer", terminal.backend());
}

/// Snapshot test: Key registration (point representation)
#[test]
fn test_snapshot_keyreg() {
    use crate::domain::KeyRegDetails;

    let txn = create_mock_txn(
        TxnType::KeyReg,
        "VALIDATOR7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAALY",
        "",
        0,
        None,
        None,
        TransactionDetails::KeyReg(KeyRegDetails {
            vote_key: Some("vote_key_base64".to_string()),
            selection_key: Some("selection_key_base64".to_string()),
            state_proof_key: None,
            vote_first: Some(1000),
            vote_last: Some(2000000),
            vote_key_dilution: Some(10000),
            non_participation: false,
        }),
    );

    let graph = TxnGraph::from_transaction(&txn);
    let widget = TxnGraphWidget::new(&graph);

    let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(widget, frame.area());
        })
        .unwrap();

    assert_snapshot!("keyreg_point", terminal.backend());
}

/// Snapshot test: App call with inner transactions
#[test]
fn test_snapshot_app_call_with_inner_txns() {
    use crate::domain::{AppCallDetails, OnComplete};

    // Inner payment
    let inner_payment = create_mock_txn(
        TxnType::Payment,
        "APPACC7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABI",
        "RECEIVER5BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
        5_000_000,
        None,
        None,
        TransactionDetails::default(),
    );

    // Inner asset transfer
    let inner_asset = create_mock_txn(
        TxnType::AssetTransfer,
        "APPACC7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABI",
        "RECEIVER5BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
        500000,
        Some(31566704),
        None,
        TransactionDetails::default(),
    );

    // Outer app call
    let outer_txn = Transaction {
        id: "OUTER_APP_CALL".to_string(),
        txn_type: TxnType::AppCall,
        from: "CALLER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        to: "1234567890".to_string(),
        timestamp: "2024-01-01 12:00:00".to_string(),
        block: 12345,
        fee: 2000,
        note: "".to_string(),
        amount: 0,
        asset_id: None,
        rekey_to: None,
        group: None,
        details: TransactionDetails::AppCall(AppCallDetails {
            app_id: 1234567890,
            created_app_id: None,
            on_complete: OnComplete::NoOp,
            approval_program: None,
            clear_state_program: None,
            app_args: vec![],
            accounts: vec![],
            foreign_apps: vec![],
            foreign_assets: vec![],
            boxes: vec![],
            global_state_schema: None,
            local_state_schema: None,
            extra_program_pages: None,
        }),
        inner_transactions: vec![inner_payment, inner_asset],
    };
    // Ensure outer has inner transactions
    assert_eq!(outer_txn.inner_transactions.len(), 2);

    let graph = TxnGraph::from_transaction(&outer_txn);
    let widget = TxnGraphWidget::new(&graph);

    let mut terminal = Terminal::new(TestBackend::new(70, 15)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(widget, frame.area());
        })
        .unwrap();

    assert_snapshot!("app_call_with_inner_txns", terminal.backend());
}

/// Snapshot test: Asset freeze
#[test]
fn test_snapshot_asset_freeze() {
    use crate::domain::AssetFreezeDetails;

    let txn = create_mock_txn(
        TxnType::AssetFreeze,
        "FREEZER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "FROZEN7BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
        0,
        Some(31566704),
        None,
        TransactionDetails::AssetFreeze(AssetFreezeDetails {
            frozen: true,
            freeze_target: "FROZEN7BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU".to_string(),
        }),
    );

    let graph = TxnGraph::from_transaction(&txn);
    let widget = TxnGraphWidget::new(&graph);

    let mut terminal = Terminal::new(TestBackend::new(50, 10)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(widget, frame.area());
        })
        .unwrap();

    assert_snapshot!("asset_freeze", terminal.backend());
}

/// Snapshot test: Asset config (create)
#[test]
fn test_snapshot_asset_config_create() {
    use crate::domain::AssetConfigDetails;

    let txn = create_mock_txn(
        TxnType::AssetConfig,
        "CREATOR7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "unknown",
        0,
        None, // No asset_id for creation
        None,
        TransactionDetails::AssetConfig(AssetConfigDetails {
            asset_id: None,
            created_asset_id: Some(987654321),
            total: Some(1_000_000_000),
            decimals: Some(6),
            default_frozen: Some(false),
            asset_name: Some("Test Token".to_string()),
            unit_name: Some("TEST".to_string()),
            url: Some("https://test.com".to_string()),
            metadata_hash: None,
            manager: Some("CREATOR7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string()),
            reserve: None,
            freeze: None,
            clawback: None,
        }),
    );

    let graph = TxnGraph::from_transaction(&txn);
    let widget = TxnGraphWidget::new(&graph);

    let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(widget, frame.area());
        })
        .unwrap();

    assert_snapshot!("asset_config_create", terminal.backend());
}

/// Snapshot test: Multiple transactions in a group
#[test]
fn test_snapshot_transaction_group() {
    let txn1 = create_mock_txn(
        TxnType::Payment,
        "SENDER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
        "RECEIVER5BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
        5_000_000,
        None,
        None,
        TransactionDetails::default(),
    );

    let txn2 = create_mock_txn(
        TxnType::AssetTransfer,
        "RECEIVER5BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBTU",
        "SENDER7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4A",
        1000000,
        Some(31566704),
        None,
        TransactionDetails::default(),
    );

    let graph = TxnGraph::from_transactions(&[txn1, txn2]);
    let widget = TxnGraphWidget::new(&graph);

    let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(widget, frame.area());
        })
        .unwrap();

    assert_snapshot!("transaction_group", terminal.backend());
}

// ============================================================================
// Static Transaction Snapshot Tests
// ============================================================================
// These tests use static fixtures from TransactionMother to validate
// graph rendering without network access. Offline-capable and deterministic.

use crate::test_utils::TransactionMother;

/// Test case for static transaction snapshot tests
struct StaticTxnTestCase {
    #[allow(dead_code)]
    name: &'static str,
    create_txn: fn() -> Transaction,
    snapshot_name: &'static str,
    width: u16,
    height: u16,
}

/// All static transaction test cases - one place to see all tested transactions
const STATIC_TXN_TEST_CASES: &[StaticTxnTestCase] = &[
    // Payment transactions
    StaticTxnTestCase {
        name: "mainnet_payment",
        create_txn: TransactionMother::mainnet_payment,
        snapshot_name: "real_mainnet_payment",
        width: 80,
        height: 15,
    },
    StaticTxnTestCase {
        name: "mainnet_payment_close_remainder",
        create_txn: TransactionMother::mainnet_payment_close_remainder,
        snapshot_name: "real_mainnet_payment_close_remainder",
        width: 80,
        height: 15,
    },
    StaticTxnTestCase {
        name: "mainnet_payment_close_remainder_recent",
        create_txn: TransactionMother::mainnet_payment_close_remainder,
        snapshot_name: "real_mainnet_payment_close_remainder_recent",
        width: 80,
        height: 15,
    },
    // Asset transfer transactions
    StaticTxnTestCase {
        name: "mainnet_asset_transfer",
        create_txn: TransactionMother::mainnet_asset_transfer,
        snapshot_name: "real_mainnet_asset_transfer",
        width: 80,
        height: 15,
    },
    StaticTxnTestCase {
        name: "mainnet_asset_opt_in",
        create_txn: TransactionMother::mainnet_asset_opt_in,
        snapshot_name: "real_mainnet_asset_opt_in",
        width: 60,
        height: 15,
    },
    StaticTxnTestCase {
        name: "mainnet_asset_close_to",
        create_txn: TransactionMother::mainnet_asset_close_to,
        snapshot_name: "real_mainnet_asset_close_to",
        width: 80,
        height: 15,
    },
    StaticTxnTestCase {
        name: "testnet_asset_clawback",
        create_txn: TransactionMother::testnet_asset_clawback,
        snapshot_name: "real_testnet_asset_clawback",
        width: 80,
        height: 15,
    },
    // Asset config transactions
    StaticTxnTestCase {
        name: "mainnet_asset_config_create",
        create_txn: TransactionMother::mainnet_asset_config_create,
        snapshot_name: "real_mainnet_asset_config_create",
        width: 60,
        height: 15,
    },
    StaticTxnTestCase {
        name: "mainnet_asset_config_reconfigure",
        create_txn: TransactionMother::mainnet_asset_config_reconfigure,
        snapshot_name: "real_mainnet_asset_config_reconfigure",
        width: 60,
        height: 15,
    },
    // Asset freeze transactions
    StaticTxnTestCase {
        name: "mainnet_asset_freeze",
        create_txn: TransactionMother::mainnet_asset_freeze,
        snapshot_name: "real_mainnet_asset_freeze",
        width: 70,
        height: 15,
    },
    StaticTxnTestCase {
        name: "mainnet_asset_freeze_recent",
        create_txn: TransactionMother::mainnet_asset_freeze,
        snapshot_name: "real_mainnet_asset_freeze_recent",
        width: 70,
        height: 15,
    },
    // Key registration transactions
    StaticTxnTestCase {
        name: "mainnet_keyreg_offline",
        create_txn: TransactionMother::mainnet_keyreg_offline,
        snapshot_name: "real_mainnet_keyreg",
        width: 60,
        height: 15,
    },
    StaticTxnTestCase {
        name: "mainnet_keyreg_online",
        create_txn: TransactionMother::mainnet_keyreg_online,
        snapshot_name: "real_mainnet_keyreg_online",
        width: 60,
        height: 15,
    },
    // App call transactions
    StaticTxnTestCase {
        name: "mainnet_app_call_inner_txns",
        create_txn: TransactionMother::mainnet_app_call_with_inner_txns,
        snapshot_name: "real_mainnet_app_call_inner_txns",
        width: 100,
        height: 30,
    },
    StaticTxnTestCase {
        name: "mainnet_app_call_mixed_inner",
        create_txn: TransactionMother::mainnet_app_call_mixed_inner,
        snapshot_name: "real_mainnet_app_call_mixed_inner",
        width: 100,
        height: 30,
    },
    StaticTxnTestCase {
        name: "mainnet_large_app_call",
        create_txn: TransactionMother::mainnet_large_app_call,
        snapshot_name: "txn_graph_widget_mainnet_snapshot",
        width: 100,
        height: 30,
    },
    // Rekey transactions
    StaticTxnTestCase {
        name: "testnet_rekey",
        create_txn: TransactionMother::testnet_rekey,
        snapshot_name: "real_testnet_rekey",
        width: 80,
        height: 15,
    },
];

/// Helper to run a single static transaction snapshot test
fn run_static_txn_snapshot_test(case: &StaticTxnTestCase) {
    let txn = (case.create_txn)();
    let graph = TxnGraph::from_transaction(&txn);
    let widget = TxnGraphWidget::new(&graph);

    let mut terminal = Terminal::new(TestBackend::new(case.width, case.height))
        .expect("terminal creation should succeed");
    terminal
        .draw(|frame| {
            frame.render_widget(widget, frame.area());
        })
        .expect("draw should succeed");

    assert_snapshot!(case.snapshot_name, terminal.backend());
}

/// Parameterized test for all static transaction fixtures
/// Runs each test case from STATIC_TXN_TEST_CASES - fully offline
#[test]
fn test_static_txn_snapshots() {
    for case in STATIC_TXN_TEST_CASES {
        run_static_txn_snapshot_test(case);
    }
}
