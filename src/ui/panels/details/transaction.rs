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
    widgets::{Block, Cell, Clear, Paragraph, Row, Table},
};

use crate::domain::{SearchResultItem, Transaction, TransactionDetails};
use crate::state::{App, DetailViewMode};
use crate::theme::{BG_COLOR, MUTED_COLOR, PRIMARY_COLOR, WARNING_COLOR};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::centered_popup_area;
use crate::widgets::{TxnGraph, TxnGraphWidget, TxnVisualCard};

/// Renders the transaction details popup with table or visual graph view.
///
/// Displays comprehensive transaction information with support for all
/// Algorand transaction types. Includes toggleable visual graph mode with
/// auto-scaling and scrolling support.
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
        let graph_width = graph.total_width();
        let graph_height = graph_widget.required_height();

        // Add padding for chrome (borders, tabs, buttons, help text)
        let chrome_h_padding: u16 = 10; // 2 border + 4 content padding + margin
        let chrome_v_padding: u16 = 10; // 2 border + tabs(2) + separator(1) + button(4) + help(1)

        // Calculate ideal dimensions with reasonable bounds
        let min_width: u16 = 50;
        let min_height: u16 = 18;
        let max_width = (area.width as f32 * 0.92) as u16;
        let max_height = (area.height as f32 * 0.92) as u16;

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

    if is_visual {
        render_visual_mode(app, &txn, &graph, &graph_widget, frame, content_area);
    } else {
        render_table_mode(&txn, app, frame, content_area);
    }

    // Render compact action bar with inline buttons
    let button_area = content_layout[3];

    let action_bar = Line::from(vec![
        Span::styled("  [C]", Style::default().fg(PRIMARY_COLOR)),
        Span::styled(" Copy", Style::default().fg(Color::White)),
        Span::raw("   "),
        Span::styled("[S]", Style::default().fg(PRIMARY_COLOR)),
        Span::styled(" Export SVG", Style::default().fg(Color::White)),
        Span::raw("   "),
        Span::styled("[Tab]", Style::default().fg(PRIMARY_COLOR)),
        Span::styled(" View", Style::default().fg(Color::White)),
        Span::raw("   "),
        Span::styled("[Esc]", Style::default().fg(PRIMARY_COLOR)),
        Span::styled(" Close", Style::default().fg(Color::White)),
    ]);

    let action_paragraph = Paragraph::new(action_bar).alignment(Alignment::Center);
    let action_rect = Rect::new(button_area.x, button_area.y + 1, button_area.width, 1);
    frame.render_widget(action_paragraph, action_rect);

    // Render minimal help text
    let help_area = content_layout[4];
    let help_text = if is_visual {
        "↑↓←→ Scroll"
    } else {
        "↑↓ Navigate"
    };

    let help_msg = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(help_msg, help_area);
}

/// Renders the visual graph view of a transaction with scrolling support.
///
/// # Arguments
///
/// * `app` - Application state for scroll position
/// * `txn` - Transaction to render (used as fallback when graph is empty)
/// * `graph` - Transaction graph structure
/// * `graph_widget` - Graph widget for rendering
/// * `frame` - Ratatui frame
/// * `area` - Content area for the graph
fn render_visual_mode(
    app: &App,
    txn: &Transaction,
    graph: &TxnGraph,
    graph_widget: &TxnGraphWidget,
    frame: &mut Frame,
    area: Rect,
) {
    let graph_lines = graph_widget.to_lines();

    // If graph has meaningful content, show it
    if !graph.columns.is_empty() {
        // Calculate padded area (minimal padding for compactness)
        let padded_area = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );

        // Calculate graph dimensions (use required_width for accurate measurement)
        let graph_height = graph_widget.required_height();
        let graph_width = graph_widget.required_width();

        // Determine if we need scrolling
        let needs_v_scroll = graph_height > padded_area.height as usize;
        let needs_h_scroll = graph_width > padded_area.width as usize;

        // Calculate max scroll values
        let max_scroll_y = graph_height.saturating_sub(padded_area.height as usize);
        let max_scroll_x = graph_width.saturating_sub(padded_area.width as usize);

        // Get scroll offsets from app state, clamped to valid range
        let scroll_x = (app.nav.graph_scroll_x as usize).min(max_scroll_x);
        let scroll_y = (app.nav.graph_scroll_y as usize).min(max_scroll_y);

        // Calculate centering offsets (when graph fits in view)
        let center_x = if !needs_h_scroll {
            (padded_area.width as usize).saturating_sub(graph_width) / 2
        } else {
            0
        };
        let center_y = if !needs_v_scroll {
            (padded_area.height as usize).saturating_sub(graph_height) / 2
        } else {
            0
        };

        // Build visible lines with centering or scrolling
        let visible_lines: Vec<Line> = if needs_v_scroll || needs_h_scroll {
            // Scrolling mode - apply scroll offsets
            graph_lines
                .into_iter()
                .skip(scroll_y)
                .take(padded_area.height as usize)
                .map(|line| {
                    if scroll_x > 0 {
                        let mut remaining_skip = scroll_x;
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

        // Show scroll indicator ONLY if scrolling is needed
        if needs_v_scroll || needs_h_scroll {
            render_scroll_indicator(
                frame,
                padded_area,
                needs_v_scroll,
                needs_h_scroll,
                scroll_y,
                scroll_x,
                max_scroll_y,
                max_scroll_x,
            );
        }
    } else {
        // Fallback to TxnVisualCard for edge cases
        let visual_card = TxnVisualCard::new(txn);
        let lines = visual_card.to_lines();

        let visual_content = Paragraph::new(lines).alignment(Alignment::Left);

        let padded_area = Rect::new(
            area.x + 2,
            area.y + 1,
            area.width.saturating_sub(4),
            area.height.saturating_sub(2),
        );
        frame.render_widget(visual_content, padded_area);
    }
}

/// Renders scroll indicators in the bottom-right corner of the graph area.
#[allow(clippy::too_many_arguments)]
fn render_scroll_indicator(
    frame: &mut Frame,
    area: Rect,
    needs_v_scroll: bool,
    needs_h_scroll: bool,
    scroll_y: usize,
    scroll_x: usize,
    max_scroll_y: usize,
    max_scroll_x: usize,
) {
    // Build a compact scroll indicator
    let scroll_hint = if needs_v_scroll && needs_h_scroll {
        // Show position with directional arrows
        let v_indicator = if scroll_y > 0 && scroll_y < max_scroll_y {
            "↕"
        } else if scroll_y > 0 {
            "↑"
        } else {
            "↓"
        };
        let h_indicator = if scroll_x > 0 && scroll_x < max_scroll_x {
            "↔"
        } else if scroll_x > 0 {
            "←"
        } else {
            "→"
        };
        format!(" {} {} ", v_indicator, h_indicator)
    } else if needs_v_scroll {
        let v_indicator = if scroll_y > 0 && scroll_y < max_scroll_y {
            "↕"
        } else if scroll_y > 0 {
            "↑"
        } else {
            "↓"
        };
        format!(" {} ", v_indicator)
    } else {
        let h_indicator = if scroll_x > 0 && scroll_x < max_scroll_x {
            "↔"
        } else if scroll_x > 0 {
            "←"
        } else {
            "→"
        };
        format!(" {} ", h_indicator)
    };

    let hint_width = scroll_hint.chars().count() as u16;
    let hint_area = Rect::new(
        area.x + area.width.saturating_sub(hint_width + 1),
        area.y + area.height.saturating_sub(1),
        hint_width,
        1,
    );

    let hint_widget =
        Paragraph::new(scroll_hint).style(Style::default().fg(Color::DarkGray).bg(BG_COLOR));
    frame.render_widget(hint_widget, hint_area);
}

/// Renders the table view of transaction details with type-specific fields.
///
/// # Arguments
///
/// * `txn` - Transaction to display
/// * `app` - Application state for navigation
/// * `frame` - Ratatui frame
/// * `area` - Content area for the table
fn render_table_mode(txn: &Transaction, app: &App, frame: &mut Frame, area: Rect) {
    let formatted_fee = format!("{:.6} Algos", txn.fee as f64 / 1_000_000.0);

    let mut details: Vec<(String, String)> = vec![
        ("Transaction ID:".to_string(), txn.id.clone()),
        ("Type:".to_string(), txn.txn_type.as_str().to_string()),
        ("From:".to_string(), txn.from.clone()),
    ];

    // Add type-specific fields
    match &txn.details {
        TransactionDetails::Payment(pay_details) => {
            render_payment_details(&mut details, txn, &formatted_fee, pay_details);
        }
        TransactionDetails::AssetTransfer(axfer_details) => {
            render_asset_transfer_details(&mut details, txn, &formatted_fee, axfer_details);
        }
        TransactionDetails::AssetConfig(acfg_details) => {
            render_asset_config_details(&mut details, txn, &formatted_fee, acfg_details);
        }
        TransactionDetails::AssetFreeze(afrz_details) => {
            render_asset_freeze_details(&mut details, txn, &formatted_fee, afrz_details);
        }
        TransactionDetails::AppCall(app_details) => {
            render_app_call_details(&mut details, txn, &formatted_fee, app_details, app);
        }
        TransactionDetails::KeyReg(keyreg_details) => {
            render_keyreg_details(&mut details, txn, &formatted_fee, keyreg_details);
        }
        TransactionDetails::StateProof(sp_details) => {
            render_state_proof_details(&mut details, txn, &formatted_fee, sp_details);
        }
        TransactionDetails::Heartbeat(hb_details) => {
            render_heartbeat_details(&mut details, txn, &formatted_fee, hb_details);
        }
        TransactionDetails::None => {
            render_default_details(&mut details, txn, &formatted_fee);
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
        .column_spacing(2);

    frame.render_widget(table, area);
}

// ============================================================================
// Transaction Type-Specific Rendering Functions
// ============================================================================

/// Adds payment transaction specific fields to the details list.
fn render_payment_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    pay_details: &crate::domain::PaymentDetails,
) {
    details.push(("To:".to_string(), txn.to.clone()));
    details.push((
        "Amount:".to_string(),
        format!("{:.6} Algos", txn.amount as f64 / 1_000_000.0),
    ));
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
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

/// Adds asset transfer transaction specific fields to the details list.
fn render_asset_transfer_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    axfer_details: &crate::domain::AssetTransferDetails,
) {
    details.push(("To:".to_string(), txn.to.clone()));
    details.push(("Amount:".to_string(), format!("{} units", txn.amount)));
    if let Some(asset_id) = txn.asset_id {
        details.push(("Asset ID:".to_string(), format!("{}", asset_id)));
    }
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
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

/// Adds asset config transaction specific fields to the details list.
fn render_asset_config_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    acfg_details: &crate::domain::AssetConfigDetails,
) {
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
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

/// Adds asset freeze transaction specific fields to the details list.
fn render_asset_freeze_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    afrz_details: &crate::domain::AssetFreezeDetails,
) {
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
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));
}

/// Adds application call transaction specific fields to the details list.
fn render_app_call_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    app_details: &crate::domain::AppCallDetails,
    app: &App,
) {
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
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
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
                        details.push((format!("    [{}]:", i), format!("{}", asset_id)));
                    }
                }
                "boxes" => {
                    for (i, box_ref) in app_details.boxes.iter().enumerate() {
                        let box_desc = format!("App: {}, Name: {}", box_ref.app_id, box_ref.name);
                        details.push((format!("    [{}]:", i), box_desc));
                    }
                }
                _ => {}
            }
        }
    }
}

/// Adds key registration transaction specific fields to the details list.
fn render_keyreg_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    keyreg_details: &crate::domain::KeyRegDetails,
) {
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
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

/// Adds state proof transaction specific fields to the details list.
fn render_state_proof_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    sp_details: &crate::domain::StateProofDetails,
) {
    if let Some(sp_type) = sp_details.state_proof_type {
        details.push(("State Proof Type:".to_string(), format!("{}", sp_type)));
    }
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));
}

/// Adds heartbeat transaction specific fields to the details list.
fn render_heartbeat_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
    hb_details: &crate::domain::HeartbeatDetails,
) {
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));

    if let Some(hb_addr) = &hb_details.hb_address {
        details.push(("Heartbeat Addr:".to_string(), hb_addr.clone()));
    }
}

/// Adds default transaction fields for unknown transaction types.
fn render_default_details(
    details: &mut Vec<(String, String)>,
    txn: &Transaction,
    formatted_fee: &str,
) {
    details.push(("To:".to_string(), txn.to.clone()));
    details.push(("Amount:".to_string(), format!("{}", txn.amount)));
    details.push(("Fee:".to_string(), formatted_fee.to_string()));
    details.push(("Block:".to_string(), format!("#{}", txn.block)));
    details.push(("Timestamp:".to_string(), txn.timestamp.clone()));
}
