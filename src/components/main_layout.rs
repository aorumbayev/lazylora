use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::{self, border, scrollbar},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};

use crate::{
    app::{App, Focus},
    constants::{BLOCK_ITEM_HEIGHT, TXN_ITEM_HEIGHT},
};

/// Renders the main header section.
pub fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(header_block.clone(), area);

    if area.height <= 2 {
        return;
    } // Not enough space for content

    // Render Title ("[lazy lora]")
    let title = Line::from(vec![
        "[".into(),
        "lazy".green().bold(),
        "lora".blue().bold(),
        "]".into(),
    ]);
    let title_paragraph = Paragraph::new(title).alignment(Alignment::Left);
    let title_width = 12; // Approximate width of the title
    let title_area = Rect::new(
        area.x + 2,
        area.y + 1,
        title_width.min(area.width.saturating_sub(4)),
        1,
    );
    frame.render_widget(title_paragraph, title_area);

    // Render Network Label if space allows
    if area.width > title_width + 25 {
        // Ensure space for title, label, and padding
        let network_text = format!("Network: {}", app.settings.selected_network.as_str());
        let network_width = network_text.len() as u16;
        let network_label = Paragraph::new(network_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Right);
        let network_area = Rect::new(
            area.right().saturating_sub(network_width + 2),
            area.y + 1,
            network_width.min(area.width.saturating_sub(title_width + 4)),
            1,
        );
        frame.render_widget(network_label, network_area);
    }
}

/// Renders the footer section with keybind hints.
pub fn render_footer(_app: &App, frame: &mut Frame, area: Rect) {
    let footer_text = "q:Quit | r:Refresh | f:Search | n:Network | Space:Live | Tab:Focus";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}

/// Renders the Blocks list pane.
pub fn render_blocks(app: &mut App, frame: &mut Frame, area: Rect) {
    let is_focused = app.focus == Focus::Blocks;

    let blocks_block = Block::default()
        .borders(Borders::ALL)
        .title(" Blocks ")
        .border_set(symbols::border::ROUNDED)
        .border_style(if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner_area = blocks_block.inner(area);
    frame.render_widget(blocks_block, area);

    let blocks_ref = match app.blocks.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            // If we can't get the lock, render an empty placeholder
            let empty_message = Paragraph::new("Loading blocks...").alignment(Alignment::Center);
            frame.render_widget(empty_message, inner_area);
            return;
        }
    };

    if blocks_ref.is_empty() {
        let no_data_message = Paragraph::new("No blocks available")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(no_data_message, inner_area);
        return;
    }

    // Convert blocks to list items
    let block_items: Vec<ListItem> = blocks_ref
        .iter()
        .enumerate()
        .map(|(i, block)| {
            let is_selected = app.block_list_state.selected() == Some(i);
            let indicator = if is_selected { "▶ " } else { "  " };
            let id_span = Span::styled(
                block.id.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            let txn_count_text = format!("{} txns", block.txn_count);
            let txn_count_span =
                Span::styled(txn_count_text.clone(), Style::default().fg(Color::Green));

            // Calculate padding to push txn_count to the right
            let available_width = inner_area.width.saturating_sub(indicator.len() as u16);
            let padding_width = available_width
                .saturating_sub(id_span.width() as u16)
                .saturating_sub(txn_count_span.width() as u16)
                .saturating_sub(1); // Account for space between id and padding
            let padding = Span::raw(" ".repeat(padding_width as usize));

            ListItem::new(vec![
                Line::from(vec![
                    indicator.into(),
                    id_span,
                    padding, // Use calculated padding
                    txn_count_span,
                ]),
                Line::from(vec![
                    Span::raw("  "), // Indent timestamp
                    Span::styled(&block.timestamp, Style::default().fg(Color::Gray)),
                ]),
                Line::from(""), // Spacer line for BLOCK_ITEM_HEIGHT = 3
            ])
            .style(if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            })
        })
        .collect();

    // Create and render the list using the state from App
    let block_list = List::new(block_items).block(Block::default());

    frame.render_stateful_widget(block_list, inner_area, &mut app.block_list_state);

    // Render scrollbar using state from App
    render_scrollbar(
        frame,
        inner_area,
        is_focused,
        blocks_ref.len(),
        BLOCK_ITEM_HEIGHT as usize,
        &app.block_list_state,
    );
}

/// Renders the Transactions list pane.
pub fn render_transactions(app: &mut App, frame: &mut Frame, area: Rect) {
    let is_focused = app.focus == Focus::Transactions;

    let txn_block = Block::default()
        .borders(Borders::ALL)
        .title(" Transactions ")
        .border_set(symbols::border::ROUNDED)
        .border_style(if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner_area = txn_block.inner(area);
    frame.render_widget(txn_block, area);

    let txns_ref = match app.transactions.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            // If we can't get the lock, render an empty placeholder
            let empty_message =
                Paragraph::new("Loading transactions...").alignment(Alignment::Center);
            frame.render_widget(empty_message, inner_area);
            return;
        }
    };

    if txns_ref.is_empty() {
        let no_data_message = Paragraph::new("No transactions available")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(no_data_message, inner_area);
        return;
    }

    // Convert transactions to list items
    let txn_items: Vec<ListItem> = txns_ref
        .iter()
        .enumerate()
        .map(|(i, txn)| {
            let is_selected = app.transaction_list_state.selected() == Some(i);
            let indicator = if is_selected { "▶ " } else { "  " };
            let txn_type_text = format!("[{}]", txn.txn_type.as_str());
            let txn_type_style = Style::default().fg(txn.txn_type.color());
            let txn_type_span = Span::styled(txn_type_text.clone(), txn_type_style);

            // Truncate ID based on available space
            let max_id_width = inner_area
                .width
                .saturating_sub(indicator.len() as u16)
                .saturating_sub(txn_type_span.width() as u16)
                .saturating_sub(2); // Account for spaces

            let id_text = if txn.id.len() as u16 > max_id_width {
                format!("{}...", &txn.id[..max_id_width.saturating_sub(3) as usize])
            } else {
                txn.id.clone()
            };
            let id_span = Span::styled(
                id_text,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            );

            // Calculate padding
            let available_width = inner_area.width.saturating_sub(indicator.len() as u16);
            let padding_width = available_width
                .saturating_sub(id_span.width() as u16)
                .saturating_sub(txn_type_span.width() as u16)
                .saturating_sub(1); // Space before type span
            let padding = Span::raw(" ".repeat(padding_width as usize));

            ListItem::new(vec![
                Line::from(vec![
                    indicator.into(),
                    id_span,
                    padding, // Use calculated padding
                    txn_type_span,
                ]),
                Line::from(vec![
                    Span::raw("  "), // Indent
                    Span::styled("From: ", Style::default().fg(Color::Gray)),
                    Span::styled(txn.from.clone(), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::raw("  "), // Indent
                    Span::styled("To:   ", Style::default().fg(Color::Gray)),
                    Span::styled(txn.to.clone(), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(""), // Spacer line for TXN_ITEM_HEIGHT = 4
            ])
            .style(if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            })
        })
        .collect();

    // Create and render the list using the state from App
    let txn_list = List::new(txn_items).block(Block::default());

    frame.render_stateful_widget(txn_list, inner_area, &mut app.transaction_list_state);

    // Render scrollbar using state from App
    render_scrollbar(
        frame,
        inner_area,
        is_focused,
        txns_ref.len(),
        TXN_ITEM_HEIGHT as usize,
        &app.transaction_list_state,
    );
}

/// Renders a vertical scrollbar if the pane is focused and content overflows.
fn render_scrollbar(
    frame: &mut Frame,
    area: Rect,
    is_focused: bool,
    total_items: usize,
    item_height: usize,
    list_state: &ListState,
) {
    // Calculate how many items fit based on height, ensuring it's at least 1 if area > 0
    let viewport_items = if item_height > 0 {
        (area.height as usize / item_height).max(1)
    } else {
        total_items // Avoid division by zero, assume all fit
    };

    // Ensure viewport doesn't exceed total items
    let viewport_content_length = viewport_items.min(total_items);

    if is_focused && total_items > viewport_content_length {
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_items)
            .viewport_content_length(viewport_content_length)
            // Use selected index for position - might feel more natural
            .position(list_state.selected().unwrap_or(0));

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(None)
            .style(Style::default().fg(Color::DarkGray));

        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}
