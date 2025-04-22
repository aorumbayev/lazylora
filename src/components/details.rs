use ratatui::{
    prelude::*,
    style::{Color, Style},
    symbols::border,
    widgets::*,
};

use crate::{
    algorand::{SearchResultItem, Transaction, TxnType},
    app::{App, PopupState},
    components::helpers::centered_fixed_popup_area,
};

/// Renders the block details popup.
pub fn render_block_details(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(index) = app.block_list_state.selected() {
        let blocks = match app.blocks.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                // If we can't get the lock, render a message
                let message =
                    Paragraph::new("Loading block details...").alignment(Alignment::Center);
                let popup_area = centered_fixed_popup_area(area, 60, 10);
                frame.render_widget(Clear, popup_area);
                frame.render_widget(message, popup_area);
                return;
            }
        };

        if let Some(block_data) = blocks.get(index) {
            let popup_area = centered_fixed_popup_area(area, 60, 10); // Use fixed size

            let popup_block = Block::default()
                .title(" Block Details ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Cyan));

            frame.render_widget(Clear, popup_area); // Clear background
            frame.render_widget(popup_block.clone(), popup_area);

            let inner_area = popup_block.inner(popup_area);

            // Create table rows
            let rows = vec![
                Row::new(vec![
                    Cell::from("Block ID:").style(Style::default().fg(Color::Yellow)),
                    Cell::from(block_data.id.to_string()),
                ]),
                Row::new(vec![
                    Cell::from("Timestamp:").style(Style::default().fg(Color::Yellow)),
                    Cell::from(block_data.timestamp.clone()),
                ]),
                Row::new(vec![
                    Cell::from("Transactions:").style(Style::default().fg(Color::Yellow)),
                    Cell::from(block_data.txn_count.to_string()),
                ]),
                // Add Proposer if available and fits?
            ];

            // Create table with fixed constraints
            let table = Table::new(rows, [Constraint::Length(15), Constraint::Min(0)])
                .block(Block::default())
                .column_spacing(1);

            let table_area = Rect {
                height: inner_area.height.saturating_sub(2), // Reserve space for close msg
                ..inner_area
            };

            frame.render_widget(table, table_area);

            // Add the close message at the bottom
            let text = "Press Esc to close";
            let text_area = Rect::new(
                popup_area.x,
                popup_area.y + popup_area.height - 2,
                popup_area.width,
                1,
            );
            let close_msg = Paragraph::new(text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(close_msg, text_area);
        }
    }
}

/// Render transaction details popup
pub fn render_transaction_details(app: &App, frame: &mut Frame, area: Rect) {
    // Determine the source of the transaction data
    let transaction_opt: Option<Transaction> = if app.viewing_search_result_details {
        // Get data from the stored detailed_search_result
        app.detailed_search_result
            .as_ref()
            .and_then(|item| match item {
                SearchResultItem::Transaction(txn) => Some(txn.clone()),
                _ => None, // Ensure we only get a Transaction
            })
    } else {
        // Find the transaction in the main transactions list state
        app.transaction_list_state
            .selected()
            .and_then(|index| match app.transactions.try_lock() {
                Ok(guard) => guard.get(index).cloned(),
                Err(_) => None,
            })
    };

    if let Some(txn) = transaction_opt {
        let popup_area = centered_fixed_popup_area(area, 76, 20); // Fixed size

        let popup_block = Block::default()
            .title(" Transaction Details ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Cyan));

        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup_block.clone(), popup_area);

        let inner_area = popup_block.inner(popup_area);

        // Split inner area for table and buttons/help text
        let detail_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Table area
                Constraint::Length(3), // Button area
                Constraint::Length(1), // Close message area
            ])
            .split(inner_area);

        let table_area = detail_chunks[0];
        let button_area = detail_chunks[1];
        let help_area = detail_chunks[2];

        // Format amount based on entity type
        let formatted_amount = match txn.txn_type {
            TxnType::Payment => format!("{:.6} Algos", txn.amount as f64 / 1_000_000.0),
            TxnType::AssetTransfer => {
                if let Some(asset_id) = txn.asset_id {
                    format!("{} units (Asset #{})", txn.amount, asset_id)
                } else {
                    format!("{} units", txn.amount)
                }
            }
            _ => txn.amount.to_string(),
        };
        let formatted_fee = format!("{:.6} Algos", txn.fee as f64 / 1_000_000.0);

        // Build the transaction details rows
        let mut details = vec![
            Row::new(vec![
                Cell::from("Transaction ID:").style(Style::default().fg(Color::Yellow)),
                Cell::from(txn.id.clone()),
            ]),
            Row::new(vec![
                Cell::from("Type:").style(Style::default().fg(Color::Yellow)),
                Cell::from(txn.txn_type.as_str().to_string()),
            ]),
            Row::new(vec![
                Cell::from("From:").style(Style::default().fg(Color::Yellow)),
                Cell::from(txn.from.clone()),
            ]),
            Row::new(vec![
                Cell::from("To:").style(Style::default().fg(Color::Yellow)),
                Cell::from(txn.to.clone()),
            ]),
            Row::new(vec![
                Cell::from("Amount:").style(Style::default().fg(Color::Yellow)),
                Cell::from(formatted_amount),
            ]),
            Row::new(vec![
                Cell::from("Fee:").style(Style::default().fg(Color::Yellow)),
                Cell::from(formatted_fee),
            ]),
            Row::new(vec![
                Cell::from("Block:").style(Style::default().fg(Color::Yellow)),
                Cell::from(txn.block.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Timestamp:").style(Style::default().fg(Color::Yellow)),
                Cell::from(txn.timestamp.clone()),
            ]),
        ];

        if let Some(asset_id) = txn.asset_id {
            details.push(Row::new(vec![
                Cell::from("Asset ID:").style(Style::default().fg(Color::Yellow)),
                Cell::from(asset_id.to_string()),
            ]));
        }
        if !txn.note.is_empty() {
            details.push(Row::new(vec![
                Cell::from("Note:").style(Style::default().fg(Color::Yellow)),
                Cell::from(txn.note.clone()),
            ]));
        }

        // Create table with proper constraints
        let table = Table::new(details, [Constraint::Length(15), Constraint::Min(0)])
            .block(Block::default())
            .column_spacing(1);

        frame.render_widget(table, table_area);

        // Render Copy Button
        let button_text = " [C] Copy TXN ID ";
        let button_width = button_text.len() as u16;
        let button_render_area = Rect {
            x: button_area.x + (button_area.width.saturating_sub(button_width)) / 2,
            y: button_area.y,
            width: button_width,
            height: 3, // Needs height for border
        };

        let button_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .border_set(border::ROUNDED);
        frame.render_widget(button_block, button_render_area);

        let button_content = Paragraph::new(button_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        // Render text inside button border
        let button_inner_area = Rect {
            x: button_render_area.x + 1,
            y: button_render_area.y + 1,
            width: button_render_area.width.saturating_sub(2),
            height: 1,
        };
        frame.render_widget(button_content, button_inner_area);

        // Add the close message at the bottom
        let help_text = "Press Esc to close";
        let close_msg = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(close_msg, help_area);
    } else {
        // Render a message if we couldn't get transaction data
        let message = Paragraph::new("Loading transaction details...").alignment(Alignment::Center);
        let popup_area = centered_fixed_popup_area(area, 60, 10);
        frame.render_widget(Clear, popup_area);
        frame.render_widget(message, popup_area);
    }
}
