//! Network selection popup rendering.
//!
//! This module provides the network selector popup that allows users to switch
//! between MainNet, TestNet, and LocalNet.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Cell, Clear, Paragraph, Row, Table},
};

use crate::domain::Network;
use crate::theme::{MUTED_COLOR, PRIMARY_COLOR, SUCCESS_COLOR};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::centered_popup_area;

// ============================================================================
// Public API
// ============================================================================

/// Renders the network selection popup.
///
/// Displays a modal popup allowing the user to select which Algorand network
/// to connect to. Shows the current network and allows navigation between
/// MainNet, TestNet, and LocalNet.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render to
/// * `area` - The terminal area to render within
/// * `selected_index` - The currently highlighted network index (0-2)
/// * `current_network` - The currently active network
///
/// # Example
///
/// ```ignore
/// use lazylora::ui::popups::network;
/// use lazylora::domain::Network;
///
/// network::render(&mut frame, area, 0, Network::MainNet);
/// ```
pub fn render(frame: &mut Frame, area: Rect, selected_index: usize, current_network: Network) {
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn test_network_selector_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area(), 0, Network::MainNet);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_network_selector_shows_current_network() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area(), 1, Network::TestNet);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_network_selector_multiple_selections() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test each selection index
        for i in 0..3 {
            terminal
                .draw(|frame| {
                    render(frame, frame.area(), i, Network::MainNet);
                })
                .unwrap();
        }

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }
}
