//! Application detail panel rendering.
//!
//! This module handles the display of detailed application information including
//! app ID, creator, state schemas, global state, and program info.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};

use crate::domain::application::ApplicationDetails;
use crate::state::{App, AppDetailTab};
use crate::theme::{
    ACCENT_COLOR, MUTED_COLOR, PRIMARY_COLOR, SECONDARY_COLOR, SUCCESS_COLOR, WARNING_COLOR,
};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::{centered_popup_area, fullscreen_popup_area};

/// Renders the application details popup with tabbed interface.
///
/// Supports tabbed navigation between Info, State, and Programs views.
///
/// # Arguments
///
/// * `app` - Application state containing application data
/// * `frame` - Ratatui frame for rendering
/// * `area` - Available screen area for rendering
pub fn render_application_details(app: &App, frame: &mut Frame, area: Rect) {
    let Some(application) = &app.data.viewed_application else {
        // Still loading or no data
        let popup_area = centered_popup_area(area, 50, 10);
        let popup_block = create_popup_block("Application Details");
        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup_block.clone(), popup_area);

        let inner_area = popup_block.inner(popup_area);
        let loading = Paragraph::new("Loading application details...")
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
    let popup_block = create_popup_block("Application Details");
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
    match app.nav.app_detail_tab {
        AppDetailTab::Info => render_info_tab(application, frame, content_area),
        AppDetailTab::State => render_state_tab(app, application, frame, content_area),
        AppDetailTab::Programs => render_programs_tab(application, frame, content_area),
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

/// Renders the tab bar for application details.
fn render_tab_bar(app: &App, frame: &mut Frame, area: Rect) {
    let current_tab = app.nav.app_detail_tab;

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
        Span::styled(" Info ", tab_style(current_tab == AppDetailTab::Info)),
        Span::raw("  "),
        Span::styled(" State ", tab_style(current_tab == AppDetailTab::State)),
        Span::raw("  "),
        Span::styled(
            " Programs ",
            tab_style(current_tab == AppDetailTab::Programs),
        ),
    ]);
    frame.render_widget(Paragraph::new(tab_bar), area);
}

/// Renders the Info tab with general application information.
fn render_info_tab(application: &ApplicationDetails, frame: &mut Frame, area: Rect) {
    let mut rows = vec![];

    // App ID
    rows.push(Row::new(vec![
        Cell::from("App ID:").style(
            Style::default()
                .fg(WARNING_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(format!("{}", application.app_id)).style(
            Style::default()
                .fg(ACCENT_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Status
    let status = if application.deleted {
        "Deleted"
    } else {
        "Active"
    };
    let status_color = if application.deleted {
        Color::Red
    } else {
        SUCCESS_COLOR
    };
    rows.push(Row::new(vec![
        Cell::from("Status:").style(
            Style::default()
                .fg(WARNING_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(status).style(Style::default().fg(status_color)),
    ]));

    rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer

    // Creator address (truncated if needed)
    let creator_display = if application.creator.len() > 50 {
        format!("{}...", &application.creator[..47])
    } else {
        application.creator.clone()
    };
    rows.push(Row::new(vec![
        Cell::from("Creator:").style(
            Style::default()
                .fg(WARNING_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(creator_display).style(Style::default().fg(SECONDARY_COLOR)),
    ]));

    // App address (truncated if needed)
    let app_address_display = if application.app_address.len() > 50 {
        format!("{}...", &application.app_address[..47])
    } else {
        application.app_address.clone()
    };
    rows.push(Row::new(vec![
        Cell::from("App Address:").style(
            Style::default()
                .fg(WARNING_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(app_address_display).style(Style::default().fg(SECONDARY_COLOR)),
    ]));

    rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer

    // Global State Schema
    rows.push(Row::new(vec![
        Cell::from("Global State:").style(
            Style::default()
                .fg(WARNING_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("").style(Style::default()),
    ]));
    rows.push(Row::new(vec![
        Cell::from("  Byte Slices:").style(Style::default().fg(MUTED_COLOR)),
        Cell::from(format!("{}", application.global_state_byte))
            .style(Style::default().fg(Color::White)),
    ]));
    rows.push(Row::new(vec![
        Cell::from("  Uint64s:").style(Style::default().fg(MUTED_COLOR)),
        Cell::from(format!("{}", application.global_state_uint))
            .style(Style::default().fg(Color::White)),
    ]));

    rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer

    // Local State Schema
    rows.push(Row::new(vec![
        Cell::from("Local State:").style(
            Style::default()
                .fg(WARNING_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("").style(Style::default()),
    ]));
    rows.push(Row::new(vec![
        Cell::from("  Byte Slices:").style(Style::default().fg(MUTED_COLOR)),
        Cell::from(format!("{}", application.local_state_byte))
            .style(Style::default().fg(Color::White)),
    ]));
    rows.push(Row::new(vec![
        Cell::from("  Uint64s:").style(Style::default().fg(MUTED_COLOR)),
        Cell::from(format!("{}", application.local_state_uint))
            .style(Style::default().fg(Color::White)),
    ]));

    // Extra program pages if any
    if let Some(extra_pages) = application.extra_program_pages {
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
        rows.push(Row::new(vec![
            Cell::from("Extra Pages:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", extra_pages)).style(Style::default().fg(Color::White)),
        ]));
    }

    // Created at round if available
    if let Some(round) = application.created_at_round {
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer
        rows.push(Row::new(vec![
            Cell::from("Created At:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("Round {}", round)).style(Style::default().fg(MUTED_COLOR)),
        ]));
    }

    let table = Table::new(rows, [Constraint::Length(20), Constraint::Min(50)])
        .block(Block::default())
        .column_spacing(2);

    frame.render_widget(table, area);
}

/// Renders the State tab with global state key-value pairs.
fn render_state_tab(app: &App, application: &ApplicationDetails, frame: &mut Frame, area: Rect) {
    // Section header
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(4),    // State list
        ])
        .split(area);

    let header = Paragraph::new(format!(
        " Global State ({} entries)",
        application.global_state.len()
    ))
    .style(
        Style::default()
            .fg(ACCENT_COLOR)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(header, sections[0]);

    if application.global_state.is_empty() {
        let empty_msg = Paragraph::new("  No global state")
            .style(Style::default().fg(MUTED_COLOR))
            .alignment(Alignment::Left);
        frame.render_widget(empty_msg, sections[1]);
    } else {
        let scroll_offset = app.nav.app_state_scroll as usize;
        let visible_height = sections[1].height as usize;

        let items: Vec<ListItem> = application
            .global_state
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(i, state)| {
                let is_selected = app.nav.app_state_index == Some(i);
                let indicator = if is_selected { "▶" } else { " " };

                let style = if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                // Type indicator
                let type_indicator = match state.value_type.as_str() {
                    "Bytes" => "[]",
                    "Uint" => "#",
                    _ => "?",
                };

                // Truncate value for display
                let value_display = if state.value.len() > 40 {
                    format!("{}...", &state.value[..37])
                } else {
                    state.value.clone()
                };

                ListItem::new(Line::from(vec![
                    Span::raw(format!("{} ", indicator)),
                    Span::styled(type_indicator, Style::default().fg(MUTED_COLOR)),
                    Span::raw(" "),
                    Span::styled(&state.key, Style::default().fg(SECONDARY_COLOR)),
                    Span::raw(" = "),
                    Span::styled(value_display, Style::default().fg(Color::White)),
                ]))
                .style(style)
            })
            .collect();

        let list = List::new(items).block(Block::default());
        frame.render_widget(list, sections[1]);
    }
}

/// Renders the Programs tab with approval and clear state program info.
fn render_programs_tab(application: &ApplicationDetails, frame: &mut Frame, area: Rect) {
    let mut rows = vec![];

    // Approval Program
    rows.push(Row::new(vec![
        Cell::from("Approval Program:").style(
            Style::default()
                .fg(WARNING_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("").style(Style::default()),
    ]));

    if let Some(ref program) = application.approval_program {
        let program_len = program.len();
        let size_display = format_program_size(program_len);
        rows.push(Row::new(vec![
            Cell::from("  Size:").style(Style::default().fg(MUTED_COLOR)),
            Cell::from(size_display).style(Style::default().fg(SUCCESS_COLOR)),
        ]));

        // Show hash-like preview (first/last bytes)
        let preview = format_program_preview(program);
        rows.push(Row::new(vec![
            Cell::from("  Preview:").style(Style::default().fg(MUTED_COLOR)),
            Cell::from(preview).style(Style::default().fg(Color::White)),
        ]));
    } else {
        rows.push(Row::new(vec![
            Cell::from("  ").style(Style::default()),
            Cell::from("(not available)").style(Style::default().fg(MUTED_COLOR)),
        ]));
    }

    rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer

    // Clear State Program
    rows.push(Row::new(vec![
        Cell::from("Clear State Program:").style(
            Style::default()
                .fg(WARNING_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("").style(Style::default()),
    ]));

    if let Some(ref program) = application.clear_state_program {
        let program_len = program.len();
        let size_display = format_program_size(program_len);
        rows.push(Row::new(vec![
            Cell::from("  Size:").style(Style::default().fg(MUTED_COLOR)),
            Cell::from(size_display).style(Style::default().fg(SUCCESS_COLOR)),
        ]));

        // Show hash-like preview (first/last bytes)
        let preview = format_program_preview(program);
        rows.push(Row::new(vec![
            Cell::from("  Preview:").style(Style::default().fg(MUTED_COLOR)),
            Cell::from(preview).style(Style::default().fg(Color::White)),
        ]));
    } else {
        rows.push(Row::new(vec![
            Cell::from("  ").style(Style::default()),
            Cell::from("(not available)").style(Style::default().fg(MUTED_COLOR)),
        ]));
    }

    rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacer

    // Extra pages info
    if let Some(extra_pages) = application.extra_program_pages {
        rows.push(Row::new(vec![
            Cell::from("Extra Program Pages:").style(
                Style::default()
                    .fg(WARNING_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{}", extra_pages)).style(Style::default().fg(Color::White)),
        ]));
    }

    let table = Table::new(rows, [Constraint::Length(22), Constraint::Min(50)])
        .block(Block::default())
        .column_spacing(2);

    frame.render_widget(table, area);
}

/// Formats the program size in a human-readable way.
fn format_program_size(len: usize) -> String {
    // Base64 encoded, so actual bytes is ~75% of this
    let approx_bytes = (len * 3) / 4;
    if approx_bytes >= 1024 {
        format!("~{:.1} KB ({} chars)", approx_bytes as f64 / 1024.0, len)
    } else {
        format!("~{} bytes ({} chars)", approx_bytes, len)
    }
}

/// Formats a program preview (first and last bytes in hex-like display).
fn format_program_preview(program: &str) -> String {
    if program.len() <= 20 {
        return program.to_string();
    }
    format!("{}...{}", &program[..8], &program[program.len() - 8..])
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::application::{AppStateValue, ApplicationDetails};
    use crate::test_utils::{mock_app, test_terminal};
    use ratatui::{Terminal, backend::TestBackend};
    use rstest::*;

    // ============================================================================
    // Fixtures
    // ============================================================================

    // Note: test_terminal and mock_app are imported from crate::test_utils

    #[fixture]
    fn mock_application() -> ApplicationDetails {
        ApplicationDetails {
            app_id: 1234567890,
            creator: "Y76M3MSY6DKBRHBL7C3NNDXGS5IIMQVQVUAB6MP4XEMMGVF2QWNPL226CA".to_string(),
            app_address: "APPADDRESSXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string(),
            deleted: false,
            global_state_byte: 10,
            global_state_uint: 5,
            local_state_byte: 2,
            local_state_uint: 3,
            extra_program_pages: Some(1),
            approval_program: Some(
                "BCACAQAmAQAxGCISQAAoMRkjEkAAGDEZIxJAAAgxGSQSQAABADEbQQAKIjUBQQAGMQA0ARJA"
                    .to_string(),
            ),
            clear_state_program: Some("BIEB".to_string()),
            global_state: vec![
                AppStateValue {
                    key: "fee_collector".to_string(),
                    value_type: "Bytes".to_string(),
                    value: "Y76M3MSY6DKBRHBL7C3NNDXGS5II...".to_string(),
                },
                AppStateValue {
                    key: "total_supply".to_string(),
                    value_type: "Uint".to_string(),
                    value: "1000000000".to_string(),
                },
                AppStateValue {
                    key: "paused".to_string(),
                    value_type: "Uint".to_string(),
                    value: "0".to_string(),
                },
            ],
            created_at_round: Some(30000000),
        }
    }

    // ============================================================================
    // Snapshot Tests
    // ============================================================================

    /// Snapshot test for application details popup - Info tab.
    #[rstest]
    #[tokio::test]
    async fn test_application_details_info_tab(
        mut test_terminal: Terminal<TestBackend>,
        #[future] mock_app: App,
        mock_application: ApplicationDetails,
    ) {
        let mut app = mock_app.await;
        app.data.viewed_application = Some(mock_application);
        app.nav.show_application_details = true;
        app.nav.app_detail_tab = AppDetailTab::Info;

        test_terminal
            .draw(|frame| {
                render_application_details(&app, frame, frame.area());
            })
            .unwrap();

        insta::assert_snapshot!("application_details_info_tab", test_terminal.backend());
    }

    /// Snapshot test for application details popup - State tab.
    #[rstest]
    #[tokio::test]
    async fn test_application_details_state_tab(
        mut test_terminal: Terminal<TestBackend>,
        #[future] mock_app: App,
        mock_application: ApplicationDetails,
    ) {
        let mut app = mock_app.await;
        app.data.viewed_application = Some(mock_application);
        app.nav.show_application_details = true;
        app.nav.app_detail_tab = AppDetailTab::State;

        test_terminal
            .draw(|frame| {
                render_application_details(&app, frame, frame.area());
            })
            .unwrap();

        insta::assert_snapshot!("application_details_state_tab", test_terminal.backend());
    }

    /// Snapshot test for application details popup - Programs tab.
    #[rstest]
    #[tokio::test]
    async fn test_application_details_programs_tab(
        mut test_terminal: Terminal<TestBackend>,
        #[future] mock_app: App,
        mock_application: ApplicationDetails,
    ) {
        let mut app = mock_app.await;
        app.data.viewed_application = Some(mock_application);
        app.nav.show_application_details = true;
        app.nav.app_detail_tab = AppDetailTab::Programs;

        test_terminal
            .draw(|frame| {
                render_application_details(&app, frame, frame.area());
            })
            .unwrap();

        insta::assert_snapshot!("application_details_programs_tab", test_terminal.backend());
    }
}
