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
    #[allow(dead_code)] // Part of AddressDisplay public API
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
    #[allow(dead_code)] // Part of AddressDisplay public API
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
    #[allow(dead_code)] // Part of AddressDisplay public API
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
    #[allow(dead_code)] // Part of AddressDisplay public API
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
    #[allow(dead_code)] // Part of AddressDisplay public API
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

    #[test]
    fn test_address_display_basic() {
        let display = AddressDisplay::new("ABCDEFGHIJKLMNOP");
        let line = display.to_line();
        assert_eq!(line.spans.len(), 1);
    }

    #[test]
    fn test_address_display_with_label() {
        let display =
            AddressDisplay::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
                .with_label("From")
                .truncate(15);
        let line = display.to_line();
        assert_eq!(line.spans.len(), 2);
    }

    #[test]
    fn test_address_display_truncation() {
        let long_addr = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let display = AddressDisplay::new(long_addr).truncate(20);
        let line = display.to_line();

        // The truncated address should contain "..."
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(content.contains("..."));
        assert!(content.len() <= 20);
    }

    #[test]
    fn test_address_display_with_color() {
        let display = AddressDisplay::new("TESTADDR").with_color(Color::Yellow);
        let line = display.to_line();
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_address_display_short_address() {
        let short_addr = "SHORT";
        let display = AddressDisplay::new(short_addr).truncate(20);
        let line = display.to_line();

        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert_eq!(content, "SHORT");
        assert!(!content.contains("..."));
    }
}
