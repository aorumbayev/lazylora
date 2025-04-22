use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    algorand::{Network, SearchResultItem, TxnType},
    app::{AddCustomNetworkState, CustomNetworkField, SearchResultsState, SearchType},
    components::helpers::centered_fixed_popup_area, // Use fixed size helper
};

/// Render network selector popup.
pub fn render_network_selector(
    frame: &mut Frame,
    area: Rect,
    available_networks: &[Network],
    selected_index: usize,
) {
    let num_options = available_networks.len() + 1; // +1 for "Add Custom..."
    let popup_height = (num_options + 4).min(10) as u16; // Keep it reasonably sized
    let popup_area = centered_fixed_popup_area(area, 40, popup_height);

    let popup_block = Block::default()
        .title(" Select Network ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(Clear, popup_area); // Clear background
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Layout for list and help text
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)]) // List, Help
        .split(inner_area);
    let list_area = chunks[0];
    let help_area = chunks[1];

    let mut list_items = Vec::new();
    for (i, network) in available_networks.iter().enumerate() {
        let is_selected = i == selected_index;
        list_items.push(
            ListItem::new(Line::from(vec![
                if is_selected { "▶ " } else { "  " }.into(),
                Span::styled(network.as_str(), Style::default()),
            ]))
            .style(if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }),
        );
    }

    // Add "Add Custom Network" option
    let add_custom_index = available_networks.len();
    let is_add_custom_selected = add_custom_index == selected_index;
    list_items.push(
        ListItem::new(Line::from(vec![
            if is_add_custom_selected { "▶ " } else { "  " }.into(),
            Span::styled("Add Custom Network...", Style::default().fg(Color::Green)),
        ]))
        .style(if is_add_custom_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }),
    );

    // Render the list (no state needed as selection is handled by styles)
    let list = List::new(list_items).block(Block::default());
    frame.render_widget(list, list_area);

    // Render help text
    let help_text = "↑↓: Move | Enter: Select | Esc: Cancel";
    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(help_msg, help_area);
}

/// Render the "Add Custom Network" popup.
pub fn render_add_custom_network_popup(
    frame: &mut Frame,
    area: Rect,
    state: &AddCustomNetworkState,
) {
    let popup_area = centered_fixed_popup_area(area, 60, 18); // Fixed size

    let popup_block = Block::default()
        .title(" Add Custom Network ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Layout for form fields and help text
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Algod URL
            Constraint::Length(3), // Indexer URL
            Constraint::Length(3), // Token
            Constraint::Min(0),    // Spacer
            Constraint::Length(1), // Help Text
        ])
        .split(inner_area);

    // Helper to render a field (avoids repetitive code)
    let render_field = |f: &mut Frame, chunk: Rect, title: &str, value: &str, is_focused: bool| {
        let border_style = if is_focused {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(border_style)
            .title(format!(" {} ", title)); // Padded title

        f.render_widget(input_block.clone(), chunk);

        // Render text with cursor if focused
        let text_area = input_block.inner(chunk);
        let display_text = if is_focused {
            format!("{}\u{2588}", value) // Add block cursor
        } else {
            value.to_string()
        };
        let input_paragraph = Paragraph::new(Text::from(display_text))
            .style(Style::default())
            .alignment(Alignment::Left);

        f.render_widget(input_paragraph, text_area);
    };

    // Render fields using the helper, accessing fields from state
    render_field(
        frame,
        chunks[0],
        "Name*",
        &state.name,
        state.focused_field == CustomNetworkField::Name, // Use enum variant
    );
    render_field(
        frame,
        chunks[1],
        "Algod URL*",
        &state.algod_url,
        state.focused_field == CustomNetworkField::AlgodUrl, // Use enum variant
    );
    render_field(
        frame,
        chunks[2],
        "Indexer URL*",
        &state.indexer_url,
        state.focused_field == CustomNetworkField::IndexerUrl, // Use enum variant
    );
    render_field(
        frame,
        chunks[3],
        "Algod Token (Optional)",
        &state.algod_token,
        state.focused_field == CustomNetworkField::Token, // Use enum variant
    );

    // Render help text
    let help_text = "Tab/↑↓: Navigate | Enter: Save | Esc: Cancel | *: Required";
    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(help_msg, chunks[5]);
}

/// Render search with type popup.
pub fn render_search_with_type_popup(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    search_type: SearchType,
) {
    let popup_area = centered_fixed_popup_area(area, 60, 12); // Adjusted height

    let popup_block = Block::default()
        .title(" Search Network ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Layout for Input, Type Selectors, Help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Input field
            Constraint::Length(1), // Type selectors
            Constraint::Min(1),    // Spacer
            Constraint::Length(1), // Help Text
        ])
        .split(inner_area);

    // Input field
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Blue)); // Focus indicator
    frame.render_widget(input_block.clone(), chunks[0]);

    let text_input_area = input_block.inner(chunks[0]);
    let input_text = format!("{}\u{2588}", query); // Add cursor
    // Calculate scroll offset to keep cursor visible
    let scroll_offset =
        (query.chars().count() as u16).saturating_sub(text_input_area.width.saturating_sub(1));
    let input = Paragraph::new(input_text)
        .style(Style::default())
        .alignment(Alignment::Left)
        .scroll((0, scroll_offset)); // Enable horizontal scroll
    frame.render_widget(input, text_input_area);

    // Search type selectors
    let search_types = [
        SearchType::Transaction,
        SearchType::Block,
        SearchType::Account,
        SearchType::Asset,
    ];
    let selector_spans: Vec<Span> = search_types
        .iter()
        .map(|t| {
            let is_selected = *t == search_type;
            let style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            };
            Span::styled(format!(" {} ", t.as_str()), style)
        })
        .collect();

    let selectors_line = Line::from(Span::styled("Type: ", Style::default().fg(Color::Gray)))
        .spans(
            selector_spans
                .into_iter()
                .flat_map(|s| [s, Span::raw(" ")])
                .collect::<Vec<_>>(),
        );

    let selectors = Paragraph::new(selectors_line).alignment(Alignment::Center);
    frame.render_widget(selectors, chunks[1]);

    // Help text
    let help_text = "Tab: Change Type | Enter: Search | Esc: Cancel";
    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(help_msg, chunks[3]);
}

/// Render search results popup.
pub fn render_search_results(
    frame: &mut Frame,
    area: Rect,
    state: &SearchResultsState, // Corrected type to SearchResultsState
) {
    let popup_area = centered_fixed_popup_area(area, 76, 20);

    let popup_block = Block::default()
        .title(" Search Results ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Layout for list and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)]) // List, Help
        .split(inner_area);
    let list_area = chunks[0];
    let help_area = chunks[1];

    let mut list_items = Vec::new();
    for (i, (_original_idx, item)) in state.results.iter().enumerate() {
        let is_selected = i == state.selected_index;
        let item_lines = match item {
            SearchResultItem::Transaction(txn) => {
                let amount_text = match txn.txn_type {
                    TxnType::Payment => format!("{:.6} Algos", txn.amount as f64 / 1_000_000.0),
                    TxnType::AssetTransfer => {
                        if let Some(asset_id) = txn.asset_id {
                            format!("{} units (Asset: {})", txn.amount, asset_id)
                        } else {
                            format!("{} units", txn.amount)
                        }
                    }
                    _ => txn.amount.to_string(),
                };
                let id_span = Span::styled(
                    txn.id.clone(),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                );
                let type_span = Span::styled(
                    format!("[{}]", txn.txn_type.as_str()),
                    Style::default().fg(txn.txn_type.color()),
                );
                vec![
                    Line::from(vec![id_span, "  ".into(), type_span]),
                    Line::from(vec![
                        Span::styled("  From: ", Style::default().fg(Color::Gray)),
                        Span::styled(txn.from.clone(), Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![
                        Span::styled("  To:   ", Style::default().fg(Color::Gray)),
                        Span::styled(txn.to.clone(), Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(vec![
                        "  ".into(),
                        Span::styled(txn.timestamp.clone(), Style::default().fg(Color::Gray)),
                        "  ".into(),
                        Span::styled(amount_text, Style::default().fg(Color::Green)),
                    ]),
                ]
            }
            SearchResultItem::Block(block) => {
                let id_span = Span::styled(
                    format!("Block # {}", block.id),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );
                let type_span = Span::styled("[Block]", Style::default().fg(Color::White));
                vec![
                    Line::from(vec![id_span, "  ".into(), type_span]),
                    Line::from(vec![
                        Span::styled("  Time: ", Style::default().fg(Color::Gray)),
                        Span::styled(block.timestamp.clone(), Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![
                        Span::styled("  Txns: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{}", block.txn_count),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Proposer: ", Style::default().fg(Color::Gray)),
                        Span::styled(block.proposer.clone(), Style::default().fg(Color::Magenta)),
                    ]),
                ]
            }
            SearchResultItem::Account(account) => {
                let id_span = Span::styled(
                    account.address.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
                let type_span = Span::styled("[Account]", Style::default().fg(Color::Yellow));
                let balance_text = format!("{:.6} Algos", account.balance as f64 / 1_000_000.0);
                vec![
                    Line::from(vec![id_span, "  ".into(), type_span]),
                    Line::from(vec![
                        Span::styled("  Balance: ", Style::default().fg(Color::Gray)),
                        Span::styled(balance_text, Style::default().fg(Color::Green)),
                    ]),
                    Line::from(vec![
                        Span::styled("  Status: ", Style::default().fg(Color::Gray)),
                        Span::styled(account.status.clone(), Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(vec![
                        Span::styled("  Assets: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{}", account.assets_count),
                            Style::default().fg(Color::Magenta),
                        ),
                    ]),
                ]
            }
            SearchResultItem::Asset(asset) => {
                let id_span = Span::styled(
                    format!("Asset # {}", asset.id),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                );
                let type_span = Span::styled("[Asset]", Style::default().fg(Color::Green));
                let name = if asset.name.is_empty() {
                    "<unnamed>"
                } else {
                    &asset.name
                };
                let unit = if asset.unit_name.is_empty() {
                    "".to_string()
                } else {
                    format!(" ({})", asset.unit_name)
                };
                let total_supply = format!("{} (decimals: {})", asset.total, asset.decimals);
                vec![
                    Line::from(vec![id_span, "  ".into(), type_span]),
                    Line::from(vec![
                        Span::styled("  Name: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{}{}", name, unit),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Creator: ", Style::default().fg(Color::Gray)),
                        Span::styled(asset.creator.clone(), Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![
                        Span::styled("  Total: ", Style::default().fg(Color::Gray)),
                        Span::styled(total_supply, Style::default().fg(Color::Magenta)),
                    ]),
                ]
            }
        };

        // Add selection indicator and spacing
        let indicator = Span::from(if is_selected { "▶ " } else { "  " });
        let indent = Span::from("  ");

        // Build the lines manually with prepended spans
        let mut final_lines: Vec<Line> = vec![];
        // First line with indicator
        let mut first_line_spans = vec![indicator.clone()];
        first_line_spans.extend(item_lines[0].spans.clone());
        final_lines.push(Line::from(first_line_spans));

        // Subsequent lines with indentation
        for line in item_lines.iter().skip(1) {
            let mut indented_line_spans = vec![indent.clone()];
            indented_line_spans.extend(line.spans.clone());
            final_lines.push(Line::from(indented_line_spans));
        }
        final_lines.push(Line::from("")); // Add spacer line

        list_items.push(ListItem::new(final_lines).style(if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        }));
    }

    // Render the list with state
    let list = List::new(list_items)
        .block(Block::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD)); // Highlight style might not be needed if we handle manually

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_index));

    frame.render_stateful_widget(list, list_area, &mut list_state);

    // Render help text
    let help_text = "↑↓: Navigate | Enter: Select/View | Esc: Cancel";
    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(help_msg, help_area);
}

/// Render a generic message popup.
pub fn render_message_popup(frame: &mut Frame, area: Rect, message: &str) {
    // Calculate appropriate popup size based on message content
    let message_lines: Vec<&str> = message.lines().collect();
    let message_line_count = message_lines.len().max(1) as u16;
    let max_line_width = message_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(20) as u16;

    // Ensure width is reasonable
    let popup_width = max_line_width.max(30).min(area.width * 8 / 10);
    let popup_height = (message_line_count + 4).min(area.height * 6 / 10); // Max 60% height

    let popup_area = centered_fixed_popup_area(area, popup_width, popup_height);

    let popup_block = Block::default()
        .title(" Message ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Yellow)); // Use Yellow for messages

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Layout for message and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)]) // Message, Help
        .split(inner_area);
    let message_area = chunks[0];
    let help_area = chunks[1];

    // Render message content
    let prompt = Paragraph::new(message)
        .style(Style::default())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(prompt, message_area);

    // Render help text
    let help_text = "Press Esc to continue";
    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(help_msg, help_area);
}
