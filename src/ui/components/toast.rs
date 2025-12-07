//! Toast notification component.
//!
//! Provides a non-blocking toast overlay that appears in the bottom-right corner
//! of the screen. Toast notifications automatically style themselves based on
//! message content (success, error, or info).

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    symbols::border,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::theme::{ERROR_COLOR, SUCCESS_COLOR};

// ============================================================================
// Constants
// ============================================================================

/// Minimum width for toast notifications.
const MIN_TOAST_WIDTH: u16 = 20;

/// Height of toast notifications.
const TOAST_HEIGHT: u16 = 3;

/// Horizontal padding from the right edge.
const TOAST_PADDING_RIGHT: u16 = 2;

/// Vertical padding from the bottom edge.
const TOAST_PADDING_BOTTOM: u16 = 2;

/// Extra padding added to message length for borders and spacing.
const TOAST_WIDTH_PADDING: u16 = 4;

// ============================================================================
// Public API
// ============================================================================

/// Renders a toast notification in the bottom-right corner.
///
/// This is a non-blocking overlay that doesn't prevent user interaction.
/// The toast automatically determines its color based on the message prefix:
/// - Messages starting with '[+]' use success color (green)
/// - Messages starting with '[x]' use error color (red)
/// - All other messages use white
///
/// # Arguments
///
/// * `frame` - The Ratatui frame to render into
/// * `area` - The area within which to position the toast
/// * `message` - The message text to display
///
/// # Example
///
/// ```rust,no_run
/// use ratatui::{Frame, layout::Rect};
/// use lazylora::ui::components::toast::render_toast;
///
/// fn render_ui(frame: &mut Frame, area: Rect) {
///     render_toast(frame, area, "[+] Operation successful");
/// }
/// ```
pub fn render_toast(frame: &mut Frame, area: Rect, message: &str) {
    let toast_area = calculate_toast_position(area, message);

    // Clear the area and draw a subtle bordered box
    frame.render_widget(Clear, toast_area);

    let toast_block = create_toast_block();
    frame.render_widget(toast_block.clone(), toast_area);

    let inner_area = toast_block.inner(toast_area);
    let text_color = determine_text_color(message);

    let toast_text = Paragraph::new(message)
        .style(Style::default().fg(text_color))
        .alignment(Alignment::Center);

    frame.render_widget(toast_text, inner_area);
}

// ============================================================================
// Internal Helpers
// ============================================================================

/// Calculates the position and dimensions for the toast notification.
///
/// Positions the toast in the bottom-right corner with appropriate padding.
///
/// # Arguments
///
/// * `area` - The containing area
/// * `message` - The message text to determine width
///
/// # Returns
///
/// A `Rect` defining the toast position and size.
#[must_use]
fn calculate_toast_position(area: Rect, message: &str) -> Rect {
    let message_len = message.chars().count() as u16;
    let toast_width = (message_len + TOAST_WIDTH_PADDING)
        .min(area.width / 2)
        .max(MIN_TOAST_WIDTH);

    // Position in bottom-right corner with padding
    let toast_x = area.x + area.width.saturating_sub(toast_width + TOAST_PADDING_RIGHT);
    let toast_y = area.y
        + area
            .height
            .saturating_sub(TOAST_HEIGHT + TOAST_PADDING_BOTTOM);

    Rect::new(toast_x, toast_y, toast_width, TOAST_HEIGHT)
}

/// Creates the styled block for the toast notification.
///
/// # Returns
///
/// A `Block` with rounded borders and dark styling.
#[must_use]
fn create_toast_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Black))
}

/// Determines the text color based on message content.
///
/// # Arguments
///
/// * `message` - The message to analyze
///
/// # Returns
///
/// The appropriate `Color` for the message type.
#[must_use]
fn determine_text_color(message: &str) -> Color {
    if message.starts_with("[+]") {
        SUCCESS_COLOR
    } else if message.starts_with("[x]") {
        ERROR_COLOR
    } else {
        Color::White
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_toast_position_variants() {
        struct TestCase {
            name: &'static str,
            area: Rect,
            message: &'static str,
            expected_height: u16,
            min_width_check: Option<u16>,
            max_width_check: Option<u16>,
            bounds_check: bool,
        }

        let cases = [
            TestCase {
                name: "normal message",
                area: Rect::new(0, 0, 100, 50),
                message: "Test message",
                expected_height: TOAST_HEIGHT,
                min_width_check: Some(MIN_TOAST_WIDTH),
                max_width_check: Some(50), // area.width / 2
                bounds_check: false,
            },
            TestCase {
                name: "long message",
                area: Rect::new(0, 0, 100, 50),
                message: "This is a very long message that should be constrained",
                expected_height: TOAST_HEIGHT,
                min_width_check: None,
                max_width_check: Some(50),
                bounds_check: false,
            },
            TestCase {
                name: "short message",
                area: Rect::new(0, 0, 100, 50),
                message: "Hi",
                expected_height: TOAST_HEIGHT,
                min_width_check: Some(MIN_TOAST_WIDTH),
                max_width_check: None,
                bounds_check: false,
            },
            TestCase {
                name: "small area",
                area: Rect::new(0, 0, 30, 10),
                message: "Test",
                expected_height: TOAST_HEIGHT,
                min_width_check: None,
                max_width_check: None,
                bounds_check: true,
            },
        ];

        for case in &cases {
            let toast_area = calculate_toast_position(case.area, case.message);

            assert_eq!(
                toast_area.height, case.expected_height,
                "{}: height",
                case.name
            );

            if let Some(min_width) = case.min_width_check {
                assert!(
                    toast_area.width >= min_width,
                    "{}: width should be >= {}",
                    case.name,
                    min_width
                );
            }

            if let Some(max_width) = case.max_width_check {
                assert!(
                    toast_area.width <= max_width,
                    "{}: width should be <= {}",
                    case.name,
                    max_width
                );
            }

            if case.bounds_check {
                assert!(
                    toast_area.x + toast_area.width <= case.area.width,
                    "{}: x bounds",
                    case.name
                );
                assert!(
                    toast_area.y + toast_area.height <= case.area.height,
                    "{}: y bounds",
                    case.name
                );
            }
        }
    }

    #[test]
    fn test_determine_text_color_variants() {
        struct TestCase {
            message: &'static str,
            expected: Color,
        }

        let cases = [
            TestCase {
                message: "[+] Success",
                expected: SUCCESS_COLOR,
            },
            TestCase {
                message: "[x] Error",
                expected: ERROR_COLOR,
            },
            TestCase {
                message: "Info message",
                expected: Color::White,
            },
            TestCase {
                message: "",
                expected: Color::White,
            },
        ];

        for case in &cases {
            assert_eq!(
                determine_text_color(case.message),
                case.expected,
                "message: '{}'",
                case.message
            );
        }
    }

    #[test]
    fn test_toast_constants_and_block() {
        assert_eq!(TOAST_HEIGHT, 3);
        assert_eq!(MIN_TOAST_WIDTH, 20);
        assert_eq!(TOAST_WIDTH_PADDING, 4);

        // Verify create_toast_block works
        let _block = create_toast_block();
    }
}
