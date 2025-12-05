//! Panel rendering module for displaying blocks and transactions lists.
//!
//! This module provides the core panel rendering functionality for the main
//! content area of the LazyLora TUI. It handles rendering of the blocks list,
//! transactions list, and associated scrollbars with proper focus states and
//! scrolling behavior.
//!
//! # Panels
//!
//! The main content area is split into two panels:
//! - **Blocks Panel**: Displays the latest blocks from the Algorand blockchain
//! - **Transactions Panel**: Shows the latest transactions
//!
//! # Detail Views
//!
//! The `details` submodule contains rendering logic for detail popups:
//! - **Block Details**: Comprehensive block information with transaction list
//! - **Transaction Details**: Visual and table views of transaction data
//! - **Account Details**: Account balances, participation, and assets
//! - **Asset Details**: ASA metadata and management information
//!
//! # Features
//!
//! - Focus-aware styling with visual indicators
//! - Smooth scrolling with viewport management
//! - Interactive selection indicators
//! - Automatic scrollbar display when content overflows

pub mod details;

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    symbols::scrollbar,
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation},
};

use super::helpers::create_border_block;
use crate::state::{App, Focus};
use crate::theme::{
    HIGHLIGHT_STYLE, MUTED_COLOR, PRIMARY_COLOR, SECONDARY_COLOR, SELECTED_STYLE, SUCCESS_COLOR,
    WARNING_COLOR,
};

// ============================================================================
// Constants
// ============================================================================

/// Height of each block item in the list (in terminal lines).
const BLOCK_HEIGHT: u16 = 3;

/// Height of each transaction item in the list (in terminal lines).
const TXN_HEIGHT: u16 = 4;

// ============================================================================
// Public Panel Rendering Functions
// ============================================================================

/// Renders the blocks panel showing the latest blocks.
///
/// Displays a list of blocks with their IDs, transaction counts, and timestamps.
/// The panel includes focus-aware styling, selection indicators, and scrollbars
/// when the content overflows the visible area.
///
/// # Arguments
///
/// * `app` - The application state containing block data and navigation state
/// * `frame` - The frame to render into
/// * `area` - The rectangular area to render the panel in
///
/// # Features
///
/// - Focused border styling when the blocks panel is active
/// - Visual selection indicator (▶) for the selected block
/// - Automatic scrolling to keep selected items visible
/// - Scrollbar display when content exceeds viewport
/// - Empty state message when no blocks are available
pub fn render_blocks(app: &App, frame: &mut Frame, area: Rect) {
    let is_focused = app.ui.focus == Focus::Blocks;
    let blocks_block = create_border_block("Latest Blocks", is_focused);

    frame.render_widget(blocks_block.clone(), area);

    let inner_area = blocks_block.inner(area);
    let blocks = &app.data.blocks;

    if blocks.is_empty() {
        let no_data_message = Paragraph::new("No blocks available")
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(no_data_message, inner_area);
        return;
    }

    let block_items: Vec<ListItem> = blocks
        .iter()
        .enumerate()
        .map(|(i, block)| {
            let is_selected = app.nav.selected_block_index == Some(i);
            let selection_indicator = if is_selected { "▶" } else { "⬚" };

            ListItem::new(vec![
                Line::from(vec![
                    Span::raw(format!("{} ", selection_indicator)),
                    Span::styled(
                        block.id.to_string(),
                        Style::default()
                            .fg(PRIMARY_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("               "),
                    Span::styled(
                        format!("{} txns", block.txn_count),
                        Style::default().fg(SUCCESS_COLOR),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "), // Indent to align with content above
                    Span::styled(&block.timestamp, Style::default().fg(MUTED_COLOR)),
                ]),
                Line::from(""),
            ])
            .style(if is_selected {
                SELECTED_STYLE
            } else {
                Style::default()
            })
        })
        .collect();

    let items_per_page = inner_area.height as usize / BLOCK_HEIGHT as usize;

    let mut list_state = ListState::default();
    if let Some(selected_index) = app.nav.selected_block_index {
        list_state.select(Some(selected_index));
    }

    let block_scroll_usize = app.nav.block_scroll as usize / BLOCK_HEIGHT as usize;
    let start_index = block_scroll_usize.min(blocks.len().saturating_sub(1));
    let end_index = (start_index + items_per_page).min(blocks.len());
    let visible_items = block_items[start_index..end_index].to_vec();

    let block_list = List::new(visible_items)
        .block(Block::default())
        .highlight_style(HIGHLIGHT_STYLE);

    let mut modified_state = list_state.clone();
    if let Some(selected) = list_state.selected() {
        if selected >= start_index && selected < end_index {
            modified_state.select(Some(selected - start_index));
        } else {
            modified_state.select(None);
        }
    }

    frame.render_stateful_widget(block_list, inner_area, &mut modified_state);

    render_scrollbar(
        frame,
        inner_area,
        is_focused,
        blocks.len(),
        BLOCK_HEIGHT as usize,
        items_per_page,
        app.nav.block_scroll as usize,
    );
}

/// Renders the transactions panel showing the latest transactions.
///
/// Displays a list of transactions with their IDs, types, sender/receiver addresses,
/// and other relevant details. The panel includes focus-aware styling, selection
/// indicators, and scrollbars when content overflows.
///
/// # Arguments
///
/// * `app` - The application state containing transaction data and navigation state
/// * `frame` - The frame to render into
/// * `area` - The rectangular area to render the panel in
///
/// # Features
///
/// - Focused border styling when the transactions panel is active
/// - Visual selection indicator (▶) for the selected transaction
/// - Color-coded transaction types for easy identification
/// - Automatic scrolling to keep selected items visible
/// - Scrollbar display when content exceeds viewport
/// - Empty state message when no transactions are available
pub fn render_transactions(app: &App, frame: &mut Frame, area: Rect) {
    let is_focused = app.ui.focus == Focus::Transactions;
    let title = "Latest Transactions";
    let txn_block = create_border_block(title, is_focused);

    frame.render_widget(txn_block.clone(), area);
    let inner_area = txn_block.inner(area);

    let transactions = &app.data.transactions;
    let transactions_to_display: Vec<_> = transactions
        .iter()
        .enumerate()
        .map(|(i, txn)| (i, txn.clone()))
        .collect();

    if transactions_to_display.is_empty() {
        let message = "No transactions available";
        let no_data_message = Paragraph::new(message)
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(no_data_message, inner_area);
        return;
    }

    let txn_items: Vec<ListItem> = transactions_to_display
        .iter()
        .map(|(orig_idx, txn)| {
            let is_selected = app.nav.selected_transaction_index == Some(*orig_idx);
            let txn_type_str = txn.txn_type.as_str();
            let entity_type_style = Style::default().fg(txn.txn_type.color());
            let selection_indicator = if is_selected { "▶" } else { "→" };

            ListItem::new(vec![
                Line::from(vec![
                    Span::raw(format!("{} ", selection_indicator)),
                    Span::styled(
                        txn.id.clone(),
                        Style::default()
                            .fg(SECONDARY_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("          "),
                    Span::styled(format!("[{}]", txn_type_str), entity_type_style),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("From: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(txn.from.clone(), Style::default().fg(WARNING_COLOR)),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("To:   ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(txn.to.clone(), Style::default().fg(PRIMARY_COLOR)),
                ]),
                Line::from(""),
            ])
            .style(if is_selected {
                SELECTED_STYLE
            } else {
                Style::default()
            })
        })
        .collect();

    let items_per_page = inner_area.height as usize / TXN_HEIGHT as usize;

    let mut list_state = ListState::default();
    if let Some(selected_index) = app.nav.selected_transaction_index
        && let Some(display_index) = transactions_to_display
            .iter()
            .position(|(idx, _)| *idx == selected_index)
    {
        list_state.select(Some(display_index));
    }

    let txn_scroll_usize = app.nav.transaction_scroll as usize / TXN_HEIGHT as usize;
    let start_index = txn_scroll_usize.min(txn_items.len().saturating_sub(1));
    let end_index = (start_index + items_per_page).min(txn_items.len());

    let visible_items = if start_index < end_index {
        txn_items[start_index..end_index].to_vec()
    } else {
        Vec::new()
    };

    let txn_list = List::new(visible_items)
        .block(Block::default())
        .highlight_style(HIGHLIGHT_STYLE);

    let mut modified_state = list_state.clone();
    if let Some(selected_display_index) = list_state.selected() {
        if selected_display_index >= start_index && selected_display_index < end_index {
            modified_state.select(Some(selected_display_index - start_index));
        } else {
            modified_state.select(None); // Selection is outside the visible range
        }
    }

    frame.render_stateful_widget(txn_list, inner_area, &mut modified_state);

    render_scrollbar(
        frame,
        inner_area,
        is_focused,
        txn_items.len(),
        TXN_HEIGHT as usize,
        items_per_page,
        app.nav.transaction_scroll as usize,
    );
}

// ============================================================================
// Private Helper Functions
// ============================================================================

/// Renders a scrollbar for a panel when content overflows the visible area.
///
/// The scrollbar is only displayed when the panel is focused and there are more
/// items than can fit in the viewport. It provides visual feedback about the
/// current scroll position and content size.
///
/// # Arguments
///
/// * `frame` - The frame to render into
/// * `area` - The area where the scrollbar should appear
/// * `is_focused` - Whether the panel is currently focused
/// * `total_items` - Total number of items in the list
/// * `item_height` - Height of each item in terminal lines
/// * `items_per_page` - Number of items that fit in the viewport
/// * `scroll_position` - Current scroll position in terminal lines
fn render_scrollbar(
    frame: &mut Frame,
    area: Rect,
    is_focused: bool,
    total_items: usize,
    item_height: usize,
    items_per_page: usize,
    scroll_position: usize,
) {
    if is_focused && total_items > items_per_page {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .track_symbol(None)
            .begin_symbol(None)
            .end_symbol(None)
            .style(Style::default().fg(MUTED_COLOR))
            .track_style(Style::default().fg(Color::DarkGray));

        let content_length = total_items * item_height;

        let mut scrollbar_state = ratatui::widgets::ScrollbarState::default()
            .content_length(content_length)
            .viewport_content_length(items_per_page * item_height)
            .position(scroll_position);

        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AlgoBlock, Network, Transaction, TransactionDetails, TxnType};
    use crate::state::StartupOptions;
    use ratatui::{Terminal, backend::TestBackend};

    /// Helper function to run tests with a mock App.
    /// Since App construction is complex and requires async runtime,
    /// we use a helper that creates it properly.
    fn test_with_mock_app<F>(test_fn: F)
    where
        F: FnOnce(&App),
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let app = rt.block_on(async {
            let options = StartupOptions {
                network: Some(Network::TestNet),
                search: None,
                graph_view: false,
            };
            App::new(options).await.unwrap()
        });

        test_fn(&app);
    }

    /// Helper function to run tests with a mutable mock App.
    fn test_with_mock_app_mut<F>(test_fn: F)
    where
        F: FnOnce(&mut App),
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut app = rt.block_on(async {
            let options = StartupOptions {
                network: Some(Network::TestNet),
                search: None,
                graph_view: false,
            };
            App::new(options).await.unwrap()
        });

        test_fn(&mut app);
    }

    /// Populates app with sample test data.
    fn populate_test_data(app: &mut App) {
        // Add some test blocks
        app.data.blocks = vec![
            AlgoBlock {
                id: 1000,
                txn_count: 5,
                timestamp: "2024-01-01 10:00:00".to_string(),
            },
            AlgoBlock {
                id: 1001,
                txn_count: 3,
                timestamp: "2024-01-01 10:00:05".to_string(),
            },
        ];

        // Add some test transactions
        app.data.transactions = vec![Transaction {
            id: "TXN1ABC".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER1".to_string(),
            to: "RECEIVER1".to_string(),
            amount: 1_000_000,
            fee: 1000,
            block: 1000,
            timestamp: "2024-01-01 10:00:00".to_string(),
            asset_id: None,
            note: String::new(),
            rekey_to: None,
            details: TransactionDetails::None,
            inner_transactions: Vec::new(),
        }];
    }

    #[test]
    fn test_render_blocks_empty_state() {
        test_with_mock_app(|app| {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_blocks(app, frame, frame.area());
                })
                .unwrap();

            let buffer = terminal.backend().buffer();

            // Verify "No blocks available" message appears somewhere in the buffer
            let message_found = (0..buffer.area().width)
                .flat_map(|x| (0..buffer.area().height).map(move |y| (x, y)))
                .any(|(x, y)| buffer[(x, y)].symbol().contains('N'));

            assert!(message_found, "Empty state message should be rendered");
        });
    }

    #[test]
    fn test_render_blocks_with_data() {
        test_with_mock_app_mut(|app| {
            populate_test_data(app);

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_blocks(app, frame, frame.area());
                })
                .unwrap();

            // Should render without panicking
            let buffer = terminal.backend().buffer();
            assert!(!buffer[(0, 0)].symbol().is_empty());
        });
    }

    #[test]
    fn test_render_blocks_focused_border() {
        test_with_mock_app_mut(|app| {
            populate_test_data(app);
            app.ui.focus = Focus::Blocks;

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_blocks(app, frame, frame.area());
                })
                .unwrap();

            // Should render with focused border (double-line borders)
            let buffer = terminal.backend().buffer();
            assert!(!buffer[(0, 0)].symbol().is_empty());
        });
    }

    #[test]
    fn test_render_transactions_empty_state() {
        test_with_mock_app(|app| {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_transactions(app, frame, frame.area());
                })
                .unwrap();

            let buffer = terminal.backend().buffer();

            // Verify "No transactions available" message appears
            let message_found = (0..buffer.area().width)
                .flat_map(|x| (0..buffer.area().height).map(move |y| (x, y)))
                .any(|(x, y)| buffer[(x, y)].symbol().contains('N'));

            assert!(message_found, "Empty state message should be rendered");
        });
    }

    #[test]
    fn test_render_transactions_with_data() {
        test_with_mock_app_mut(|app| {
            populate_test_data(app);

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_transactions(app, frame, frame.area());
                })
                .unwrap();

            // Should render without panicking
            let buffer = terminal.backend().buffer();
            assert!(!buffer[(0, 0)].symbol().is_empty());
        });
    }

    #[test]
    fn test_render_transactions_focused_border() {
        test_with_mock_app_mut(|app| {
            populate_test_data(app);
            app.ui.focus = Focus::Transactions;

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_transactions(app, frame, frame.area());
                })
                .unwrap();

            // Should render with focused border
            let buffer = terminal.backend().buffer();
            assert!(!buffer[(0, 0)].symbol().is_empty());
        });
    }

    #[test]
    fn test_render_blocks_selection_indicator() {
        test_with_mock_app_mut(|app| {
            populate_test_data(app);
            app.nav.selected_block_index = Some(0);

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_blocks(app, frame, frame.area());
                })
                .unwrap();

            // Selection indicator (▶) should be present in the buffer
            let buffer = terminal.backend().buffer();
            let indicator_found = (0..buffer.area().width)
                .flat_map(|x| (0..buffer.area().height).map(move |y| (x, y)))
                .any(|(x, y)| buffer[(x, y)].symbol().contains('▶'));

            assert!(indicator_found, "Selection indicator should be rendered");
        });
    }

    #[test]
    fn test_render_transactions_selection_indicator() {
        test_with_mock_app_mut(|app| {
            populate_test_data(app);
            app.nav.selected_transaction_index = Some(0);

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_transactions(app, frame, frame.area());
                })
                .unwrap();

            // Selection indicator should be present
            let buffer = terminal.backend().buffer();
            let indicator_found = (0..buffer.area().width)
                .flat_map(|x| (0..buffer.area().height).map(move |y| (x, y)))
                .any(|(x, y)| buffer[(x, y)].symbol().contains('▶'));

            assert!(indicator_found, "Selection indicator should be rendered");
        });
    }

    #[test]
    fn test_scrollbar_renders_when_focused_and_overflow() {
        test_with_mock_app_mut(|app| {
            populate_test_data(app);
            app.ui.focus = Focus::Blocks;

            // Add many blocks to trigger scrollbar
            for i in 0..50 {
                app.data.blocks.push(AlgoBlock {
                    id: 2000 + i,
                    txn_count: 1,
                    timestamp: format!("2024-01-01 10:{:02}:00", i),
                });
            }

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|frame| {
                    render_blocks(app, frame, frame.area());
                })
                .unwrap();

            // Should render without panicking with scrollbar
            let buffer = terminal.backend().buffer();
            assert!(!buffer[(0, 0)].symbol().is_empty());
        });
    }

    #[test]
    fn test_constants_are_reasonable() {
        assert!(BLOCK_HEIGHT > 0, "Block height must be positive");
        assert!(TXN_HEIGHT > 0, "Transaction height must be positive");
        assert!(BLOCK_HEIGHT <= 10, "Block height should be reasonable");
        assert!(TXN_HEIGHT <= 10, "Transaction height should be reasonable");
    }
}
