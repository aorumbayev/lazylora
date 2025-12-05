//! Account detail panel rendering.
//!
//! This module handles the display of detailed account information including
//! balances, rewards, asset holdings, participation status, and NFD integration.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Cell, Clear, Paragraph, Row, Table},
};

use crate::state::App;
use crate::theme::{
    ACCENT_COLOR, MUTED_COLOR, PRIMARY_COLOR, SECONDARY_COLOR, SUCCESS_COLOR, WARNING_COLOR,
};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::centered_popup_area;

/// Renders the account details popup with comprehensive account information.
///
/// Displays account balances, participation status, asset holdings, and
/// optional NFD (Non-Fungible Domain) information if available.
///
/// # Arguments
///
/// * `app` - Application state containing account data
/// * `frame` - Ratatui frame for rendering
/// * `area` - Available screen area for rendering
pub fn render_account_details(app: &App, frame: &mut Frame, area: Rect) {
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
