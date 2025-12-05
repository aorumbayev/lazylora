//! Amount display widget.
//!
//! Renders formatted amounts with proper units for ALGO and ASA tokens.

#![allow(dead_code)] // Transitional phase - items will be used after integration

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::widgets::helpers::{ALGO_SYMBOL, ASSET_SYMBOL, format_algo_amount, format_asset_amount};

// ============================================================================
// AmountDisplay Widget
// ============================================================================

/// Renders formatted amounts with proper units.
///
/// # Example
///
/// ```text
/// ◈ 5.000000 ALGO
/// ◆ 1,000 USDC (ASA #31566704)
/// ```
///
/// # Usage
///
/// ```ignore
/// use crate::widgets::common::AmountDisplay;
///
/// // Display ALGO amount
/// let algo_display = AmountDisplay::algo(5_000_000);
///
/// // Display ASA amount with unit name
/// let asset_display = AmountDisplay::asset(1000, Some(31566704), Some(6))
///     .with_unit_name("USDC");
/// ```
#[derive(Debug, Clone)]
pub struct AmountDisplay {
    amount: u64,
    asset_id: Option<u64>,
    decimals: Option<u64>,
    unit_name: Option<String>,
    is_algo: bool,
}

impl AmountDisplay {
    /// Create an Algo amount display.
    ///
    /// # Arguments
    ///
    /// * `microalgos` - The amount in microAlgos (1 ALGO = 1,000,000 microAlgos)
    ///
    /// # Returns
    ///
    /// A new `AmountDisplay` configured for ALGO
    #[must_use]
    pub const fn algo(microalgos: u64) -> Self {
        Self {
            amount: microalgos,
            asset_id: None,
            decimals: Some(6),
            unit_name: None,
            is_algo: true,
        }
    }

    /// Create an asset amount display.
    ///
    /// # Arguments
    ///
    /// * `amount` - The raw asset amount
    /// * `asset_id` - Optional ASA ID
    /// * `decimals` - Optional decimal places for formatting
    ///
    /// # Returns
    ///
    /// A new `AmountDisplay` configured for an ASA
    #[must_use]
    pub const fn asset(amount: u64, asset_id: Option<u64>, decimals: Option<u64>) -> Self {
        Self {
            amount,
            asset_id,
            decimals,
            unit_name: None,
            is_algo: false,
        }
    }

    /// Set a custom unit name for the asset.
    ///
    /// # Arguments
    ///
    /// * `name` - The unit name to display (e.g., "USDC", "PLANET")
    ///
    /// # Returns
    ///
    /// Self with the unit name set
    #[must_use]
    pub fn with_unit_name(mut self, name: impl Into<String>) -> Self {
        self.unit_name = Some(name.into());
        self
    }

    /// Generate the display line.
    ///
    /// # Returns
    ///
    /// A `Line` element representing the formatted amount
    #[must_use]
    pub fn to_line(&self) -> Line<'static> {
        if self.is_algo {
            let formatted = format_algo_amount(self.amount);
            Line::from(vec![
                Span::styled(
                    format!("{ALGO_SYMBOL} "),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(formatted, Style::default().fg(Color::Green)),
            ])
        } else {
            let formatted = format_asset_amount(self.amount, self.decimals);
            let unit = self
                .unit_name
                .clone()
                .unwrap_or_else(|| "units".to_string());
            let asset_info = self
                .asset_id
                .map(|id| format!(" (ASA #{id})"))
                .unwrap_or_default();

            Line::from(vec![
                Span::styled(
                    format!("{ASSET_SYMBOL} "),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{formatted} {unit}"),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(asset_info, Style::default().fg(Color::DarkGray)),
            ])
        }
    }
}

impl Widget for AmountDisplay {
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
    fn test_amount_display_algo() {
        let display = AmountDisplay::algo(5_000_000);
        let line = display.to_line();
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_amount_display_asset() {
        let display = AmountDisplay::asset(1000, Some(31566704), Some(6));
        let line = display.to_line();
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_amount_display_asset_with_unit_name() {
        let display = AmountDisplay::asset(1000, Some(31566704), Some(6)).with_unit_name("USDC");
        let line = display.to_line();
        assert!(!line.spans.is_empty());

        // Check that the line contains "USDC"
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(content.contains("USDC"));
    }

    #[test]
    fn test_amount_display_asset_no_id() {
        let display = AmountDisplay::asset(500, None, None);
        let line = display.to_line();
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_amount_display_zero_algo() {
        let display = AmountDisplay::algo(0);
        let line = display.to_line();
        let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(content.contains("0.000000"));
    }
}
