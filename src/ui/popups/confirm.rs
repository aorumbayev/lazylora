//! Quit confirmation popup rendering.
//!
//! This module provides a simple confirmation popup asking the user
//! if they want to quit the application.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Clear, Paragraph},
};

use crate::theme::{MUTED_COLOR, PRIMARY_COLOR};
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::centered_popup_area;

// ============================================================================
// Public API
// ============================================================================

/// Renders the quit confirmation popup.
///
/// Displays a centered modal popup asking "Are you sure you want to close lazy lora?"
/// with y/n options.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render to
/// * `area` - The terminal area to render within
pub fn render(frame: &mut Frame, area: Rect) {
    let popup_width = 50;
    let popup_height = 7;

    let popup_area = centered_popup_area(area, popup_width, popup_height);

    let popup_block = create_popup_block("Confirm Quit");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    // Message
    let message = "Are you sure you want to close lazy lora?";
    let message_area = Rect::new(inner_area.x, inner_area.y + 1, inner_area.width, 1);

    let message_widget = Paragraph::new(message)
        .style(Style::default())
        .alignment(Alignment::Center);

    frame.render_widget(message_widget, message_area);

    // Separator
    let separator = "─".repeat(popup_area.width.saturating_sub(2) as usize);
    let separator_area = Rect::new(
        popup_area.x + 1,
        popup_area.y + popup_area.height - 3,
        popup_area.width - 2,
        1,
    );

    let separator_widget = Paragraph::new(separator)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(separator_widget, separator_area);

    // Help text with highlighted keys
    let help_area = Rect::new(
        popup_area.x,
        popup_area.y + popup_area.height - 2,
        popup_area.width,
        1,
    );

    let help_text = ratatui::text::Line::from(vec![
        ratatui::text::Span::styled(
            "y",
            Style::default()
                .fg(PRIMARY_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        ratatui::text::Span::styled(":Yes  ", Style::default().fg(MUTED_COLOR)),
        ratatui::text::Span::styled(
            "n",
            Style::default()
                .fg(PRIMARY_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        ratatui::text::Span::styled("/", Style::default().fg(MUTED_COLOR)),
        ratatui::text::Span::styled(
            "Esc",
            Style::default()
                .fg(PRIMARY_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        ratatui::text::Span::styled(":No", Style::default().fg(MUTED_COLOR)),
    ]);

    let help_msg = Paragraph::new(help_text).alignment(Alignment::Center);

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
    fn test_confirm_popup_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area());
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_confirm_popup_small_terminal() {
        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area());
            })
            .unwrap();

        // Should render without panicking even on smaller terminal
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }
}
