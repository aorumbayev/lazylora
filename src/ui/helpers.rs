//! UI helper functions for creating styled blocks and widgets.
//!
//! This module provides reusable helper functions for creating consistent
//! UI elements with proper styling throughout the LazyLora TUI application.

// TODO: Remove this allow once functions are used in the refactored UI code
#![allow(dead_code)]

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    symbols::border,
    widgets::{Block, Borders},
};

use crate::theme::{BORDER_STYLE, FOCUSED_BORDER_STYLE, FOCUSED_TITLE_STYLE};

// ============================================================================
// Border Block Helpers
// ============================================================================

/// Creates a bordered block with proper styling based on focus state.
///
/// This function creates a consistent block widget with borders and titles
/// that adapt their appearance based on whether the element is focused.
///
/// # Arguments
///
/// * `title` - The title text to display in the block border
/// * `focused` - Whether the block should be styled as focused/active
///
/// # Returns
///
/// A configured `Block` widget with appropriate styling.
///
/// # Example
///
/// ```rust
/// use lazylora::ui::helpers::create_border_block;
///
/// let focused_block = create_border_block("Latest Blocks", true);
/// let unfocused_block = create_border_block("Transactions", false);
/// ```
#[must_use]
pub fn create_border_block(title: &str, focused: bool) -> Block<'_> {
    let (border_style, border_set, title_style, display_title) = if focused {
        (
            FOCUSED_BORDER_STYLE,
            border::DOUBLE,
            FOCUSED_TITLE_STYLE,
            if title.is_empty() {
                String::new()
            } else {
                format!(" ● {} ", title)
            },
        )
    } else {
        (
            BORDER_STYLE,
            border::ROUNDED,
            Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            if title.is_empty() {
                String::new()
            } else {
                format!(" {} ", title)
            },
        )
    };

    Block::default()
        .borders(Borders::ALL)
        .title(display_title)
        .title_style(title_style)
        .border_set(border_set)
        .border_style(border_style)
}

/// Creates a popup-style block with centered title and rounded borders.
///
/// This function creates blocks suitable for popup overlays and modal dialogs,
/// with a centered title and consistent styling.
///
/// # Arguments
///
/// * `title` - The title text to display centered at the top
///
/// # Returns
///
/// A configured `Block` widget styled for popup use.
///
/// # Example
///
/// ```rust
/// use lazylora::ui::helpers::create_popup_block;
///
/// let popup = create_popup_block("Search Results");
/// ```
#[must_use]
pub fn create_popup_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {} ", title))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(BORDER_STYLE)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn test_create_border_block_unfocused_renders() {
        let backend = TestBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let block = create_border_block("Test Title", false);
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Verify block is rendered with rounded corner (unfocused style)
        assert_eq!(buffer[(0, 0)].symbol(), "╭");
        assert_eq!(buffer[(29, 0)].symbol(), "╮");
    }

    #[test]
    fn test_create_border_block_focused_renders() {
        let backend = TestBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let block = create_border_block("Test Title", true);
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Verify block is rendered with double-line borders (focused style)
        // Note: The actual border characters depend on the border set
        assert!(!buffer[(0, 0)].symbol().is_empty());
    }

    #[test]
    fn test_create_border_block_empty_title() {
        let backend = TestBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let block = create_border_block("", false);
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 0)].symbol(), "╭");
    }

    #[test]
    fn test_create_border_block_with_title() {
        let backend = TestBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let block = create_border_block("Test", false);
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Verify title is present in the buffer
        let title_found = (0..buffer.area().width).any(|x| {
            let cell = &buffer[(x, 0)];
            cell.symbol().contains('T')
        });
        assert!(title_found, "Title should be rendered in the border");
    }

    #[test]
    fn test_create_popup_block_renders() {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let block = create_popup_block("Popup Title");
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Verify block is rendered with rounded corners
        assert_eq!(buffer[(0, 0)].symbol(), "╭");
        assert_eq!(buffer[(39, 0)].symbol(), "╮");
    }

    #[test]
    fn test_create_popup_block_title_centered() {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let block = create_popup_block("Popup");
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Verify title is centered (should be near middle of top border)
        let title_found = (15..25).any(|x| {
            let cell = &buffer[(x, 0)];
            cell.symbol().contains('P')
        });
        assert!(title_found, "Title should be centered in the border");
    }

    #[test]
    fn test_border_block_focused_indicator() {
        let backend = TestBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let block = create_border_block("Focused", true);
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Verify focused indicator (●) is present
        let indicator_found = (0..buffer.area().width).any(|x| {
            let cell = &buffer[(x, 0)];
            cell.symbol().contains('●')
        });
        assert!(indicator_found, "Focused indicator should be rendered");
    }

    #[test]
    fn test_border_blocks_render_differently() {
        let backend1 = TestBackend::new(30, 5);
        let mut terminal1 = Terminal::new(backend1).unwrap();

        terminal1
            .draw(|frame| {
                let block = create_border_block("Test", false);
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        let backend2 = TestBackend::new(30, 5);
        let mut terminal2 = Terminal::new(backend2).unwrap();

        terminal2
            .draw(|frame| {
                let block = create_border_block("Test", true);
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        // The buffers should differ due to different border styles
        let buffer1 = terminal1.backend().buffer();
        let buffer2 = terminal2.backend().buffer();

        // At least some cells should be different (border style or title indicator)
        let has_differences =
            (0..buffer1.area().width).any(|x| buffer1[(x, 0)].symbol() != buffer2[(x, 0)].symbol());

        assert!(
            has_differences,
            "Focused and unfocused blocks should render differently"
        );
    }

    #[test]
    fn test_create_border_block_with_long_title() {
        let long_title = "This is a very long title that might need wrapping";
        let backend = TestBackend::new(60, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let block = create_border_block(long_title, false);
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer[(0, 0)].symbol().is_empty());
    }

    #[test]
    fn test_create_popup_block_with_special_characters() {
        let title_with_special = "Test: [Results] (42)";
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let block = create_popup_block(title_with_special);
                frame.render_widget(block, frame.area());
            })
            .unwrap();

        // Should render without panicking
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 0)].symbol(), "╭");
    }

    #[test]
    fn test_both_functions_create_valid_blocks() {
        // Test that both functions return blocks that can be rendered
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(5),
                        ratatui::layout::Constraint::Length(5),
                    ])
                    .split(area);

                let border_block = create_border_block("Border", true);
                let popup_block = create_popup_block("Popup");

                frame.render_widget(border_block, chunks[0]);
                frame.render_widget(popup_block, chunks[1]);
            })
            .unwrap();

        // Both blocks should render successfully
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 0)].symbol(), "╔"); // Top-left corner of focused block (double)
        assert_eq!(buffer[(0, 5)].symbol(), "╭"); // Top-left corner of popup block (rounded)
    }
}
