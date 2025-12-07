//! Asset detail panel rendering.
//!
//! This module handles the display of detailed asset information including
//! supply, decimals, management addresses, and metadata.

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
use crate::ui::layout::{centered_popup_area, fullscreen_popup_area};

/// Renders the asset details popup.
///
/// Displays ASA metadata, supply, and management addresses.
///
/// # Arguments
///
/// * `app` - Application state containing asset data
/// * `frame` - Ratatui frame for rendering
/// * `area` - Available screen area for rendering
pub fn render_asset_details(app: &App, frame: &mut Frame, area: Rect) {
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

    let popup_area = if app.ui.detail_fullscreen {
        fullscreen_popup_area(area)
    } else {
        centered_popup_area(area, 85, 30)
    };
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
    let help_text = "[C] Copy  [Y] JSON  [O] Open  [Esc] Close";
    frame.render_widget(
        Paragraph::new(help_text)
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Center),
        content_layout[1],
    );
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    use rstest::*;

    use crate::test_utils::{AssetMother, mock_app, test_terminal};

    // ============================================================================
    // Snapshot Tests
    // ============================================================================

    /// Snapshot test for USDC asset details popup.
    ///
    /// Uses static USDC fixture (Asset ID 31566704) - a well-known ASA with all fields populated.
    #[rstest]
    #[tokio::test]
    async fn test_asset_details_usdc(
        mut test_terminal: Terminal<TestBackend>,
        #[future] mock_app: App,
    ) {
        let mut app = mock_app.await;
        app.data.viewed_asset = Some(AssetMother::usdc());
        app.nav.show_asset_details = true;

        test_terminal
            .draw(|frame| {
                render_asset_details(&app, frame, frame.area());
            })
            .unwrap();

        insta::assert_snapshot!("asset_details_usdc", test_terminal.backend());
    }

    /// Snapshot test for goUSD asset details popup.
    ///
    /// Uses static goUSD fixture (Asset ID 672913181) - tests asset with metadata hash.
    #[rstest]
    #[tokio::test]
    async fn test_asset_details_gousd(
        mut test_terminal: Terminal<TestBackend>,
        #[future] mock_app: App,
    ) {
        let mut app = mock_app.await;
        app.data.viewed_asset = Some(AssetMother::gousd());
        app.nav.show_asset_details = true;

        test_terminal
            .draw(|frame| {
                render_asset_details(&app, frame, frame.area());
            })
            .unwrap();

        insta::assert_snapshot!("asset_details_gousd", test_terminal.backend());
    }
}
