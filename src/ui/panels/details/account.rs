//! Account detail panel rendering.
//!
//! This module handles the display of detailed account information including
//! balances, rewards, asset holdings, participation status, and NFD integration.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};

use crate::domain::account::AccountDetails;
use crate::state::{AccountDetailTab, App};
use crate::theme::{
    ACCENT_COLOR, MUTED_COLOR, PRIMARY_COLOR, SECONDARY_COLOR, SUCCESS_COLOR, WARNING_COLOR,
};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::{centered_popup_area, fullscreen_popup_area};

/// Renders the account details popup with tabbed interface.
///
/// Supports tabbed navigation between Info, Assets, and Apps views.
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

    let popup_area = if app.ui.detail_fullscreen {
        fullscreen_popup_area(area)
    } else {
        centered_popup_area(area, 85, 34)
    };
    let popup_block = create_popup_block("Account Details");
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
    render_tab_bar(app, frame, content_layout[0]);

    // Separator
    let separator = "─".repeat(inner_area.width as usize);
    frame.render_widget(
        Paragraph::new(separator).style(Style::default().fg(Color::DarkGray)),
        content_layout[1],
    );

    // Content based on tab
    let content_area = content_layout[2];
    match app.nav.account_detail_tab {
        AccountDetailTab::Info => render_info_tab(account, frame, content_area),
        AccountDetailTab::Assets => render_assets_tab(app, account, frame, content_area),
        AccountDetailTab::Apps => render_apps_tab(app, account, frame, content_area),
    }

    // Help text
    let help_text = "[Tab] Switch  [↑↓] Navigate  [C] Copy  [Y] JSON  [O] Open  [Esc] Close";
    frame.render_widget(
        Paragraph::new(help_text)
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center),
        content_layout[3],
    );
}

/// Renders the tab bar for account details.
fn render_tab_bar(app: &App, frame: &mut Frame, area: Rect) {
    let current_tab = app.nav.account_detail_tab;

    let tab_style = |is_active: bool| {
        if is_active {
            Style::default()
                .bg(PRIMARY_COLOR)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MUTED_COLOR)
        }
    };

    let tab_bar = Line::from(vec![
        Span::raw("  "),
        Span::styled(" Info ", tab_style(current_tab == AccountDetailTab::Info)),
        Span::raw("  "),
        Span::styled(
            " Assets ",
            tab_style(current_tab == AccountDetailTab::Assets),
        ),
        Span::raw("  "),
        Span::styled(" Apps ", tab_style(current_tab == AccountDetailTab::Apps)),
    ]);
    frame.render_widget(Paragraph::new(tab_bar), area);
}

/// Renders the Info tab with general account information.
fn render_info_tab(account: &AccountDetails, frame: &mut Frame, area: Rect) {
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

    let table = Table::new(rows, [Constraint::Length(20), Constraint::Min(50)])
        .block(Block::default())
        .column_spacing(2);

    frame.render_widget(table, area);
}

/// Renders the Assets tab with asset holdings and created assets.
fn render_assets_tab(app: &App, account: &AccountDetails, frame: &mut Frame, area: Rect) {
    // Split area for holdings and created assets
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Holdings header
            Constraint::Min(8),    // Holdings list
            Constraint::Length(1), // Created header
            Constraint::Min(4),    // Created list
        ])
        .split(area);

    // Asset Holdings section
    let holdings_header = Paragraph::new(format!(
        " Asset Holdings ({} total)",
        account.total_assets_opted_in
    ))
    .style(
        Style::default()
            .fg(ACCENT_COLOR)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(holdings_header, sections[0]);

    if account.assets.is_empty() {
        let empty_msg = Paragraph::new("  No assets held")
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Left);
        frame.render_widget(empty_msg, sections[1]);
    } else {
        let scroll_offset = app.nav.account_item_scroll as usize;
        let visible_height = sections[1].height as usize;

        let items: Vec<ListItem> = account
            .assets
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(i, asset)| {
                let is_selected = app.nav.account_item_index == Some(i)
                    && app.nav.account_detail_tab == AccountDetailTab::Assets;
                let indicator = if is_selected { "▶" } else { " " };
                let frozen_indicator = if asset.is_frozen { " [frozen]" } else { "" };

                let style = if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::raw(format!("{} ", indicator)),
                    Span::styled(
                        format!("Asset #{}", asset.asset_id),
                        Style::default().fg(SECONDARY_COLOR),
                    ),
                    Span::raw(": "),
                    Span::styled(
                        format!("{}", asset.amount),
                        Style::default().fg(SUCCESS_COLOR),
                    ),
                    Span::styled(frozen_indicator, Style::default().fg(Color::Red)),
                ]))
                .style(style)
            })
            .collect();

        let list = List::new(items).block(Block::default());
        frame.render_widget(list, sections[1]);
    }

    // Created Assets section
    let created_header = Paragraph::new(format!(
        " Created Assets ({} total)",
        account.total_created_assets
    ))
    .style(
        Style::default()
            .fg(ACCENT_COLOR)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(created_header, sections[2]);

    if account.created_assets.is_empty() {
        let empty_msg = Paragraph::new("  No assets created")
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Left);
        frame.render_widget(empty_msg, sections[3]);
    } else {
        let items: Vec<ListItem> = account
            .created_assets
            .iter()
            .take(sections[3].height as usize)
            .map(|asset| {
                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("#{}", asset.asset_id),
                        Style::default().fg(SECONDARY_COLOR),
                    ),
                    Span::raw(": "),
                    Span::styled(&asset.name, Style::default().fg(Color::White)),
                    Span::raw(" ("),
                    Span::styled(&asset.unit_name, Style::default().fg(MUTED_COLOR)),
                    Span::raw(")"),
                ]))
            })
            .collect();

        let list = List::new(items).block(Block::default());
        frame.render_widget(list, sections[3]);
    }
}

/// Renders the Apps tab with opted-in and created applications.
fn render_apps_tab(app: &App, account: &AccountDetails, frame: &mut Frame, area: Rect) {
    // Split area for opted apps and created apps
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Opted header
            Constraint::Min(8),    // Opted list
            Constraint::Length(1), // Created header
            Constraint::Min(4),    // Created list
        ])
        .split(area);

    // Apps Opted In section
    let opted_header = Paragraph::new(format!(
        " Apps Opted In ({} total)",
        account.total_apps_opted_in
    ))
    .style(
        Style::default()
            .fg(SECONDARY_COLOR)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(opted_header, sections[0]);

    if account.apps_local_state.is_empty() {
        let empty_msg = Paragraph::new("  No apps opted into")
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Left);
        frame.render_widget(empty_msg, sections[1]);
    } else {
        let scroll_offset = app.nav.account_item_scroll as usize;
        let visible_height = sections[1].height as usize;

        let items: Vec<ListItem> = account
            .apps_local_state
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(i, app_state)| {
                let is_selected = app.nav.account_item_index == Some(i)
                    && app.nav.account_detail_tab == AccountDetailTab::Apps;
                let indicator = if is_selected { "▶" } else { " " };

                let style = if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::raw(format!("{} ", indicator)),
                    Span::styled(
                        format!("App #{}", app_state.app_id),
                        Style::default().fg(PRIMARY_COLOR),
                    ),
                    Span::raw(" - "),
                    Span::styled(
                        format!(
                            "{} uint, {} bytes",
                            app_state.schema_num_uint, app_state.schema_num_byte_slice
                        ),
                        Style::default().fg(MUTED_COLOR),
                    ),
                ]))
                .style(style)
            })
            .collect();

        let list = List::new(items).block(Block::default());
        frame.render_widget(list, sections[1]);
    }

    // Created Apps section
    let created_header = Paragraph::new(format!(
        " Created Apps ({} total)",
        account.total_created_apps
    ))
    .style(
        Style::default()
            .fg(SECONDARY_COLOR)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(created_header, sections[2]);

    if account.created_apps.is_empty() {
        let empty_msg = Paragraph::new("  No apps created")
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Left);
        frame.render_widget(empty_msg, sections[3]);
    } else {
        let items: Vec<ListItem> = account
            .created_apps
            .iter()
            .take(sections[3].height as usize)
            .map(|app_info| {
                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("App #{}", app_info.app_id),
                        Style::default().fg(PRIMARY_COLOR),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items).block(Block::default());
        frame.render_widget(list, sections[3]);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    use rstest::*;

    use crate::domain::account::{AccountAssetHolding, AccountDetails};
    use crate::test_utils::{AccountMother, mock_app, test_terminal};

    // ============================================================================
    // Fixtures
    // ============================================================================

    // Note: test_terminal, mock_app, and AccountMother are imported from crate::test_utils

    #[fixture]
    fn mock_account() -> AccountDetails {
        AccountDetails {
            address: "Y76M3MSY6DKBRHBL7C3NNDXGS5IIMQVQVUAB6MP4XEMMGVF2QWNPL226CA".to_string(),
            balance: 17_398_150_061_870, // ~17.4M ALGO
            min_balance: 100_000,
            pending_rewards: 150_061_870,
            rewards: 398_150_061_870,
            status: "Online".to_string(),
            total_apps_opted_in: 5,
            total_assets_opted_in: 42,
            total_created_apps: 3,
            total_created_assets: 10,
            assets: vec![
                AccountAssetHolding {
                    asset_id: 31566704,
                    amount: 1_000_000_000,
                    is_frozen: false,
                },
                AccountAssetHolding {
                    asset_id: 312769,
                    amount: 500_000_000,
                    is_frozen: true,
                },
            ],
            nfd: None,
            ..Default::default()
        }
    }

    // ============================================================================
    // Snapshot Tests
    // ============================================================================

    /// Snapshot test for account details popup - Info tab.
    ///
    /// Uses a mock account with various features (assets, apps, etc) to avoid flaky
    /// balance changes from live participating accounts.
    #[rstest]
    #[tokio::test]
    async fn test_account_details_snapshot(
        mut test_terminal: Terminal<TestBackend>,
        #[future] mock_app: App,
        mock_account: AccountDetails,
    ) {
        let mut app = mock_app.await;
        app.data.viewed_account = Some(mock_account);
        app.nav.show_account_details = true;
        app.nav.account_detail_tab = AccountDetailTab::Info;

        test_terminal
            .draw(|frame| {
                render_account_details(&app, frame, frame.area());
            })
            .unwrap();

        insta::assert_snapshot!("account_details", test_terminal.backend());
    }

    /// Snapshot test for account with NFD name.
    ///
    /// Tests that NFD names are displayed prominently with verification badge.
    /// Uses a static fixture (silvio.algo-style) to avoid network calls.
    #[rstest]
    #[tokio::test]
    async fn test_account_details_with_nfd_snapshot(
        mut test_terminal: Terminal<TestBackend>,
        #[future] mock_app: App,
    ) {
        let mut app = mock_app.await;
        app.data.viewed_account = Some(AccountMother::with_nfd());
        app.nav.show_account_details = true;
        app.nav.account_detail_tab = AccountDetailTab::Info;

        test_terminal
            .draw(|frame| {
                render_account_details(&app, frame, frame.area());
            })
            .unwrap();

        insta::assert_snapshot!("account_details_with_nfd", test_terminal.backend());
    }
}
