//! Search results popup rendering.
//!
//! This module provides the search results popup that displays the results
//! of a search query with formatted information for each entity type.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, Paragraph},
};

use crate::domain::{SearchResultItem, TxnType};
use crate::theme::{
    ACCENT_COLOR, HIGHLIGHT_STYLE, MUTED_COLOR, PRIMARY_COLOR, SECONDARY_COLOR, SELECTED_STYLE,
    SUCCESS_COLOR, WARNING_COLOR,
};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::centered_popup_area;

// ============================================================================
// Public API
// ============================================================================

/// Renders the search results popup.
///
/// Displays a list of search results with appropriate formatting for each entity type:
/// - Transactions: ID, type, from/to addresses, amount
/// - Blocks: ID, timestamp, transaction count, proposer
/// - Accounts: Address, balance, status, asset count
/// - Assets: ID, name, creator, total supply
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render to
/// * `area` - The terminal area to render within
/// * `results` - The search results to display
///
/// # Example
///
/// ```ignore
/// use lazylora::ui::popups::search_results;
///
/// let results = vec![(0, SearchResultItem::Transaction(txn))];
/// search_results::render(&mut frame, area, &results);
/// ```
pub fn render(frame: &mut Frame, area: Rect, results: &[(usize, SearchResultItem)]) {
    let popup_area = centered_popup_area(area, 80, 22);

    let popup_block = create_popup_block("Search Results");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    let mut list_items = Vec::new();
    for (i, (_idx, item)) in results.iter().enumerate() {
        let is_selected = i == 0;
        let selection_indicator = if is_selected { "▶" } else { "⬚" };

        let list_item = match item {
            SearchResultItem::Transaction(txn) => {
                let amount_text = match txn.txn_type {
                    TxnType::Payment => {
                        format!("{:.6} Algos", txn.amount as f64 / 1_000_000.0)
                    }
                    TxnType::AssetTransfer => {
                        if let Some(asset_id) = txn.asset_id {
                            format!("{} units (Asset: {})", txn.amount, asset_id)
                        } else {
                            format!("{} units", txn.amount)
                        }
                    }
                    _ => format!("{}", txn.amount),
                };

                let id_span = Span::styled(
                    txn.id.clone(),
                    Style::default()
                        .fg(SECONDARY_COLOR)
                        .add_modifier(Modifier::BOLD),
                );
                let type_span = Span::styled(
                    format!("[{}]", txn.txn_type.as_str()),
                    Style::default().fg(txn.txn_type.color()),
                );

                let line1 = Line::from(vec![
                    Span::raw(format!("{} ", selection_indicator)),
                    id_span,
                    "  ".into(),
                    type_span,
                ]);
                let line2 = Line::from(vec![
                    Span::styled("  From: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(txn.from.clone(), Style::default().fg(WARNING_COLOR)),
                ]);
                let line3 = Line::from(vec![
                    Span::styled("  To:   ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(txn.to.clone(), Style::default().fg(PRIMARY_COLOR)),
                ]);
                let line4 = Line::from(vec![
                    "  ".into(),
                    Span::styled(txn.timestamp.clone(), Style::default().fg(MUTED_COLOR)),
                    "  ".into(),
                    Span::styled(amount_text, Style::default().fg(SUCCESS_COLOR)),
                ]);
                vec![line1, line2, line3, line4, Line::from("")]
            }
            SearchResultItem::Block(block) => {
                let id_span = Span::styled(
                    format!("Block # {}", block.id),
                    Style::default()
                        .fg(PRIMARY_COLOR)
                        .add_modifier(Modifier::BOLD),
                );
                let type_span = Span::styled("[Block]", Style::default().fg(Color::White));

                let line1 = Line::from(vec![
                    Span::raw(format!("{} ", selection_indicator)),
                    id_span,
                    "  ".into(),
                    type_span,
                ]);
                let line2 = Line::from(vec![
                    Span::styled("  Time: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(block.timestamp.clone(), Style::default().fg(WARNING_COLOR)),
                ]);
                let line3 = Line::from(vec![
                    Span::styled("  Txns: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(
                        format!("{}", block.txn_count),
                        Style::default().fg(SUCCESS_COLOR),
                    ),
                ]);
                let line4 = Line::from(vec![
                    Span::styled("  Proposer: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(block.proposer.clone(), Style::default().fg(ACCENT_COLOR)),
                ]);
                vec![line1, line2, line3, line4, Line::from("")]
            }
            SearchResultItem::Account(account) => {
                let id_span = Span::styled(
                    account.address.clone(),
                    Style::default()
                        .fg(WARNING_COLOR)
                        .add_modifier(Modifier::BOLD),
                );
                let type_span = Span::styled("[Account]", Style::default().fg(WARNING_COLOR));
                let balance_text = format!("{:.6} Algos", account.balance as f64 / 1_000_000.0);

                let line1 = Line::from(vec![
                    Span::raw(format!("{} ", selection_indicator)),
                    id_span,
                    "  ".into(),
                    type_span,
                ]);
                let line2 = Line::from(vec![
                    Span::styled("  Balance: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(balance_text, Style::default().fg(SUCCESS_COLOR)),
                ]);
                let line3 = Line::from(vec![
                    Span::styled("  Status: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(account.status.clone(), Style::default().fg(PRIMARY_COLOR)),
                ]);
                let line4 = Line::from(vec![
                    Span::styled("  Assets: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(
                        format!("{}", account.assets_count),
                        Style::default().fg(ACCENT_COLOR),
                    ),
                ]);
                vec![line1, line2, line3, line4, Line::from("")]
            }
            SearchResultItem::Asset(asset) => {
                let id_span = Span::styled(
                    format!("Asset # {}", asset.id),
                    Style::default()
                        .fg(SUCCESS_COLOR)
                        .add_modifier(Modifier::BOLD),
                );
                let type_span = Span::styled("[Asset]", Style::default().fg(SUCCESS_COLOR));
                let name = if asset.name.is_empty() {
                    "<unnamed>".to_string()
                } else {
                    asset.name.clone()
                };
                let unit = if asset.unit_name.is_empty() {
                    "".to_string()
                } else {
                    format!(" ({})", asset.unit_name)
                };
                let total_supply = format!("{} (decimals: {})", asset.total, asset.decimals);

                let line1 = Line::from(vec![
                    Span::raw(format!("{} ", selection_indicator)),
                    id_span,
                    "  ".into(),
                    type_span,
                ]);
                let line2 = Line::from(vec![
                    Span::styled("  Name: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(
                        format!("{}{}", name, unit),
                        Style::default().fg(PRIMARY_COLOR),
                    ),
                ]);
                let line3 = Line::from(vec![
                    Span::styled("  Creator: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(asset.creator.clone(), Style::default().fg(WARNING_COLOR)),
                ]);
                let line4 = Line::from(vec![
                    Span::styled("  Total: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(total_supply, Style::default().fg(ACCENT_COLOR)),
                ]);
                vec![line1, line2, line3, line4, Line::from("")]
            }
            SearchResultItem::Application(app) => {
                let id_span = Span::styled(
                    format!("App # {}", app.app_id),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                );
                let type_span = Span::styled("[App]", Style::default().fg(Color::Blue));
                let status = if app.deleted { "Deleted" } else { "Active" };
                let status_color = if app.deleted {
                    Color::Red
                } else {
                    SUCCESS_COLOR
                };

                let line1 = Line::from(vec![
                    Span::raw(format!("{} ", selection_indicator)),
                    id_span,
                    "  ".into(),
                    type_span,
                ]);
                let line2 = Line::from(vec![
                    Span::styled("  Creator: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(app.creator.clone(), Style::default().fg(WARNING_COLOR)),
                ]);
                let line3 = Line::from(vec![
                    Span::styled("  Status: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(status, Style::default().fg(status_color)),
                ]);
                let line4 = Line::from("");
                vec![line1, line2, line3, line4, Line::from("")]
            }
        };

        list_items.push(ListItem::new(list_item).style(if is_selected {
            SELECTED_STYLE
        } else {
            Style::default()
        }));
    }

    let txn_list = List::new(list_items)
        .block(Block::default())
        .highlight_style(HIGHLIGHT_STYLE);

    frame.render_widget(txn_list, inner_area);

    let help_text = "j/k:Navigate  Enter:Select  Esc:Close";
    let help_area = Rect::new(
        popup_area.x + (popup_area.width - help_text.len() as u16) / 2,
        popup_area.y + popup_area.height - 2,
        help_text.len() as u16,
        1,
    );

    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg, help_area);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn test_search_results_renders_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area(), &[]);
            })
            .unwrap();

        // Should render without panicking even with empty results
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_search_results_renders_with_data() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Create a mock transaction result
        use crate::domain::{Transaction, TransactionDetails};

        let txn = Transaction {
            id: "TEST123".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER".to_string(),
            to: "RECEIVER".to_string(),
            amount: 1_000_000,
            fee: 1000,
            block: 12345,
            timestamp: "2024-01-01".to_string(),
            asset_id: None,
            note: String::new(),
            rekey_to: None,
            group: None,
            inner_transactions: Vec::new(),
            details: TransactionDetails::None,
        };

        let results = vec![(0, SearchResultItem::Transaction(Box::new(txn)))];

        terminal
            .draw(|frame| {
                render(frame, frame.area(), &results);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }
}
