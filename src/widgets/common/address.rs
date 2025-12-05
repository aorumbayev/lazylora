//! Address display widget.
//!
//! Renders truncated addresses with optional labels and colors.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::widgets::helpers::truncate_address;

// ============================================================================
// AddressDisplay Widget
// ============================================================================

/// Renders truncated addresses with optional labels.
///
/// # Example
///
/// ```text
/// From: AAAAAAA...AAAAAA
/// ```
///
/// # Usage
///
/// ```ignore
/// use crate::widgets::common::AddressDisplay;
/// use ratatui::style::Color;
///
/// let display = AddressDisplay::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
///     .with_label("From")
///     .truncate(20)
///     .with_color(Color::Yellow);
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AddressDisplay {
    address: String,
    label: Option<String>,
    max_len: usize,
    color: Color,
}

impl AddressDisplay {
    /// Default maximum length for address truncation.
    #[allow(dead_code)]
    pub const DEFAULT_MAX_LEN: usize = 20;

    /// Create a new address display.
    ///
    /// # Arguments
    ///
    /// * `address` - The full address to display
    ///
    /// # Returns
    ///
    /// A new `AddressDisplay` with default settings
    #[must_use]
    #[allow(dead_code)]
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            label: None,
            max_len: Self::DEFAULT_MAX_LEN,
            color: Color::Cyan,
        }
    }

    /// Add a label prefix.
    ///
    /// # Arguments
    ///
    /// * `label` - The label to display before the address (e.g., "From", "To")
    ///
    /// # Returns
    ///
    /// Self with the label set
    #[must_use]
    #[allow(dead_code)]
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set the maximum length for truncation.
    ///
    /// # Arguments
    ///
    /// * `max_len` - Maximum character length for the truncated address
    ///
    /// # Returns
    ///
    /// Self with the new truncation length
    #[must_use]
    #[allow(dead_code)]
    pub const fn truncate(mut self, max_len: usize) -> Self {
        self.max_len = max_len;
        self
    }

    /// Set the address color.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to use for the address text
    ///
    /// # Returns
    ///
    /// Self with the new color
    #[must_use]
    #[allow(dead_code)]
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Generate the display line.
    ///
    /// # Returns
    ///
    /// A `Line` element representing the formatted address
    #[must_use]
    pub fn to_line(&self) -> Line<'static> {
        let truncated = truncate_address(&self.address, self.max_len);

        match &self.label {
            Some(label) => Line::from(vec![
                Span::styled(
                    format!("{label}: "),
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(truncated, Style::default().fg(self.color)),
            ]),
            None => Line::from(Span::styled(truncated, Style::default().fg(self.color))),
        }
    }
}

impl Widget for AddressDisplay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = self.to_line();

        if area.height == 0 {
            return;
        }

        let y = area.y;
        let mut x = area.x;

        for span in line.spans.iter() {
            let content = span.content.as_ref();
            for ch in content.chars() {
                if x >= area.x + area.width {
                    break;
                }
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(ch).set_style(span.style);
                }
                x += 1;
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests AddressDisplay behavior across all configurations.
    ///
    /// Validates: truncation, labels, colors, and content rendering.
    #[test]
    fn test_address_display_behavior() {
        struct TestCase {
            name: &'static str,
            address: &'static str,
            label: Option<&'static str>,
            max_len: Option<usize>,
            color: Option<Color>,
            // Expected behavior
            expect_spans: usize,
            expect_truncated: bool,
            expect_content_contains: Option<&'static str>,
            expect_content_equals: Option<&'static str>,
        }

        let test_cases = [
            TestCase {
                name: "basic address without label",
                address: "ABCDEFGHIJKLMNOP",
                label: None,
                max_len: None,
                color: None,
                expect_spans: 1,
                expect_truncated: false,
                expect_content_contains: None,
                expect_content_equals: None,
            },
            TestCase {
                name: "address with label produces two spans",
                address: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                label: Some("From"),
                max_len: Some(15),
                color: None,
                expect_spans: 2,
                expect_truncated: true,
                expect_content_contains: Some("From:"),
                expect_content_equals: None,
            },
            TestCase {
                name: "long address is truncated with ellipsis",
                address: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                label: None,
                max_len: Some(20),
                color: None,
                expect_spans: 1,
                expect_truncated: true,
                expect_content_contains: Some("..."),
                expect_content_equals: None,
            },
            TestCase {
                name: "custom color preserves content",
                address: "TESTADDR",
                label: None,
                max_len: None,
                color: Some(Color::Yellow),
                expect_spans: 1,
                expect_truncated: false,
                expect_content_contains: Some("TESTADDR"),
                expect_content_equals: None,
            },
            TestCase {
                name: "short address not truncated",
                address: "SHORT",
                label: None,
                max_len: Some(20),
                color: None,
                expect_spans: 1,
                expect_truncated: false,
                expect_content_contains: None,
                expect_content_equals: Some("SHORT"),
            },
        ];

        for tc in test_cases {
            let mut display = AddressDisplay::new(tc.address);
            if let Some(label) = tc.label {
                display = display.with_label(label);
            }
            if let Some(max_len) = tc.max_len {
                display = display.truncate(max_len);
            }
            if let Some(color) = tc.color {
                display = display.with_color(color);
            }

            let line = display.to_line();
            let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();

            assert_eq!(
                line.spans.len(),
                tc.expect_spans,
                "{}: expected {} spans, got {}",
                tc.name,
                tc.expect_spans,
                line.spans.len()
            );

            if tc.expect_truncated {
                assert!(
                    content.contains("..."),
                    "{}: expected truncation ellipsis",
                    tc.name
                );
            } else {
                assert!(
                    !content.contains("..."),
                    "{}: unexpected truncation ellipsis",
                    tc.name
                );
            }

            if let Some(expected) = tc.expect_content_contains {
                assert!(
                    content.contains(expected),
                    "{}: expected content to contain '{}'",
                    tc.name,
                    expected
                );
            }

            if let Some(expected) = tc.expect_content_equals {
                assert_eq!(content, expected, "{}: content mismatch", tc.name);
            }
        }
    }
}
