//! Amount display widget.
//!
//! Renders formatted amounts with proper units for ALGO and ASA tokens.

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
#[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    /// Tests AmountDisplay behavior for both ALGO and asset amounts.
    ///
    /// Validates: formatting, unit names, asset IDs, and edge cases.
    #[test]
    fn test_amount_display_behavior() {
        struct TestCase {
            name: &'static str,
            display: AmountDisplay,
            expect_content_contains: Vec<&'static str>,
            expect_content_excludes: Vec<&'static str>,
        }

        let test_cases = [
            TestCase {
                name: "algo amount formats correctly",
                display: AmountDisplay::algo(5_000_000),
                expect_content_contains: vec!["5.000000", "ALGO"],
                expect_content_excludes: vec!["ASA"],
            },
            TestCase {
                name: "zero algo amount shows zeros",
                display: AmountDisplay::algo(0),
                expect_content_contains: vec!["0.000000"],
                expect_content_excludes: vec![],
            },
            TestCase {
                name: "asset with ID shows ASA reference",
                display: AmountDisplay::asset(1000, Some(31566704), Some(6)),
                expect_content_contains: vec!["ASA #31566704"],
                expect_content_excludes: vec![],
            },
            TestCase {
                name: "asset with unit name displays name",
                display: AmountDisplay::asset(1000, Some(31566704), Some(6)).with_unit_name("USDC"),
                expect_content_contains: vec!["USDC", "ASA #31566704"],
                expect_content_excludes: vec![],
            },
            TestCase {
                name: "asset without ID uses default unit",
                display: AmountDisplay::asset(500, None, None),
                expect_content_contains: vec!["units"],
                expect_content_excludes: vec!["ASA #"],
            },
        ];

        for tc in test_cases {
            let line = tc.display.to_line();
            let content: String = line.spans.iter().map(|s| s.content.to_string()).collect();

            assert!(
                !line.spans.is_empty(),
                "{}: should produce non-empty spans",
                tc.name
            );

            for expected in &tc.expect_content_contains {
                assert!(
                    content.contains(expected),
                    "{}: expected content to contain '{}', got '{}'",
                    tc.name,
                    expected,
                    content
                );
            }

            for excluded in &tc.expect_content_excludes {
                assert!(
                    !content.contains(excluded),
                    "{}: expected content to NOT contain '{}', got '{}'",
                    tc.name,
                    excluded,
                    content
                );
            }
        }
    }
}
