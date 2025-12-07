//! Transaction visual card widget.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use super::TxnFlowDiagram;
use crate::domain::{Transaction, TransactionDetails};
use crate::widgets::helpers::{
    format_algo_amount, format_with_commas, truncate_address, txn_type_icon,
};

// ============================================================================
// TxnVisualCard Widget
// ============================================================================

/// Visual card widget for displaying transaction details.
pub struct TxnVisualCard<'a> {
    txn: &'a Transaction,
    show_flow: bool,
    show_details: bool,
    compact: bool,
}

impl<'a> TxnVisualCard<'a> {
    /// Create a new transaction visual card.
    #[must_use]
    pub const fn new(txn: &'a Transaction) -> Self {
        Self {
            txn,
            show_flow: true,
            show_details: true,
            compact: false,
        }
    }

    /// Hide the flow diagram.
    #[allow(dead_code)]
    #[must_use]
    pub const fn without_flow(mut self) -> Self {
        self.show_flow = false;
        self
    }

    /// Hide the details section.
    #[allow(dead_code)]
    #[must_use]
    pub const fn without_details(mut self) -> Self {
        self.show_details = false;
        self
    }

    /// Enable compact mode.
    #[allow(dead_code)]
    #[must_use]
    pub const fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Generate all lines for the card.
    #[must_use]
    pub fn to_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Transaction ID header
        let id_display = truncate_address(&self.txn.id, 52);
        lines.push(Line::from(vec![
            Span::styled(
                "TXN: ",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                id_display,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Type badge inline
        let icon = txn_type_icon(self.txn.txn_type);
        let type_name = self.txn.txn_type.as_str();
        let color = self.txn.txn_type.color();
        lines.push(Line::from(vec![
            Span::styled(
                "Type: ",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{} {}", icon, type_name),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(""));

        // Flow diagram
        if self.show_flow && !self.compact {
            let flow = TxnFlowDiagram::new(self.txn);
            lines.extend(flow.to_lines());
            lines.push(Line::from(""));
        }

        // Type-specific details section
        if self.show_details {
            // Add type-specific information
            match &self.txn.details {
                TransactionDetails::Payment(pay_details) => {
                    // Amount for payment
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Amount: ",
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format_algo_amount(self.txn.amount),
                            Style::default().fg(Color::Green),
                        ),
                    ]));

                    // Close remainder if present
                    if let Some(close_to) = &pay_details.close_remainder_to {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Close To: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                truncate_address(close_to, 30),
                                Style::default().fg(Color::Yellow),
                            ),
                        ]));
                    }
                    if let Some(close_amount) = pay_details.close_amount {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Close Amount: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format_algo_amount(close_amount),
                                Style::default().fg(Color::Green),
                            ),
                        ]));
                    }
                }
                TransactionDetails::AssetTransfer(axfer_details) => {
                    // Amount and asset
                    let amount_str = if let Some(asset_id) = self.txn.asset_id {
                        format!(
                            "{} units (ASA #{})",
                            format_with_commas(self.txn.amount),
                            asset_id
                        )
                    } else {
                        format!("{} units", format_with_commas(self.txn.amount))
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Amount: ",
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(amount_str, Style::default().fg(Color::Yellow)),
                    ]));

                    // Clawback info
                    if let Some(asset_sender) = &axfer_details.asset_sender {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Clawback From: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                truncate_address(asset_sender, 30),
                                Style::default().fg(Color::Magenta),
                            ),
                        ]));
                    }
                    // Close to info
                    if let Some(close_to) = &axfer_details.close_to {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Close To: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                truncate_address(close_to, 30),
                                Style::default().fg(Color::Yellow),
                            ),
                        ]));
                    }
                }
                TransactionDetails::AssetConfig(acfg_details) => {
                    // Determine action
                    let action = if acfg_details.created_asset_id.is_some()
                        || (acfg_details.asset_id.is_none() && acfg_details.total.is_some())
                    {
                        "Create Asset"
                    } else if acfg_details.total.is_none() && acfg_details.asset_id.is_some() {
                        "Destroy Asset"
                    } else {
                        "Modify Asset"
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Action: ",
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(action.to_string(), Style::default().fg(Color::Cyan)),
                    ]));

                    // Asset details for creation
                    if let Some(name) = &acfg_details.asset_name {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Asset Name: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(name.clone(), Style::default().fg(Color::White)),
                        ]));
                    }
                    if let Some(unit) = &acfg_details.unit_name {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Unit: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(unit.clone(), Style::default().fg(Color::White)),
                        ]));
                    }
                    if let Some(total) = acfg_details.total {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Total: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format_with_commas(total),
                                Style::default().fg(Color::Green),
                            ),
                        ]));
                    }
                    if let Some(decimals) = acfg_details.decimals {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Decimals: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("{}", decimals),
                                Style::default().fg(Color::White),
                            ),
                        ]));
                    }
                }
                TransactionDetails::AssetFreeze(afrz_details) => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Action: ",
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            if afrz_details.frozen {
                                "Freeze"
                            } else {
                                "Unfreeze"
                            }
                            .to_string(),
                            Style::default().fg(if afrz_details.frozen {
                                Color::Red
                            } else {
                                Color::Green
                            }),
                        ),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Target: ",
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            truncate_address(&afrz_details.freeze_target, 30),
                            Style::default().fg(Color::Yellow),
                        ),
                    ]));
                }
                TransactionDetails::AppCall(app_details) => {
                    // On-complete action
                    lines.push(Line::from(vec![
                        Span::styled(
                            "On-Complete: ",
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            app_details.on_complete.as_str().to_string(),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]));

                    // Created app ID
                    if let Some(created_id) = app_details.created_app_id {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Created App: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("#{}", created_id),
                                Style::default().fg(Color::Green),
                            ),
                        ]));
                    }

                    // App arguments count
                    if !app_details.app_args.is_empty() {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "App Args: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("{}", app_details.app_args.len()),
                                Style::default().fg(Color::White),
                            ),
                        ]));
                    }

                    // Foreign references
                    if !app_details.foreign_apps.is_empty() {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Foreign Apps: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("{}", app_details.foreign_apps.len()),
                                Style::default().fg(Color::White),
                            ),
                        ]));
                    }
                    if !app_details.foreign_assets.is_empty() {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Foreign Assets: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("{}", app_details.foreign_assets.len()),
                                Style::default().fg(Color::White),
                            ),
                        ]));
                    }
                    if !app_details.boxes.is_empty() {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Box Refs: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("{}", app_details.boxes.len()),
                                Style::default().fg(Color::White),
                            ),
                        ]));
                    }
                }
                TransactionDetails::KeyReg(keyreg_details) => {
                    if keyreg_details.non_participation {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Status: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "Going Offline".to_string(),
                                Style::default().fg(Color::Red),
                            ),
                        ]));
                    } else if keyreg_details.vote_key.is_some() {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Status: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "Going Online".to_string(),
                                Style::default().fg(Color::Green),
                            ),
                        ]));
                        if let (Some(first), Some(last)) =
                            (keyreg_details.vote_first, keyreg_details.vote_last)
                        {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    "Valid Rounds: ",
                                    Style::default()
                                        .fg(Color::Gray)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!("{} - {}", first, last),
                                    Style::default().fg(Color::White),
                                ),
                            ]));
                        }
                        if let Some(dilution) = keyreg_details.vote_key_dilution {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    "Key Dilution: ",
                                    Style::default()
                                        .fg(Color::Gray)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!("{}", dilution),
                                    Style::default().fg(Color::White),
                                ),
                            ]));
                        }
                    }
                }
                TransactionDetails::StateProof(sp_details) => {
                    if let Some(sp_type) = sp_details.state_proof_type {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Proof Type: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(format!("{}", sp_type), Style::default().fg(Color::White)),
                        ]));
                    }
                }
                TransactionDetails::Heartbeat(hb_details) => {
                    if let Some(hb_addr) = &hb_details.hb_address {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "HB Address: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                truncate_address(hb_addr, 30),
                                Style::default().fg(Color::Cyan),
                            ),
                        ]));
                    }
                }
                TransactionDetails::None => {
                    // Fallback amount display for unknown types
                    if self.txn.amount > 0 {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Amount: ",
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("{}", self.txn.amount),
                                Style::default().fg(Color::White),
                            ),
                        ]));
                    }
                }
            }

            lines.push(Line::from("")); // Spacer

            // Fee
            let fee_formatted = format_algo_amount(self.txn.fee);
            lines.push(Line::from(vec![
                Span::styled(
                    "Fee: ",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(fee_formatted, Style::default().fg(Color::Red)),
            ]));

            // Block
            lines.push(Line::from(vec![
                Span::styled(
                    "Block: ",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("#{}", self.txn.block),
                    Style::default().fg(Color::Cyan),
                ),
            ]));

            // Timestamp
            lines.push(Line::from(vec![
                Span::styled(
                    "Time: ",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(self.txn.timestamp.clone(), Style::default().fg(Color::Gray)),
            ]));
        }

        lines
    }
}

impl Widget for TxnVisualCard<'_> {
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
