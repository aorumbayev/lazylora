use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::{border, line, scrollbar},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, Table, Wrap,
    },
};

use crate::algorand::Transaction;
use crate::app_state::{App, Focus, PopupState};

/// Render the entire application UI
pub fn render(app: &App, frame: &mut Frame) {
    let size = frame.area();

    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(3),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(size);

    render_header(app, frame, chunks[0]);
    render_main_content(app, frame, chunks[1]);
    render_footer(app, frame, chunks[2]);

    match &app.popup_state {
        PopupState::NetworkSelect(selected_index) => {
            render_network_selector(app, frame, size, *selected_index);
        }
        PopupState::Search(query) => {
            render_search_popup(app, frame, size, query);
        }
        PopupState::Message(message) => {
            render_message_popup(app, frame, size, message);
        }
        PopupState::SearchResults(results) => {
            render_search_results(app, frame, size, results);
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
        .constraints([
            Constraint::Length(3), // Title area
            Constraint::Min(10),   // Content area
        ])
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
        .constraints([
            Constraint::Ratio(1, 2), // Blocks
            Constraint::Ratio(1, 2), // Transactions
        ])
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

    let mut block_items = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        let is_selected = app.selected_block_index == Some(i);

        let block_text = vec![
            Line::from(vec![
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
                "               ".into(),
                Span::styled(
                    format!("{} txns", block.txn_count),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                "  ".into(),
                Span::styled(&block.timestamp, Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
        ];

        let item = ListItem::new(block_text);
        block_items.push(if is_selected {
            item.style(Style::default().bg(Color::DarkGray))
        } else {
            item
        });
    }

    let block_height = 3_usize;
    let items_per_page = inner_area.height as usize / block_height;
    let block_scroll_usize = app.block_scroll as usize;
    let adjusted_scroll = (block_scroll_usize / block_height) * block_height;
    let start_index = (adjusted_scroll / block_height).min(blocks.len().saturating_sub(1));
    let end_index = (start_index + items_per_page).min(blocks.len());
    let visible_items = block_items[start_index..end_index].to_vec();

    let block_list = List::new(visible_items)
        .style(Style::default())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        );

    frame.render_widget(block_list, inner_area);

    if is_focused && blocks.len() > items_per_page {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .begin_symbol(Some(line::BOTTOM_LEFT))
            .end_symbol(Some(line::TOP_LEFT));

        let content_length = blocks.len() * block_height;
        let mut scroll_state = ratatui::widgets::ScrollbarState::default()
            .content_length(content_length)
            .position(adjusted_scroll);

        frame.render_stateful_widget(scrollbar, inner_area, &mut scroll_state);
    }
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

    let txn_items: Vec<ListItem> = if !app.filtered_transactions.is_empty() {
        app.filtered_transactions
            .iter()
            .map(|(orig_index, txn)| {
                let is_selected = app.selected_transaction_index == Some(*orig_index);
                create_transaction_list_item(
                    txn.id.clone(),
                    txn.from.clone(),
                    txn.to.clone(),
                    txn.txn_type.as_str().to_string(),
                    txn.txn_type.color(),
                    is_selected,
                )
            })
            .collect()
    } else {
        let transactions = app.transactions.lock().unwrap();

        if transactions.is_empty() {
            let no_data_message = Paragraph::new("No transactions available")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(no_data_message, inner_area);
            return;
        }

        transactions
            .iter()
            .enumerate()
            .map(|(i, txn)| {
                let is_selected = app.selected_transaction_index == Some(i);
                create_transaction_list_item(
                    txn.id.clone(),
                    txn.from.clone(),
                    txn.to.clone(),
                    txn.txn_type.as_str().to_string(),
                    txn.txn_type.color(),
                    is_selected,
                )
            })
            .collect()
    };

    if txn_items.is_empty() {
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

    let txn_height = 4_usize;
    let items_per_page = inner_area.height as usize / txn_height;
    let txn_scroll_usize = app.transaction_scroll as usize;
    let adjusted_scroll = (txn_scroll_usize / txn_height) * txn_height;
    let start_index = (adjusted_scroll / txn_height).min(txn_items.len().saturating_sub(1));
    let end_index = (start_index + items_per_page).min(txn_items.len());

    let visible_items = if start_index < end_index {
        txn_items[start_index..end_index].to_vec()
    } else {
        Vec::new()
    };

    let txn_list = List::new(visible_items)
        .style(Style::default())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        );

    frame.render_widget(txn_list, inner_area);

    if is_focused && txn_items.len() > items_per_page {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .begin_symbol(Some(line::BOTTOM_LEFT))
            .end_symbol(Some(line::TOP_LEFT));

        let content_length = txn_items.len() * txn_height;
        let mut scroll_state = ratatui::widgets::ScrollbarState::default()
            .content_length(content_length)
            .position(adjusted_scroll);

        frame.render_stateful_widget(scrollbar, inner_area, &mut scroll_state);
    }
}

fn create_transaction_list_item(
    id: String,
    from: String,
    to: String,
    txn_type: String,
    txn_color: Color,
    is_selected: bool,
) -> ListItem<'static> {
    let txn_type_style = Style::default().fg(txn_color);

    let txn_text = vec![
        Line::from(vec![
            if is_selected {
                "▶ ".into()
            } else {
                "→ ".into()
            },
            Span::styled(
                id,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            "          ".into(),
            Span::styled(format!("[{}]", txn_type), txn_type_style),
        ]),
        Line::from(vec![
            Span::styled("  From: ", Style::default().fg(Color::Gray)),
            Span::styled(from, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  To:   ", Style::default().fg(Color::Gray)),
            Span::styled(to, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
    ];

    let item = ListItem::new(txn_text);
    if is_selected {
        item.style(Style::default().bg(Color::DarkGray))
    } else {
        item
    }
}

/// Render block details popup
fn render_block_details(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(index) = app.selected_block_index {
        let blocks = app.blocks.lock().unwrap();
        if let Some(block) = blocks.get(index) {
            let popup_width = 60.min(area.width.saturating_sub(4));
            let popup_height = 15.min(area.height.saturating_sub(4));
            let popup_x = (area.width.saturating_sub(popup_width)) / 2;
            let popup_y = (area.height.saturating_sub(popup_height)) / 2;

            let popup_area = Rect::new(
                area.x + popup_x,
                area.y + popup_y,
                popup_width,
                popup_height,
            );

            frame.render_widget(Clear, popup_area);

            let popup_block = Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title(" Block Details ")
                .border_style(Style::default().fg(Color::Cyan));

            frame.render_widget(popup_block.clone(), popup_area);
            let inner_area = popup_block.inner(popup_area);

            let details = vec![
                ("Block ID:", format!("{}", block.id)),
                ("Transactions:", format!("{}", block.txn_count)),
                ("Timestamp:", block.timestamp.clone()),
            ];

            let rows: Vec<Row> = details
                .into_iter()
                .map(|(label, value)| {
                    Row::new(vec![
                        Cell::from(label).style(Style::default().fg(Color::Yellow)),
                        Cell::from(value),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                vec![Constraint::Percentage(30), Constraint::Percentage(70)],
            )
            .block(Block::default())
            .column_spacing(1);

            frame.render_widget(table, inner_area);

            render_close_message(frame, popup_area);
        }
    }
}

/// Render transaction details popup
fn render_transaction_details(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(index) = app.selected_transaction_index {
        let transactions = app.transactions.lock().unwrap();

        if let Some(txn) = transactions.get(index) {
            let popup_width = 60.min(area.width.saturating_sub(4));
            let popup_height = 20.min(area.height.saturating_sub(4));
            let popup_x = (area.width.saturating_sub(popup_width)) / 2;
            let popup_y = (area.height.saturating_sub(popup_height)) / 2;

            let popup_area = Rect::new(
                area.x + popup_x,
                area.y + popup_y,
                popup_width,
                popup_height,
            );

            frame.render_widget(Clear, popup_area);

            let popup_block = Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title(" Transaction Details ")
                .border_style(Style::default().fg(Color::Cyan));

            frame.render_widget(popup_block.clone(), popup_area);
            let inner_area = popup_block.inner(popup_area);

            let details = vec![
                ("Transaction ID:", txn.id.clone()),
                ("Type:", txn.txn_type.as_str().to_string()),
                ("From:", txn.from.clone()),
                ("To:", txn.to.clone()),
            ];

            let rows: Vec<Row> = details
                .into_iter()
                .map(|(label, value)| {
                    let value_style = if label == "Type:" {
                        Style::default().fg(txn.txn_type.color())
                    } else {
                        Style::default()
                    };

                    Row::new(vec![
                        Cell::from(label).style(Style::default().fg(Color::Yellow)),
                        Cell::from(value).style(value_style),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                vec![Constraint::Percentage(30), Constraint::Percentage(70)],
            )
            .block(Block::default())
            .column_spacing(1);

            frame.render_widget(table, inner_area);

            render_close_message(frame, popup_area);
        }
    }
}

fn render_close_message(frame: &mut Frame, popup_area: Rect) {
    let close_text = "[ESC] Close";
    let close_width = close_text.len() as u16;
    let close_area = Rect::new(
        popup_area.x + (popup_area.width - close_width) / 2,
        popup_area.y + popup_area.height - 2,
        close_width,
        1,
    );

    let close_paragraph = Paragraph::new(close_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(close_paragraph, close_area);
}

fn render_footer(_app: &App, frame: &mut Frame, area: Rect) {
    let footer_text = " [q]uit [r]efresh [f]ind [n]etwork [Space] Toggle Live [Enter] View Details [Tab] Switch Panel [↑/↓] Navigate";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });

    frame.render_widget(footer, area);
}

/// Render network selector
fn render_network_selector(_app: &App, frame: &mut Frame, area: Rect, selected_index: usize) {
    let popup_width = 30.min(area.width.saturating_sub(4));
    let popup_height = 12.min(area.height.saturating_sub(4));
    let popup_x = area.width.saturating_sub(popup_width).saturating_div(2);
    let popup_y = area.height.saturating_sub(popup_height).saturating_div(2);

    let popup_area = Rect::new(
        area.x.saturating_add(popup_x),
        area.y.saturating_add(popup_y),
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(" Select Network ")
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(popup_block.clone(), popup_area);
    let inner_area = popup_block.inner(popup_area);

    let networks = ["MainNet", "TestNet", "LocalNet"];
    let items: Vec<ListItem> = networks
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == selected_index {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if i == selected_index { "> " } else { "  " };
            ListItem::new(format!("{}{}", prefix, name)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, inner_area);

    let help_text = "[↑/↓] Navigate [Enter] Select [Esc] Cancel";
    let help_width = help_text.len().min(popup_area.width as usize - 4) as u16;
    let help_x = popup_area
        .x
        .saturating_add((popup_area.width.saturating_sub(help_width)).saturating_div(2));
    let help_y = popup_area
        .y
        .saturating_add(popup_area.height.saturating_sub(2));

    let help_area = Rect::new(help_x, help_y, help_width, 1);
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(help_paragraph, help_area);
}

/// Render search popup
fn render_search_popup(_app: &App, frame: &mut Frame, area: Rect, query: &str) {
    let popup_width = 50.min(area.width.saturating_sub(4));
    let popup_height = 7.min(area.height.saturating_sub(4));
    let popup_x = area.width.saturating_sub(popup_width).saturating_div(2);
    let popup_y = area.height.saturating_sub(popup_height).saturating_div(2);

    let popup_area = Rect::new(
        area.x.saturating_add(popup_x),
        area.y.saturating_add(popup_y),
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(" Search Transactions ")
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(popup_block.clone(), popup_area);
    let inner_area = popup_block.inner(popup_area);

    let instructions = Paragraph::new("Enter transaction ID, sender, or receiver address:")
        .style(Style::default())
        .alignment(Alignment::Left);

    let instructions_area = Rect::new(inner_area.x, inner_area.y, inner_area.width, 1);
    frame.render_widget(instructions, instructions_area);

    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default());

    let search_input_area = Rect::new(inner_area.x, inner_area.y + 2, inner_area.width, 3);
    frame.render_widget(search_block.clone(), search_input_area);

    let query_text = format!("{}_", query);
    let query_paragraph = Paragraph::new(query_text)
        .style(Style::default())
        .alignment(Alignment::Left);

    let query_text_area = Rect::new(
        search_input_area.x + 1,
        search_input_area.y + 1,
        search_input_area.width - 2,
        1,
    );

    frame.render_widget(query_paragraph, query_text_area);
}

/// Render a message popup
fn render_message_popup(_app: &App, frame: &mut Frame, area: Rect, message: &str) {
    let popup_width = 50.min(area.width.saturating_sub(4));
    let popup_height = 5.min(area.height.saturating_sub(4));
    let popup_x = area.width.saturating_sub(popup_width).saturating_div(2);
    let popup_y = area.height.saturating_sub(popup_height).saturating_div(2);

    let popup_area = Rect::new(
        area.x.saturating_add(popup_x),
        area.y.saturating_add(popup_y),
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(" Message ")
        .border_style(Style::default().fg(Color::Yellow));

    frame.render_widget(popup_block.clone(), popup_area);
    let inner_area = popup_block.inner(popup_area);

    let message_paragraph = Paragraph::new(message)
        .style(Style::default())
        .alignment(Alignment::Center);

    let message_area = Rect::new(inner_area.x, inner_area.y + 1, inner_area.width, 1);
    frame.render_widget(message_paragraph, message_area);

    render_close_message(frame, popup_area);
}

/// Render search results popup
fn render_search_results(
    _app: &App,
    frame: &mut Frame,
    area: Rect,
    results: &Vec<(usize, Transaction)>,
) {
    let popup_width = 70.min(area.width.saturating_sub(4));
    let popup_height = 20.min(area.height.saturating_sub(4));
    let popup_x = area.width.saturating_sub(popup_width).saturating_div(2);
    let popup_y = area.height.saturating_sub(popup_height).saturating_div(2);

    let popup_area = Rect::new(
        area.x.saturating_add(popup_x),
        area.y.saturating_add(popup_y),
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(" Search Results ")
        .border_style(Style::default().fg(Color::Green));

    frame.render_widget(popup_block.clone(), popup_area);
    let inner_area = popup_block.inner(popup_area);

    let txn_items: Vec<ListItem> = results
        .iter()
        .map(|(_orig_index, txn)| {
            create_transaction_list_item(
                txn.id.clone(),
                txn.from.clone(),
                txn.to.clone(),
                txn.txn_type.as_str().to_string(),
                txn.txn_type.color(),
                false,
            )
        })
        .collect();

    let txn_height = 4_usize;
    let items_per_page = inner_area.height as usize / txn_height;
    let visible_items = txn_items
        .iter()
        .take(items_per_page)
        .cloned()
        .collect::<Vec<_>>();

    let txn_list = List::new(visible_items)
        .style(Style::default())
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(txn_list, inner_area);
    render_close_message(frame, popup_area);
}
