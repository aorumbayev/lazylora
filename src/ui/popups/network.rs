//! Network selection popup rendering.
//!
//! This module provides the network selector popup that allows users to switch
//! between built-in networks (MainNet, TestNet, LocalNet) and custom user-defined networks.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Cell, Clear, Paragraph, Row, Table},
};

use crate::domain::NetworkConfig;
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
/// all available networks (built-in and custom).
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render to
/// * `area` - The terminal area to render within
/// * `selected_index` - The currently highlighted network index
/// * `current_network` - The currently active network configuration
/// * `networks` - All available networks (built-in + custom)
///
/// # Example
///
/// ```ignore
/// use lazylora::ui::popups::network;
/// use lazylora::domain::NetworkConfig;
///
/// let networks = vec![NetworkConfig::BuiltIn(Network::MainNet)];
/// network::render(&mut frame, area, 0, &networks[0], &networks);
/// ```
pub fn render(
    frame: &mut Frame,
    area: Rect,
    selected_index: usize,
    current_network: &NetworkConfig,
    networks: &[NetworkConfig],
) {
    // Dynamic height based on network count (min 10, max 20)
    let popup_height = (networks.len() as u16 + 6).clamp(10, 20);
    let popup_area = centered_popup_area(area, 40, popup_height);

    let popup_block = create_popup_block("Select Network");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    let rows: Vec<Row> = networks
        .iter()
        .enumerate()
        .map(|(i, net)| {
            let is_selected = i == selected_index;
            let is_current = net == current_network;

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

            // Mark custom networks with [Custom] suffix
            let network_name = net.as_str();
            let suffix = match net {
                NetworkConfig::Custom(_) => " [Custom]",
                NetworkConfig::BuiltIn(_) => "",
            };
            let network_text = if is_current {
                format!("{}{} (current)", network_name, suffix)
            } else {
                format!("{}{}", network_name, suffix)
            };

            Row::new(vec![
                Cell::from(indicator).style(style),
                Cell::from(network_text).style(style),
            ])
        })
        .collect();

    let table = Table::new(rows, [Constraint::Length(3), Constraint::Min(20)])
        .block(Block::default())
        .column_spacing(1);

    frame.render_widget(table, inner_area);

    // Build help text - show delete option only for custom networks
    let selected_is_custom = networks
        .get(selected_index)
        .map(|n| matches!(n, NetworkConfig::Custom(_)))
        .unwrap_or(false);

    let help_text = if selected_is_custom {
        "j/k:Move  Enter:Select  a:Add  d:Delete  Esc:Close"
    } else {
        "j/k:Move  Enter:Select  a:Add  Esc:Close"
    };

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
    use crate::domain::{CustomNetwork, Network};
    use ratatui::{Terminal, backend::TestBackend};

    fn builtin_networks() -> Vec<NetworkConfig> {
        vec![
            NetworkConfig::BuiltIn(Network::MainNet),
            NetworkConfig::BuiltIn(Network::TestNet),
            NetworkConfig::BuiltIn(Network::LocalNet),
        ]
    }

    #[test]
    fn test_network_selector_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let networks = builtin_networks();
        let current = &networks[0];

        terminal
            .draw(|frame| {
                render(frame, frame.area(), 0, current, &networks);
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
        let networks = builtin_networks();
        let current = &networks[1]; // TestNet

        terminal
            .draw(|frame| {
                render(frame, frame.area(), 1, current, &networks);
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
        let networks = builtin_networks();
        let current = &networks[0];

        // Test each selection index
        for i in 0..3 {
            terminal
                .draw(|frame| {
                    render(frame, frame.area(), i, current, &networks);
                })
                .unwrap();
        }

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_network_selector_with_custom_networks() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut networks = builtin_networks();
        networks.push(NetworkConfig::Custom(CustomNetwork::new(
            "MyCustomNet",
            "http://indexer.local",
            "http://algod.local",
        )));
        let current = &networks[0];

        terminal
            .draw(|frame| {
                render(frame, frame.area(), 3, current, &networks);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_network_selector_custom_network_is_current() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let custom = NetworkConfig::Custom(CustomNetwork::new(
            "MyCustomNet",
            "http://indexer.local",
            "http://algod.local",
        ));
        let mut networks = builtin_networks();
        networks.push(custom.clone());
        let current = &networks[3]; // Custom network is current

        terminal
            .draw(|frame| {
                render(frame, frame.area(), 3, current, &networks);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }
}
