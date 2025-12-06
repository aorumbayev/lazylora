//! UI rendering module for the LazyLora TUI.
//!
//! This module provides the main rendering entry point and orchestrates
//! rendering of all UI components including panels, popups, and overlays.
//!
//! # Module Structure
//!
//! - `panels` - Main content panels (blocks, transactions, details)
//! - `popups` - Modal dialogs (network selector, search, messages)
//! - `components` - Reusable UI components (toast notifications)
//! - `layout` - Layout calculations and structs
//! - `header` - Header bar rendering
//! - `footer` - Footer bar rendering
//! - `helpers` - Shared helper functions for creating styled blocks

pub mod components;
pub mod footer;
pub mod header;
pub mod helpers;
pub mod layout;
pub mod panels;
pub mod popups;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::state::{App, PopupState};

use layout::{HEADER_HEIGHT, SEARCH_BAR_HEIGHT};

// ============================================================================
// Main Render Entry Point
// ============================================================================

/// Main render function that orchestrates all UI rendering.
///
/// This function is the entry point for the UI layer and handles:
/// 1. Main layout (header, search bar, content, footer)
/// 2. Popup overlays based on current popup state
/// 3. Detail views when viewing specific items
/// 4. Toast notifications as non-blocking overlays
///
/// # Arguments
///
/// * `app` - The application state containing all data to render
/// * `frame` - The ratatui frame to render to
pub fn render(app: &App, frame: &mut Frame) {
    let size = frame.area();

    // Main layout: header, search bar, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Length(SEARCH_BAR_HEIGHT),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(size);

    // Render main UI structure
    header::render_header(frame, chunks[0], app);
    header::render_search_bar(frame, chunks[1], app);
    render_main_content(app, frame, chunks[2]);
    footer::render(frame, chunks[3], app);

    // Render popup overlays (if any)
    render_popups(app, frame, size);

    // Render detail views (if no popup active)
    if app.ui.popup_state == PopupState::None {
        render_detail_views(app, frame, size);
    }

    // Render toast notification on top of everything (non-blocking overlay)
    if let Some((message, _)) = &app.ui.toast {
        components::render_toast(frame, size, message);
    }
}

// ============================================================================
// Internal Rendering Functions
// ============================================================================

/// Render the main content area (blocks and transactions panels)
fn render_main_content(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    // Split content area for blocks and transactions (50/50)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    // Render block and transaction panels
    panels::render_blocks(app, frame, content_chunks[0]);
    panels::render_transactions(app, frame, content_chunks[1]);
}

/// Render popup overlays based on current popup state
fn render_popups(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    match &app.ui.popup_state {
        PopupState::NetworkSelect(selected_index) => {
            popups::network::render(frame, area, *selected_index, app.network);
        }
        PopupState::SearchWithType(query, search_type) => {
            popups::search::render(frame, area, query, *search_type);
        }
        PopupState::Message(message) => {
            popups::message::render(frame, area, message);
        }
        PopupState::SearchResults(results) => {
            popups::search_results::render(frame, area, results);
        }
        PopupState::None => {}
    }
}

/// Render detail views (block, transaction, account, asset details)
fn render_detail_views(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    if app.nav.show_block_details {
        panels::details::block::render_block_details(app, frame, area);
    } else if app.nav.show_transaction_details {
        panels::details::transaction::render_transaction_details(app, frame, area);
    } else if app.nav.show_account_details {
        panels::details::account::render_account_details(app, frame, area);
    } else if app.nav.show_asset_details {
        panels::details::asset::render_asset_details(app, frame, area);
    }
}
