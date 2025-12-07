//! Message popup rendering.
//!
//! This module provides a generic message popup for displaying informational
//! messages, warnings, or errors to the user.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Clear, Paragraph, Wrap},
};

use crate::theme::MUTED_COLOR;
use crate::ui::helpers::create_popup_block;
use crate::ui::layout::centered_popup_area;

// ============================================================================
// Public API
// ============================================================================

/// Renders a message popup with auto-sized dimensions.
///
/// Displays a centered modal popup with the given message. The popup
/// automatically sizes itself based on the message content, calculating
/// appropriate width and height to fit the text.
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render to
/// * `area` - The terminal area to render within
/// * `message` - The message text to display (supports multi-line)
///
/// # Example
///
/// ```ignore
/// use lazylora::ui::popups::message;
///
/// message::render(&mut frame, area, "Operation completed successfully!");
/// ```
pub fn render(frame: &mut Frame, area: Rect, message: &str) {
    let message_lines = message.lines().count().max(1) as u16;
    let longest_line = message
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(message.chars().count()) as u16;

    let popup_width = 40.max(longest_line + 6).min(area.width * 8 / 10);
    let popup_height = 6.max(message_lines + 4);

    let popup_area = centered_popup_area(area, popup_width, popup_height);

    let popup_block = create_popup_block("Message");
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block.clone(), popup_area);

    let inner_area = popup_block.inner(popup_area);

    let message_area = Rect::new(
        inner_area.x,
        inner_area.y,
        inner_area.width,
        inner_area.height.saturating_sub(2), // Reserve space for help text
    );

    let prompt = Paragraph::new(message)
        .style(Style::default())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(prompt, message_area);

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

    let help_text = "Esc:Close  Enter:Close  q:Close";
    let help_area = Rect::new(
        popup_area.x,
        popup_area.y + popup_area.height - 2,
        popup_area.width,
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
    fn test_message_popup_renders() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area(), "Test message");
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_message_popup_multiline() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let message = "Line 1\nLine 2\nLine 3";

        terminal
            .draw(|frame| {
                render(frame, frame.area(), message);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_message_popup_long_text() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let long_message = "This is a very long message that should wrap properly when displayed in the popup. It should automatically size the popup to fit the content.";

        terminal
            .draw(|frame| {
                render(frame, frame.area(), long_message);
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_message_popup_empty_string() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render(frame, frame.area(), "");
            })
            .unwrap();

        // Should render without panicking even with empty message
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }

    #[test]
    fn test_message_popup_sizing() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test various message lengths
        let messages = vec![
            "Short",
            "Medium length message here",
            "A very long message that will test the width calculation and wrapping behavior of the popup renderer",
        ];

        for msg in messages {
            terminal
                .draw(|frame| {
                    render(frame, frame.area(), msg);
                })
                .unwrap();
        }

        // Should render all variants without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.area().is_empty());
    }
}
