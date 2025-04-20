use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::{border, scrollbar},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, Table, Wrap,
    },
};

use crate::algorand::{Transaction, TxnType};
use crate::app_state::{App, Focus, PopupState, SearchType};

const BLOCK_HEIGHT: u16 = 3;
const TXN_HEIGHT: u16 = 4;
const HEADER_HEIGHT: u16 = 3;
const TITLE_HEIGHT: u16 = 3;

/// Render the entire application UI
pub fn render(app: &App, frame: &mut Frame) {
    let size = frame.area();

    let chunks = Layout::default()
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(size);

    render_header(app, frame, chunks[0]);
    render_main_content(app, frame, chunks[1]);
    render_footer(app, frame, chunks[2]);

    match &app.popup_state {
        PopupState::NetworkSelect(selected_index) => {
            render_network_selector(frame, size, *selected_index);
        }
        PopupState::Search(query, search_type) => {
            render_search_popup(frame, size, query, search_type);
        }
        PopupState::Message(message) => {
            render_message_popup(frame, size, message);
        }
        PopupState::SearchResults(results) => {
            render_search_results(frame, size, results);
        }
        PopupState::None => {
            if app.show_block_details {
                render_block_details(app, frame, size);
            } else if app.show_transaction_details {
                render_transaction_details(app, frame, size);
            }
        }
    }
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(header_block.clone(), area);

    if area.height <= 2 {
        return;
    }

    let title = Line::from(vec![
        "[".into(),
        "lazy".green().bold(),
        "lora".blue().bold(),
        "]".into(),
    ]);

    let title_paragraph = Paragraph::new(title)
        .style(Style::default())
        .alignment(Alignment::Left);

    let title_area = Rect::new(
        area.x + 2,
        area.y + 1,
        12.min(area.width.saturating_sub(2)),
        1,
    );
    frame.render_widget(title_paragraph, title_area);

    if area.width > 40 {
        let network_text = format!("Network: {}", app.network.as_str());
        let network_label = Paragraph::new(network_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Right);

        let network_area = Rect::new(area.right() - 20, area.y + 1, 18, 1);
        frame.render_widget(network_label, network_area);
    }
}

fn render_main_content(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(TITLE_HEIGHT), Constraint::Min(10)])
        .split(area);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(title_block.clone(), chunks[0]);

    let title = Paragraph::new("Explore").style(Style::default().add_modifier(Modifier::BOLD));
    let title_area = Rect::new(chunks[0].x + 2, chunks[0].y + 1, 10, 1);
    frame.render_widget(title, title_area);

    let show_live = *app.show_live.lock().unwrap();
    let checkbox_text = format!("[{}] Show live", if show_live { "✓" } else { " " });
    let checkbox = Paragraph::new(checkbox_text).style(Style::default().fg(if show_live {
        Color::Green
    } else {
        Color::Gray
    }));

    let checkbox_area = Rect::new(chunks[0].right() - 15, chunks[0].y + 1, 15, 1);
    frame.render_widget(checkbox, checkbox_area);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(chunks[1]);

    render_blocks(app, frame, content_chunks[0]);
    render_transactions(app, frame, content_chunks[1]);
}

fn render_blocks(app: &App, frame: &mut Frame, area: Rect) {
    let is_focused = app.focus == Focus::Blocks;
    let style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let blocks_block = Block::default()
        .borders(Borders::ALL)
        .title(" Latest Blocks ")
        .title_style(Style::default().add_modifier(Modifier::BOLD))
        .border_set(border::ROUNDED)
        .border_style(style);

    frame.render_widget(blocks_block.clone(), area);

    let inner_area = blocks_block.inner(area);
    let blocks = app.blocks.lock().unwrap();

    if blocks.is_empty() {
        let no_data_message = Paragraph::new("No blocks available")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(no_data_message, inner_area);
        return;
    }

    // Convert blocks to list items
    let block_items: Vec<ListItem> = blocks
        .iter()
        .enumerate()
        .map(|(i, block)| {
            let is_selected = app.selected_block_index == Some(i);
            ListItem::new(vec![
                Line::from(vec![
                    // Add consistent icon for all rows, but different based on selection
                    if is_selected {
                        "▶ ".into()
                    } else {
                        "⬚ ".into()
                    },
                    Span::styled(
                        block.id.to_string(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("               "),
                    Span::styled(
                        format!("{} txns", block.txn_count),
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "), // Indent to align with content above
                    Span::styled(&block.timestamp, Style::default().fg(Color::Gray)),
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

    // Calculate how many items can be displayed
    let items_per_page = inner_area.height as usize / BLOCK_HEIGHT as usize;

    // Create a stateful list
    let mut list_state = ratatui::widgets::ListState::default();
    if let Some(selected_index) = app.selected_block_index {
        list_state.select(Some(selected_index));

        // Just check if selected item is visible; app.block_scroll is immutable here
        let block_scroll_usize = app.block_scroll as usize / BLOCK_HEIGHT as usize;
        let visible_start = block_scroll_usize;
        let visible_end = visible_start + items_per_page;

        if selected_index < visible_start || selected_index >= visible_end {
            // Selected item is not in visible area
            // We can't modify app.block_scroll here because it's a reference
            // The scroll adjustment should happen in the app logic instead
        }
    }

    // Determine visible range of items
    let block_scroll_usize = app.block_scroll as usize / BLOCK_HEIGHT as usize;
    let start_index = block_scroll_usize.min(blocks.len().saturating_sub(1));
    let end_index = (start_index + items_per_page).min(blocks.len());
    let visible_items = block_items[start_index..end_index].to_vec();

    // Create and render the list - no highlight_symbol as we're adding it manually
    let block_list = List::new(visible_items)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    // Render list with state
    let mut modified_state = list_state.clone();
    if let Some(selected) = list_state.selected() {
        if selected >= start_index && selected < end_index {
            modified_state.select(Some(selected - start_index));
        } else {
            modified_state.select(None);
        }
    }

    frame.render_stateful_widget(block_list, inner_area, &mut modified_state);

    // Render scrollbar
    render_scrollbar(
        frame,
        inner_area,
        is_focused,
        blocks.len(),
        BLOCK_HEIGHT as usize,
        items_per_page,
        app.block_scroll as usize,
    );
}

fn render_transactions(app: &App, frame: &mut Frame, area: Rect) {
    let is_focused = app.focus == Focus::Transactions;
    let style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let title = if !app.filtered_transactions.is_empty() {
        " Search Results "
    } else {
        " Latest Transactions "
    };

    let txn_block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().add_modifier(Modifier::BOLD))
        .border_set(border::ROUNDED)
        .border_style(style);

    frame.render_widget(txn_block.clone(), area);
    let inner_area = txn_block.inner(area);

    // Determine the source of transactions to display
    let transactions_to_display = if !app.filtered_transactions.is_empty() {
        // If we have filtered transactions, use those
        app.filtered_transactions
            .iter()
            .map(|(i, txn)| (*i, txn.clone()))
            .collect::<Vec<_>>()
    } else {
        // Otherwise, use all transactions with their indices
        let transactions = app.transactions.lock().unwrap();
        transactions
            .iter()
            .enumerate()
            .map(|(i, txn)| (i, txn.clone()))
            .collect::<Vec<_>>()
    };

    if transactions_to_display.is_empty() {
        let message = if !app.filtered_transactions.is_empty() {
            "No matching transactions found"
        } else {
            "No transactions available"
        };

        let no_data_message = Paragraph::new(message)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(no_data_message, inner_area);
        return;
    }

    // Convert transactions to list items
    let txn_items: Vec<ListItem> = transactions_to_display
        .iter()
        .enumerate()
        .map(|(_list_idx, (orig_idx, txn))| {
            let is_selected = app.selected_transaction_index == Some(*orig_idx);
            let txn_type_style = Style::default().fg(txn.txn_type.color());

            ListItem::new(vec![
                Line::from(vec![
                    // Add consistent icon for all rows, but different based on selection
                    if is_selected {
                        "▶ ".into()
                    } else {
                        "→ ".into()
                    },
                    Span::styled(
                        txn.id.clone(),
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("          "),
                    Span::styled(format!("[{}]", txn.txn_type.as_str()), txn_type_style),
                ]),
                Line::from(vec![
                    Span::raw("  "), // Indent to align with content above
                    Span::styled("From: ", Style::default().fg(Color::Gray)),
                    Span::styled(txn.from.clone(), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::raw("  "), // Indent to align with content above
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

    // Calculate how many items can be displayed
    let items_per_page = inner_area.height as usize / TXN_HEIGHT as usize;

    // Create a stateful list
    let mut list_state = ratatui::widgets::ListState::default();
    if let Some(selected_index) = app.selected_transaction_index {
        list_state.select(Some(selected_index));
    }

    // Determine visible range of items
    let txn_scroll_usize = app.transaction_scroll as usize / TXN_HEIGHT as usize;
    let start_index = txn_scroll_usize.min(txn_items.len().saturating_sub(1));
    let end_index = (start_index + items_per_page).min(txn_items.len());

    let visible_items = if start_index < end_index {
        txn_items[start_index..end_index].to_vec()
    } else {
        Vec::new()
    };

    // Create and render the list - no highlight_symbol as we're adding it manually
    let txn_list = List::new(visible_items)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    // Render list with state
    let mut modified_state = list_state.clone();
    if let Some(selected) = list_state.selected() {
        if selected >= start_index && selected < end_index {
            modified_state.select(Some(selected - start_index));
        } else {
            modified_state.select(None);
        }
    }

    frame.render_stateful_widget(txn_list, inner_area, &mut modified_state);

    // Render scrollbar
    render_scrollbar(
        frame,
        inner_area,
        is_focused,
        txn_items.len(),
        TXN_HEIGHT as usize,
        items_per_page,
        app.transaction_scroll as usize,
    );
}

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
        // Use ratatui's built-in scrollbar widget
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .track_symbol(None)
            .begin_symbol(None)
            .end_symbol(None)
            .style(Style::default().fg(Color::Gray))
            .track_style(Style::default().fg(Color::DarkGray));

        // Calculate proper scroll position and content length
        let content_length = total_items * item_height;

        // Create stateful scrollbar
        let mut scrollbar_state = ratatui::widgets::ScrollbarState::default()
            .content_length(content_length)
            .viewport_content_length(items_per_page * item_height)
            .position(scroll_position);

        // Render the scrollbar
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

/// Render block details popup
fn render_block_details(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(index) = app.selected_block_index {
        let blocks = app.blocks.lock().unwrap();
        if let Some(block_data) = blocks.get(index) {
            let popup_area = centered_popup_area(area, 60, 15);

            // Create an outer block with title for the popup
            let popup_block = Block::default()
                .title(" Block Details ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Cyan));

            // Render the popup background
            frame.render_widget(Clear, popup_area);
            frame.render_widget(popup_block.clone(), popup_area);

            let inner_area = popup_block.inner(popup_area);

            // Create table rows
            let rows = vec![
                Row::new(vec![
                    Cell::from("Block ID:").style(Style::default().fg(Color::Yellow)),
                    Cell::from(format!("{}", block_data.id)),
                ]),
                Row::new(vec![
                    Cell::from("Transactions:").style(Style::default().fg(Color::Yellow)),
                    Cell::from(format!("{}", block_data.txn_count)),
                ]),
                Row::new(vec![
                    Cell::from("Timestamp:").style(Style::default().fg(Color::Yellow)),
                    Cell::from(block_data.timestamp.clone()),
                ]),
            ];

            // Create table with proper constraints
            let table = Table::new(
                rows,
                [Constraint::Percentage(30), Constraint::Percentage(70)],
            )
            .block(Block::default())
            .column_spacing(1);

            frame.render_widget(table, inner_area);

            // Add the close message at the bottom
            let text = "Press Esc to close";
            let text_area = Rect::new(
                popup_area.x + (popup_area.width - text.len() as u16) / 2,
                popup_area.y + popup_area.height - 2,
                text.len() as u16,
                1,
            );

            let close_msg = Paragraph::new(text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);

            frame.render_widget(close_msg, text_area);
        }
    }
}

/// Helper function to create a centered popup area
fn centered_popup_area(parent: Rect, width: u16, height: u16) -> Rect {
    // Ensure popup doesn't exceed terminal boundaries
    let popup_width = width.min(parent.width.saturating_sub(4));
    let popup_height = height.min(parent.height.saturating_sub(4));

    // Center the popup
    let popup_x = parent.x + (parent.width.saturating_sub(popup_width)) / 2;
    let popup_y = parent.y + (parent.height.saturating_sub(popup_height)) / 2;

    Rect::new(popup_x, popup_y, popup_width, popup_height)
}

/// Render transaction details popup
fn render_transaction_details(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(index) = app.selected_transaction_index {
        let transactions = app.transactions.lock().unwrap();

        if let Some(txn) = transactions.get(index) {
            // Increase popup size to fit more information
            let popup_area = centered_popup_area(area, 76, 25);

            // Create an outer block with title for the popup
            let popup_block = Block::default()
                .title(" Transaction Details ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Cyan));

            // Render the popup background
            frame.render_widget(Clear, popup_area);
            frame.render_widget(popup_block.clone(), popup_area);

            let inner_area = popup_block.inner(popup_area);

            // Format amount based on transaction type
            let formatted_amount = match txn.txn_type {
                TxnType::Payment => {
                    // Convert microAlgos to Algos (1 Algo = 1,000,000 microAlgos)
                    format!("{:.6} Algos", txn.amount as f64 / 1_000_000.0)
                }
                TxnType::AssetTransfer => {
                    if let Some(asset_id) = txn.asset_id {
                        format!("{} units of Asset ID: {}", txn.amount, asset_id)
                    } else {
                        format!("{} units", txn.amount)
                    }
                }
                _ => format!("{}", txn.amount),
            };

            // Format fee from microAlgos to Algos
            let formatted_fee = format!("{:.6} Algos", txn.fee as f64 / 1_000_000.0);

            // Build the transaction details
            let mut details = vec![
                ("Transaction ID:", txn.id.clone()),
                ("Type:", txn.txn_type.as_str().to_string()),
                ("From:", txn.from.clone()),
                ("To:", txn.to.clone()),
                ("Timestamp:", txn.timestamp.clone()),
                ("Block:", format!("{}", txn.block)),
                ("Fee:", formatted_fee),
                ("Amount:", formatted_amount),
                ("Note:", txn.note.clone()),
            ];

            // Add asset ID if it's an asset transfer
            if let Some(asset_id) = txn.asset_id {
                details.push(("Asset ID:", format!("{}", asset_id)));
            }

            // Create table rows
            let rows: Vec<Row> = details
                .into_iter()
                .map(|(label, value)| {
                    Row::new(vec![
                        Cell::from(label).style(Style::default().fg(Color::Yellow)),
                        Cell::from(value), // Using Cell::from directly for text wrapping
                    ])
                })
                .collect();

            // Create table with proper constraints
            let table = Table::new(rows, [Constraint::Length(15), Constraint::Min(40)])
                .block(Block::default())
                .column_spacing(1)
                .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

            frame.render_widget(table, inner_area);

            // Add copy button
            let button_text = "[C] Copy TXN ID";
            let button_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_set(border::ROUNDED);

            let button_width = button_text.len() as u16 + 4;
            let button_height = 3;
            let button_x = inner_area.x + (inner_area.width - button_width) / 2;
            let button_y = inner_area.y + inner_area.height - button_height - 2;

            let button_area = Rect::new(button_x, button_y, button_width, button_height);

            frame.render_widget(button_block, button_area);

            let button_content = Paragraph::new(button_text)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center);

            let button_inner_area = Rect::new(
                button_area.x + 1,
                button_area.y + 1,
                button_area.width - 2,
                button_area.height - 2,
            );

            frame.render_widget(button_content, button_inner_area);

            // Add the close message at the bottom
            let text = "Press Esc to close";
            let text_area = Rect::new(
                popup_area.x + (popup_area.width - text.len() as u16) / 2,
                popup_area.y + popup_area.height - 1,
                text.len() as u16,
                1,
            );

            let close_msg = Paragraph::new(text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);

            frame.render_widget(close_msg, text_area);
        }
    }
}

fn render_footer(_app: &App, frame: &mut Frame, area: Rect) {
    let footer_text = "q:Quit  r:Refresh  f:Search  n:Network  Space:Live  Tab:Focus";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

/// Render network selector
fn render_network_selector(frame: &mut Frame, area: Rect, selected_index: usize) {
    let popup_area = centered_popup_area(area, 30, 12);

    // Create an outer block with title for the popup
    let popup_block = Block::default()
        .title(" Select Network ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    // Render the popup background
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    let networks = ["MainNet", "TestNet", "LocalNet"];
    let rows: Vec<Row> = networks
        .iter()
        .enumerate()
        .map(|(i, net)| {
            let style = if i == selected_index {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![if i == selected_index { "> " } else { "  " }, *net]).style(style)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Length(2), Constraint::Min(10)])
        .block(Block::default())
        .column_spacing(1);

    frame.render_widget(table, inner_area);

    // Add the help message at the bottom
    let help_text = "↑↓:Move  Enter:Select  Esc:Cancel";
    let text_area = Rect::new(
        popup_area.x + (popup_area.width - help_text.len() as u16) / 2,
        popup_area.y + popup_area.height - 2,
        help_text.len() as u16,
        1,
    );

    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg, text_area);
}

/// Render search popup
fn render_search_popup(frame: &mut Frame, area: Rect, query: &str, search_type: &SearchType) {
    let popup_area = centered_popup_area(area, 50, 10);

    // Create an outer block with title for the popup
    let popup_block = Block::default()
        .title(" Search Transactions ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    // Render the popup background
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // First, render the search type selector
    let search_type_text = format!("Search type: {}", search_type.as_str());
    let search_type_area = Rect::new(inner_area.x, inner_area.y, inner_area.width, 1);

    let search_type_para = Paragraph::new(search_type_text)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Left);

    frame.render_widget(search_type_para, search_type_area);

    // Then render the search input area
    let query_prompt = format!(
        "Search query: {}{}",
        query,
        if query.is_empty() { "" } else { "_" }
    );

    let prompt_area = Rect::new(inner_area.x, inner_area.y + 2, inner_area.width, 1);

    let prompt = Paragraph::new(query_prompt)
        .style(Style::default())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(prompt, prompt_area);

    // Add the help message at the bottom
    let help_text = "Tab:Change Type  Enter:Search  Esc:Cancel";
    let text_area = Rect::new(
        popup_area.x + (popup_area.width - help_text.len() as u16) / 2,
        popup_area.y + popup_area.height - 2,
        help_text.len() as u16,
        1,
    );

    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg, text_area);
}

/// Render a message popup
fn render_message_popup(frame: &mut Frame, area: Rect, message: &str) {
    let popup_area = centered_popup_area(area, 40, 6);

    // Create an outer block with title for the popup
    let popup_block = Block::default()
        .title(" Message ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    // Render the popup background
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    let prompt = Paragraph::new(message)
        .style(Style::default())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(prompt, inner_area);

    // Add the help message at the bottom
    let help_text = "Press Esc to continue";
    let text_area = Rect::new(
        popup_area.x + (popup_area.width - help_text.len() as u16) / 2,
        popup_area.y + popup_area.height - 2,
        help_text.len() as u16,
        1,
    );

    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg, text_area);
}

/// Render search results popup
fn render_search_results(frame: &mut Frame, area: Rect, results: &[(usize, Transaction)]) {
    let popup_area = centered_popup_area(area, 76, 20);

    // Create an outer block with title for the popup
    let popup_block = Block::default()
        .title(" Search Results ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    // Render the popup background
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    let mut list_items = Vec::new();
    for (i, (_idx, txn)) in results.iter().enumerate() {
        let is_selected = i == 0;

        // Format amount based on transaction type
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
            _ => "".to_string(),
        };

        // Use Line::from with word wrapping for long content
        let id_span = Span::styled(
            txn.id.clone(),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );

        let from_span = Span::styled(txn.from.clone(), Style::default().fg(Color::Yellow));
        let to_span = Span::styled(txn.to.clone(), Style::default().fg(Color::Cyan));

        // Create lines with proper styling
        let line1 = Line::from(vec![
            if is_selected {
                "▶ ".into()
            } else {
                "⬚ ".into()
            },
            id_span,
            "  ".into(),
            Span::styled(
                format!("[{}]", txn.txn_type.as_str()),
                Style::default().fg(txn.txn_type.color()),
            ),
        ]);

        // For the address lines, use a more structured approach that allows wrapping
        let line2_prefix = "  From: ";
        let line2 = Line::from(vec![
            Span::styled(line2_prefix, Style::default().fg(Color::Gray)),
            from_span,
        ]);

        let line3_prefix = "  To:   ";
        let line3 = Line::from(vec![
            Span::styled(line3_prefix, Style::default().fg(Color::Gray)),
            to_span,
        ]);

        let line4 = Line::from(vec![
            "  ".into(),
            Span::styled(txn.timestamp.clone(), Style::default().fg(Color::Gray)),
            "  ".into(),
            Span::styled(amount_text, Style::default().fg(Color::Green)),
        ]);

        let item_text = vec![line1, line2, line3, line4, Line::from("")];
        list_items.push(ListItem::new(item_text).style(if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        }));
    }

    // Create a list with wrapping enabled
    let txn_list = List::new(list_items)
        .block(Block::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(txn_list, inner_area);

    // Add the help message at the bottom
    let help_text = "Enter:Select  Esc:Cancel";
    let text_area = Rect::new(
        popup_area.x + (popup_area.width - help_text.len() as u16) / 2,
        popup_area.y + popup_area.height - 2,
        help_text.len() as u16,
        1,
    );

    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg, text_area);
}
