//! UI helper functions for creating styled blocks and widgets.
//!
//! This module provides reusable helper functions for creating consistent
//! UI elements with proper styling throughout the LazyLora TUI application.

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
    use insta::assert_snapshot;
    use ratatui::{
        Terminal,
        backend::TestBackend,
        layout::{Constraint, Direction, Layout},
    };

    /// Consolidated test for all border block and popup block visual states.
    ///
    /// Per commandments: "One snapshot test > Twenty cell assertions"
    /// and "10+ tests for the same widget with minor data variations" is a smell.
    #[test]
    fn test_all_block_states() {
        let backend = TestBackend::new(50, 25);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let areas = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ])
                    .split(frame.area());

                // Border blocks - unfocused states
                frame.render_widget(create_border_block("Unfocused", false), areas[0]);
                frame.render_widget(create_border_block("", false), areas[1]);

                // Border blocks - focused states
                frame.render_widget(create_border_block("Focused", true), areas[2]);
                frame.render_widget(create_border_block("", true), areas[3]);

                // Popup blocks
                frame.render_widget(create_popup_block("Popup Title"), areas[4]);
                frame.render_widget(create_popup_block("Short"), areas[5]);
                frame.render_widget(create_popup_block("Test: [Results] (42)"), areas[6]);
            })
            .unwrap();

        assert_snapshot!(terminal.backend());
    }
}
