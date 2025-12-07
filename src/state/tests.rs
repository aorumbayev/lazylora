//! Tests for the state module.

use tokio::sync::{mpsc, watch};

use super::{
    App, DataState, DetailViewMode, Focus, NavigationState, PopupState, SearchType, UiState,
};
use crate::client::AlgoClient;
use crate::commands::{AppCommand, InputContext, map_key};
use crate::domain::{
    AlgoBlock, BlockDetails, BlockInfo, Network, NetworkConfig, SearchResultItem, Transaction,
    TxnType,
};

// ========================================================================
// Test Helper Functions
// ========================================================================

/// Creates a test App instance without network operations.
fn create_test_app() -> App {
    let (message_tx, message_rx) = mpsc::unbounded_channel();
    let (live_updates_tx, _live_updates_rx) = watch::channel(true);
    let (network_tx, _network_rx) = watch::channel(NetworkConfig::BuiltIn(Network::TestNet));
    let network_config = NetworkConfig::BuiltIn(Network::TestNet);

    App {
        nav: NavigationState::new(),
        data: DataState::new(),
        ui: UiState::new(),
        network: Network::TestNet,
        network_config: network_config.clone(),
        available_networks: vec![network_config.clone()],
        show_live: true,
        exit: false,
        animation_tick: 0,
        message_tx,
        message_rx,
        live_updates_tx,
        network_tx,
        client: AlgoClient::from_config(&network_config).expect("test client should build"),
        startup_options: None,
    }
}

/// Creates a test block with the given ID and transaction count.
fn create_test_block(id: u64, txn_count: u16) -> AlgoBlock {
    AlgoBlock::new(id, txn_count, format!("2023-11-14 {id}"))
}

/// Creates a test transaction with the given ID and type.
fn create_test_transaction(id: &str, txn_type: TxnType, block: u64) -> Transaction {
    Transaction {
        id: id.to_string(),
        txn_type,
        from: "SENDER".to_string(),
        to: "RECEIVER".to_string(),
        timestamp: "2023-11-14".to_string(),
        block,
        fee: 1000,
        note: "Test".to_string(),
        amount: 1_000_000,
        asset_id: None,
        rekey_to: None,
        group: None,
        details: crate::domain::TransactionDetails::None,
        inner_transactions: Vec::new(),
    }
}

// ========================================================================
// merge_blocks() Tests
// ========================================================================

#[test]
fn test_merge_blocks_all_scenarios() {
    struct TestCase {
        name: &'static str,
        initial_blocks: Vec<AlgoBlock>,
        new_blocks: Vec<AlgoBlock>,
        expected_len: usize,
        expected_order: Vec<u64>, // Expected IDs in descending order
    }

    let cases = [
        TestCase {
            name: "empty input preserves existing blocks",
            initial_blocks: vec![create_test_block(100, 5)],
            new_blocks: vec![],
            expected_len: 1,
            expected_order: vec![100],
        },
        TestCase {
            name: "adds new blocks in descending order",
            initial_blocks: vec![create_test_block(100, 5)],
            new_blocks: vec![create_test_block(101, 3), create_test_block(102, 7)],
            expected_len: 3,
            expected_order: vec![102, 101, 100],
        },
        TestCase {
            name: "deduplicates existing blocks",
            initial_blocks: vec![create_test_block(101, 3), create_test_block(100, 5)],
            new_blocks: vec![create_test_block(101, 3), create_test_block(102, 7)],
            expected_len: 3,
            expected_order: vec![102, 101, 100],
        },
        TestCase {
            name: "maintains sort order with interleaved blocks",
            initial_blocks: vec![
                create_test_block(105, 2),
                create_test_block(103, 4),
                create_test_block(101, 6),
            ],
            new_blocks: vec![create_test_block(104, 1), create_test_block(102, 3)],
            expected_len: 5,
            expected_order: vec![105, 104, 103, 102, 101],
        },
    ];

    for case in cases {
        let mut app = create_test_app();
        app.data.blocks = case.initial_blocks;
        app.merge_blocks(case.new_blocks);

        assert_eq!(app.data.blocks.len(), case.expected_len, "{}", case.name);
        for (block, expected_id) in app.data.blocks.iter().zip(&case.expected_order) {
            assert_eq!(
                block.id, *expected_id,
                "{}: block order mismatch",
                case.name
            );
        }
    }

    // Test capacity limit separately (requires different setup)
    let mut app = create_test_app();
    app.data
        .blocks
        .extend((0..98).map(|i| create_test_block(1000 - i, 1)));
    let new_blocks: Vec<AlgoBlock> = (1..=5).map(|i| create_test_block(2000 + i, 1)).collect();
    app.merge_blocks(new_blocks);

    assert_eq!(app.data.blocks.len(), 100, "truncates to 100 blocks");
    assert_eq!(app.data.blocks[0].id, 2005, "keeps newest blocks first");
    assert_eq!(app.data.blocks[99].id, 906, "keeps newest blocks last");
}

// ========================================================================
// merge_transactions() Tests
// ========================================================================

#[test]
fn test_merge_transactions_all_scenarios() {
    struct TestCase {
        name: &'static str,
        initial_txns: Vec<Transaction>,
        new_txns: Vec<Transaction>,
        expected_len: usize,
        expected_ids: Vec<&'static str>,
        unexpected_ids: Vec<&'static str>,
    }

    let cases = [
        TestCase {
            name: "empty input preserves existing transactions",
            initial_txns: vec![create_test_transaction("TXN1", TxnType::Payment, 100)],
            new_txns: vec![],
            expected_len: 1,
            expected_ids: vec!["TXN1"],
            unexpected_ids: vec![],
        },
        TestCase {
            name: "adds new transactions",
            initial_txns: vec![create_test_transaction("TXN1", TxnType::Payment, 100)],
            new_txns: vec![
                create_test_transaction("TXN2", TxnType::AppCall, 101),
                create_test_transaction("TXN3", TxnType::AssetTransfer, 102),
            ],
            expected_len: 3,
            expected_ids: vec!["TXN1", "TXN2", "TXN3"],
            unexpected_ids: vec![],
        },
        TestCase {
            name: "deduplicates transactions by ID",
            initial_txns: vec![
                create_test_transaction("TXN1", TxnType::Payment, 100),
                create_test_transaction("TXN2", TxnType::AppCall, 101),
            ],
            new_txns: vec![
                create_test_transaction("TXN2", TxnType::AppCall, 101),
                create_test_transaction("TXN3", TxnType::AssetTransfer, 102),
            ],
            expected_len: 3,
            expected_ids: vec!["TXN1", "TXN2", "TXN3"],
            unexpected_ids: vec![],
        },
    ];

    for case in cases {
        let mut app = create_test_app();
        app.data.transactions = case.initial_txns;
        app.merge_transactions(case.new_txns);

        assert_eq!(
            app.data.transactions.len(),
            case.expected_len,
            "{}",
            case.name
        );

        let ids: Vec<&str> = app
            .data
            .transactions
            .iter()
            .map(|t| t.id.as_str())
            .collect();
        for expected_id in case.expected_ids {
            assert!(
                ids.contains(&expected_id),
                "{}: missing expected ID {expected_id}",
                case.name
            );
        }
        for unexpected_id in case.unexpected_ids {
            assert!(
                !ids.contains(&unexpected_id),
                "{}: found unexpected ID {unexpected_id}",
                case.name
            );
        }
    }

    // Test capacity limit with new transactions
    let mut app = create_test_app();
    app.data.transactions.extend(
        (0..98).map(|i| create_test_transaction(&format!("OLD_TXN_{i}"), TxnType::Payment, 100)),
    );
    let new_txns: Vec<Transaction> = (1..=5)
        .map(|i| create_test_transaction(&format!("NEW_TXN_{i}"), TxnType::Payment, 200))
        .collect();
    app.merge_transactions(new_txns);

    assert_eq!(app.data.transactions.len(), 100, "caps at 100 transactions");
    let ids: Vec<&str> = app
        .data
        .transactions
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    assert!(ids.contains(&"NEW_TXN_1"), "keeps new transactions");
    assert!(ids.contains(&"NEW_TXN_5"), "keeps new transactions");

    // Test prioritization: new transactions replace old when at capacity
    let mut app = create_test_app();
    app.data.transactions.extend(
        (0..100).map(|i| create_test_transaction(&format!("OLD_{i}"), TxnType::Payment, 100)),
    );
    let new_txns: Vec<Transaction> = (1..=10)
        .map(|i| create_test_transaction(&format!("NEW_{i}"), TxnType::Payment, 200))
        .collect();
    app.merge_transactions(new_txns);

    assert_eq!(app.data.transactions.len(), 100, "prioritizes new over old");
    let ids: Vec<&str> = app
        .data
        .transactions
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    for i in 1..=10 {
        assert!(
            ids.contains(&format!("NEW_{i}").as_str()),
            "all new transactions present"
        );
    }
    assert!(
        !ids.contains(&"OLD_99"),
        "oldest transactions dropped when at capacity"
    );
}

// ========================================================================
// sync_selections() Tests
// ========================================================================

#[test]
fn test_sync_selections_all_scenarios() {
    struct BlockTestCase {
        name: &'static str,
        blocks: Vec<AlgoBlock>,
        selected_id: Option<u64>,
        expected_index: Option<usize>,
        expected_id_after: Option<u64>,
    }

    let block_cases = [
        BlockTestCase {
            name: "block still exists after merge",
            blocks: vec![
                create_test_block(103, 1),
                create_test_block(102, 2),
                create_test_block(101, 3),
            ],
            selected_id: Some(102),
            expected_index: Some(1),
            expected_id_after: Some(102),
        },
        BlockTestCase {
            name: "block removed clears selection",
            blocks: vec![create_test_block(103, 1), create_test_block(101, 3)],
            selected_id: Some(102),
            expected_index: None,
            expected_id_after: None,
        },
        BlockTestCase {
            name: "no selection remains None",
            blocks: vec![create_test_block(100, 1)],
            selected_id: None,
            expected_index: None,
            expected_id_after: None,
        },
    ];

    for case in block_cases {
        let mut app = create_test_app();
        app.data.blocks = case.blocks;
        app.nav.selected_block_id = case.selected_id;
        app.sync_selections();

        assert_eq!(
            app.nav.selected_block_index, case.expected_index,
            "{}",
            case.name
        );
        assert_eq!(
            app.nav.selected_block_id, case.expected_id_after,
            "{}",
            case.name
        );
    }

    struct TxnTestCase {
        name: &'static str,
        transactions: Vec<Transaction>,
        selected_id: Option<String>,
        expected_index: Option<usize>,
        expected_id_after: Option<String>,
    }

    let txn_cases = [
        TxnTestCase {
            name: "transaction still exists after merge",
            transactions: vec![
                create_test_transaction("TXN1", TxnType::Payment, 100),
                create_test_transaction("TXN2", TxnType::AppCall, 101),
                create_test_transaction("TXN3", TxnType::AssetTransfer, 102),
            ],
            selected_id: Some("TXN2".to_string()),
            expected_index: Some(1),
            expected_id_after: Some("TXN2".to_string()),
        },
        TxnTestCase {
            name: "transaction removed clears selection",
            transactions: vec![
                create_test_transaction("TXN1", TxnType::Payment, 100),
                create_test_transaction("TXN3", TxnType::AssetTransfer, 102),
            ],
            selected_id: Some("TXN2".to_string()),
            expected_index: None,
            expected_id_after: None,
        },
    ];

    for case in txn_cases {
        let mut app = create_test_app();
        app.data.transactions = case.transactions;
        app.nav.selected_transaction_id = case.selected_id;
        app.sync_selections();

        assert_eq!(
            app.nav.selected_transaction_index, case.expected_index,
            "{}",
            case.name
        );
        assert_eq!(
            app.nav.selected_transaction_id, case.expected_id_after,
            "{}",
            case.name
        );
    }
}

// ========================================================================
// move_selection_up/down() Tests
// ========================================================================

#[test]
fn test_navigation_all_scenarios() {
    struct TestCase {
        name: &'static str,
        focus: Focus,
        initial_index: Option<usize>,
        initial_id: Option<String>,
        action: fn(&mut App),
        expected_index: Option<usize>,
        expected_id: Option<String>,
    }

    // Setup: 3-item block list [103, 102, 101]
    let block_list = vec![
        create_test_block(103, 1),
        create_test_block(102, 2),
        create_test_block(101, 3),
    ];

    let cases = [
        TestCase {
            name: "move up from middle",
            focus: Focus::Blocks,
            initial_index: Some(1),
            initial_id: Some("102".to_string()),
            action: App::move_selection_up,
            expected_index: Some(0),
            expected_id: Some("103".to_string()),
        },
        TestCase {
            name: "move up from top stays at top",
            focus: Focus::Blocks,
            initial_index: Some(0),
            initial_id: Some("103".to_string()),
            action: App::move_selection_up,
            expected_index: Some(0),
            expected_id: Some("103".to_string()),
        },
        TestCase {
            name: "move up with no selection selects first",
            focus: Focus::Blocks,
            initial_index: None,
            initial_id: None,
            action: App::move_selection_up,
            expected_index: Some(0),
            expected_id: Some("103".to_string()),
        },
        TestCase {
            name: "move down from middle",
            focus: Focus::Blocks,
            initial_index: Some(1),
            initial_id: Some("102".to_string()),
            action: App::move_selection_down,
            expected_index: Some(2),
            expected_id: Some("101".to_string()),
        },
        TestCase {
            name: "move down from bottom stays at bottom",
            focus: Focus::Blocks,
            initial_index: Some(2),
            initial_id: Some("101".to_string()),
            action: App::move_selection_down,
            expected_index: Some(2),
            expected_id: Some("101".to_string()),
        },
        TestCase {
            name: "move down with no selection selects first",
            focus: Focus::Blocks,
            initial_index: None,
            initial_id: None,
            action: App::move_selection_down,
            expected_index: Some(0),
            expected_id: Some("103".to_string()),
        },
    ];

    for case in cases {
        let mut app = create_test_app();
        app.ui.focus = case.focus;
        app.data.blocks = block_list.clone();
        app.nav.selected_block_index = case.initial_index;
        app.nav.selected_block_id = case.initial_id.as_ref().and_then(|s| s.parse().ok());

        (case.action)(&mut app);

        assert_eq!(
            app.nav.selected_block_index, case.expected_index,
            "{}",
            case.name
        );
        assert_eq!(
            app.nav.selected_block_id,
            case.expected_id.as_ref().and_then(|s| s.parse().ok()),
            "{}",
            case.name
        );
    }

    // Test empty list behavior
    let mut app = create_test_app();
    app.ui.focus = Focus::Blocks;
    app.data.blocks = Vec::new();

    app.move_selection_up();
    assert_eq!(app.nav.selected_block_index, None, "empty list move up");

    app.move_selection_down();
    assert_eq!(app.nav.selected_block_index, None, "empty list move down");

    // Test transaction navigation
    let mut app = create_test_app();
    app.ui.focus = Focus::Transactions;
    app.data.transactions = vec![
        create_test_transaction("TXN1", TxnType::Payment, 100),
        create_test_transaction("TXN2", TxnType::AppCall, 101),
        create_test_transaction("TXN3", TxnType::AssetTransfer, 102),
    ];

    app.move_selection_down();
    assert_eq!(
        app.nav.selected_transaction_index,
        Some(0),
        "transactions: first down selects 0"
    );

    app.move_selection_down();
    assert_eq!(
        app.nav.selected_transaction_index,
        Some(1),
        "transactions: second down moves to 1"
    );

    app.move_selection_up();
    assert_eq!(
        app.nav.selected_transaction_index,
        Some(0),
        "transactions: up moves back to 0"
    );
}

// ========================================================================
// handle_dismiss() Tests
// ========================================================================

#[test]
fn test_handle_dismiss_all_contexts() {
    #[allow(dead_code)]
    struct TestCase {
        name: &'static str,
        setup: fn(&mut App),
        verify: fn(&App),
    }

    let cases = [
        TestCase {
            name: "transaction details clears graph scroll and view state",
            setup: |app| {
                app.nav.show_transaction_details = true;
                app.data.viewed_transaction =
                    Some(create_test_transaction("TXN1", TxnType::Payment, 100));
                app.nav.graph_scroll_x = 10;
                app.nav.graph_scroll_y = 5;
            },
            verify: |app| {
                assert!(!app.nav.show_transaction_details);
                assert!(!app.nav.show_block_details);
                assert!(!app.nav.show_account_details);
                assert!(!app.nav.show_asset_details);
                assert!(app.data.viewed_transaction.is_none());
                assert_eq!(app.nav.graph_scroll_x, 0);
                assert_eq!(app.nav.graph_scroll_y, 0);
            },
        },
        TestCase {
            name: "block details closes view",
            setup: |app| {
                app.nav.show_block_details = true;
                app.data.block_details = Some(BlockDetails::new(
                    BlockInfo::new(
                        100,
                        "2023-11-14".to_string(),
                        5,
                        "PROPOSER".to_string(),
                        "SEED".to_string(),
                    ),
                    Vec::new(),
                ));
            },
            verify: |app| {
                assert!(!app.nav.show_block_details);
            },
        },
        TestCase {
            name: "search popup clears search results",
            setup: |app| {
                app.ui.popup_state =
                    PopupState::SearchWithType("query".to_string(), SearchType::Transaction);
                app.data.filtered_search_results.push((
                    0,
                    SearchResultItem::Transaction(Box::new(create_test_transaction(
                        "TXN1",
                        TxnType::Payment,
                        100,
                    ))),
                ));
            },
            verify: |app| {
                assert_eq!(app.ui.popup_state, PopupState::None);
                assert!(app.data.filtered_search_results.is_empty());
                assert!(!app.ui.viewing_search_result);
            },
        },
        TestCase {
            name: "search results popup clears results",
            setup: |app| {
                app.ui.popup_state = PopupState::SearchResults(vec![(
                    0,
                    SearchResultItem::Transaction(Box::new(create_test_transaction(
                        "TXN1",
                        TxnType::Payment,
                        100,
                    ))),
                )]);
                app.data.filtered_search_results.push((
                    0,
                    SearchResultItem::Transaction(Box::new(create_test_transaction(
                        "TXN1",
                        TxnType::Payment,
                        100,
                    ))),
                ));
            },
            verify: |app| {
                assert_eq!(app.ui.popup_state, PopupState::None);
                assert!(app.data.filtered_search_results.is_empty());
            },
        },
        TestCase {
            name: "network select popup dismisses",
            setup: |app| {
                app.ui.popup_state = PopupState::NetworkSelect(1);
            },
            verify: |app| {
                assert_eq!(app.ui.popup_state, PopupState::None);
            },
        },
        TestCase {
            name: "message popup dismisses",
            setup: |app| {
                app.ui.popup_state = PopupState::Message("Test message".to_string());
            },
            verify: |app| {
                assert_eq!(app.ui.popup_state, PopupState::None);
            },
        },
        TestCase {
            name: "no popup or details does nothing",
            setup: |app| {
                app.ui.popup_state = PopupState::None;
                app.nav.show_transaction_details = false;
            },
            verify: |app| {
                assert_eq!(app.ui.popup_state, PopupState::None);
            },
        },
    ];

    for case in cases {
        let mut app = create_test_app();
        (case.setup)(&mut app);
        app.handle_dismiss();
        (case.verify)(&app);
    }
}

// ========================================================================
// get_input_context() Tests
// ========================================================================

#[test]
fn test_get_input_context_all_states() {
    struct TestCase {
        name: &'static str,
        setup: fn(&mut App),
        expected: InputContext,
    }

    let cases = [
        TestCase {
            name: "main context by default",
            setup: |_| {},
            expected: InputContext::Main,
        },
        TestCase {
            name: "network select popup",
            setup: |app| {
                app.ui.popup_state = PopupState::NetworkSelect(0);
            },
            expected: InputContext::NetworkSelect,
        },
        TestCase {
            name: "search input popup",
            setup: |app| {
                app.ui.popup_state =
                    PopupState::SearchWithType(String::new(), SearchType::Transaction);
            },
            expected: InputContext::SearchInput,
        },
        TestCase {
            name: "search results popup",
            setup: |app| {
                app.ui.popup_state = PopupState::SearchResults(vec![(
                    0,
                    SearchResultItem::Transaction(Box::new(create_test_transaction(
                        "TXN1",
                        TxnType::Payment,
                        100,
                    ))),
                )]);
            },
            expected: InputContext::SearchResults,
        },
        TestCase {
            name: "message popup",
            setup: |app| {
                app.ui.popup_state = PopupState::Message("Test".to_string());
            },
            expected: InputContext::MessagePopup,
        },
        TestCase {
            name: "block detail view",
            setup: |app| {
                app.nav.show_block_details = true;
            },
            expected: InputContext::BlockDetailView,
        },
        TestCase {
            name: "transaction detail view (table mode)",
            setup: |app| {
                app.nav.show_transaction_details = true;
                app.ui.detail_view_mode = DetailViewMode::Table;
            },
            expected: InputContext::TxnDetailViewTable,
        },
        TestCase {
            name: "transaction detail view (visual mode)",
            setup: |app| {
                app.nav.show_transaction_details = true;
                app.ui.detail_view_mode = DetailViewMode::Visual;
            },
            expected: InputContext::DetailView,
        },
        TestCase {
            name: "account detail view",
            setup: |app| {
                app.nav.show_account_details = true;
            },
            expected: InputContext::AccountDetailView,
        },
        TestCase {
            name: "asset detail view",
            setup: |app| {
                app.nav.show_asset_details = true;
            },
            expected: InputContext::DetailView,
        },
        TestCase {
            name: "popup precedence over detail view",
            setup: |app| {
                app.ui.popup_state = PopupState::NetworkSelect(0);
                app.nav.show_transaction_details = true;
            },
            expected: InputContext::NetworkSelect,
        },
    ];

    for case in cases {
        let mut app = create_test_app();
        (case.setup)(&mut app);

        assert_eq!(app.get_input_context(), case.expected, "{}", case.name);
    }
}

// ========================================================================
// handle_search_results() Tests
// ========================================================================

#[test]
fn test_handle_search_results_empty() {
    let mut app = create_test_app();

    app.handle_search_results(Vec::new());

    // Should show error message
    assert!(matches!(app.ui.popup_state, PopupState::Message(_)));
    assert!(app.data.filtered_search_results.is_empty());
}

#[test]
fn test_handle_search_results_with_results() {
    let mut app = create_test_app();

    let results = vec![
        SearchResultItem::Transaction(Box::new(create_test_transaction(
            "TXN1",
            TxnType::Payment,
            100,
        ))),
        SearchResultItem::Transaction(Box::new(create_test_transaction(
            "TXN2",
            TxnType::AppCall,
            101,
        ))),
    ];

    app.handle_search_results(results);

    // Should show search results popup
    assert!(matches!(app.ui.popup_state, PopupState::SearchResults(_)));
    assert_eq!(app.data.filtered_search_results.len(), 2);
}

/// Tests the complete flow from key event to network form state change.
/// This validates that typing in the network form actually updates the form state.
#[tokio::test]
async fn test_network_form_key_to_state_flow() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    let mut app = create_test_app();

    // Open network form popup
    app.ui.open_network_form(0);
    assert!(matches!(app.ui.popup_state, PopupState::NetworkForm(_)));

    // Verify input context is correct
    let context = app.get_input_context();
    assert_eq!(context, InputContext::NetworkForm);

    // Create a key event for 'T'
    let key_event = KeyEvent {
        code: KeyCode::Char('T'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    };

    // Map the key to a command
    let command = map_key(key_event, &context);
    assert!(matches!(command, AppCommand::TypeChar('T')));

    // Execute the command
    app.execute_command(command).await.unwrap();

    // Verify the character was added to the active field (Name)
    if let PopupState::NetworkForm(form) = &app.ui.popup_state {
        assert_eq!(form.name, "T");
    } else {
        panic!("Expected NetworkForm popup");
    }

    // Type more characters: 'e', 's', 't'
    for c in ['e', 's', 't'] {
        let key_event = KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        let command = map_key(key_event, &app.get_input_context());
        app.execute_command(command).await.unwrap();
    }

    if let PopupState::NetworkForm(form) = &app.ui.popup_state {
        assert_eq!(form.name, "Test");
    } else {
        panic!("Expected NetworkForm popup");
    }

    // Test backspace
    let backspace_event = KeyEvent {
        code: KeyCode::Backspace,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    };
    let command = map_key(backspace_event, &app.get_input_context());
    assert!(matches!(command, AppCommand::Backspace));
    app.execute_command(command).await.unwrap();

    if let PopupState::NetworkForm(form) = &app.ui.popup_state {
        assert_eq!(form.name, "Tes");
    } else {
        panic!("Expected NetworkForm popup");
    }

    // Test Tab to move to next field
    let tab_event = KeyEvent {
        code: KeyCode::Tab,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    };
    let command = map_key(tab_event, &app.get_input_context());
    assert!(matches!(command, AppCommand::NetworkFormNextField));
    app.execute_command(command).await.unwrap();

    // Type in the URL field
    for c in ['h', 't', 't', 'p'] {
        let key_event = KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        };
        let command = map_key(key_event, &app.get_input_context());
        app.execute_command(command).await.unwrap();
    }

    if let PopupState::NetworkForm(form) = &app.ui.popup_state {
        assert_eq!(form.name, "Tes");
        assert_eq!(form.algod_url, "http");
    } else {
        panic!("Expected NetworkForm popup");
    }
}
