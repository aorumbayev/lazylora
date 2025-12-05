//! Transaction list widget.
//!
//! Displays a list of Algorand transactions with selection and scrolling support.

#![allow(dead_code)] // Transitional phase - items will be used after integration

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, StatefulWidget},
};

use crate::domain::Transaction;

use super::state::{TransactionListState, render_list_scrollbar};

// ============================================================================
// TransactionListWidget
// ============================================================================

/// A widget that displays a list of transactions with selection and scrolling.
///
/// This widget implements `StatefulWidget` and requires a `TransactionListState`
/// to track selection and scroll position.
///
/// # Example
///
/// ```text
/// ┌─ Latest Transactions ───────────────┐
/// │ ▶ ABCD...WXYZ            [Payment]  │
/// │   From: SENDER...ADDR               │
/// │   To:   RECEIVER...ADDR             │
/// │                                     │
/// │ → EFGH...STUV            [App Call] │
/// │   From: CALLER...ADDR               │
/// │   To:   APP#12345                   │
/// │                                     │
/// └─────────────────────────────────────┘
/// ```
///
/// # Usage
///
/// ```ignore
/// use crate::widgets::list::{TransactionListWidget, TransactionListState};
///
/// let transactions = vec![/* ... */];
/// let widget = TransactionListWidget::new(&transactions).focused(true);
/// let mut state = TransactionListState::with_selection(0);
///
/// // Render with frame.render_stateful_widget(widget, area, &mut state);
/// ```
#[derive(Debug)]
pub struct TransactionListWidget<'a> {
    /// Slice of transactions to display.
    transactions: &'a [Transaction],
    /// Whether this widget is currently focused.
    focused: bool,
    /// Height of each transaction item in rows.
    item_height: u16,
}

impl<'a> TransactionListWidget<'a> {
    /// Height of each transaction item in the list (in rows).
    pub const DEFAULT_ITEM_HEIGHT: u16 = 4;

    /// Creates a new `TransactionListWidget` with the given transactions.
    ///
    /// # Arguments
    ///
    /// * `transactions` - A slice of `Transaction` items to display
    ///
    /// # Returns
    ///
    /// A new `TransactionListWidget` with default settings
    #[must_use]
    pub const fn new(transactions: &'a [Transaction]) -> Self {
        Self {
            transactions,
            focused: false,
            item_height: Self::DEFAULT_ITEM_HEIGHT,
        }
    }

    /// Sets whether this widget is focused.
    ///
    /// When focused, the scrollbar will be rendered if content exceeds the viewport.
    ///
    /// # Arguments
    ///
    /// * `focused` - Whether the widget should be considered focused
    ///
    /// # Returns
    ///
    /// Self with the focus state updated
    #[must_use]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the item height.
    ///
    /// # Arguments
    ///
    /// * `height` - The height of each item in rows
    ///
    /// # Returns
    ///
    /// Self with the item height updated
    #[must_use]
    pub const fn item_height(mut self, height: u16) -> Self {
        self.item_height = height;
        self
    }

    /// Returns the number of transactions.
    ///
    /// # Returns
    ///
    /// The number of transactions in the list
    #[must_use]
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Returns true if there are no transactions.
    ///
    /// # Returns
    ///
    /// `true` if the transactions slice is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
}

impl StatefulWidget for TransactionListWidget<'_> {
    type State = TransactionListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Handle empty state
        if self.transactions.is_empty() {
            let empty_msg = "No transactions available";
            let x = area.x + (area.width.saturating_sub(empty_msg.len() as u16)) / 2;
            let y = area.y + area.height / 2;

            if y < area.y + area.height && x < area.x + area.width {
                let style = Style::default().fg(Color::Gray);
                buf.set_string(x, y, empty_msg, style);
            }
            return;
        }

        // Build list items
        let txn_items: Vec<ListItem> = self
            .transactions
            .iter()
            .enumerate()
            .map(|(i, txn)| {
                let is_selected = state.selected_index == Some(i);
                let txn_type_str = txn.txn_type.as_str();
                let entity_type_style = Style::default().fg(txn.txn_type.color());
                let selection_indicator = if is_selected { "▶" } else { "→" };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw(format!("{selection_indicator} ")),
                        Span::styled(
                            txn.id.clone(),
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("          "),
                        Span::styled(format!("[{txn_type_str}]"), entity_type_style),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled("From: ", Style::default().fg(Color::Gray)),
                        Span::styled(txn.from.clone(), Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled("To:   ", Style::default().fg(Color::Gray)),
                        Span::styled(txn.to.clone(), Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(""),
                ])
                .style(if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                })
            })
            .collect();

        let items_per_page = area.height as usize / self.item_height as usize;
        let txn_scroll_usize = state.scroll_position as usize / self.item_height as usize;
        let start_index = txn_scroll_usize.min(self.transactions.len().saturating_sub(1));
        let end_index = (start_index + items_per_page).min(self.transactions.len());

        let visible_items = if start_index < end_index {
            txn_items[start_index..end_index].to_vec()
        } else {
            Vec::new()
        };

        // Create internal ListState for highlighting
        let mut list_state = ListState::default();
        if let Some(selected) = state.selected_index
            && selected >= start_index
            && selected < end_index
        {
            list_state.select(Some(selected - start_index));
        }

        let txn_list = List::new(visible_items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        StatefulWidget::render(txn_list, area, buf, &mut list_state);

        // Render scrollbar if focused and content exceeds viewport
        if self.focused && self.transactions.len() > items_per_page {
            render_list_scrollbar(
                area,
                buf,
                self.transactions.len(),
                self.item_height as usize,
                items_per_page,
                state.scroll_position as usize,
            );
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{TransactionDetails, TxnType};

    fn create_sample_transactions() -> Vec<Transaction> {
        vec![
            Transaction {
                id: "TXID1ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCD".to_string(),
                txn_type: TxnType::Payment,
                from: "SENDER1AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
                to: "RECEIVER1BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".to_string(),
                timestamp: "Mon, 01 Jan 2024 12:00:00".to_string(),
                block: 12345678,
                fee: 1000,
                note: "Test payment".to_string(),
                amount: 5_000_000,
                asset_id: None,
                rekey_to: None,
                details: TransactionDetails::default(),
                inner_transactions: Vec::new(),
            },
            Transaction {
                id: "TXID2ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890EFGH".to_string(),
                txn_type: TxnType::AssetTransfer,
                from: "SENDER2CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC".to_string(),
                to: "RECEIVER2DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD".to_string(),
                timestamp: "Mon, 01 Jan 2024 11:59:55".to_string(),
                block: 12345677,
                fee: 1000,
                note: "".to_string(),
                amount: 100,
                asset_id: Some(31566704),
                rekey_to: None,
                details: TransactionDetails::default(),
                inner_transactions: Vec::new(),
            },
            Transaction {
                id: "TXID3ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890IJKL".to_string(),
                txn_type: TxnType::AppCall,
                from: "CALLER1EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE".to_string(),
                to: "123456".to_string(),
                timestamp: "Mon, 01 Jan 2024 11:59:50".to_string(),
                block: 12345676,
                fee: 2000,
                note: "".to_string(),
                amount: 0,
                asset_id: None,
                rekey_to: None,
                details: TransactionDetails::default(),
                inner_transactions: Vec::new(),
            },
        ]
    }

    #[test]
    fn test_transaction_list_widget_new() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions);

        assert_eq!(widget.len(), 3);
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_transaction_list_widget_empty() {
        let transactions: Vec<Transaction> = vec![];
        let widget = TransactionListWidget::new(&transactions);

        assert_eq!(widget.len(), 0);
        assert!(widget.is_empty());
    }

    #[test]
    fn test_transaction_list_widget_focused() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions).focused(true);

        // Widget should be constructed without errors
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_transaction_list_widget_item_height() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions).item_height(6);

        // Widget should be constructed without errors
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_transaction_list_widget_render_empty() {
        let transactions: Vec<Transaction> = vec![];
        let widget = TransactionListWidget::new(&transactions);
        let mut state = TransactionListState::new();

        let area = Rect::new(0, 0, 80, 20);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render "No transactions available" message
        let content = buf_to_string(&buf);
        assert!(content.contains("No transactions available"));
    }

    #[test]
    fn test_transaction_list_widget_render_with_transactions() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions);
        let mut state = TransactionListState::new();

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render transaction info
        let content = buf_to_string(&buf);
        assert!(content.contains("From:"));
        assert!(content.contains("To:"));
    }

    #[test]
    fn test_transaction_list_widget_render_with_selection() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions);
        let mut state = TransactionListState::with_selection(0);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render selection indicator for first item
        let content = buf_to_string(&buf);
        assert!(content.contains("▶")); // Selected indicator
    }

    #[test]
    fn test_transaction_list_widget_render_with_different_types() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions);
        let mut state = TransactionListState::new();

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should show different transaction types
        let content = buf_to_string(&buf);
        // Payment and other types should be rendered with their badges
        assert!(content.contains("[Payment]") || content.contains("Payment"));
    }

    // Helper function to convert buffer to string for testing
    fn buf_to_string(buf: &Buffer) -> String {
        let area = buf.area;
        let mut result = String::new();

        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell((x, y)) {
                    result.push_str(cell.symbol());
                }
            }
            result.push('\n');
        }

        result
    }
}
