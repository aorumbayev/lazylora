//! Transaction type badge widget.
//!
//! Displays a colored badge showing the transaction type with an icon.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::domain::TxnType;
use crate::widgets::helpers::txn_type_icon;

// ============================================================================
// TxnTypeBadge Widget
// ============================================================================

/// A colored badge showing the transaction type with an icon.
///
/// # Example
///
/// ```text
/// ┌─────────────────┐
/// │ [$] Payment     │
/// └─────────────────┘
/// ```
///
/// # Usage
///
/// ```ignore
/// use crate::widgets::common::TxnTypeBadge;
/// use crate::domain::TxnType;
///
/// let badge = TxnTypeBadge::new(TxnType::Payment);
/// let compact_badge = TxnTypeBadge::new(TxnType::AppCall).compact();
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TxnTypeBadge {
    txn_type: TxnType,
    compact: bool,
}

impl TxnTypeBadge {
    /// Create a new transaction type badge.
    ///
    /// # Arguments
    ///
    /// * `txn_type` - The transaction type to display
    ///
    /// # Returns
    ///
    /// A new `TxnTypeBadge` instance
    #[must_use]
    #[allow(dead_code)] // Part of TxnTypeBadge public API
    pub const fn new(txn_type: TxnType) -> Self {
        Self {
            txn_type,
            compact: false,
        }
    }

    /// Create a compact badge (icon only).
    ///
    /// # Returns
    ///
    /// Self with compact mode enabled
    #[must_use]
    #[allow(dead_code)] // Part of TxnTypeBadge public API
    pub const fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Get the lines for rendering this badge.
    ///
    /// # Returns
    ///
    /// A vector of `Line` elements representing the badge
    #[must_use]
    pub fn to_lines(&self) -> Vec<Line<'static>> {
        let icon = txn_type_icon(self.txn_type);
        let color = self.txn_type.color();

        if self.compact {
            let content = format!(" {icon} ");
            let width = content.len();
            vec![
                Line::from(format!("┌{}┐", "─".repeat(width))),
                Line::from(vec![
                    Span::raw("│"),
                    Span::styled(
                        content,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("│"),
                ]),
                Line::from(format!("└{}┘", "─".repeat(width))),
            ]
        } else {
            let name = self.txn_type.as_str();
            let content = format!(" {icon} {name} ");
            let width = content.len();
            vec![
                Line::from(format!("┌{}┐", "─".repeat(width))),
                Line::from(vec![
                    Span::raw("│"),
                    Span::styled(
                        content,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("│"),
                ]),
                Line::from(format!("└{}┘", "─".repeat(width))),
            ]
        }
    }
}

impl Widget for TxnTypeBadge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.to_lines();

        for (i, line) in lines.iter().enumerate() {
            if i >= area.height as usize {
                break;
            }
            let y = area.y + i as u16;
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
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_txn_type_badge_to_lines() {
        let badge = TxnTypeBadge::new(TxnType::Payment);
        let lines = badge.to_lines();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_txn_type_badge_compact() {
        let badge = TxnTypeBadge::new(TxnType::Payment).compact();
        let lines = badge.to_lines();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_txn_type_badge_different_types() {
        for txn_type in [
            TxnType::Payment,
            TxnType::AppCall,
            TxnType::AssetTransfer,
            TxnType::AssetConfig,
            TxnType::AssetFreeze,
            TxnType::KeyReg,
            TxnType::StateProof,
            TxnType::Heartbeat,
            TxnType::Unknown,
        ] {
            let badge = TxnTypeBadge::new(txn_type);
            let lines = badge.to_lines();
            assert_eq!(lines.len(), 3, "Badge for {txn_type:?} should have 3 lines");
        }
    }
}
