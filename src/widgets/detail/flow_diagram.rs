//! Transaction flow diagram widget.
//!
//! This module provides an ASCII art visualization of transaction flow,
//! showing sender and receiver entities with an arrow between them.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::domain::{Transaction, TxnType};
use crate::widgets::helpers::{
    MICROALGOS_PER_ALGO, format_with_commas, truncate_address, txn_type_icon,
};

// ============================================================================
// TxnFlowDiagram Widget
// ============================================================================

/// ASCII art showing the flow of a transaction.
///
/// # Example
///
/// ```text
/// Payment:
///   ┌──────────────┐         ┌──────────────┐
///   │   SENDER     │──[$]───▶│   RECEIVER   │
///   │ ABC...XYZ    │  5 ALGO │ DEF...UVW    │
///   └──────────────┘         └──────────────┘
/// ```
///
/// # Usage
///
/// ```ignore
/// use crate::widgets::detail::TxnFlowDiagram;
///
/// let diagram = TxnFlowDiagram::new(&transaction);
/// let lines = diagram.to_lines();
/// ```
#[derive(Debug, Clone)]
pub struct TxnFlowDiagram<'a> {
    txn: &'a Transaction,
    box_width: usize,
}

impl<'a> TxnFlowDiagram<'a> {
    #[must_use]
    pub const fn new(txn: &'a Transaction) -> Self {
        Self { txn, box_width: 16 }
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_box_width(mut self, width: usize) -> Self {
        self.box_width = width;
        self
    }

    /// Get the sender label based on transaction type.
    fn sender_label(&self) -> &'static str {
        match self.txn.txn_type {
            TxnType::AppCall => "CALLER",
            _ => "SENDER",
        }
    }

    /// Get the receiver label based on transaction type.
    fn receiver_label(&self) -> String {
        match self.txn.txn_type {
            TxnType::AppCall => {
                if self.txn.to != "unknown" && self.txn.to != "0" {
                    format!("APP #{}", self.txn.to)
                } else {
                    "NEW APP".to_string()
                }
            }
            TxnType::AssetConfig => "ASSET CFG".to_string(),
            TxnType::AssetFreeze => "FROZEN".to_string(),
            TxnType::KeyReg => "CONSENSUS".to_string(),
            TxnType::StateProof => "STATE".to_string(),
            TxnType::Heartbeat => "NETWORK".to_string(),
            _ => "RECEIVER".to_string(),
        }
    }

    /// Get the transfer description (amount or action).
    fn transfer_description(&self) -> String {
        match self.txn.txn_type {
            TxnType::Payment => {
                let algos = self.txn.amount as f64 / MICROALGOS_PER_ALGO;
                if algos >= 1.0 {
                    format!("{:.2} ALGO", algos)
                } else {
                    format!("{:.6} ALGO", algos)
                }
            }
            TxnType::AssetTransfer => {
                if let Some(asset_id) = self.txn.asset_id {
                    format!("{} ASA", format_with_commas(self.txn.amount))
                        + &format!("\n#{}", asset_id)
                } else {
                    format!("{} ASA", format_with_commas(self.txn.amount))
                }
            }
            TxnType::AppCall => "call".to_string(),
            TxnType::AssetConfig => "config".to_string(),
            TxnType::AssetFreeze => "freeze".to_string(),
            TxnType::KeyReg => "keyreg".to_string(),
            TxnType::StateProof => "proof".to_string(),
            TxnType::Heartbeat => "beat".to_string(),
            TxnType::Unknown => "???".to_string(),
        }
    }

    /// Generate the flow diagram lines.
    ///
    /// # Returns
    ///
    /// A vector of `Line` elements representing the flow diagram
    #[must_use]
    pub fn to_lines(&self) -> Vec<Line<'static>> {
        let box_w = self.box_width;
        let inner_w = box_w - 2; // Account for borders

        let sender_label = self.sender_label();
        let receiver_label = self.receiver_label();
        let from_addr = truncate_address(&self.txn.from, inner_w.saturating_sub(2));
        let to_addr = if matches!(
            self.txn.txn_type,
            TxnType::AppCall
                | TxnType::AssetConfig
                | TxnType::KeyReg
                | TxnType::StateProof
                | TxnType::Heartbeat
        ) {
            String::new()
        } else {
            truncate_address(&self.txn.to, inner_w.saturating_sub(2))
        };

        let icon = txn_type_icon(self.txn.txn_type);
        let transfer_desc = self.transfer_description();
        let transfer_lines: Vec<&str> = transfer_desc.lines().collect();
        let transfer_line1 = transfer_lines.first().copied().unwrap_or("");
        let transfer_line2 = transfer_lines.get(1).copied().unwrap_or("");

        let color = self.txn.txn_type.color();

        // Center text in box
        let center = |s: &str, w: usize| -> String {
            let len = s.chars().count();
            if len >= w {
                s.chars().take(w).collect()
            } else {
                let padding = w - len;
                let left = padding / 2;
                let right = padding - left;
                format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
            }
        };

        // Build the diagram
        let top_border = format!("┌{}┐", "─".repeat(inner_w));
        let bottom_border = format!("└{}┘", "─".repeat(inner_w));
        let arrow_segment = format!("──{}───▶", icon);
        let gap = 9; // Width of arrow segment

        // Line 1: Top borders
        let line1 = Line::from(format!("  {}{}{}", top_border, " ".repeat(gap), top_border));

        // Line 2: Labels with arrow
        let sender_centered = center(sender_label, inner_w);
        let receiver_centered = center(&receiver_label, inner_w);
        let line2 = Line::from(vec![
            Span::raw("  │"),
            Span::styled(
                sender_centered,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("│"),
            Span::styled(
                arrow_segment.clone(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("│"),
            Span::styled(
                receiver_centered,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("│"),
        ]);

        // Line 3: Addresses with transfer amount
        let from_centered = center(&from_addr, inner_w);
        let to_centered = center(&to_addr, inner_w);
        let transfer_centered = center(transfer_line1, gap);
        let line3 = Line::from(vec![
            Span::raw("  │"),
            Span::styled(from_centered, Style::default().fg(Color::Yellow)),
            Span::raw("│"),
            Span::styled(transfer_centered, Style::default().fg(Color::Green)),
            Span::raw("│"),
            Span::styled(to_centered, Style::default().fg(Color::Cyan)),
            Span::raw("│"),
        ]);

        // Line 4: Bottom info (asset ID if applicable) or empty
        let empty_box = center("", inner_w);
        let transfer2_centered = center(transfer_line2, gap);
        let line4 = Line::from(vec![
            Span::raw("  │"),
            Span::styled(empty_box.clone(), Style::default()),
            Span::raw("│"),
            Span::styled(transfer2_centered, Style::default().fg(Color::DarkGray)),
            Span::raw("│"),
            Span::styled(empty_box, Style::default()),
            Span::raw("│"),
        ]);

        // Line 5: Bottom borders
        let line5 = Line::from(format!(
            "  {}{}{}",
            bottom_border,
            " ".repeat(gap),
            bottom_border
        ));

        vec![line1, line2, line3, line4, line5]
    }
}

impl Widget for TxnFlowDiagram<'_> {
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
    use crate::domain::TransactionDetails;

    fn create_test_payment() -> Transaction {
        Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER_ADDRESS_VERY_LONG_ADDRESS".to_string(),
            to: "RECEIVER_ADDRESS_VERY_LONG".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 5_000_000,
            asset_id: None,
            rekey_to: None,
            group: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        }
    }

    #[test]
    fn test_txn_flow_diagram_configurations() {
        let txn = create_test_payment();

        // Test default configuration
        let diagram = TxnFlowDiagram::new(&txn);
        assert_eq!(diagram.box_width, 16, "default box width");

        // Test custom configuration
        let diagram_custom = TxnFlowDiagram::new(&txn).with_box_width(20);
        assert_eq!(diagram_custom.box_width, 20, "custom box width");

        // Test line generation
        let lines = diagram.to_lines();
        assert_eq!(lines.len(), 5, "diagram has 5 lines");
    }

    #[test]
    fn test_txn_flow_diagram_labels_by_type() {
        struct TestCase {
            name: &'static str,
            txn_type: TxnType,
            to_field: &'static str,
            expected_sender: &'static str,
            expected_receiver: &'static str,
        }

        let cases = [
            TestCase {
                name: "payment",
                txn_type: TxnType::Payment,
                to_field: "RECEIVER",
                expected_sender: "SENDER",
                expected_receiver: "RECEIVER",
            },
            TestCase {
                name: "app call with id",
                txn_type: TxnType::AppCall,
                to_field: "12345",
                expected_sender: "CALLER",
                expected_receiver: "APP #12345",
            },
            TestCase {
                name: "app creation",
                txn_type: TxnType::AppCall,
                to_field: "unknown",
                expected_sender: "CALLER",
                expected_receiver: "NEW APP",
            },
        ];

        for case in &cases {
            let mut txn = create_test_payment();
            txn.txn_type = case.txn_type;
            txn.to = case.to_field.to_string();

            let diagram = TxnFlowDiagram::new(&txn);
            assert_eq!(
                diagram.sender_label(),
                case.expected_sender,
                "{}: sender",
                case.name
            );
            assert_eq!(
                diagram.receiver_label(),
                case.expected_receiver,
                "{}: receiver",
                case.name
            );
        }
    }

    #[test]
    fn test_txn_flow_diagram_transfer_descriptions() {
        struct TestCase {
            name: &'static str,
            txn_type: TxnType,
            amount: u64,
            asset_id: Option<u64>,
            expected_contains: &'static str,
        }

        let cases = [
            TestCase {
                name: "payment",
                txn_type: TxnType::Payment,
                amount: 5_000_000,
                asset_id: None,
                expected_contains: "ALGO",
            },
            TestCase {
                name: "asset transfer with id",
                txn_type: TxnType::AssetTransfer,
                amount: 1000,
                asset_id: Some(31566704),
                expected_contains: "ASA",
            },
            TestCase {
                name: "asset transfer without id",
                txn_type: TxnType::AssetTransfer,
                amount: 500,
                asset_id: None,
                expected_contains: "ASA",
            },
        ];

        for case in &cases {
            let mut txn = create_test_payment();
            txn.txn_type = case.txn_type;
            txn.amount = case.amount;
            txn.asset_id = case.asset_id;

            let diagram = TxnFlowDiagram::new(&txn);
            let desc = diagram.transfer_description();
            assert!(
                desc.contains(case.expected_contains),
                "{}: expected '{}' in '{}'",
                case.name,
                case.expected_contains,
                desc
            );
        }
    }
}
