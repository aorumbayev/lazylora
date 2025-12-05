use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::{border, scrollbar},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, Table, Wrap,
    },
};

use crate::algorand::{
    AlgoBlock, AlgoClient, BlockDetails, Network, SearchResultItem, Transaction,
    TransactionDetails, TxnType,
};
use crate::app_state::{App, BlockDetailTab, DetailViewMode, Focus, PopupState, SearchType};
use crate::widgets::{TxnGraph, TxnGraphWidget, TxnVisualCard};

const BLOCK_HEIGHT: u16 = 3;
const TXN_HEIGHT: u16 = 4;
const HEADER_HEIGHT: u16 = 3;
const TITLE_HEIGHT: u16 = 3;

const PRIMARY_COLOR: Color = Color::Cyan;
const SECONDARY_COLOR: Color = Color::Blue;
const SUCCESS_COLOR: Color = Color::Green;
const WARNING_COLOR: Color = Color::Yellow;
#[allow(dead_code)]
const ERROR_COLOR: Color = Color::Red;
const MUTED_COLOR: Color = Color::Gray;
const ACCENT_COLOR: Color = Color::Magenta;

const BORDER_STYLE: Style = Style::new().fg(Color::DarkGray);
const FOCUSED_BORDER_STYLE: Style = Style::new().fg(PRIMARY_COLOR);
const TITLE_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);
const FOCUSED_TITLE_STYLE: Style = Style::new().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD);
const SELECTED_STYLE: Style = Style::new().bg(Color::DarkGray);
const HIGHLIGHT_STYLE: Style = Style::new()
    .bg(Color::DarkGray)
    .add_modifier(Modifier::BOLD);

fn create_border_block(title: &str, focused: bool) -> Block<'_> {
    let (border_style, border_set, title_style, display_title) = if focused {
        (
            FOCUSED_BORDER_STYLE,
            border::DOUBLE,
            FOCUSED_TITLE_STYLE,
            if title.is_empty() {
                String::new()
            } else {
                format!(" ● {} ", title)
            },
        )
    } else {
        (
            BORDER_STYLE,
            border::ROUNDED,
            TITLE_STYLE.fg(Color::DarkGray),
            if title.is_empty() {
                String::new()
            } else {
                format!(" {} ", title)
            },
        )
    };

    Block::default()
        .borders(Borders::ALL)
        .title(display_title)
        .title_style(title_style)
        .border_set(border_set)
        .border_style(border_style)
}

fn create_popup_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {} ", title))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(BORDER_STYLE)
}

fn centered_popup_area(parent: Rect, width: u16, height: u16) -> Rect {
    let popup_width = width.min(parent.width.saturating_sub(4));
    let popup_height = height.min(parent.height.saturating_sub(4));

    let popup_x = parent.x + (parent.width.saturating_sub(popup_width)) / 2;
    let popup_y = parent.y + (parent.height.saturating_sub(popup_height)) / 2;

    Rect::new(popup_x, popup_y, popup_width, popup_height)
}

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

    match &app.ui.popup_state {
        PopupState::NetworkSelect(selected_index) => {
            render_network_selector(frame, size, *selected_index, app.network);
        }
        PopupState::SearchWithType(query, search_type) => {
            render_search_with_type_popup(frame, size, query, *search_type);
        }
        PopupState::Message(message) => {
            render_message_popup(frame, size, message);
        }
        PopupState::SearchResults(results) => {
            render_search_results(frame, size, results);
        }
        PopupState::None => {
            if app.nav.show_block_details {
                render_block_details(app, frame, size);
            } else if app.nav.show_transaction_details {
                render_transaction_details(app, frame, size);
            } else if app.nav.show_account_details {
                render_account_details(app, frame, size);
            } else if app.nav.show_asset_details {
                render_asset_details(app, frame, size);
            }
        }
    }

    // Render toast notification on top of everything (non-blocking overlay)
    if let Some((message, _)) = &app.ui.toast {
        render_toast(frame, size, message);
    }
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let header_block = create_border_block("", false);
    frame.render_widget(header_block.clone(), area);

    if area.height <= 2 {
        return;
    }

    // Create the title with shimmer effect when live mode is enabled
    let title = if app.show_live {
        // Calculate shimmer effect using sine wave for breathing glow
        // The animation_tick increments every 100ms, so we scale appropriately
        let time = app.animation_tick as f32 * 0.15; // Adjust speed of shimmer

        // Create phase-shifted sine waves for different parts of the logo
        // This creates a "traveling shimmer" effect
        let bracket_glow = ((time * 0.8).sin() + 1.0) / 2.0; // 0.0 to 1.0
        let lazy_glow = ((time * 0.8 + 0.5).sin() + 1.0) / 2.0;
        let lora_glow = ((time * 0.8 + 1.0).sin() + 1.0) / 2.0;

        // Map glow values to color intensity (keeping base colors visible)
        // Green for "lazy": ranges from dim green to bright green
        let lazy_green = (120.0 + lazy_glow * 135.0) as u8; // 120-255
        let lazy_color = Color::Rgb(
            (50.0 * lazy_glow) as u8, // slight red tint at peak
            lazy_green,
            (80.0 * lazy_glow) as u8, // slight blue tint at peak
        );

        // Blue/Cyan for "lora": ranges from dim cyan to bright cyan
        let lora_blue = (140.0 + lora_glow * 115.0) as u8; // 140-255
        let lora_green = (180.0 + lora_glow * 75.0) as u8; // 180-255
        let lora_color = Color::Rgb(
            (100.0 * lora_glow) as u8, // slight red at peak
            lora_green,
            lora_blue,
        );

        // Brackets: subtle white shimmer
        let bracket_intensity = (100.0 + bracket_glow * 155.0) as u8; // 100-255
        let bracket_color = Color::Rgb(bracket_intensity, bracket_intensity, bracket_intensity);

        Line::from(vec![
            Span::styled("[", Style::default().fg(bracket_color)),
            Span::styled(
                "lazy",
                Style::default().fg(lazy_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "lora",
                Style::default().fg(lora_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("]", Style::default().fg(bracket_color)),
        ])
    } else {
        // Static logo when live mode is off
        Line::from(vec![
            "[".into(),
            "lazy".green().bold(),
            "lora".blue().bold(),
            "]".into(),
        ])
    };

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
        let network_style = Style::default()
            .fg(SUCCESS_COLOR)
            .add_modifier(Modifier::BOLD);

        let network_label = Paragraph::new(network_text)
            .style(network_style)
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

    let title_block = create_border_block("", false);
    frame.render_widget(title_block.clone(), chunks[0]);

    let title = Paragraph::new("Explore").style(TITLE_STYLE);
    let title_area = Rect::new(chunks[0].x + 2, chunks[0].y + 1, 10, 1);
    frame.render_widget(title, title_area);

    let show_live = app.show_live;
    let checkbox_text = format!("[{}] Show live", if show_live { "✓" } else { " " });
    let checkbox_style = if show_live {
        Style::default()
            .fg(SUCCESS_COLOR)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(MUTED_COLOR)
    };
    let checkbox = Paragraph::new(checkbox_text).style(checkbox_style);

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

fn render_transactions(app: &App, frame: &mut Frame, area: Rect) {
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

fn render_block_details(app: &App, frame: &mut Frame, area: Rect) {
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

fn render_transaction_details(app: &App, frame: &mut Frame, area: Rect) {
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

    // Calculate popup size - fullscreen or normal
    let popup_area = if app.ui.detail_fullscreen {
        // Fullscreen: use almost all available area with small margin
        Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        )
    } else {
        // Normal: centered popup
        centered_popup_area(area, 85, 32)
    };

    let popup_block = create_popup_block("Transaction Details");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Create layout: tab bar at top, content area, button, help text
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Length(1), // Separator
            Constraint::Min(10),   // Main content
            Constraint::Length(4), // Button area
            Constraint::Length(1), // Help text
        ])
        .split(inner_area);

    // Render tab bar
    let is_visual = app.ui.detail_view_mode == DetailViewMode::Visual;
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
        Span::raw("  "),
        Span::styled(" Table ", table_style),
        Span::raw("  "),
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

    if is_visual {
        // Visual mode: use TxnGraph for sophisticated visualization
        let graph = TxnGraph::from_transaction(&txn);
        let graph_widget = TxnGraphWidget::new(&graph);
        let graph_lines = graph_widget.to_lines();

        // If graph has meaningful content (multiple columns), show it
        // Otherwise fall back to TxnVisualCard
        if !graph.columns.is_empty() {
            // Calculate padded area
            let padded_area = Rect::new(
                content_area.x + 2,
                content_area.y + 1,
                content_area.width.saturating_sub(4),
                content_area.height.saturating_sub(2),
            );

            // Calculate graph dimensions
            let graph_height = graph_widget.required_height();
            let graph_width = graph.total_width();

            // Get scroll offsets from app state
            let scroll_x = app.nav.graph_scroll_x as usize;
            let scroll_y = app.nav.graph_scroll_y as usize;

            // Clamp scroll to valid range
            let max_scroll_y = graph_height.saturating_sub(padded_area.height as usize);
            let max_scroll_x = graph_width.saturating_sub(padded_area.width as usize);
            let clamped_scroll_y = scroll_y.min(max_scroll_y);
            let clamped_scroll_x = scroll_x.min(max_scroll_x);

            // Calculate centering offsets (when graph fits in view)
            let center_x = if graph_width < padded_area.width as usize {
                (padded_area.width as usize - graph_width) / 2
            } else {
                0
            };
            let center_y = if graph_height < padded_area.height as usize {
                (padded_area.height as usize - graph_height) / 2
            } else {
                0
            };

            // Determine if we need scrolling (only when content exceeds view)
            let needs_scroll = graph_height > padded_area.height as usize
                || graph_width > padded_area.width as usize;

            // Build visible lines with centering or scrolling
            let visible_lines: Vec<Line> = if needs_scroll {
                // Scrolling mode - apply scroll offsets
                graph_lines
                    .into_iter()
                    .skip(clamped_scroll_y)
                    .take(padded_area.height as usize)
                    .map(|line| {
                        if clamped_scroll_x > 0 {
                            let mut remaining_skip = clamped_scroll_x;
                            let mut new_spans = Vec::new();

                            for span in line.spans {
                                let content = span.content.to_string();
                                let char_count = content.chars().count();

                                if remaining_skip >= char_count {
                                    remaining_skip -= char_count;
                                    continue;
                                }

                                if remaining_skip > 0 {
                                    let new_content: String =
                                        content.chars().skip(remaining_skip).collect();
                                    new_spans.push(Span::styled(new_content, span.style));
                                    remaining_skip = 0;
                                } else {
                                    new_spans.push(Span::styled(content, span.style));
                                }
                            }

                            Line::from(new_spans)
                        } else {
                            line
                        }
                    })
                    .collect()
            } else {
                // Centering mode - add padding for both horizontal and vertical centering
                let mut centered_lines: Vec<Line> = Vec::new();

                // Add top padding for vertical centering
                for _ in 0..center_y {
                    centered_lines.push(Line::from(""));
                }

                // Add graph lines with horizontal centering
                for line in graph_lines {
                    if center_x > 0 {
                        let mut new_spans = vec![Span::raw(" ".repeat(center_x))];
                        new_spans.extend(line.spans);
                        centered_lines.push(Line::from(new_spans));
                    } else {
                        centered_lines.push(line);
                    }
                }

                centered_lines
            };

            let visual_content = Paragraph::new(visible_lines).alignment(Alignment::Left);

            frame.render_widget(visual_content, padded_area);

            // Show scroll indicators if content exceeds visible area
            let needs_v_scroll = graph_height > padded_area.height as usize;
            let needs_h_scroll = graph_width > padded_area.width as usize;

            if needs_v_scroll || needs_h_scroll {
                // Render scroll indicator at bottom right of content area
                let scroll_hint = if needs_v_scroll && needs_h_scroll {
                    format!("Scroll: {},{} (arrows)", clamped_scroll_x, clamped_scroll_y)
                } else if needs_v_scroll {
                    format!("Scroll: {} (up/down)", clamped_scroll_y)
                } else {
                    format!("Scroll: {} (left/right)", clamped_scroll_x)
                };

                let hint_width = scroll_hint.len() as u16;
                let hint_area = Rect::new(
                    padded_area.x + padded_area.width.saturating_sub(hint_width + 1),
                    padded_area.y + padded_area.height.saturating_sub(1),
                    hint_width,
                    1,
                );

                let hint_widget =
                    Paragraph::new(scroll_hint).style(Style::default().fg(Color::DarkGray));
                frame.render_widget(hint_widget, hint_area);
            }
        } else {
            // Fallback to TxnVisualCard for edge cases
            let visual_card = TxnVisualCard::new(&txn);
            let lines = visual_card.to_lines();

            let visual_content = Paragraph::new(lines).alignment(Alignment::Left);

            let padded_area = Rect::new(
                content_area.x + 2,
                content_area.y + 1,
                content_area.width.saturating_sub(4),
                content_area.height.saturating_sub(2),
            );
            frame.render_widget(visual_content, padded_area);
        }
    } else {
        // Table mode: type-specific rendering
        let formatted_fee = format!("{:.6} Algos", txn.fee as f64 / 1_000_000.0);

        let mut details: Vec<(String, String)> = vec![
            ("Transaction ID:".to_string(), txn.id.clone()),
            ("Type:".to_string(), txn.txn_type.as_str().to_string()),
            ("From:".to_string(), txn.from.clone()),
        ];

        // Add type-specific fields
        match &txn.details {
            TransactionDetails::Payment(pay_details) => {
                details.push(("To:".to_string(), txn.to.clone()));
                details.push((
                    "Amount:".to_string(),
                    format!("{:.6} Algos", txn.amount as f64 / 1_000_000.0),
                ));
                details.push(("Fee:".to_string(), formatted_fee));
                details.push(("Block:".to_string(), format!("#{}", txn.block)));
                details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

                if let Some(close_to) = &pay_details.close_remainder_to {
                    details.push(("Close To:".to_string(), close_to.clone()));
                }
                if let Some(close_amount) = pay_details.close_amount {
                    details.push((
                        "Close Amount:".to_string(),
                        format!("{:.6} Algos", close_amount as f64 / 1_000_000.0),
                    ));
                }
            }
            TransactionDetails::AssetTransfer(axfer_details) => {
                details.push(("To:".to_string(), txn.to.clone()));
                details.push(("Amount:".to_string(), format!("{} units", txn.amount)));
                if let Some(asset_id) = txn.asset_id {
                    details.push(("Asset ID:".to_string(), format!("{}", asset_id)));
                }
                details.push(("Fee:".to_string(), formatted_fee));
                details.push(("Block:".to_string(), format!("#{}", txn.block)));
                details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

                if let Some(asset_sender) = &axfer_details.asset_sender {
                    details.push(("Clawback From:".to_string(), asset_sender.clone()));
                }
                if let Some(close_to) = &axfer_details.close_to {
                    details.push(("Close To:".to_string(), close_to.clone()));
                }
                if let Some(close_amount) = axfer_details.close_amount {
                    details.push((
                        "Close Amount:".to_string(),
                        format!("{} units", close_amount),
                    ));
                }
            }
            TransactionDetails::AssetConfig(acfg_details) => {
                details.push(("Fee:".to_string(), formatted_fee));
                details.push(("Block:".to_string(), format!("#{}", txn.block)));
                details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

                // Determine if creation or modification
                if let Some(created_id) = acfg_details.created_asset_id {
                    details.push(("Created Asset ID:".to_string(), format!("{}", created_id)));
                } else if let Some(asset_id) = acfg_details.asset_id {
                    details.push(("Asset ID:".to_string(), format!("{}", asset_id)));
                }

                if let Some(name) = &acfg_details.asset_name {
                    details.push(("Asset Name:".to_string(), name.clone()));
                }
                if let Some(unit) = &acfg_details.unit_name {
                    details.push(("Unit Name:".to_string(), unit.clone()));
                }
                if let Some(total) = acfg_details.total {
                    details.push(("Total Supply:".to_string(), format!("{}", total)));
                }
                if let Some(decimals) = acfg_details.decimals {
                    details.push(("Decimals:".to_string(), format!("{}", decimals)));
                }
                if let Some(url) = &acfg_details.url {
                    details.push(("URL:".to_string(), url.clone()));
                }
                if let Some(manager) = &acfg_details.manager {
                    details.push(("Manager:".to_string(), manager.clone()));
                }
                if let Some(reserve) = &acfg_details.reserve {
                    details.push(("Reserve:".to_string(), reserve.clone()));
                }
                if let Some(freeze) = &acfg_details.freeze {
                    details.push(("Freeze:".to_string(), freeze.clone()));
                }
                if let Some(clawback) = &acfg_details.clawback {
                    details.push(("Clawback:".to_string(), clawback.clone()));
                }
            }
            TransactionDetails::AssetFreeze(afrz_details) => {
                details.push((
                    "Freeze Target:".to_string(),
                    afrz_details.freeze_target.clone(),
                ));
                if let Some(asset_id) = txn.asset_id {
                    details.push(("Asset ID:".to_string(), format!("{}", asset_id)));
                }
                details.push((
                    "Frozen:".to_string(),
                    if afrz_details.frozen { "Yes" } else { "No" }.to_string(),
                ));
                details.push(("Fee:".to_string(), formatted_fee));
                details.push(("Block:".to_string(), format!("#{}", txn.block)));
                details.push(("Timestamp:".to_string(), txn.timestamp.clone()));
            }
            TransactionDetails::AppCall(app_details) => {
                let app_id_str = if app_details.app_id == 0 {
                    "Creating...".to_string()
                } else {
                    format!("{}", app_details.app_id)
                };
                details.push(("App ID:".to_string(), app_id_str));
                details.push((
                    "On-Complete:".to_string(),
                    app_details.on_complete.as_str().to_string(),
                ));
                details.push(("Fee:".to_string(), formatted_fee));
                details.push(("Block:".to_string(), format!("#{}", txn.block)));
                details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

                if let Some(created_id) = app_details.created_app_id {
                    details.push(("Created App ID:".to_string(), format!("{}", created_id)));
                }

                // Build expandable sections list
                let mut expandable_sections: Vec<(&str, &str, usize)> = Vec::new();
                if !app_details.app_args.is_empty() {
                    expandable_sections.push(("app_args", "App Args", app_details.app_args.len()));
                }
                if !app_details.accounts.is_empty() {
                    expandable_sections.push(("accounts", "Accounts", app_details.accounts.len()));
                }
                if !app_details.foreign_apps.is_empty() {
                    expandable_sections.push((
                        "foreign_apps",
                        "Foreign Apps",
                        app_details.foreign_apps.len(),
                    ));
                }
                if !app_details.foreign_assets.is_empty() {
                    expandable_sections.push((
                        "foreign_assets",
                        "Foreign Assets",
                        app_details.foreign_assets.len(),
                    ));
                }
                if !app_details.boxes.is_empty() {
                    expandable_sections.push(("boxes", "Box Refs", app_details.boxes.len()));
                }

                // Add expandable sections as rows
                for (idx, (section_id, label, count)) in expandable_sections.iter().enumerate() {
                    let is_expanded = app.ui.is_section_expanded(section_id);
                    let is_selected = app.ui.detail_section_index == Some(idx);
                    let indicator = if is_expanded { "▼" } else { "▶" };
                    let selection_mark = if is_selected { "→ " } else { "  " };

                    details.push((
                        format!("{}{} {}:", selection_mark, indicator, label),
                        format!("{} item(s)", count),
                    ));

                    // If expanded, show the items
                    if is_expanded {
                        match *section_id {
                            "app_args" => {
                                for (i, arg) in app_details.app_args.iter().enumerate() {
                                    let truncated = if arg.len() > 40 {
                                        format!("{}...", &arg[..40])
                                    } else {
                                        arg.clone()
                                    };
                                    details.push((format!("    [{}]:", i), truncated));
                                }
                            }
                            "accounts" => {
                                for (i, acc) in app_details.accounts.iter().enumerate() {
                                    let truncated = if acc.len() > 40 {
                                        format!("{}...", &acc[..40])
                                    } else {
                                        acc.clone()
                                    };
                                    details.push((format!("    [{}]:", i), truncated));
                                }
                            }
                            "foreign_apps" => {
                                for (i, app_id) in app_details.foreign_apps.iter().enumerate() {
                                    details.push((format!("    [{}]:", i), format!("{}", app_id)));
                                }
                            }
                            "foreign_assets" => {
                                for (i, asset_id) in app_details.foreign_assets.iter().enumerate() {
                                    details
                                        .push((format!("    [{}]:", i), format!("{}", asset_id)));
                                }
                            }
                            "boxes" => {
                                for (i, box_ref) in app_details.boxes.iter().enumerate() {
                                    let box_desc =
                                        format!("App: {}, Name: {}", box_ref.app_id, box_ref.name);
                                    details.push((format!("    [{}]:", i), box_desc));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            TransactionDetails::KeyReg(keyreg_details) => {
                details.push(("Fee:".to_string(), formatted_fee));
                details.push(("Block:".to_string(), format!("#{}", txn.block)));
                details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

                if keyreg_details.non_participation {
                    details.push(("Status:".to_string(), "Going Offline".to_string()));
                } else if keyreg_details.vote_key.is_some() {
                    details.push(("Status:".to_string(), "Going Online".to_string()));
                    if let Some(vote_key) = &keyreg_details.vote_key {
                        let truncated = if vote_key.len() > 20 {
                            format!("{}...", &vote_key[..20])
                        } else {
                            vote_key.clone()
                        };
                        details.push(("Vote Key:".to_string(), truncated));
                    }
                    if let Some(sel_key) = &keyreg_details.selection_key {
                        let truncated = if sel_key.len() > 20 {
                            format!("{}...", &sel_key[..20])
                        } else {
                            sel_key.clone()
                        };
                        details.push(("Selection Key:".to_string(), truncated));
                    }
                    if let Some(vote_first) = keyreg_details.vote_first {
                        details.push(("Vote First:".to_string(), format!("{}", vote_first)));
                    }
                    if let Some(vote_last) = keyreg_details.vote_last {
                        details.push(("Vote Last:".to_string(), format!("{}", vote_last)));
                    }
                    if let Some(dilution) = keyreg_details.vote_key_dilution {
                        details.push(("Key Dilution:".to_string(), format!("{}", dilution)));
                    }
                }
            }
            TransactionDetails::StateProof(sp_details) => {
                if let Some(sp_type) = sp_details.state_proof_type {
                    details.push(("State Proof Type:".to_string(), format!("{}", sp_type)));
                }
                details.push(("Fee:".to_string(), formatted_fee));
                details.push(("Block:".to_string(), format!("#{}", txn.block)));
                details.push(("Timestamp:".to_string(), txn.timestamp.clone()));
            }
            TransactionDetails::Heartbeat(hb_details) => {
                details.push(("Fee:".to_string(), formatted_fee));
                details.push(("Block:".to_string(), format!("#{}", txn.block)));
                details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

                if let Some(hb_addr) = &hb_details.hb_address {
                    details.push(("Heartbeat Addr:".to_string(), hb_addr.clone()));
                }
            }
            TransactionDetails::None => {
                // Fallback for unknown types
                details.push(("To:".to_string(), txn.to.clone()));
                details.push(("Amount:".to_string(), format!("{}", txn.amount)));
                details.push(("Fee:".to_string(), formatted_fee));
                details.push(("Block:".to_string(), format!("#{}", txn.block)));
                details.push(("Timestamp:".to_string(), txn.timestamp.clone()));
            }
        }

        let rows: Vec<Row> = details
            .into_iter()
            .map(|(label, value)| {
                Row::new(vec![
                    Cell::from(label).style(
                        Style::default()
                            .fg(WARNING_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(value).style(Style::default().fg(PRIMARY_COLOR)),
                ])
            })
            .collect();

        let table = Table::new(rows, [Constraint::Length(18), Constraint::Min(50)])
            .block(Block::default())
            .column_spacing(2)
            .row_highlight_style(HIGHLIGHT_STYLE);

        frame.render_widget(table, content_area);
    }

    // Render copy button
    let button_area = content_layout[3];
    let button_text = "[C] Copy TXN ID";
    let button_block = Block::default()
        .borders(Borders::ALL)
        .border_style(BORDER_STYLE)
        .border_set(border::ROUNDED);

    let button_width = button_text.len() as u16 + 4;
    let button_height = 3;
    let button_x = button_area.x + (button_area.width - button_width) / 2;
    let button_y = button_area.y;

    let button_rect = Rect::new(button_x, button_y, button_width, button_height);

    frame.render_widget(button_block, button_rect);

    let button_content = Paragraph::new(button_text)
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    let button_inner_area = Rect::new(
        button_rect.x + 1,
        button_rect.y + 1,
        button_rect.width - 2,
        button_rect.height - 2,
    );

    frame.render_widget(button_content, button_inner_area);

    // Render help text with Tab info
    let help_text = "Tab: Switch View | Arrows: Scroll | [S] Export SVG | [C] Copy | Esc: Close";
    let help_area = content_layout[4];

    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg, help_area);
}

fn render_footer(_app: &App, frame: &mut Frame, area: Rect) {
    let footer_text = "q:Quit  r:Refresh  f:Search  n:Network  Space:Live  Tab:Focus";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

fn render_network_selector(
    frame: &mut Frame,
    area: Rect,
    selected_index: usize,
    current_network: Network,
) {
    let popup_area = centered_popup_area(area, 35, 14);

    let popup_block = create_popup_block("Select Network (Esc:Cancel)");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    let networks = ["MainNet", "TestNet", "LocalNet"];
    let network_types = [Network::MainNet, Network::TestNet, Network::LocalNet];

    let rows: Vec<Row> = networks
        .iter()
        .enumerate()
        .map(|(i, net)| {
            let is_selected = i == selected_index;
            let is_current = network_types[i] == current_network;

            let indicator = if is_current && is_selected {
                "◉ " // Both current and selected
            } else if is_current {
                "● " // Current network
            } else if is_selected {
                "▶ " // Selected in UI
            } else {
                "  " // Neither
            };

            let style = if is_selected {
                Style::default()
                    .fg(PRIMARY_COLOR)
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default()
                    .fg(SUCCESS_COLOR)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(MUTED_COLOR)
            };

            let network_text = if is_current {
                format!("{} (current)", net)
            } else {
                net.to_string()
            };

            Row::new(vec![
                Cell::from(indicator).style(style),
                Cell::from(network_text).style(style),
            ])
        })
        .collect();

    let table = Table::new(rows, [Constraint::Length(3), Constraint::Min(15)])
        .block(Block::default())
        .column_spacing(1);

    frame.render_widget(table, inner_area);

    let help_text = "↑↓:Move Enter:Select";
    let help_area = Rect::new(
        inner_area.x,
        inner_area.y + inner_area.height.saturating_sub(1),
        inner_area.width,
        1,
    );

    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg, help_area);
}

fn render_search_with_type_popup(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    search_type: SearchType,
) {
    let popup_area = centered_popup_area(area, 65, 20);

    let popup_block = create_popup_block("Search Algorand Network");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(BORDER_STYLE)
        .title(" Enter search term ")
        .title_alignment(Alignment::Left);

    let input_area = Rect::new(inner_area.x + 2, inner_area.y + 2, inner_area.width - 4, 3);

    frame.render_widget(input_block.clone(), input_area);

    let text_input_area = input_block.inner(input_area);

    let input_text = format!("{}{}", query, "▏");

    let input = Paragraph::new(input_text)
        .style(Style::default())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(input, text_input_area);

    let selector_y = input_area.y + 4;
    let selector_height = 1;
    let selector_width = inner_area.width / 5;
    let spacing = 2;

    let search_types = [
        SearchType::Transaction,
        SearchType::Block,
        SearchType::Account,
        SearchType::Asset,
    ];

    let mut x_offset = inner_area.x + (inner_area.width - (4 * selector_width + 3 * spacing)) / 2;

    for t in &search_types {
        let is_selected = *t == search_type;
        let button_style = if is_selected {
            Style::default()
                .bg(PRIMARY_COLOR)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        };

        let button_rect = Rect::new(x_offset, selector_y, selector_width, selector_height);

        let button = Paragraph::new(t.as_str())
            .style(button_style)
            .alignment(Alignment::Center);

        frame.render_widget(button, button_rect);

        x_offset += selector_width + spacing;
    }

    let suggestions_y = selector_y + 3;
    let suggestions_area = Rect::new(inner_area.x + 2, suggestions_y, inner_area.width - 4, 4);

    let suggestion = AlgoClient::get_search_suggestions(query, search_type);

    let suggestion_color = if suggestion.contains("Valid") {
        SUCCESS_COLOR
    } else if suggestion.contains("too short")
        || suggestion.contains("too long")
        || suggestion.contains("invalid")
    {
        WARNING_COLOR
    } else if suggestion.contains("Enter") {
        MUTED_COLOR
    } else {
        PRIMARY_COLOR
    };

    let suggestions_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(MUTED_COLOR))
        .title(" Suggestions ")
        .title_alignment(Alignment::Left);

    frame.render_widget(suggestions_block.clone(), suggestions_area);

    let suggestions_inner = suggestions_block.inner(suggestions_area);

    let suggestion_text = Paragraph::new(suggestion)
        .style(Style::default().fg(suggestion_color))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(suggestion_text, suggestions_inner);

    let help_text1 = "Search directly queries the Algorand network";
    let help_text2 = "Use Tab to switch between search types";

    let help_area1 = Rect::new(inner_area.x + 2, suggestions_y + 5, inner_area.width - 4, 1);
    let help_area2 = Rect::new(inner_area.x + 2, suggestions_y + 6, inner_area.width - 4, 1);

    let help_msg1 = Paragraph::new(help_text1)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    let help_msg2 = Paragraph::new(help_text2)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg1, help_area1);
    frame.render_widget(help_msg2, help_area2);

    let control_text = "Tab: Change Type  Enter: Search  Esc: Cancel";
    let control_area = Rect::new(
        popup_area.x + (popup_area.width - control_text.len() as u16) / 2,
        popup_area.y + popup_area.height - 2,
        control_text.len() as u16,
        1,
    );

    let control_msg = Paragraph::new(control_text)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    frame.render_widget(control_msg, control_area);
}

fn render_search_results(frame: &mut Frame, area: Rect, results: &[(usize, SearchResultItem)]) {
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

    let help_text = "↑↓: Navigate  Enter: Select  Esc: Cancel";
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

fn render_account_details(app: &App, frame: &mut Frame, area: Rect) {
    let Some(account) = &app.data.viewed_account else {
        // Still loading or no data
        let popup_area = centered_popup_area(area, 50, 10);
        let popup_block = create_popup_block("Account Details");
        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup_block.clone(), popup_area);

        let inner_area = popup_block.inner(popup_area);
        let loading = Paragraph::new("Loading account details...")
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(loading, inner_area);
        return;
    };

    let popup_area = centered_popup_area(area, 85, 34);
    let popup_block = create_popup_block("Account Details");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Layout: content area and help text
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Main content
            Constraint::Length(1), // Help text
        ])
        .split(inner_area);

    let content_area = content_layout[0];

    // Format balances in Algos
    let balance_algos = format!("{:.6} Algos", account.balance as f64 / 1_000_000.0);
    let min_balance_algos = format!("{:.6} Algos", account.min_balance as f64 / 1_000_000.0);
    let pending_rewards_algos =
        format!("{:.6} Algos", account.pending_rewards as f64 / 1_000_000.0);
    let rewards_algos = format!("{:.6} Algos", account.rewards as f64 / 1_000_000.0);

    // Truncate address for display if needed
    let address_display = if account.address.len() > 50 {
        format!("{}...", &account.address[..47])
    } else {
        account.address.clone()
    };

    let mut rows = vec![];

    // Show NFD name prominently if available
    if let Some(ref nfd) = account.nfd {
        rows.push(Row::new(vec![
            Cell::from("NFD Name:").style(
                Style::default()
                    .fg(ACCENT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(nfd.name.clone()).style(
                Style::default()
                    .fg(ACCENT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        if nfd.is_verified {
            rows.push(Row::new(vec![
                Cell::from("NFD Status:").style(
                    Style::default()
                        .fg(WARNING_COLOR)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Verified").style(Style::default().fg(SUCCESS_COLOR)),
            ]));
        }
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
    }

    rows.extend(vec![
        Row::new(vec![
            Cell::from("Address:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(address_display).style(Style::default().fg(WARNING_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Status:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(account.status.clone()).style(if account.status == "Online" {
                Style::default().fg(SUCCESS_COLOR)
            } else {
                Style::default().fg(MUTED_COLOR)
            }),
        ]),
        Row::new(vec![Cell::from(""), Cell::from("")]), // Spacer
        Row::new(vec![
            Cell::from("Balance:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(balance_algos).style(Style::default().fg(SUCCESS_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Min Balance:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(min_balance_algos).style(Style::default().fg(MUTED_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Pending Rewards:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(pending_rewards_algos).style(Style::default().fg(PRIMARY_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Total Rewards:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(rewards_algos).style(Style::default().fg(PRIMARY_COLOR)),
        ]),
        Row::new(vec![Cell::from(""), Cell::from("")]), // Spacer
        Row::new(vec![
            Cell::from("Assets Opted In:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", account.total_assets_opted_in))
                .style(Style::default().fg(ACCENT_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Created Assets:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", account.total_created_assets))
                .style(Style::default().fg(ACCENT_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Apps Opted In:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", account.total_apps_opted_in))
                .style(Style::default().fg(SECONDARY_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Created Apps:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", account.total_created_apps))
                .style(Style::default().fg(SECONDARY_COLOR)),
        ]),
    ]);

    // Add authorized address if rekeyed
    if let Some(ref auth_addr) = account.auth_addr {
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
        let auth_display = if auth_addr.len() > 40 {
            format!("{}...", &auth_addr[..37])
        } else {
            auth_addr.clone()
        };
        rows.push(Row::new(vec![
            Cell::from("Rekeyed To:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(auth_display).style(Style::default().fg(Color::Red)),
        ]));
    }

    // Add participation info if online
    if let Some(ref participation) = account.participation {
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
        rows.push(Row::new(vec![
            Cell::from("Participation:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("").style(Style::default()),
        ]));
        rows.push(Row::new(vec![
            Cell::from("  Vote First:").style(Style::default().fg(MUTED_COLOR)),
            Cell::from(format!("{}", participation.vote_first))
                .style(Style::default().fg(Color::White)),
        ]));
        rows.push(Row::new(vec![
            Cell::from("  Vote Last:").style(Style::default().fg(MUTED_COLOR)),
            Cell::from(format!("{}", participation.vote_last))
                .style(Style::default().fg(Color::White)),
        ]));
        rows.push(Row::new(vec![
            Cell::from("  Key Dilution:").style(Style::default().fg(MUTED_COLOR)),
            Cell::from(format!("{}", participation.vote_key_dilution))
                .style(Style::default().fg(Color::White)),
        ]));
    }

    // Show first few asset holdings if any
    if !account.assets.is_empty() {
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
        rows.push(Row::new(vec![
            Cell::from("Asset Holdings:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("(showing first {})", account.assets.len().min(5)))
                .style(Style::default().fg(MUTED_COLOR)),
        ]));
        for asset in account.assets.iter().take(5) {
            let frozen_indicator = if asset.is_frozen { " [frozen]" } else { "" };
            rows.push(Row::new(vec![
                Cell::from(format!("  Asset #{}:", asset.asset_id))
                    .style(Style::default().fg(MUTED_COLOR)),
                Cell::from(format!("{}{}", asset.amount, frozen_indicator))
                    .style(Style::default().fg(SUCCESS_COLOR)),
            ]));
        }
    }

    let table = Table::new(rows, [Constraint::Length(20), Constraint::Min(50)])
        .block(Block::default())
        .column_spacing(2);

    frame.render_widget(table, content_area);

    // Help text
    let help_text = "Esc: Close";
    frame.render_widget(
        Paragraph::new(help_text)
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center),
        content_layout[1],
    );
}

fn render_asset_details(app: &App, frame: &mut Frame, area: Rect) {
    let Some(asset) = &app.data.viewed_asset else {
        // Still loading or no data
        let popup_area = centered_popup_area(area, 50, 10);
        let popup_block = create_popup_block("Asset Details");
        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup_block.clone(), popup_area);

        let inner_area = popup_block.inner(popup_area);
        let loading = Paragraph::new("Loading asset details...")
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(loading, inner_area);
        return;
    };

    let popup_area = centered_popup_area(area, 85, 30);
    let popup_block = create_popup_block("Asset Details");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Layout: content area and help text
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Main content
            Constraint::Length(1), // Help text
        ])
        .split(inner_area);

    let content_area = content_layout[0];

    // Format display values
    let name_display = if asset.name.is_empty() {
        "<unnamed>".to_string()
    } else {
        asset.name.clone()
    };
    let unit_display = if asset.unit_name.is_empty() {
        "-".to_string()
    } else {
        asset.unit_name.clone()
    };

    // Format total supply with decimals
    let total_display = if asset.decimals > 0 {
        let divisor = 10u64.pow(asset.decimals as u32);
        format!(
            "{:.prec$} {}",
            asset.total as f64 / divisor as f64,
            unit_display,
            prec = asset.decimals as usize
        )
    } else {
        format!("{} {}", asset.total, unit_display)
    };

    let creator_display = if asset.creator.len() > 40 {
        format!("{}...", &asset.creator[..37])
    } else {
        asset.creator.clone()
    };

    let mut rows = vec![
        Row::new(vec![
            Cell::from("Asset ID:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", asset.id)).style(
                Style::default()
                    .fg(SUCCESS_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Row::new(vec![
            Cell::from("Name:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(name_display).style(Style::default().fg(PRIMARY_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Unit Name:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(unit_display.clone()).style(Style::default().fg(PRIMARY_COLOR)),
        ]),
        Row::new(vec![Cell::from(""), Cell::from("")]), // Spacer
        Row::new(vec![
            Cell::from("Total Supply:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(total_display).style(Style::default().fg(SUCCESS_COLOR)),
        ]),
        Row::new(vec![
            Cell::from("Decimals:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", asset.decimals)).style(Style::default().fg(Color::White)),
        ]),
        Row::new(vec![
            Cell::from("Default Frozen:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(if asset.default_frozen { "Yes" } else { "No" }).style(
                if asset.default_frozen {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(SUCCESS_COLOR)
                },
            ),
        ]),
        Row::new(vec![Cell::from(""), Cell::from("")]), // Spacer
        Row::new(vec![
            Cell::from("Creator:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(creator_display).style(Style::default().fg(WARNING_COLOR)),
        ]),
    ];

    // Add URL if present
    if !asset.url.is_empty() {
        let url_display = if asset.url.len() > 50 {
            format!("{}...", &asset.url[..47])
        } else {
            asset.url.clone()
        };
        rows.push(Row::new(vec![
            Cell::from("URL:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(url_display).style(Style::default().fg(SECONDARY_COLOR)),
        ]));
    }

    // Add management addresses section
    rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
    rows.push(Row::new(vec![
        Cell::from("Management:").style(
            Style::default()
                .fg(WARNING_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("").style(Style::default()),
    ]));

    // Helper to format optional address
    let format_addr = |addr: &Option<String>| -> String {
        match addr {
            Some(a) if a.len() > 30 => format!("{}...", &a[..27]),
            Some(a) => a.clone(),
            None => "-".to_string(),
        }
    };

    rows.push(Row::new(vec![
        Cell::from("  Manager:").style(Style::default().fg(MUTED_COLOR)),
        Cell::from(format_addr(&asset.manager)).style(Style::default().fg(ACCENT_COLOR)),
    ]));
    rows.push(Row::new(vec![
        Cell::from("  Reserve:").style(Style::default().fg(MUTED_COLOR)),
        Cell::from(format_addr(&asset.reserve)).style(Style::default().fg(ACCENT_COLOR)),
    ]));
    rows.push(Row::new(vec![
        Cell::from("  Freeze:").style(Style::default().fg(MUTED_COLOR)),
        Cell::from(format_addr(&asset.freeze)).style(Style::default().fg(ACCENT_COLOR)),
    ]));
    rows.push(Row::new(vec![
        Cell::from("  Clawback:").style(Style::default().fg(MUTED_COLOR)),
        Cell::from(format_addr(&asset.clawback)).style(Style::default().fg(ACCENT_COLOR)),
    ]));

    // Add metadata hash if present
    if let Some(ref hash) = asset.metadata_hash {
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
        let hash_display = if hash.len() > 40 {
            format!("{}...", &hash[..37])
        } else {
            hash.clone()
        };
        rows.push(Row::new(vec![
            Cell::from("Metadata Hash:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(hash_display).style(Style::default().fg(MUTED_COLOR)),
        ]));
    }

    // Add created round if present
    if let Some(round) = asset.created_at_round {
        rows.push(Row::new(vec![
            Cell::from("Created Round:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", round)).style(Style::default().fg(MUTED_COLOR)),
        ]));
    }

    // Add deleted status if true
    if asset.deleted {
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
        rows.push(Row::new(vec![
            Cell::from("Status:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("DELETED")
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]));
    }

    let table = Table::new(rows, [Constraint::Length(18), Constraint::Min(50)])
        .block(Block::default())
        .column_spacing(2);

    frame.render_widget(table, content_area);

    // Help text
    let help_text = "Esc: Close";
    frame.render_widget(
        Paragraph::new(help_text)
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center),
        content_layout[1],
    );
}

fn render_message_popup(frame: &mut Frame, area: Rect, message: &str) {
    let message_lines = message.lines().count().max(1) as u16;
    let longest_line = message
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(message.chars().count()) as u16;

    let popup_width = 40.max(longest_line + 6).min(area.width * 8 / 10);
    let popup_height = 6.max(message_lines + 4);

    let popup_area = centered_popup_area(area, popup_width, popup_height);

    let popup_block = create_popup_block("Message");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    let message_area = Rect::new(
        inner_area.x,
        inner_area.y,
        inner_area.width,
        inner_area.height.saturating_sub(2), // Reserve space for help text
    );

    let prompt = Paragraph::new(message)
        .style(Style::default())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(prompt, message_area);

    let separator = "─".repeat(popup_area.width.saturating_sub(2) as usize);
    let separator_area = Rect::new(
        popup_area.x + 1,
        popup_area.y + popup_area.height - 3,
        popup_area.width - 2,
        1,
    );

    let separator_widget = Paragraph::new(separator)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(separator_widget, separator_area);

    let help_text = "Press Esc to continue";
    let help_area = Rect::new(
        popup_area.x,
        popup_area.y + popup_area.height - 2,
        popup_area.width,
        1,
    );

    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(MUTED_COLOR))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg, help_area);
}

/// Renders a toast notification in the bottom-right corner.
/// This is a non-blocking overlay that doesn't prevent user interaction.
fn render_toast(frame: &mut Frame, area: Rect, message: &str) {
    let message_len = message.chars().count() as u16;
    let toast_width = (message_len + 4).min(area.width / 2).max(20);
    let toast_height = 3;

    // Position in bottom-right corner with some padding
    let toast_x = area.x + area.width.saturating_sub(toast_width + 2);
    let toast_y = area.y + area.height.saturating_sub(toast_height + 2);

    let toast_area = Rect::new(toast_x, toast_y, toast_width, toast_height);

    // Clear the area and draw a subtle bordered box
    frame.render_widget(Clear, toast_area);

    let toast_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(toast_block.clone(), toast_area);

    let inner_area = toast_block.inner(toast_area);

    // Determine color based on message content
    let text_color = if message.starts_with('✓') {
        SUCCESS_COLOR
    } else if message.starts_with('✗') {
        Color::Red
    } else {
        Color::White
    };

    let toast_text = Paragraph::new(message)
        .style(Style::default().fg(text_color))
        .alignment(Alignment::Center);

    frame.render_widget(toast_text, inner_area);
}
