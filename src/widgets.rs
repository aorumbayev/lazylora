//! ASCII visualization widgets for the TUI.
//!
//! This module provides reusable widget components that render rich ASCII visualizations
//! for Algorand transactions, inspired by the AlgoKit-Lora web UI.

#![allow(dead_code)] // Public API methods may not be used internally yet

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::algorand::{Transaction, TransactionDetails, TxnType};

// ============================================================================
// Constants
// ============================================================================

/// Algorand symbol for display
const ALGO_SYMBOL: &str = "◈";

/// Asset symbol for display
const ASSET_SYMBOL: &str = "◆";

/// Number of microAlgos per Algo
const MICROALGOS_PER_ALGO: f64 = 1_000_000.0;

// ============================================================================
// Helper Functions
// ============================================================================

/// Truncate an address to fit in the given width.
///
/// If the address is longer than `max_len`, it will be truncated with an ellipsis
/// in the middle (e.g., "AAAA...AAAA").
///
/// # Arguments
///
/// * `addr` - The address to truncate
/// * `max_len` - The maximum length of the resulting string
///
/// # Returns
///
/// A truncated address string or the original if it fits
#[must_use]
pub fn truncate_address(addr: &str, max_len: usize) -> String {
    if addr.len() <= max_len {
        return addr.to_string();
    }

    if max_len < 7 {
        // Need at least "A...A" (5 chars) + some buffer
        return addr.chars().take(max_len).collect();
    }

    // Reserve 3 chars for "..."
    let available = max_len - 3;
    let prefix_len = available.div_ceil(2); // Round up for prefix
    let suffix_len = available / 2;

    let prefix: String = addr.chars().take(prefix_len).collect();
    let suffix: String = addr.chars().skip(addr.len() - suffix_len).collect();

    format!("{}...{}", prefix, suffix)
}

/// Format microAlgos to Algos with proper decimals.
///
/// # Arguments
///
/// * `microalgos` - The amount in microAlgos
///
/// # Returns
///
/// A formatted string like "5.000000 ALGO"
#[must_use]
pub fn format_algo_amount(microalgos: u64) -> String {
    let algos = microalgos as f64 / MICROALGOS_PER_ALGO;
    format!("{:.6} ALGO", algos)
}

/// Format asset amount with optional decimals.
///
/// # Arguments
///
/// * `amount` - The raw asset amount
/// * `decimals` - Optional decimal places for formatting
///
/// # Returns
///
/// A formatted string with commas for thousands
#[must_use]
pub fn format_asset_amount(amount: u64, decimals: Option<u64>) -> String {
    match decimals {
        Some(d) if d > 0 => {
            let divisor = 10_u64.pow(d as u32) as f64;
            let formatted = amount as f64 / divisor;
            format_with_commas_f64(formatted, d as usize)
        }
        _ => format_with_commas(amount),
    }
}

/// Format a number with commas for thousands separators.
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format a floating point number with commas and specified decimal places.
fn format_with_commas_f64(n: f64, decimals: usize) -> String {
    let int_part = n.trunc() as u64;
    let frac_part = n.fract();

    let int_formatted = format_with_commas(int_part);

    if decimals == 0 {
        int_formatted
    } else {
        let frac_str = format!("{:.prec$}", frac_part, prec = decimals);
        // Skip the "0." prefix
        let frac_digits = &frac_str[2..];
        format!("{}.{}", int_formatted, frac_digits)
    }
}

/// Get the ASCII icon for a transaction type.
///
/// Returns ASCII-safe icons that work in all terminals.
#[must_use]
pub const fn txn_type_icon(txn_type: TxnType) -> &'static str {
    match txn_type {
        TxnType::Payment => "[$]",
        TxnType::AppCall => "[A]",
        TxnType::AssetTransfer => "[>]",
        TxnType::AssetConfig => "[*]",
        TxnType::AssetFreeze => "[#]",
        TxnType::KeyReg => "[K]",
        TxnType::StateProof => "[S]",
        TxnType::Heartbeat => "[H]",
        TxnType::Unknown => "[?]",
    }
}

/// Get the short code for a transaction type.
#[must_use]
pub const fn txn_type_code(txn_type: TxnType) -> &'static str {
    match txn_type {
        TxnType::Payment => "PAY",
        TxnType::AppCall => "APP",
        TxnType::AssetTransfer => "AXF",
        TxnType::AssetConfig => "ACF",
        TxnType::AssetFreeze => "AFZ",
        TxnType::KeyReg => "KEY",
        TxnType::StateProof => "STP",
        TxnType::Heartbeat => "HBT",
        TxnType::Unknown => "???",
    }
}

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
#[derive(Debug, Clone)]
pub struct TxnTypeBadge {
    txn_type: TxnType,
    compact: bool,
}

impl TxnTypeBadge {
    /// Create a new transaction type badge.
    #[must_use]
    pub const fn new(txn_type: TxnType) -> Self {
        Self {
            txn_type,
            compact: false,
        }
    }

    /// Create a compact badge (icon only).
    #[must_use]
    pub const fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Get the lines for rendering this badge.
    #[must_use]
    pub fn to_lines(&self) -> Vec<Line<'static>> {
        let icon = txn_type_icon(self.txn_type);
        let color = self.txn_type.color();

        if self.compact {
            let content = format!(" {} ", icon);
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
            let content = format!(" {} {} ", icon, name);
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
#[derive(Debug, Clone)]
pub struct TxnFlowDiagram<'a> {
    txn: &'a Transaction,
    box_width: usize,
}

impl<'a> TxnFlowDiagram<'a> {
    /// Create a new transaction flow diagram.
    #[must_use]
    pub const fn new(txn: &'a Transaction) -> Self {
        Self { txn, box_width: 16 }
    }

    /// Set custom box width.
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
    #[must_use]
    pub fn asset(amount: u64, asset_id: Option<u64>, decimals: Option<u64>) -> Self {
        Self {
            amount,
            asset_id,
            decimals,
            unit_name: None,
            is_algo: false,
        }
    }

    /// Set a custom unit name for the asset.
    #[must_use]
    pub fn with_unit_name(mut self, name: impl Into<String>) -> Self {
        self.unit_name = Some(name.into());
        self
    }

    /// Generate the display line.
    #[must_use]
    pub fn to_line(&self) -> Line<'static> {
        if self.is_algo {
            let formatted = format_algo_amount(self.amount);
            Line::from(vec![
                Span::styled(
                    format!("{} ", ALGO_SYMBOL),
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
                .map(|id| format!(" (ASA #{})", id))
                .unwrap_or_default();

            Line::from(vec![
                Span::styled(
                    format!("{} ", ASSET_SYMBOL),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} {}", formatted, unit),
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
// AddressDisplay Widget
// ============================================================================

/// Renders truncated addresses with optional labels.
///
/// # Example
///
/// ```text
/// From: AAAAAAA...AAAAAA
/// ```
#[derive(Debug, Clone)]
pub struct AddressDisplay {
    address: String,
    label: Option<String>,
    max_len: usize,
    color: Color,
}

impl AddressDisplay {
    /// Create a new address display.
    #[must_use]
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            label: None,
            max_len: 20,
            color: Color::Cyan,
        }
    }

    /// Add a label prefix.
    #[must_use]
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set the maximum length for truncation.
    #[must_use]
    pub const fn truncate(mut self, max_len: usize) -> Self {
        self.max_len = max_len;
        self
    }

    /// Set the address color.
    #[must_use]
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Generate the display line.
    #[must_use]
    pub fn to_line(&self) -> Line<'static> {
        let truncated = truncate_address(&self.address, self.max_len);

        match &self.label {
            Some(label) => Line::from(vec![
                Span::styled(
                    format!("{}: ", label),
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
// TxnVisualCard Widget
// ============================================================================

/// A complete visual card combining badge, flow diagram, and details.
///
/// This widget provides a comprehensive view of a transaction including:
/// - Transaction type badge
/// - Visual flow diagram
/// - Amount and fee information
/// - Timestamp and block details
#[derive(Debug, Clone)]
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
    #[must_use]
    pub const fn without_flow(mut self) -> Self {
        self.show_flow = false;
        self
    }

    /// Hide the details section.
    #[must_use]
    pub const fn without_details(mut self) -> Self {
        self.show_details = false;
        self
    }

    /// Enable compact mode.
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

// ============================================================================
// StatefulWidget Imports
// ============================================================================

use ratatui::{
    symbols::scrollbar,
    widgets::{
        List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
    },
};

use crate::algorand::AlgoBlock;

// ============================================================================
// BlockListWidget State & Widget
// ============================================================================

/// State for the block list widget.
///
/// This state tracks the currently selected block index and scroll position,
/// allowing the widget to maintain its state across renders.
#[derive(Debug, Default, Clone)]
pub struct BlockListState {
    /// Currently selected block index in the list.
    pub selected_index: Option<usize>,
    /// Scroll position (in pixels/rows).
    pub scroll_position: u16,
}

impl BlockListState {
    /// Creates a new `BlockListState` with no selection.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected_index: None,
            scroll_position: 0,
        }
    }

    /// Creates a new `BlockListState` with the given selection.
    #[must_use]
    pub const fn with_selection(index: usize) -> Self {
        Self {
            selected_index: Some(index),
            scroll_position: 0,
        }
    }

    /// Sets the selected index.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected_index = index;
    }

    /// Returns the currently selected index.
    #[must_use]
    pub const fn selected(&self) -> Option<usize> {
        self.selected_index
    }
}

/// A widget that displays a list of blocks with selection and scrolling.
///
/// This widget implements `StatefulWidget` and requires a `BlockListState`
/// to track selection and scroll position.
///
/// # Example
///
/// ```text
/// ┌─ Latest Blocks ─────────────────────┐
/// │ ▶ 12345678          15 txns         │
/// │   Mon, 01 Jan 2024 12:00:00         │
/// │                                     │
/// │ ⬚ 12345677          8 txns          │
/// │   Mon, 01 Jan 2024 11:59:55         │
/// │                                     │
/// └─────────────────────────────────────┘
/// ```
#[derive(Debug)]
pub struct BlockListWidget<'a> {
    /// Slice of blocks to display.
    blocks: &'a [AlgoBlock],
    /// Whether this widget is currently focused.
    focused: bool,
    /// Height of each block item in rows.
    item_height: u16,
}

impl<'a> BlockListWidget<'a> {
    /// Height of each block item in the list (in rows).
    pub const DEFAULT_ITEM_HEIGHT: u16 = 3;

    /// Creates a new `BlockListWidget` with the given blocks.
    #[must_use]
    pub const fn new(blocks: &'a [AlgoBlock]) -> Self {
        Self {
            blocks,
            focused: false,
            item_height: Self::DEFAULT_ITEM_HEIGHT,
        }
    }

    /// Sets whether this widget is focused.
    #[must_use]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the item height.
    #[must_use]
    pub const fn item_height(mut self, height: u16) -> Self {
        self.item_height = height;
        self
    }

    /// Returns the number of blocks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Returns true if there are no blocks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

impl StatefulWidget for BlockListWidget<'_> {
    type State = BlockListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Handle empty state
        if self.blocks.is_empty() {
            let empty_msg = "No blocks available";
            let x = area.x + (area.width.saturating_sub(empty_msg.len() as u16)) / 2;
            let y = area.y + area.height / 2;

            if y < area.y + area.height && x < area.x + area.width {
                let style = Style::default().fg(Color::Gray);
                buf.set_string(x, y, empty_msg, style);
            }
            return;
        }

        // Build list items
        let block_items: Vec<ListItem> = self
            .blocks
            .iter()
            .enumerate()
            .map(|(i, block)| {
                let is_selected = state.selected_index == Some(i);
                let selection_indicator = if is_selected { "▶" } else { "⬚" };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw(format!("{} ", selection_indicator)),
                        Span::styled(
                            block.id.to_string(),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("               "),
                        Span::styled(
                            format!("{} txns", block.txn_count),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(&block.timestamp, Style::default().fg(Color::Gray)),
                    ]),
                    Line::from(""),
                ])
                .style(if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                })
            })
            .collect();

        let items_per_page = area.height as usize / self.item_height as usize;
        let block_scroll_usize = state.scroll_position as usize / self.item_height as usize;
        let start_index = block_scroll_usize.min(self.blocks.len().saturating_sub(1));
        let end_index = (start_index + items_per_page).min(self.blocks.len());
        let visible_items = block_items[start_index..end_index].to_vec();

        // Create internal ListState for highlighting
        let mut list_state = ListState::default();
        if let Some(selected) = state.selected_index
            && selected >= start_index
            && selected < end_index
        {
            list_state.select(Some(selected - start_index));
        }

        let block_list = List::new(visible_items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        StatefulWidget::render(block_list, area, buf, &mut list_state);

        // Render scrollbar if focused and content exceeds viewport
        if self.focused && self.blocks.len() > items_per_page {
            render_list_scrollbar(
                area,
                buf,
                self.blocks.len(),
                self.item_height as usize,
                items_per_page,
                state.scroll_position as usize,
            );
        }
    }
}

// ============================================================================
// TransactionListWidget State & Widget
// ============================================================================

/// State for the transaction list widget.
///
/// This state tracks the currently selected transaction index and scroll position,
/// allowing the widget to maintain its state across renders.
#[derive(Debug, Default, Clone)]
pub struct TransactionListState {
    /// Currently selected transaction index in the list.
    pub selected_index: Option<usize>,
    /// Scroll position (in pixels/rows).
    pub scroll_position: u16,
}

impl TransactionListState {
    /// Creates a new `TransactionListState` with no selection.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected_index: None,
            scroll_position: 0,
        }
    }

    /// Creates a new `TransactionListState` with the given selection.
    #[must_use]
    pub const fn with_selection(index: usize) -> Self {
        Self {
            selected_index: Some(index),
            scroll_position: 0,
        }
    }

    /// Sets the selected index.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected_index = index;
    }

    /// Returns the currently selected index.
    #[must_use]
    pub const fn selected(&self) -> Option<usize> {
        self.selected_index
    }
}

/// A widget that displays a list of transactions with selection and scrolling.
///
/// This widget implements `StatefulWidget` and requires a `TransactionListState`
/// to track selection and scroll position.
///
/// # Example
///
/// ```text
/// ┌─ Latest Transactions ───────────────┐
/// │ ▶ ABCD...WXYZ            [Payment]  │
/// │   From: SENDER...ADDR               │
/// │   To:   RECEIVER...ADDR             │
/// │                                     │
/// │ → EFGH...STUV            [App Call] │
/// │   From: CALLER...ADDR               │
/// │   To:   APP#12345                   │
/// │                                     │
/// └─────────────────────────────────────┘
/// ```
#[derive(Debug)]
pub struct TransactionListWidget<'a> {
    /// Slice of transactions to display.
    transactions: &'a [Transaction],
    /// Whether this widget is currently focused.
    focused: bool,
    /// Height of each transaction item in rows.
    item_height: u16,
}

impl<'a> TransactionListWidget<'a> {
    /// Height of each transaction item in the list (in rows).
    pub const DEFAULT_ITEM_HEIGHT: u16 = 4;

    /// Creates a new `TransactionListWidget` with the given transactions.
    #[must_use]
    pub const fn new(transactions: &'a [Transaction]) -> Self {
        Self {
            transactions,
            focused: false,
            item_height: Self::DEFAULT_ITEM_HEIGHT,
        }
    }

    /// Sets whether this widget is focused.
    #[must_use]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the item height.
    #[must_use]
    pub const fn item_height(mut self, height: u16) -> Self {
        self.item_height = height;
        self
    }

    /// Returns the number of transactions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Returns true if there are no transactions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
}

impl StatefulWidget for TransactionListWidget<'_> {
    type State = TransactionListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Handle empty state
        if self.transactions.is_empty() {
            let empty_msg = "No transactions available";
            let x = area.x + (area.width.saturating_sub(empty_msg.len() as u16)) / 2;
            let y = area.y + area.height / 2;

            if y < area.y + area.height && x < area.x + area.width {
                let style = Style::default().fg(Color::Gray);
                buf.set_string(x, y, empty_msg, style);
            }
            return;
        }

        // Build list items
        let txn_items: Vec<ListItem> = self
            .transactions
            .iter()
            .enumerate()
            .map(|(i, txn)| {
                let is_selected = state.selected_index == Some(i);
                let txn_type_str = txn.txn_type.as_str();
                let entity_type_style = Style::default().fg(txn.txn_type.color());
                let selection_indicator = if is_selected { "▶" } else { "→" };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw(format!("{} ", selection_indicator)),
                        Span::styled(
                            txn.id.clone(),
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("          "),
                        Span::styled(format!("[{}]", txn_type_str), entity_type_style),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled("From: ", Style::default().fg(Color::Gray)),
                        Span::styled(txn.from.clone(), Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled("To:   ", Style::default().fg(Color::Gray)),
                        Span::styled(txn.to.clone(), Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(""),
                ])
                .style(if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                })
            })
            .collect();

        let items_per_page = area.height as usize / self.item_height as usize;
        let txn_scroll_usize = state.scroll_position as usize / self.item_height as usize;
        let start_index = txn_scroll_usize.min(self.transactions.len().saturating_sub(1));
        let end_index = (start_index + items_per_page).min(self.transactions.len());

        let visible_items = if start_index < end_index {
            txn_items[start_index..end_index].to_vec()
        } else {
            Vec::new()
        };

        // Create internal ListState for highlighting
        let mut list_state = ListState::default();
        if let Some(selected) = state.selected_index
            && selected >= start_index
            && selected < end_index
        {
            list_state.select(Some(selected - start_index));
        }

        let txn_list = List::new(visible_items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        StatefulWidget::render(txn_list, area, buf, &mut list_state);

        // Render scrollbar if focused and content exceeds viewport
        if self.focused && self.transactions.len() > items_per_page {
            render_list_scrollbar(
                area,
                buf,
                self.transactions.len(),
                self.item_height as usize,
                items_per_page,
                state.scroll_position as usize,
            );
        }
    }
}

// ============================================================================
// Scrollbar Helper
// ============================================================================

/// Renders a scrollbar for a list widget.
///
/// This helper function renders a vertical scrollbar on the right side of the
/// given area, properly sized based on the content length and viewport.
fn render_list_scrollbar(
    area: Rect,
    buf: &mut Buffer,
    total_items: usize,
    item_height: usize,
    items_per_page: usize,
    scroll_position: usize,
) {
    if total_items <= items_per_page {
        return;
    }

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .symbols(scrollbar::VERTICAL)
        .track_symbol(None)
        .begin_symbol(None)
        .end_symbol(None)
        .style(Style::default().fg(Color::Gray))
        .track_style(Style::default().fg(Color::DarkGray));

    let content_length = total_items * item_height;

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(content_length)
        .viewport_content_length(items_per_page * item_height)
        .position(scroll_position);

    scrollbar.render(area, buf, &mut scrollbar_state);
}

// ============================================================================
// Transaction Graph Visualization (Lora-style)
// ============================================================================

/// Entity type for graph columns - matches algokit-lora's Vertical type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphEntityType {
    /// Account address column
    Account,
    /// Application ID column
    Application,
    /// Asset ID column
    Asset,
}

impl GraphEntityType {
    /// Get the column header prefix for this entity type
    #[must_use]
    pub const fn header_prefix(&self) -> &'static str {
        match self {
            Self::Account => "",
            Self::Application => "App #",
            Self::Asset => "ASA #",
        }
    }

    /// Get the header color for this entity type
    #[must_use]
    pub const fn header_color(&self) -> Color {
        match self {
            Self::Account => Color::Yellow,
            Self::Application => Color::Cyan,
            Self::Asset => Color::Magenta,
        }
    }
}

/// A vertical column in the graph representing an entity
#[derive(Debug, Clone)]
pub struct GraphColumn {
    /// Type of entity (Account, Application, Asset)
    pub entity_type: GraphEntityType,
    /// Entity identifier (address for accounts, ID for apps/assets)
    pub entity_id: String,
    /// Display label (truncated address or "App #123")
    pub label: String,
    /// Column index (0-based from left)
    pub index: usize,
}

impl GraphColumn {
    /// Create a new account column
    #[must_use]
    pub fn account(address: &str, index: usize, label_width: usize) -> Self {
        Self {
            entity_type: GraphEntityType::Account,
            entity_id: address.to_string(),
            label: truncate_address(address, label_width),
            index,
        }
    }

    /// Create a new application column
    #[must_use]
    pub fn application(app_id: u64, index: usize) -> Self {
        Self {
            entity_type: GraphEntityType::Application,
            entity_id: app_id.to_string(),
            label: format!("App #{}", app_id),
            index,
        }
    }

    /// Create a new asset column
    #[must_use]
    pub fn asset(asset_id: u64, index: usize) -> Self {
        Self {
            entity_type: GraphEntityType::Asset,
            entity_id: asset_id.to_string(),
            label: format!("ASA #{}", asset_id),
            index,
        }
    }
}

/// Visual representation type for a transaction - matches algokit-lora
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphRepresentation {
    /// Arrow between two different columns (sender → receiver)
    Vector,
    /// Curved arrow when sender = receiver (e.g., opt-in)
    SelfLoop,
    /// Single point marker (KeyReg, StateProof, Heartbeat)
    Point,
}

/// A horizontal row in the graph representing a transaction
#[derive(Debug, Clone)]
pub struct GraphRow {
    /// Transaction ID
    pub txn_id: String,
    /// Transaction type
    pub txn_type: TxnType,
    /// Source column index (None for Point type)
    pub from_col: Option<usize>,
    /// Target column index (None for Point type)
    pub to_col: Option<usize>,
    /// Visual representation type
    pub representation: GraphRepresentation,
    /// Row index (0-based from top)
    pub index: usize,
    /// Nesting depth for inner transactions (0 = top level)
    pub depth: usize,
    /// Parent transaction index (None if top level)
    pub parent_index: Option<usize>,
    /// Display label (amount, action, etc.)
    pub label: String,
    /// Whether this row has children (inner transactions)
    pub has_children: bool,
    /// Whether this is the last child in its parent group
    pub is_last_child: bool,
}

/// Complete transaction graph structure
#[derive(Debug, Clone)]
pub struct TxnGraph {
    /// Column definitions (entities)
    pub columns: Vec<GraphColumn>,
    /// Row definitions (transactions)
    pub rows: Vec<GraphRow>,
    /// Column width in characters
    pub column_width: usize,
    /// Spacing between columns
    pub column_spacing: usize,
}

impl TxnGraph {
    /// Default column width
    pub const DEFAULT_COLUMN_WIDTH: usize = 12;
    /// Default spacing between columns
    pub const DEFAULT_COLUMN_SPACING: usize = 8;

    /// Create a new empty graph
    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            column_width: Self::DEFAULT_COLUMN_WIDTH,
            column_spacing: Self::DEFAULT_COLUMN_SPACING,
        }
    }

    /// Set column width
    #[must_use]
    pub const fn with_column_width(mut self, width: usize) -> Self {
        self.column_width = width;
        self
    }

    /// Set column spacing
    #[must_use]
    pub const fn with_column_spacing(mut self, spacing: usize) -> Self {
        self.column_spacing = spacing;
        self
    }

    /// Build a graph from a single transaction (including inner transactions)
    #[must_use]
    pub fn from_transaction(txn: &Transaction) -> Self {
        let mut graph = Self::new();
        graph.add_transaction_recursive(txn, 0, None, false);
        graph.finalize_tree_structure();
        graph
    }

    /// Build a graph from multiple transactions (e.g., inner transactions)
    #[must_use]
    pub fn from_transactions(transactions: &[Transaction]) -> Self {
        let mut graph = Self::new();
        let total = transactions.len();
        for (i, txn) in transactions.iter().enumerate() {
            let is_last = i == total - 1;
            graph.add_transaction_recursive(txn, i, None, is_last);
        }
        graph.finalize_tree_structure();
        graph
    }

    /// Add a transaction to the graph (legacy method for backward compatibility)
    pub fn add_transaction(
        &mut self,
        txn: &Transaction,
        row_index: usize,
        parent_index: Option<usize>,
    ) {
        self.add_transaction_recursive(txn, row_index, parent_index, false);
    }

    /// Add a transaction and its inner transactions recursively to the graph
    fn add_transaction_recursive(
        &mut self,
        txn: &Transaction,
        _row_index: usize,
        parent_index: Option<usize>,
        is_last_child: bool,
    ) {
        let depth = parent_index.map_or(0, |p_idx| {
            // Find parent row and get its depth + 1
            self.rows.get(p_idx).map_or(1, |parent| parent.depth + 1)
        });

        // Determine representation and columns
        let (representation, from_col, to_col) = self.determine_representation(txn);

        // Create the row
        let label = self.create_row_label(txn);
        let has_children = !txn.inner_transactions.is_empty();
        let current_row_index = self.rows.len();

        let row = GraphRow {
            txn_id: txn.id.clone(),
            txn_type: txn.txn_type,
            from_col,
            to_col,
            representation,
            index: current_row_index,
            depth,
            parent_index,
            label,
            has_children,
            is_last_child,
        };

        self.rows.push(row);

        // Recursively add inner transactions
        let inner_count = txn.inner_transactions.len();
        for (i, inner_txn) in txn.inner_transactions.iter().enumerate() {
            let inner_is_last = i == inner_count - 1;
            self.add_transaction_recursive(inner_txn, i, Some(current_row_index), inner_is_last);
        }
    }

    /// Finalize tree structure by updating is_last_child flags based on siblings
    fn finalize_tree_structure(&mut self) {
        // Group rows by parent_index
        let mut children_by_parent: std::collections::HashMap<Option<usize>, Vec<usize>> =
            std::collections::HashMap::new();

        for (idx, row) in self.rows.iter().enumerate() {
            children_by_parent
                .entry(row.parent_index)
                .or_default()
                .push(idx);
        }

        // Mark last child in each group
        for children in children_by_parent.values() {
            if let Some(&last_idx) = children.last()
                && let Some(row) = self.rows.get_mut(last_idx)
            {
                row.is_last_child = true;
            }
        }
    }

    /// Determine visual representation and column indices for a transaction
    fn determine_representation(
        &mut self,
        txn: &Transaction,
    ) -> (GraphRepresentation, Option<usize>, Option<usize>) {
        match txn.txn_type {
            // Point representation for single-entity transactions
            TxnType::KeyReg | TxnType::StateProof | TxnType::Heartbeat => {
                let col = self.get_or_create_account_column(&txn.from);
                (GraphRepresentation::Point, Some(col), None)
            }

            // App calls: Account → Application
            TxnType::AppCall => {
                let from_col = self.get_or_create_account_column(&txn.from);
                if txn.to != "unknown" && txn.to != "0" && !txn.to.is_empty() {
                    if let Ok(app_id) = txn.to.parse::<u64>() {
                        let to_col = self.get_or_create_app_column(app_id);
                        if from_col == to_col {
                            (GraphRepresentation::SelfLoop, Some(from_col), Some(to_col))
                        } else {
                            (GraphRepresentation::Vector, Some(from_col), Some(to_col))
                        }
                    } else {
                        (GraphRepresentation::Point, Some(from_col), None)
                    }
                } else {
                    // App creation
                    (GraphRepresentation::Point, Some(from_col), None)
                }
            }

            // Asset config: May involve asset column
            TxnType::AssetConfig => {
                let from_col = self.get_or_create_account_column(&txn.from);
                if let Some(asset_id) = txn.asset_id {
                    let to_col = self.get_or_create_asset_column(asset_id);
                    (GraphRepresentation::Vector, Some(from_col), Some(to_col))
                } else {
                    (GraphRepresentation::Point, Some(from_col), None)
                }
            }

            // Asset freeze: Account → Account (frozen account)
            TxnType::AssetFreeze => {
                let from_col = self.get_or_create_account_column(&txn.from);
                if !txn.to.is_empty() && txn.to != txn.from {
                    let to_col = self.get_or_create_account_column(&txn.to);
                    (GraphRepresentation::Vector, Some(from_col), Some(to_col))
                } else {
                    (
                        GraphRepresentation::SelfLoop,
                        Some(from_col),
                        Some(from_col),
                    )
                }
            }

            // Payment and Asset Transfer: Account → Account
            TxnType::Payment | TxnType::AssetTransfer => {
                let from_col = self.get_or_create_account_column(&txn.from);
                if txn.to.is_empty() || txn.to == txn.from {
                    // Self-transfer (e.g., opt-in)
                    (
                        GraphRepresentation::SelfLoop,
                        Some(from_col),
                        Some(from_col),
                    )
                } else {
                    let to_col = self.get_or_create_account_column(&txn.to);
                    if from_col == to_col {
                        (GraphRepresentation::SelfLoop, Some(from_col), Some(to_col))
                    } else {
                        (GraphRepresentation::Vector, Some(from_col), Some(to_col))
                    }
                }
            }

            TxnType::Unknown => {
                let col = self.get_or_create_account_column(&txn.from);
                (GraphRepresentation::Point, Some(col), None)
            }
        }
    }

    /// Get or create an account column, returning its index
    fn get_or_create_account_column(&mut self, address: &str) -> usize {
        // Check if column exists
        for col in &self.columns {
            if col.entity_type == GraphEntityType::Account && col.entity_id == address {
                return col.index;
            }
        }

        // Create new column
        let index = self.columns.len();
        self.columns
            .push(GraphColumn::account(address, index, self.column_width));
        index
    }

    /// Get or create an application column, returning its index
    fn get_or_create_app_column(&mut self, app_id: u64) -> usize {
        let id_str = app_id.to_string();

        // Check if column exists
        for col in &self.columns {
            if col.entity_type == GraphEntityType::Application && col.entity_id == id_str {
                return col.index;
            }
        }

        // Create new column
        let index = self.columns.len();
        self.columns.push(GraphColumn::application(app_id, index));
        index
    }

    /// Get or create an asset column, returning its index
    fn get_or_create_asset_column(&mut self, asset_id: u64) -> usize {
        let id_str = asset_id.to_string();

        // Check if column exists
        for col in &self.columns {
            if col.entity_type == GraphEntityType::Asset && col.entity_id == id_str {
                return col.index;
            }
        }

        // Create new column
        let index = self.columns.len();
        self.columns.push(GraphColumn::asset(asset_id, index));
        index
    }

    /// Create a display label for a transaction row
    fn create_row_label(&self, txn: &Transaction) -> String {
        match txn.txn_type {
            TxnType::Payment => {
                let algos = txn.amount as f64 / MICROALGOS_PER_ALGO;
                if algos >= 1.0 {
                    format!("{:.2}A", algos)
                } else if algos > 0.0 {
                    format!("{:.4}A", algos)
                } else {
                    "0A".to_string()
                }
            }
            TxnType::AssetTransfer => {
                if let Some(asset_id) = txn.asset_id {
                    if txn.amount == 0 && txn.from == txn.to {
                        format!("opt-in #{}", asset_id)
                    } else {
                        format!("{}", txn.amount)
                    }
                } else {
                    format!("{}", txn.amount)
                }
            }
            TxnType::AppCall => {
                if let TransactionDetails::AppCall(details) = &txn.details {
                    details.on_complete.as_str().to_string()
                } else {
                    "call".to_string()
                }
            }
            TxnType::AssetConfig => "config".to_string(),
            TxnType::AssetFreeze => "freeze".to_string(),
            TxnType::KeyReg => "keyreg".to_string(),
            TxnType::StateProof => "proof".to_string(),
            TxnType::Heartbeat => "beat".to_string(),
            TxnType::Unknown => "?".to_string(),
        }
    }

    /// Calculate total width needed for the graph
    #[must_use]
    pub fn total_width(&self) -> usize {
        if self.columns.is_empty() {
            return 0;
        }
        let num_cols = self.columns.len();
        num_cols * self.column_width + (num_cols.saturating_sub(1)) * self.column_spacing
    }

    /// Calculate the x position for a column center
    #[must_use]
    pub fn column_center_x(&self, col_index: usize) -> usize {
        col_index * (self.column_width + self.column_spacing) + self.column_width / 2
    }

    /// Calculate the x position for a column start
    #[must_use]
    pub fn column_start_x(&self, col_index: usize) -> usize {
        col_index * (self.column_width + self.column_spacing)
    }

    // ========================================================================
    // SVG Export
    // ========================================================================

    /// Export the transaction graph to SVG format
    ///
    /// Generates a standalone SVG file with:
    /// - Column headers with circled numbers (①②③)
    /// - Tree structure for nested transactions
    /// - Arrows for transfers between entities
    /// - Self-loops for self-transfers
    /// - Points for single-entity operations
    ///
    /// Uses Tokyo Night color scheme for consistency with TUI.
    #[must_use]
    pub fn to_svg(&self) -> String {
        if self.columns.is_empty() || self.rows.is_empty() {
            return Self::empty_svg();
        }

        // SVG dimensions and styling constants
        const ROW_HEIGHT: usize = 50;
        const HEADER_HEIGHT: usize = 80;
        const LABEL_WIDTH: usize = 180;
        const PADDING: usize = 20;
        const COL_WIDTH: usize = 100;
        const COL_SPACING: usize = 60;

        // Tokyo Night colors
        const BG_COLOR: &str = "#1a1b26";
        const TEXT_COLOR: &str = "#c0caf5";
        const HEADER_COLOR: &str = "#7aa2f7";
        const LABEL_COLOR: &str = "#9ece6a";
        const TREE_COLOR: &str = "#565f89";
        const ARROW_PAYMENT: &str = "#9ece6a";
        const ARROW_ASSET: &str = "#bb9af7";
        const ARROW_APPCALL: &str = "#7dcfff";
        const POINT_COLOR: &str = "#f7768e";
        const GRID_COLOR: &str = "#24283b";

        let num_cols = self.columns.len();
        let num_rows = self.rows.len();
        let graph_width = num_cols * COL_WIDTH + (num_cols.saturating_sub(1)) * COL_SPACING;
        let total_width = LABEL_WIDTH + graph_width + PADDING * 2;
        let total_height = HEADER_HEIGHT + num_rows * ROW_HEIGHT + PADDING * 2;

        let mut svg = String::new();

        // SVG header
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<defs>
  <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
    <polygon points="0 0, 10 3.5, 0 7" fill="{}"/>
  </marker>
  <marker id="arrowhead-asset" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
    <polygon points="0 0, 10 3.5, 0 7" fill="{}"/>
  </marker>
  <marker id="arrowhead-app" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
    <polygon points="0 0, 10 3.5, 0 7" fill="{}"/>
  </marker>
</defs>
<rect width="100%" height="100%" fill="{}"/>
"#,
            total_width,
            total_height,
            total_width,
            total_height,
            ARROW_PAYMENT,
            ARROW_ASSET,
            ARROW_APPCALL,
            BG_COLOR
        ));

        // Draw vertical grid lines for columns
        for (i, _col) in self.columns.iter().enumerate() {
            let x = LABEL_WIDTH + i * (COL_WIDTH + COL_SPACING) + COL_WIDTH / 2;
            svg.push_str(&format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1" stroke-dasharray="4,4" opacity="0.5"/>"#,
                x,
                HEADER_HEIGHT,
                x,
                total_height - PADDING,
                GRID_COLOR
            ));
            svg.push('\n');
        }

        // Draw column headers
        let circled_numbers = ["①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨", "⑩"];
        for (i, col) in self.columns.iter().enumerate() {
            let x = LABEL_WIDTH + i * (COL_WIDTH + COL_SPACING) + COL_WIDTH / 2;
            let y = PADDING + 20;

            // Circled number
            let num = if i < circled_numbers.len() {
                circled_numbers[i]
            } else {
                "⓪"
            };
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" fill="{}" font-family="monospace" font-size="16" text-anchor="middle">{}</text>"#,
                x, y, HEADER_COLOR, num
            ));
            svg.push('\n');

            // Entity label
            let label = Self::truncate_label(&col.label, 12);
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" fill="{}" font-family="monospace" font-size="12" text-anchor="middle">{}</text>"#,
                x,
                y + 20,
                TEXT_COLOR,
                Self::escape_xml(&label)
            ));
            svg.push('\n');

            // Entity type
            let type_label = match col.entity_type {
                GraphEntityType::Account => "Account",
                GraphEntityType::Application => "App",
                GraphEntityType::Asset => "Asset",
            };
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" fill="{}" font-family="monospace" font-size="10" text-anchor="middle" opacity="0.7">{}</text>"#,
                x,
                y + 35,
                TEXT_COLOR,
                type_label
            ));
            svg.push('\n');
        }

        // Draw rows
        for (row_idx, row) in self.rows.iter().enumerate() {
            let y = HEADER_HEIGHT + row_idx * ROW_HEIGHT + ROW_HEIGHT / 2;

            // Draw tree prefix
            let tree_prefix = self.build_tree_prefix(row);
            if !tree_prefix.is_empty() {
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" fill="{}" font-family="monospace" font-size="12">{}</text>"#,
                    PADDING,
                    y + 4,
                    TREE_COLOR,
                    Self::escape_xml(&tree_prefix)
                ));
                svg.push('\n');
            }

            // Draw row label (transaction type + details)
            let label = Self::truncate_label(&row.label, 20);
            let label_x = PADDING + row.depth * 20 + tree_prefix.chars().count() * 8;
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" fill="{}" font-family="monospace" font-size="11">{}</text>"#,
                label_x,
                y + 4,
                LABEL_COLOR,
                Self::escape_xml(&label)
            ));
            svg.push('\n');

            // Draw the graph element (arrow, self-loop, or point)
            match row.representation {
                GraphRepresentation::Vector => {
                    if let (Some(from), Some(to)) = (row.from_col, row.to_col) {
                        let x1 = LABEL_WIDTH + from * (COL_WIDTH + COL_SPACING) + COL_WIDTH / 2;
                        let x2 = LABEL_WIDTH + to * (COL_WIDTH + COL_SPACING) + COL_WIDTH / 2;

                        let arrow_color = match row.txn_type {
                            TxnType::Payment => ARROW_PAYMENT,
                            TxnType::AssetTransfer | TxnType::AssetConfig | TxnType::AssetFreeze => {
                                ARROW_ASSET
                            }
                            TxnType::AppCall => ARROW_APPCALL,
                            _ => ARROW_PAYMENT,
                        };

                        let marker_id = match row.txn_type {
                            TxnType::AssetTransfer | TxnType::AssetConfig | TxnType::AssetFreeze => {
                                "arrowhead-asset"
                            }
                            TxnType::AppCall => "arrowhead-app",
                            _ => "arrowhead",
                        };

                        svg.push_str(&format!(
                            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="2" marker-end="url(#{})"/>"#,
                            x1, y, x2, y, arrow_color, marker_id
                        ));
                        svg.push('\n');
                    }
                }
                GraphRepresentation::SelfLoop => {
                    if let Some(col) = row.from_col {
                        let cx = LABEL_WIDTH + col * (COL_WIDTH + COL_SPACING) + COL_WIDTH / 2;
                        let arrow_color = match row.txn_type {
                            TxnType::Payment => ARROW_PAYMENT,
                            TxnType::AssetTransfer => ARROW_ASSET,
                            TxnType::AppCall => ARROW_APPCALL,
                            _ => ARROW_PAYMENT,
                        };

                        // Draw a small loop arc
                        svg.push_str(&format!(
                            r#"<path d="M {} {} C {} {} {} {} {} {}" fill="none" stroke="{}" stroke-width="2"/>"#,
                            cx,
                            y - 5,
                            cx + 25,
                            y - 25,
                            cx + 25,
                            y + 25,
                            cx,
                            y + 5,
                            arrow_color
                        ));
                        svg.push('\n');

                        // Small arrow at the end
                        svg.push_str(&format!(
                            r#"<polygon points="{},{} {},{} {},{}" fill="{}"/>"#,
                            cx,
                            y + 5,
                            cx + 6,
                            y + 10,
                            cx + 6,
                            y,
                            arrow_color
                        ));
                        svg.push('\n');
                    }
                }
                GraphRepresentation::Point => {
                    if let Some(col) = row.from_col {
                        let cx = LABEL_WIDTH + col * (COL_WIDTH + COL_SPACING) + COL_WIDTH / 2;
                        svg.push_str(&format!(
                            r#"<circle cx="{}" cy="{}" r="6" fill="{}"/>"#,
                            cx, y, POINT_COLOR
                        ));
                        svg.push('\n');
                    }
                }
            }
        }

        // Close SVG
        svg.push_str("</svg>\n");
        svg
    }

    /// Generate an empty SVG with a message
    fn empty_svg() -> String {
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 100" width="400" height="100">
<rect width="100%" height="100%" fill="#1a1b26"/>
<text x="200" y="50" fill="#c0caf5" font-family="monospace" font-size="14" text-anchor="middle">No graph data</text>
</svg>
"##
        .to_string()
    }

    /// Build tree prefix string for a row (├─, └─, │, etc.)
    fn build_tree_prefix(&self, row: &GraphRow) -> String {
        if row.depth == 0 {
            return String::new();
        }

        let mut prefix = String::new();

        // Build prefix based on ancestry
        for d in 1..row.depth {
            // Check if there's a sibling at this depth level
            let has_sibling = self.has_sibling_at_depth(row, d);
            if has_sibling {
                prefix.push_str("│ ");
            } else {
                prefix.push_str("  ");
            }
        }

        // Add connector for current level
        if row.is_last_child {
            prefix.push_str("└─");
        } else {
            prefix.push_str("├─");
        }

        prefix
    }

    /// Check if a row has siblings at a given depth level
    fn has_sibling_at_depth(&self, row: &GraphRow, depth: usize) -> bool {
        // Find the ancestor at the given depth
        let mut ancestor_idx = row.parent_index;
        let mut current_depth = row.depth - 1;

        while current_depth > depth {
            if let Some(idx) = ancestor_idx {
                if let Some(ancestor) = self.rows.get(idx) {
                    ancestor_idx = ancestor.parent_index;
                    current_depth -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Check if ancestor has more children after this row's branch
        if let Some(idx) = ancestor_idx {
            for (i, r) in self.rows.iter().enumerate() {
                if i > row.index && r.parent_index == Some(idx) {
                    return true;
                }
            }
        }

        false
    }

    /// Truncate a label to max length with ellipsis
    fn truncate_label(label: &str, max_len: usize) -> String {
        if label.len() <= max_len {
            label.to_string()
        } else {
            format!("{}…", &label[..max_len - 1])
        }
    }

    /// Escape special XML characters
    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

impl Default for TxnGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TxnGraphWidget - ASCII Renderer
// ============================================================================

/// Widget that renders a transaction graph as ASCII art
#[derive(Debug, Clone)]
pub struct TxnGraphWidget<'a> {
    graph: &'a TxnGraph,
    /// Show column headers
    show_headers: bool,
    /// Show row labels (amounts, actions)
    show_labels: bool,
    /// Center the graph in the available area
    center: bool,
}

impl<'a> TxnGraphWidget<'a> {
    /// Row height for transactions
    const ROW_HEIGHT: usize = 2;
    /// Header height (including separator)
    const HEADER_HEIGHT: usize = 3;

    /// Create a new graph widget
    #[must_use]
    pub const fn new(graph: &'a TxnGraph) -> Self {
        Self {
            graph,
            show_headers: true,
            show_labels: true,
            center: true,
        }
    }

    /// Hide column headers
    #[must_use]
    pub const fn without_headers(mut self) -> Self {
        self.show_headers = false;
        self
    }

    /// Hide row labels
    #[must_use]
    pub const fn without_labels(mut self) -> Self {
        self.show_labels = false;
        self
    }

    /// Disable centering
    #[must_use]
    pub const fn without_centering(mut self) -> Self {
        self.center = false;
        self
    }

    /// Generate lines for the graph
    #[must_use]
    pub fn to_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        if self.graph.columns.is_empty() || self.graph.rows.is_empty() {
            lines.push(Line::from(Span::styled(
                "No graph data",
                Style::default().fg(Color::Gray),
            )));
            return lines;
        }

        // Calculate max tree prefix width for alignment
        let max_prefix_width = self.calculate_max_prefix_width();

        // Render column headers with prefix padding
        if self.show_headers {
            lines.extend(self.render_headers_with_padding(max_prefix_width));
        }

        // Render each transaction row
        for row in &self.graph.rows {
            lines.extend(self.render_row_with_padding(row, max_prefix_width));
        }

        lines
    }

    /// Calculate the maximum tree prefix width across all rows
    fn calculate_max_prefix_width(&self) -> usize {
        self.graph
            .rows
            .iter()
            .map(|row| {
                if row.depth == 0 {
                    0
                } else {
                    // Each depth level adds 3 characters ("│  " or "├──" etc.)
                    row.depth * 3
                }
            })
            .max()
            .unwrap_or(0)
    }

    /// Render column headers with consistent padding for tree prefix alignment
    fn render_headers_with_padding(&self, prefix_padding: usize) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let col_width = self.graph.column_width;
        let col_spacing = self.graph.column_spacing;

        // Header labels row with padding
        let mut header_spans = Vec::new();

        // Add padding to match tree prefix width
        if prefix_padding > 0 {
            header_spans.push(Span::raw(" ".repeat(prefix_padding)));
        }

        for (i, col) in self.graph.columns.iter().enumerate() {
            if i > 0 {
                header_spans.push(Span::raw(" ".repeat(col_spacing)));
            }

            // Center the label in the column
            let label = &col.label;
            let label_len = label.chars().count();
            let padding_total = col_width.saturating_sub(label_len);
            let padding_left = padding_total / 2;
            let padding_right = padding_total - padding_left;

            let padded_label = format!(
                "{}{}{}",
                " ".repeat(padding_left),
                label,
                " ".repeat(padding_right)
            );

            header_spans.push(Span::styled(
                padded_label,
                Style::default()
                    .fg(col.entity_type.header_color())
                    .add_modifier(Modifier::BOLD),
            ));
        }
        lines.push(Line::from(header_spans));

        // Separator line with column markers
        let mut sep_spans = Vec::new();

        // Add padding to match tree prefix width
        if prefix_padding > 0 {
            sep_spans.push(Span::raw(" ".repeat(prefix_padding)));
        }

        for (i, _col) in self.graph.columns.iter().enumerate() {
            if i > 0 {
                sep_spans.push(Span::raw(" ".repeat(col_spacing)));
            }

            // Create a centered marker
            let marker_padding = col_width / 2;
            let marker = format!(
                "{}│{}",
                " ".repeat(marker_padding),
                " ".repeat(col_width.saturating_sub(marker_padding + 1))
            );
            sep_spans.push(Span::styled(marker, Style::default().fg(Color::DarkGray)));
        }
        lines.push(Line::from(sep_spans));

        // Empty line for spacing
        lines.push(Line::from(""));

        lines
    }

    /// Render a single transaction row with consistent padding
    fn render_row_with_padding(
        &self,
        row: &GraphRow,
        max_prefix_width: usize,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let color = row.txn_type.color();

        // Generate tree prefix based on depth and position
        let tree_prefix = self.generate_tree_prefix(row);
        let prefix_len = tree_prefix.chars().count();

        // Calculate padding needed to align with max prefix width
        let extra_padding = max_prefix_width.saturating_sub(prefix_len);
        let full_prefix = format!("{}{}", " ".repeat(extra_padding), tree_prefix);

        match row.representation {
            GraphRepresentation::Vector => {
                if let (Some(from_col), Some(to_col)) = (row.from_col, row.to_col) {
                    lines.push(self.render_vector_line(
                        from_col,
                        to_col,
                        color,
                        &row.label,
                        &full_prefix,
                    ));
                }
            }
            GraphRepresentation::SelfLoop => {
                if let Some(col) = row.from_col {
                    lines.push(self.render_self_loop_line(col, color, &row.label, &full_prefix));
                }
            }
            GraphRepresentation::Point => {
                if let Some(col) = row.from_col {
                    lines.push(self.render_point_line(col, color, &row.label, &full_prefix));
                }
            }
        }

        // Add spacing between rows (also with prefix padding for consistency)
        let spacing_prefix = " ".repeat(max_prefix_width);
        lines.push(Line::from(Span::raw(spacing_prefix)));

        lines
    }

    /// Generate tree prefix characters for inner transaction nesting
    fn generate_tree_prefix(&self, row: &GraphRow) -> String {
        if row.depth == 0 {
            return String::new();
        }

        // Simplified tree prefix - just use indentation based on depth
        // Each depth level adds 2 spaces for visual nesting
        " ".repeat(row.depth * 2)
    }

    /// Render a vector (arrow) between two columns
    fn render_vector_line(
        &self,
        from_col: usize,
        to_col: usize,
        color: Color,
        label: &str,
        tree_prefix: &str,
    ) -> Line<'static> {
        let col_width = self.graph.column_width;
        let col_spacing = self.graph.column_spacing;
        let total_cols = self.graph.columns.len();

        let mut spans = Vec::new();

        // Add tree prefix if present
        if !tree_prefix.is_empty() {
            spans.push(Span::raw(tree_prefix.to_string()));
        }

        let (left_col, right_col, is_left_to_right) = if from_col <= to_col {
            (from_col, to_col, true)
        } else {
            (to_col, from_col, false)
        };

        let center = col_width / 2;

        for col_idx in 0..total_cols {
            if col_idx > 0 {
                // Spacing between columns
                if col_idx > left_col && col_idx <= right_col {
                    // Draw arrow line through spacing
                    spans.push(Span::styled(
                        "─".repeat(col_spacing),
                        Style::default().fg(color),
                    ));
                } else {
                    spans.push(Span::raw(" ".repeat(col_spacing)));
                }
            }

            // Column content
            if col_idx == left_col && col_idx == right_col {
                // Self-reference (should not happen for Vector, but handle it)
                let col_content = format!(
                    "{}●{}",
                    " ".repeat(center),
                    " ".repeat(col_width.saturating_sub(center + 1))
                );
                spans.push(Span::styled(col_content, Style::default().fg(color)));
            } else if col_idx == left_col {
                // Start of arrow - marker at center, then line to the right
                let marker = if is_left_to_right { "●" } else { "◀" };
                let left_padding = " ".repeat(center);
                let right_fill = "─".repeat(col_width.saturating_sub(center + 1));
                let col_content = format!("{}{}{}", left_padding, marker, right_fill);
                spans.push(Span::styled(col_content, Style::default().fg(color)));
            } else if col_idx == right_col {
                // End of arrow - line from left, then marker at center
                let marker = if is_left_to_right { "▶" } else { "●" };
                let left_fill = "─".repeat(center);
                let right_padding = " ".repeat(col_width.saturating_sub(center + 1));
                let col_content = format!("{}{}{}", left_fill, marker, right_padding);
                spans.push(Span::styled(col_content, Style::default().fg(color)));
            } else if col_idx > left_col && col_idx < right_col {
                // Middle column - draw line through entire width
                let col_content = "─".repeat(col_width);
                spans.push(Span::styled(col_content, Style::default().fg(color)));
            } else {
                // Non-participating column - show vertical column marker
                let col_content = format!(
                    "{}│{}",
                    " ".repeat(center),
                    " ".repeat(col_width.saturating_sub(center + 1))
                );
                spans.push(Span::styled(
                    col_content,
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }

        // Add label if showing
        if self.show_labels && !label.is_empty() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(label.to_string(), Style::default().fg(color)));
        }

        Line::from(spans)
    }

    /// Render a self-loop (curved arrow to same column)
    fn render_self_loop_line(
        &self,
        col: usize,
        color: Color,
        label: &str,
        tree_prefix: &str,
    ) -> Line<'static> {
        let col_width = self.graph.column_width;
        let col_spacing = self.graph.column_spacing;
        let total_cols = self.graph.columns.len();

        let mut spans = Vec::new();

        // Add tree prefix if present
        if !tree_prefix.is_empty() {
            spans.push(Span::styled(
                tree_prefix.to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let center = col_width / 2;

        for col_idx in 0..total_cols {
            if col_idx > 0 {
                spans.push(Span::raw(" ".repeat(col_spacing)));
            }

            if col_idx == col {
                // Self-loop marker: ↺
                let col_content = format!(
                    "{}↺{}",
                    " ".repeat(center),
                    " ".repeat(col_width.saturating_sub(center + 1))
                );
                spans.push(Span::styled(col_content, Style::default().fg(color)));
            } else {
                // Non-participating column - show vertical column marker
                let col_content = format!(
                    "{}│{}",
                    " ".repeat(center),
                    " ".repeat(col_width.saturating_sub(center + 1))
                );
                spans.push(Span::styled(
                    col_content,
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }

        // Add label if showing
        if self.show_labels && !label.is_empty() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(label.to_string(), Style::default().fg(color)));
        }

        Line::from(spans)
    }

    /// Render a point (single marker)
    fn render_point_line(
        &self,
        col: usize,
        color: Color,
        label: &str,
        tree_prefix: &str,
    ) -> Line<'static> {
        let col_width = self.graph.column_width;
        let col_spacing = self.graph.column_spacing;
        let total_cols = self.graph.columns.len();

        let mut spans = Vec::new();

        // Add tree prefix if present
        if !tree_prefix.is_empty() {
            spans.push(Span::styled(
                tree_prefix.to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let center = col_width / 2;

        for col_idx in 0..total_cols {
            if col_idx > 0 {
                spans.push(Span::raw(" ".repeat(col_spacing)));
            }

            if col_idx == col {
                // Point marker: ◉
                let col_content = format!(
                    "{}◉{}",
                    " ".repeat(center),
                    " ".repeat(col_width.saturating_sub(center + 1))
                );
                spans.push(Span::styled(col_content, Style::default().fg(color)));
            } else {
                // Non-participating column - show vertical column marker
                let col_content = format!(
                    "{}│{}",
                    " ".repeat(center),
                    " ".repeat(col_width.saturating_sub(center + 1))
                );
                spans.push(Span::styled(
                    col_content,
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }

        // Add label if showing
        if self.show_labels && !label.is_empty() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(label.to_string(), Style::default().fg(color)));
        }

        Line::from(spans)
    }

    /// Calculate required height for rendering
    #[must_use]
    pub fn required_height(&self) -> usize {
        let header_height = if self.show_headers {
            Self::HEADER_HEIGHT
        } else {
            0
        };
        let rows_height = self.graph.rows.len() * Self::ROW_HEIGHT;
        header_height + rows_height
    }
}

impl Widget for TxnGraphWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.to_lines();
        let total_width = self.graph.total_width();

        // Calculate horizontal centering offset
        let x_offset = if self.center && total_width < area.width as usize {
            ((area.width as usize - total_width) / 2) as u16
        } else {
            0
        };

        for (i, line) in lines.iter().enumerate() {
            if i >= area.height as usize {
                break;
            }
            let y = area.y + i as u16;
            let mut x = area.x + x_offset;

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
    use crate::algorand::TransactionDetails;

    #[test]
    fn test_truncate_address_short() {
        let addr = "ABCDEFGH";
        assert_eq!(truncate_address(addr, 20), "ABCDEFGH");
    }

    #[test]
    fn test_truncate_address_exact() {
        let addr = "ABCDEFGHIJ";
        assert_eq!(truncate_address(addr, 10), "ABCDEFGHIJ");
    }

    #[test]
    fn test_truncate_address_long() {
        let addr = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let result = truncate_address(addr, 20);
        assert_eq!(result.len(), 20);
        assert!(result.contains("..."));
        assert!(result.starts_with("AAAA"));
        assert!(result.ends_with("AAAA"));
    }

    #[test]
    fn test_truncate_address_very_short_max() {
        let addr = "ABCDEFGHIJ";
        let result = truncate_address(addr, 5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_format_algo_amount() {
        assert_eq!(format_algo_amount(0), "0.000000 ALGO");
        assert_eq!(format_algo_amount(1_000_000), "1.000000 ALGO");
        assert_eq!(format_algo_amount(5_500_000), "5.500000 ALGO");
        assert_eq!(format_algo_amount(123_456), "0.123456 ALGO");
    }

    #[test]
    fn test_format_asset_amount_no_decimals() {
        assert_eq!(format_asset_amount(1000, None), "1,000");
        assert_eq!(format_asset_amount(1_000_000, None), "1,000,000");
    }

    #[test]
    fn test_format_asset_amount_with_decimals() {
        assert_eq!(format_asset_amount(100_000_000, Some(6)), "100.000000");
        assert_eq!(format_asset_amount(1_500_000, Some(6)), "1.500000");
    }

    #[test]
    fn test_format_with_commas() {
        assert_eq!(format_with_commas(0), "0");
        assert_eq!(format_with_commas(999), "999");
        assert_eq!(format_with_commas(1000), "1,000");
        assert_eq!(format_with_commas(1_000_000), "1,000,000");
        assert_eq!(format_with_commas(1_234_567_890), "1,234,567,890");
    }

    #[test]
    fn test_txn_type_icon() {
        assert_eq!(txn_type_icon(TxnType::Payment), "[$]");
        assert_eq!(txn_type_icon(TxnType::AppCall), "[A]");
        assert_eq!(txn_type_icon(TxnType::AssetTransfer), "[>]");
        assert_eq!(txn_type_icon(TxnType::AssetConfig), "[*]");
        assert_eq!(txn_type_icon(TxnType::AssetFreeze), "[#]");
        assert_eq!(txn_type_icon(TxnType::KeyReg), "[K]");
        assert_eq!(txn_type_icon(TxnType::StateProof), "[S]");
        assert_eq!(txn_type_icon(TxnType::Heartbeat), "[H]");
        assert_eq!(txn_type_icon(TxnType::Unknown), "[?]");
    }

    #[test]
    fn test_txn_type_code() {
        assert_eq!(txn_type_code(TxnType::Payment), "PAY");
        assert_eq!(txn_type_code(TxnType::AppCall), "APP");
        assert_eq!(txn_type_code(TxnType::AssetTransfer), "AXF");
        assert_eq!(txn_type_code(TxnType::Unknown), "???");
    }

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
    fn test_address_display_with_label() {
        let display =
            AddressDisplay::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
                .with_label("From")
                .truncate(15);
        let line = display.to_line();
        assert_eq!(line.spans.len(), 2);
    }

    #[test]
    fn test_txn_flow_diagram() {
        let txn = Transaction {
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
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let diagram = TxnFlowDiagram::new(&txn);
        let lines = diagram.to_lines();
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn test_txn_visual_card() {
        let txn = Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER".to_string(),
            to: "RECEIVER".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 5_000_000,
            asset_id: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let card = TxnVisualCard::new(&txn);
        let lines = card.to_lines();
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_txn_visual_card_compact() {
        let txn = Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::AssetTransfer,
            from: "SENDER".to_string(),
            to: "RECEIVER".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 100,
            asset_id: Some(31566704),
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let card = TxnVisualCard::new(&txn).compact().without_flow();
        let lines = card.to_lines();
        assert!(!lines.is_empty());
    }

    // ========================================================================
    // BlockListWidget Tests
    // ========================================================================

    fn create_sample_blocks() -> Vec<AlgoBlock> {
        vec![
            AlgoBlock {
                id: 12345678,
                txn_count: 15,
                timestamp: "Mon, 01 Jan 2024 12:00:00".to_string(),
            },
            AlgoBlock {
                id: 12345677,
                txn_count: 8,
                timestamp: "Mon, 01 Jan 2024 11:59:55".to_string(),
            },
            AlgoBlock {
                id: 12345676,
                txn_count: 22,
                timestamp: "Mon, 01 Jan 2024 11:59:50".to_string(),
            },
        ]
    }

    #[test]
    fn test_block_list_state_new() {
        let state = BlockListState::new();
        assert!(state.selected_index.is_none());
        assert_eq!(state.scroll_position, 0);
    }

    #[test]
    fn test_block_list_state_with_selection() {
        let state = BlockListState::with_selection(2);
        assert_eq!(state.selected_index, Some(2));
        assert_eq!(state.scroll_position, 0);
    }

    #[test]
    fn test_block_list_state_select() {
        let mut state = BlockListState::new();
        state.select(Some(5));
        assert_eq!(state.selected(), Some(5));

        state.select(None);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn test_block_list_widget_new() {
        let blocks = create_sample_blocks();
        let widget = BlockListWidget::new(&blocks);

        assert_eq!(widget.len(), 3);
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_block_list_widget_empty() {
        let blocks: Vec<AlgoBlock> = vec![];
        let widget = BlockListWidget::new(&blocks);

        assert_eq!(widget.len(), 0);
        assert!(widget.is_empty());
    }

    #[test]
    fn test_block_list_widget_focused() {
        let blocks = create_sample_blocks();
        let widget = BlockListWidget::new(&blocks).focused(true);

        // Widget should be constructed without errors
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_block_list_widget_item_height() {
        let blocks = create_sample_blocks();
        let widget = BlockListWidget::new(&blocks).item_height(5);

        // Widget should be constructed without errors
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_block_list_widget_render_empty() {
        let blocks: Vec<AlgoBlock> = vec![];
        let widget = BlockListWidget::new(&blocks);
        let mut state = BlockListState::new();

        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render "No blocks available" message
        let content = buf_to_string(&buf);
        assert!(content.contains("No blocks available"));
    }

    #[test]
    fn test_block_list_widget_render_with_blocks() {
        let blocks = create_sample_blocks();
        let widget = BlockListWidget::new(&blocks);
        let mut state = BlockListState::new();

        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render block IDs
        let content = buf_to_string(&buf);
        assert!(content.contains("12345678"));
    }

    #[test]
    fn test_block_list_widget_render_with_selection() {
        let blocks = create_sample_blocks();
        let widget = BlockListWidget::new(&blocks);
        let mut state = BlockListState::with_selection(0);

        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render selection indicator for first item
        let content = buf_to_string(&buf);
        assert!(content.contains("▶")); // Selected indicator
    }

    // ========================================================================
    // TransactionListWidget Tests
    // ========================================================================

    fn create_sample_transactions() -> Vec<Transaction> {
        vec![
            Transaction {
                id: "TXID1ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCD".to_string(),
                txn_type: TxnType::Payment,
                from: "SENDER1AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
                to: "RECEIVER1BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".to_string(),
                timestamp: "Mon, 01 Jan 2024 12:00:00".to_string(),
                block: 12345678,
                fee: 1000,
                note: "Test payment".to_string(),
                amount: 5_000_000,
                asset_id: None,
                details: TransactionDetails::default(),
                inner_transactions: Vec::new(),
            },
            Transaction {
                id: "TXID2ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890EFGH".to_string(),
                txn_type: TxnType::AssetTransfer,
                from: "SENDER2CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC".to_string(),
                to: "RECEIVER2DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD".to_string(),
                timestamp: "Mon, 01 Jan 2024 11:59:55".to_string(),
                block: 12345677,
                fee: 1000,
                note: "".to_string(),
                amount: 100,
                asset_id: Some(31566704),
                details: TransactionDetails::default(),
                inner_transactions: Vec::new(),
            },
            Transaction {
                id: "TXID3ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890IJKL".to_string(),
                txn_type: TxnType::AppCall,
                from: "CALLER1EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE".to_string(),
                to: "123456".to_string(),
                timestamp: "Mon, 01 Jan 2024 11:59:50".to_string(),
                block: 12345676,
                fee: 2000,
                note: "".to_string(),
                amount: 0,
                asset_id: None,
                details: TransactionDetails::default(),
                inner_transactions: Vec::new(),
            },
        ]
    }

    #[test]
    fn test_transaction_list_state_new() {
        let state = TransactionListState::new();
        assert!(state.selected_index.is_none());
        assert_eq!(state.scroll_position, 0);
    }

    #[test]
    fn test_transaction_list_state_with_selection() {
        let state = TransactionListState::with_selection(1);
        assert_eq!(state.selected_index, Some(1));
        assert_eq!(state.scroll_position, 0);
    }

    #[test]
    fn test_transaction_list_state_select() {
        let mut state = TransactionListState::new();
        state.select(Some(3));
        assert_eq!(state.selected(), Some(3));

        state.select(None);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn test_transaction_list_widget_new() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions);

        assert_eq!(widget.len(), 3);
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_transaction_list_widget_empty() {
        let transactions: Vec<Transaction> = vec![];
        let widget = TransactionListWidget::new(&transactions);

        assert_eq!(widget.len(), 0);
        assert!(widget.is_empty());
    }

    #[test]
    fn test_transaction_list_widget_focused() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions).focused(true);

        // Widget should be constructed without errors
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_transaction_list_widget_item_height() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions).item_height(6);

        // Widget should be constructed without errors
        assert!(!widget.is_empty());
    }

    #[test]
    fn test_transaction_list_widget_render_empty() {
        let transactions: Vec<Transaction> = vec![];
        let widget = TransactionListWidget::new(&transactions);
        let mut state = TransactionListState::new();

        let area = Rect::new(0, 0, 80, 20);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render "No transactions available" message
        let content = buf_to_string(&buf);
        assert!(content.contains("No transactions available"));
    }

    #[test]
    fn test_transaction_list_widget_render_with_transactions() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions);
        let mut state = TransactionListState::new();

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render transaction info
        let content = buf_to_string(&buf);
        assert!(content.contains("From:"));
        assert!(content.contains("To:"));
    }

    #[test]
    fn test_transaction_list_widget_render_with_selection() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions);
        let mut state = TransactionListState::with_selection(0);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should render selection indicator for first item
        let content = buf_to_string(&buf);
        assert!(content.contains("▶")); // Selected indicator
    }

    #[test]
    fn test_transaction_list_widget_render_with_different_types() {
        let transactions = create_sample_transactions();
        let widget = TransactionListWidget::new(&transactions);
        let mut state = TransactionListState::new();

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        widget.render(area, &mut buf, &mut state);

        // Should show different transaction types
        let content = buf_to_string(&buf);
        // Payment and other types should be rendered with their badges
        assert!(content.contains("[Payment]") || content.contains("Payment"));
    }

    // ========================================================================
    // State Update Tests
    // ========================================================================

    #[test]
    fn test_block_list_state_updates() {
        let mut state = BlockListState::new();

        // Initial state
        assert!(state.selected().is_none());

        // Select first item
        state.select(Some(0));
        assert_eq!(state.selected(), Some(0));

        // Update scroll position
        state.scroll_position = 6;
        assert_eq!(state.scroll_position, 6);

        // Select different item
        state.select(Some(2));
        assert_eq!(state.selected(), Some(2));

        // Clear selection
        state.select(None);
        assert!(state.selected().is_none());
    }

    #[test]
    fn test_transaction_list_state_updates() {
        let mut state = TransactionListState::new();

        // Initial state
        assert!(state.selected().is_none());

        // Select first item
        state.select(Some(0));
        assert_eq!(state.selected(), Some(0));

        // Update scroll position
        state.scroll_position = 8;
        assert_eq!(state.scroll_position, 8);

        // Select different item
        state.select(Some(5));
        assert_eq!(state.selected(), Some(5));

        // Clear selection
        state.select(None);
        assert!(state.selected().is_none());
    }

    // Helper function to convert buffer to string for testing
    fn buf_to_string(buf: &Buffer) -> String {
        let area = buf.area;
        let mut result = String::new();

        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell((x, y)) {
                    result.push_str(cell.symbol());
                }
            }
            result.push('\n');
        }

        result
    }

    // ========================================================================
    // TxnGraph Tests
    // ========================================================================

    #[test]
    fn test_graph_entity_type_header_prefix() {
        assert_eq!(GraphEntityType::Account.header_prefix(), "");
        assert_eq!(GraphEntityType::Application.header_prefix(), "App #");
        assert_eq!(GraphEntityType::Asset.header_prefix(), "ASA #");
    }

    #[test]
    fn test_graph_column_account() {
        let col = GraphColumn::account(
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            0,
            12,
        );
        assert_eq!(col.entity_type, GraphEntityType::Account);
        assert_eq!(col.index, 0);
        assert!(col.label.len() <= 12);
    }

    #[test]
    fn test_graph_column_application() {
        let col = GraphColumn::application(12345, 1);
        assert_eq!(col.entity_type, GraphEntityType::Application);
        assert_eq!(col.entity_id, "12345");
        assert_eq!(col.label, "App #12345");
        assert_eq!(col.index, 1);
    }

    #[test]
    fn test_graph_column_asset() {
        let col = GraphColumn::asset(31566704, 2);
        assert_eq!(col.entity_type, GraphEntityType::Asset);
        assert_eq!(col.entity_id, "31566704");
        assert_eq!(col.label, "ASA #31566704");
        assert_eq!(col.index, 2);
    }

    #[test]
    fn test_txn_graph_new() {
        let graph = TxnGraph::new();
        assert!(graph.columns.is_empty());
        assert!(graph.rows.is_empty());
        assert_eq!(graph.column_width, TxnGraph::DEFAULT_COLUMN_WIDTH);
        assert_eq!(graph.column_spacing, TxnGraph::DEFAULT_COLUMN_SPACING);
    }

    #[test]
    fn test_txn_graph_with_column_width() {
        let graph = TxnGraph::new().with_column_width(20);
        assert_eq!(graph.column_width, 20);
    }

    #[test]
    fn test_txn_graph_with_column_spacing() {
        let graph = TxnGraph::new().with_column_spacing(12);
        assert_eq!(graph.column_spacing, 12);
    }

    #[test]
    fn test_txn_graph_from_payment_transaction() {
        let txn = Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER_ADDRESS".to_string(),
            to: "RECEIVER_ADDRESS".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 5_000_000,
            asset_id: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let graph = TxnGraph::from_transaction(&txn);

        // Should have 2 columns (sender and receiver)
        assert_eq!(graph.columns.len(), 2);
        // Should have 1 row
        assert_eq!(graph.rows.len(), 1);
        // Should be Vector representation (arrow between two columns)
        assert_eq!(graph.rows[0].representation, GraphRepresentation::Vector);
    }

    #[test]
    fn test_txn_graph_from_self_transfer() {
        let same_address = "SAME_ADDRESS";
        let txn = Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::Payment,
            from: same_address.to_string(),
            to: same_address.to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 0,
            asset_id: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let graph = TxnGraph::from_transaction(&txn);

        // Should have 1 column (same sender and receiver)
        assert_eq!(graph.columns.len(), 1);
        // Should be SelfLoop representation
        assert_eq!(graph.rows[0].representation, GraphRepresentation::SelfLoop);
    }

    #[test]
    fn test_txn_graph_from_keyreg_transaction() {
        let txn = Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::KeyReg,
            from: "SENDER_ADDRESS".to_string(),
            to: "".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 0,
            asset_id: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let graph = TxnGraph::from_transaction(&txn);

        // Should have 1 column (just sender)
        assert_eq!(graph.columns.len(), 1);
        // Should be Point representation
        assert_eq!(graph.rows[0].representation, GraphRepresentation::Point);
    }

    #[test]
    fn test_txn_graph_total_width() {
        let txn = Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER".to_string(),
            to: "RECEIVER".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 5_000_000,
            asset_id: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let graph = TxnGraph::from_transaction(&txn);

        // 2 columns of width 12 + 1 spacing of 8 = 32
        let expected_width = 2 * TxnGraph::DEFAULT_COLUMN_WIDTH + TxnGraph::DEFAULT_COLUMN_SPACING;
        assert_eq!(graph.total_width(), expected_width);
    }

    #[test]
    fn test_txn_graph_column_center_x() {
        let graph = TxnGraph::new();

        // First column: center at column_width/2 = 6
        assert_eq!(graph.column_center_x(0), TxnGraph::DEFAULT_COLUMN_WIDTH / 2);

        // Second column: center at column_width + spacing + column_width/2 = 12 + 8 + 6 = 26
        let second_col_center = TxnGraph::DEFAULT_COLUMN_WIDTH
            + TxnGraph::DEFAULT_COLUMN_SPACING
            + TxnGraph::DEFAULT_COLUMN_WIDTH / 2;
        assert_eq!(graph.column_center_x(1), second_col_center);
    }

    #[test]
    fn test_txn_graph_widget_to_lines() {
        let txn = Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER".to_string(),
            to: "RECEIVER".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 5_000_000,
            asset_id: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let graph = TxnGraph::from_transaction(&txn);
        let widget = TxnGraphWidget::new(&graph);
        let lines = widget.to_lines();

        // Should have lines (headers + rows)
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_txn_graph_widget_without_headers() {
        let txn = Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER".to_string(),
            to: "RECEIVER".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 5_000_000,
            asset_id: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let graph = TxnGraph::from_transaction(&txn);
        let widget_with_headers = TxnGraphWidget::new(&graph);
        let widget_without_headers = TxnGraphWidget::new(&graph).without_headers();

        let lines_with = widget_with_headers.to_lines();
        let lines_without = widget_without_headers.to_lines();

        // Without headers should have fewer lines
        assert!(lines_without.len() < lines_with.len());
    }

    #[test]
    fn test_txn_graph_widget_required_height() {
        let txn = Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER".to_string(),
            to: "RECEIVER".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 5_000_000,
            asset_id: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let graph = TxnGraph::from_transaction(&txn);
        let widget = TxnGraphWidget::new(&graph);

        // Header height (3) + 1 row * row height (2) = 5
        assert_eq!(widget.required_height(), 5);
    }

    #[test]
    fn test_txn_graph_empty() {
        let graph = TxnGraph::new();
        let widget = TxnGraphWidget::new(&graph);
        let lines = widget.to_lines();

        // Should show "No graph data" message
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_txn_graph_to_svg_empty() {
        let graph = TxnGraph::new();
        let svg = graph.to_svg();

        // Should produce valid SVG with "No graph data" message
        assert!(svg.contains("<?xml version"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("No graph data"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_txn_graph_to_svg_payment() {
        let txn = Transaction {
            id: "test-txn-id".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER".to_string(),
            to: "RECEIVER".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 5_000_000,
            asset_id: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let graph = TxnGraph::from_transaction(&txn);
        let svg = graph.to_svg();

        // Should produce valid SVG with transaction content
        assert!(svg.contains("<?xml version"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        // Should have column headers
        assert!(svg.contains("Account"));
        // Should have arrow marker definitions
        assert!(svg.contains("<marker"));
        assert!(svg.contains("arrowhead"));
        // Should have circled number for column
        assert!(svg.contains("①"));
    }

    #[test]
    fn test_txn_graph_to_svg_self_loop() {
        let txn = Transaction {
            id: "self-transfer".to_string(),
            txn_type: TxnType::Payment,
            from: "ACCOUNT1".to_string(),
            to: "ACCOUNT1".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 1_000_000,
            asset_id: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let graph = TxnGraph::from_transaction(&txn);
        let svg = graph.to_svg();

        // Should have a path for self-loop
        assert!(svg.contains("<path"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_txn_graph_to_svg_escapes_xml() {
        // Test XML escaping
        let escaped = TxnGraph::escape_xml("<script>&test</script>");
        assert_eq!(escaped, "&lt;script&gt;&amp;test&lt;/script&gt;");
    }

    #[test]
    fn test_txn_graph_truncate_label() {
        assert_eq!(TxnGraph::truncate_label("short", 10), "short");
        assert_eq!(TxnGraph::truncate_label("a_very_long_label", 10), "a_very_lo…");
    }
}
