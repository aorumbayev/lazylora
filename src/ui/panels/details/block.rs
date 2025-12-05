//! Block detail panel rendering.
//!
//! This module handles the display of detailed block information including
//! block metadata, transaction type breakdown, and the list of transactions
//! within a block.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::scrollbar,
    text::{Line, Span},
    widgets::{
        Block, Cell, Clear, List, ListItem, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table,
    },
};

use crate::domain::{AlgoBlock, BlockDetails};
use crate::state::{App, BlockDetailTab};
use crate::theme::{
    ACCENT_COLOR, HIGHLIGHT_STYLE, MUTED_COLOR, PRIMARY_COLOR, SECONDARY_COLOR, SELECTED_STYLE,
    SUCCESS_COLOR, WARNING_COLOR,
};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::centered_popup_area;

/// Renders the block details popup with tabbed interface.
///
/// Displays comprehensive block information including metadata and transactions.
/// Supports tabbed navigation between Info and Transactions views.
///
/// # Arguments
///
/// * `app` - Application state containing block data and navigation state
/// * `frame` - Ratatui frame for rendering
/// * `area` - Available screen area for rendering
pub fn render_block_details(app: &App, frame: &mut Frame, area: Rect) {
    let Some(index) = app.nav.selected_block_index else {
        return;
    };
    let Some(block_data) = app.data.blocks.get(index) else {
        return;
    };

    // Use block_details if loaded, otherwise show basic info
    let block_details = app.data.block_details.as_ref();

    let popup_area = centered_popup_area(area, 85, 32);
    let popup_block = create_popup_block("Block Details");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Layout: tab bar, separator, content, help text
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Length(1), // Separator
            Constraint::Min(10),   // Main content
            Constraint::Length(1), // Help text
        ])
        .split(inner_area);

    // Render tab bar
    let is_info_tab = app.nav.block_detail_tab == BlockDetailTab::Info;
    let info_style = if is_info_tab {
        Style::default()
            .bg(PRIMARY_COLOR)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(MUTED_COLOR)
    };
    let txn_style = if !is_info_tab {
        Style::default()
            .bg(PRIMARY_COLOR)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(MUTED_COLOR)
    };

    let tab_bar = Line::from(vec![
        Span::raw("  "),
        Span::styled(" Info ", info_style),
        Span::raw("  "),
        Span::styled(" Transactions ", txn_style),
    ]);
    frame.render_widget(Paragraph::new(tab_bar), content_layout[0]);

    // Separator
    let separator = "─".repeat(inner_area.width as usize);
    frame.render_widget(
        Paragraph::new(separator).style(Style::default().fg(Color::DarkGray)),
        content_layout[1],
    );

    // Content based on tab
    let content_area = content_layout[2];

    if is_info_tab {
        render_block_info_tab(block_data, block_details, frame, content_area);
    } else {
        render_block_transactions_tab(app, block_details, frame, content_area);
    }

    // Help text
    let help_text = "Tab: Switch Tab | ↑↓: Navigate | Enter: View Txn | Esc: Close";
    frame.render_widget(
        Paragraph::new(help_text)
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center),
        content_layout[3],
    );
}

/// Renders the Info tab of block details showing metadata and transaction type breakdown.
///
/// # Arguments
///
/// * `block_data` - Basic block information
/// * `block_details` - Optional detailed block information from API
/// * `frame` - Ratatui frame for rendering
/// * `area` - Available area for the tab content
fn render_block_info_tab(
    block_data: &AlgoBlock,
    block_details: Option<&BlockDetails>,
    frame: &mut Frame,
    area: Rect,
) {
    // Basic block info as rows
    let mut rows = vec![
        Row::new(vec![
            Cell::from("Block ID:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", block_data.id)).style(Style::default().fg(PRIMARY_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Transactions:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", block_data.txn_count))
                .style(Style::default().fg(SUCCESS_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Timestamp:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(block_data.timestamp.clone()).style(Style::default().fg(MUTED_COLOR)),
        ]),
    ];

    // Add type breakdown if we have detailed info
    if let Some(details) = block_details {
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
        rows.push(Row::new(vec![
            Cell::from("Proposer:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(details.info.proposer.clone()).style(Style::default().fg(ACCENT_COLOR)),
        ]));

        // Type breakdown
        if !details.txn_type_counts.is_empty() {
            rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
            rows.push(Row::new(vec![
                Cell::from("Transaction Types:").style(
                    Style::default()
                        .fg(WARNING_COLOR)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("").style(Style::default()),
            ]));

            // Sort by count descending for better UX
            let mut type_counts: Vec<_> = details.txn_type_counts.iter().collect();
            type_counts.sort_by(|a, b| b.1.cmp(a.1));

            for (txn_type, count) in type_counts {
                rows.push(Row::new(vec![
                    Cell::from(format!("  {}:", txn_type.as_str()))
                        .style(Style::default().fg(txn_type.color())),
                    Cell::from(format!("{}", count)).style(Style::default().fg(Color::White)),
                ]));
            }
        }
    }

    let table = Table::new(rows, [Constraint::Length(20), Constraint::Min(50)])
        .block(Block::default())
        .column_spacing(2);

    frame.render_widget(table, area);
}

/// Renders the Transactions tab showing the list of transactions in the block.
///
/// # Arguments
///
/// * `app` - Application state for navigation and selection tracking
/// * `block_details` - Optional detailed block information containing transaction list
/// * `frame` - Ratatui frame for rendering
/// * `area` - Available area for the tab content
fn render_block_transactions_tab(
    app: &App,
    block_details: Option<&BlockDetails>,
    frame: &mut Frame,
    area: Rect,
) {
    let Some(details) = block_details else {
        // Still loading
        let loading = Paragraph::new("Loading transactions...")
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(loading, area);
        return;
    };

    if details.transactions.is_empty() {
        let empty = Paragraph::new("No transactions in this block")
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let item_height: u16 = 2; // Each transaction takes 2 lines
    let items_per_page = area.height as usize / item_height as usize;
    let scroll_offset = app.nav.block_txn_scroll as usize / item_height as usize;

    // Calculate visible range
    let start_index = scroll_offset.min(details.transactions.len().saturating_sub(1));
    let end_index = (start_index + items_per_page + 1).min(details.transactions.len());

    // Render transactions as a list with scrolling
    let txn_items: Vec<ListItem> = details
        .transactions
        .iter()
        .enumerate()
        .skip(start_index)
        .take(end_index - start_index)
        .map(|(i, txn)| {
            let is_selected = app.nav.block_txn_index == Some(i);
            let indicator = if is_selected { "▶" } else { " " };

            ListItem::new(vec![
                Line::from(vec![
                    Span::raw(format!("{} ", indicator)),
                    Span::styled(
                        txn.id.chars().take(20).collect::<String>() + "...",
                        Style::default()
                            .fg(SECONDARY_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        format!("[{}]", txn.txn_type.as_str()),
                        Style::default().fg(txn.txn_type.color()),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("   "),
                    Span::styled("From: ", Style::default().fg(MUTED_COLOR)),
                    Span::styled(
                        txn.from.chars().take(20).collect::<String>() + "...",
                        Style::default().fg(WARNING_COLOR),
                    ),
                ]),
            ])
            .style(if is_selected {
                SELECTED_STYLE
            } else {
                Style::default()
            })
        })
        .collect();

    let txn_list = List::new(txn_items)
        .block(Block::default())
        .highlight_style(HIGHLIGHT_STYLE);

    frame.render_widget(txn_list, area);

    // Render scrollbar if needed
    let total_items = details.transactions.len();
    if total_items > items_per_page {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .track_symbol(None)
            .begin_symbol(None)
            .end_symbol(None)
            .style(Style::default().fg(MUTED_COLOR))
            .track_style(Style::default().fg(Color::DarkGray));

        let content_length = total_items * item_height as usize;
        let mut scrollbar_state = ratatui::widgets::ScrollbarState::default()
            .content_length(content_length)
            .viewport_content_length(items_per_page * item_height as usize)
            .position(app.nav.block_txn_scroll as usize);

        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}
