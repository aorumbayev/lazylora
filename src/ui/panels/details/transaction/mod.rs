//! Transaction detail panel rendering.
//!
//! This module handles the display of detailed transaction information with
//! both table and visual graph views. Supports all Algorand transaction types
//! including payment, asset transfer, app calls, key registration, etc.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

use crate::domain::{SearchResultItem, Transaction};
use crate::state::{App, DetailViewMode};
use crate::theme::{MUTED_COLOR, PRIMARY_COLOR};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::centered_popup_area;
use crate::widgets::{TxnGraph, TxnGraphWidget};

mod transaction_table;
mod transaction_visual;

// Re-export public items from submodules
pub use transaction_table::{build_flat_row_list_for_copy, build_info_details, get_flat_row_count};

// ============================================================================
// Main Rendering Entry Point
// ============================================================================

/// Renders the transaction details popup with table or visual graph view.
///
/// Supports all Algorand transaction types with toggleable visual graph mode.
///
/// # Arguments
///
/// * `app` - Application state containing transaction data and UI state
/// * `frame` - Ratatui frame for rendering
/// * `area` - Available screen area for rendering
pub fn render_transaction_details(app: &App, frame: &mut Frame, area: Rect) {
    // First check if we have a directly viewed transaction (from block details)
    let transaction_opt: Option<Transaction> = if let Some(txn) = &app.data.viewed_transaction {
        Some(txn.clone())
    } else if app.ui.viewing_search_result {
        app.nav.selected_transaction_id.as_ref().and_then(|txn_id| {
            app.data
                .filtered_search_results
                .iter()
                .find_map(|(_, item)| match item {
                    SearchResultItem::Transaction(t) if &t.id == txn_id => Some((**t).clone()),
                    _ => None,
                })
        })
    } else {
        app.nav.selected_transaction_index.and_then(|index| {
            let transactions = &app.data.transactions;
            transactions.get(index).cloned()
        })
    };

    let Some(txn) = transaction_opt else {
        // Transaction not found - show error popup
        let popup_area = centered_popup_area(area, 50, 10);
        let popup_block = create_popup_block("Transaction Details");
        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup_block.clone(), popup_area);

        let inner_area = popup_block.inner(popup_area);
        let error_msg = Paragraph::new("Transaction data not available.\n\nPress Esc to close.")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(error_msg, inner_area);
        return;
    };

    // Pre-calculate graph dimensions for auto-scaling
    let is_visual = app.ui.detail_view_mode == DetailViewMode::Visual;
    let graph = TxnGraph::from_transaction(&txn);
    let graph_widget = TxnGraphWidget::new(&graph);

    // Calculate popup size - fullscreen or auto-scaled
    let popup_area = if app.ui.detail_fullscreen {
        // Fullscreen: use almost all available area with small margin
        Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        )
    } else if is_visual && !graph.columns.is_empty() {
        // Auto-scale popup based on graph content
        // Use required_width which includes type indicator column and tree prefix
        let graph_width = graph_widget.required_width();
        let graph_height = graph_widget.required_height();

        // Chrome padding breakdown:
        // Horizontal: 2 (border) + 2 (inner padding) + 4 (centering margin) = 8
        // The centering margin ensures the graph doesn't touch the edges and has
        // room for visual centering since most lines are shorter than the widest line.
        // Vertical: 2 (border) + 1 (tab) + 1 (separator) + 3 (buttons) + 1 (help) = 8
        let chrome_h_padding: u16 = 8;
        let chrome_v_padding: u16 = 8;

        // Calculate ideal dimensions - expand to fit content
        let min_width: u16 = 60;
        let min_height: u16 = 20;
        // Allow up to 95% of available space to maximize content visibility
        let max_width = (area.width as f32 * 0.95) as u16;
        let max_height = (area.height as f32 * 0.95) as u16;

        let ideal_width = (graph_width as u16)
            .saturating_add(chrome_h_padding)
            .clamp(min_width, max_width);
        let ideal_height = (graph_height as u16)
            .saturating_add(chrome_v_padding)
            .clamp(min_height, max_height);

        centered_popup_area(area, ideal_width, ideal_height)
    } else {
        // Table mode or fallback: standard size
        centered_popup_area(area, 80, 28)
    };

    let popup_block = create_popup_block("Transaction Details");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Create layout: tab bar, separator, content, buttons, help
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Length(1), // Separator
            Constraint::Min(6),    // Main content
            Constraint::Length(3), // Button area (more compact)
            Constraint::Length(1), // Help text
        ])
        .split(inner_area);

    // Render tab bar
    let visual_style = if is_visual {
        Style::default()
            .bg(PRIMARY_COLOR)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(MUTED_COLOR)
    };
    let table_style = if !is_visual {
        Style::default()
            .bg(PRIMARY_COLOR)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(MUTED_COLOR)
    };

    let tab_bar = Line::from(vec![
        Span::raw(" "),
        Span::styled(" Table ", table_style),
        Span::raw(" "),
        Span::styled(" Visual ", visual_style),
    ]);
    let tab_paragraph = Paragraph::new(tab_bar);
    frame.render_widget(tab_paragraph, content_layout[0]);

    // Render separator
    let separator = "─".repeat(inner_area.width as usize);
    let separator_widget = Paragraph::new(separator).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(separator_widget, content_layout[1]);

    // Render content based on view mode
    let content_area = content_layout[2];

    let needs_scroll = if is_visual {
        transaction_visual::render_visual_mode(
            app,
            &txn,
            &graph,
            &graph_widget,
            frame,
            content_area,
        )
    } else {
        transaction_table::render_table_mode(&txn, app, frame, content_area);
        false // Table mode doesn't scroll in this implementation
    };

    // Render compact action bar with inline buttons
    let button_area = content_layout[3];

    // Build action bar - Export SVG only available in Visual mode
    let mut action_spans = vec![
        Span::styled("  [C]", Style::default().fg(PRIMARY_COLOR)),
        Span::styled(" Copy", Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled("[Y]", Style::default().fg(PRIMARY_COLOR)),
        Span::styled(" JSON", Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled("[O]", Style::default().fg(PRIMARY_COLOR)),
        Span::styled(" Open", Style::default().fg(Color::White)),
        Span::raw("  "),
    ];

    // Only show Export SVG in Visual mode (graph view)
    if is_visual {
        action_spans.extend([
            Span::styled("[S]", Style::default().fg(PRIMARY_COLOR)),
            Span::styled(" SVG", Style::default().fg(Color::White)),
            Span::raw("  "),
        ]);
    }

    action_spans.extend([
        Span::styled("[Tab]", Style::default().fg(PRIMARY_COLOR)),
        Span::styled(" View", Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled("[Esc]", Style::default().fg(PRIMARY_COLOR)),
        Span::styled(" Close", Style::default().fg(Color::White)),
    ]);

    let action_bar = Line::from(action_spans);

    let action_paragraph = Paragraph::new(action_bar).alignment(Alignment::Center);
    let action_rect = Rect::new(button_area.x, button_area.y + 1, button_area.width, 1);
    frame.render_widget(action_paragraph, action_rect);

    // Render minimal help text
    let help_area = content_layout[4];

    let help_text = if is_visual && needs_scroll {
        "↑↓←→ Scroll"
    } else if is_visual {
        "" // No scrolling needed, don't confuse users
    } else {
        "↑↓/jk Navigate  [C] Copy"
    };

    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg, help_area);
}

/// Builds the full transaction details for legacy/copy functionality.
///
/// This includes all fields and is used by copy operations.
#[must_use]
pub fn build_transaction_details(txn: &Transaction, _app: &App) -> Vec<(String, String)> {
    build_info_details(txn)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    use rstest::*;

    use crate::domain::Network;
    use crate::state::StartupOptions;

    /// Fixture: mainnet client for fetching real transactions
    #[fixture]
    fn mainnet_client() -> crate::client::AlgoClient {
        crate::client::AlgoClient::new(Network::MainNet).expect("test client should build")
    }

    // ========================================================================
    // Transaction Popup Snapshot Tests
    // ========================================================================

    /// Test case for transaction popup rendering tests
    struct PopupTestCase {
        txn_id: &'static str,
        view_mode: DetailViewMode,
        snapshot_name: &'static str,
        width: u16,
        height: u16,
    }

    /// All popup test cases
    const POPUP_TEST_CASES: &[PopupTestCase] = &[
        PopupTestCase {
            txn_id: "RSTLLBOXL3LIVU6JDP2MYP7DR6624F4M7NDXERCKSETCLRNADWHQ",
            view_mode: DetailViewMode::Visual,
            snapshot_name: "transaction_popup_visual_mode",
            width: 120,
            height: 40,
        },
        PopupTestCase {
            txn_id: "RSTLLBOXL3LIVU6JDP2MYP7DR6624F4M7NDXERCKSETCLRNADWHQ",
            view_mode: DetailViewMode::Table,
            snapshot_name: "transaction_popup_table_mode",
            width: 100,
            height: 35,
        },
    ];

    /// Parameterized test for all transaction popup rendering scenarios
    #[tokio::test]
    async fn test_transaction_popup_snapshots() {
        use crate::client::AlgoClient;

        let client = AlgoClient::new(Network::MainNet).expect("test client should build");
        let options = StartupOptions {
            network: Some(Network::MainNet),
            search: None,
            graph_view: false,
        };

        for case in POPUP_TEST_CASES {
            let txn = client
                .get_transaction_by_id(case.txn_id)
                .await
                .unwrap_or_else(|e| panic!("Failed to fetch {}: {}", case.txn_id, e))
                .unwrap_or_else(|| panic!("Transaction not found: {}", case.txn_id));

            let mut app = App::new(options.clone())
                .await
                .expect("Failed to create app");
            app.data.viewed_transaction = Some(txn);
            app.ui.detail_view_mode = case.view_mode.clone();

            let mut terminal = Terminal::new(TestBackend::new(case.width, case.height))
                .expect("terminal creation should succeed");
            terminal
                .draw(|frame| {
                    render_transaction_details(&app, frame, frame.area());
                })
                .expect("draw should succeed");

            insta::assert_snapshot!(case.snapshot_name, terminal.backend());
        }
    }
}
