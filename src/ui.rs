use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    widgets::{Block, Borders, Paragraph},
};

// Components module is declared in main.rs
// No need for `mod components;` here

use crate::{
    app::{App, PopupState}, // Use crate::app
    components::{
        // Use crate::components::* paths
        details::{render_block_details, render_transaction_details},
        main_layout::{render_blocks, render_header, render_transactions},
        popups::{
            render_add_custom_network_popup, render_message_popup, render_network_selector,
            render_search_results, render_search_with_type_popup,
        },
    },
    constants::{FOOTER_HEIGHT, HEADER_HEIGHT, TITLE_HEIGHT}, // Use constants module
};

// Remove unused constants
// const BLOCK_HEIGHT: u16 = 3;
// const TXN_HEIGHT: u16 = 4;

/// Main rendering function for the entire application UI.
pub fn render(app: &mut App, frame: &mut Frame) {
    let size = frame.area();

    // Main layout: Header, Content, Footer
    let main_chunks = Layout::default()
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(0), // Content area takes remaining space
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(size);

    let header_area = main_chunks[0];
    let content_area = main_chunks[1];
    let footer_area = main_chunks[2];

    render_header(app, frame, header_area);
    render_main_content(app, frame, content_area);
    crate::components::main_layout::render_footer(app, frame, footer_area);

    // Render popups or details over the main content if necessary
    match &app.popup_state {
        PopupState::NetworkSelect {
            available_networks,
            selected_index,
        } => {
            render_network_selector(frame, size, available_networks, *selected_index);
        }
        PopupState::AddCustomNetwork(state) => {
            // Pass the state struct directly
            render_add_custom_network_popup(frame, size, state);
        }
        PopupState::SearchWithType { query, search_type } => {
            // Destructure here
            render_search_with_type_popup(frame, size, query, *search_type);
        }
        PopupState::Message(message) => {
            render_message_popup(frame, size, message);
        }
        PopupState::SearchResults(state) => {
            // Pass the state struct directly
            render_search_results(frame, size, state);
        }
        PopupState::None => {
            // Only render details if no popup is active
            if app.show_block_details {
                render_block_details(app, frame, size);
            } else if app.show_transaction_details {
                render_transaction_details(app, frame, size);
            }
        }
    }
}

/// Renders the central content area (Title, Blocks, Transactions).
fn render_main_content(app: &mut App, frame: &mut Frame, area: Rect) {
    // Layout for Title bar and Content panes
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(TITLE_HEIGHT),
            Constraint::Min(0), // Panes area takes remaining space
        ])
        .split(area);

    let title_area = content_chunks[0];
    let panes_area = content_chunks[1];

    // Render Title Bar
    render_title_bar(app, frame, title_area);

    // Layout for Blocks and Transactions panes
    let panes_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 2), // Left pane (Blocks)
            Constraint::Ratio(1, 2), // Right pane (Transactions)
        ])
        .split(panes_area);

    // Render Blocks and Transactions panes (delegated to components::main_layout)
    render_blocks(app, frame, panes_chunks[0]);
    render_transactions(app, frame, panes_chunks[1]);
}

/// Renders the title bar section below the main header.
fn render_title_bar(app: &App, frame: &mut Frame, area: Rect) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(title_block.clone(), area);

    // Render "Explore" title
    let title = Paragraph::new("Explore").style(Style::default().add_modifier(Modifier::BOLD));
    // Position title inside the border
    let title_inner_area = Rect::new(
        area.x + 2,
        area.y + 1,
        10.min(area.width.saturating_sub(4)),
        1,
    );
    frame.render_widget(title, title_inner_area);

    // Render "Show live" checkbox
    // Use try_lock to avoid blocking, falling back to a default if lock fails
    let show_live = app.show_live.try_lock().map(|guard| *guard).unwrap_or(true);
    let checkbox_text = format!("[{}] Show live", if show_live { "✓" } else { " " });
    let checkbox = Paragraph::new(checkbox_text.clone()).style(Style::default().fg(if show_live {
        Color::Green
    } else {
        Color::Gray
    }));

    // Position checkbox towards the right inside the border
    let checkbox_width = checkbox_text.len() as u16;
    let checkbox_area = Rect::new(
        area.right()
            .saturating_sub(checkbox_width + 2)
            .max(area.x + 1),
        area.y + 1,
        checkbox_width.min(area.width.saturating_sub(2)),
        1,
    );
    frame.render_widget(checkbox, checkbox_area);
}
