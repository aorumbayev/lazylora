//! ASCII renderer for transaction graphs.
//!
//! This module provides the `TxnGraphWidget` which renders a transaction graph
//! as ASCII art in the terminal using Ratatui.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use super::txn_graph::TxnGraph;
use super::types::{GraphRepresentation, GraphRow};

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
    /// Row height for transactions (single line, no spacing)
    const ROW_HEIGHT: usize = 1;
    /// Header height (circled numbers + label row + type subtitle + underline + connector row)
    const HEADER_HEIGHT: usize = 5;

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
    #[allow(dead_code)]
    #[must_use]
    pub const fn without_headers(mut self) -> Self {
        self.show_headers = false;
        self
    }

    /// Hide row labels
    #[allow(dead_code)]
    #[must_use]
    pub const fn without_labels(mut self) -> Self {
        self.show_labels = false;
        self
    }

    /// Disable centering
    #[allow(dead_code)]
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

        // Render each transaction row with visual grouping
        for (idx, row) in self.graph.rows.iter().enumerate() {
            // Add subtle group separator for new transaction groups
            if self.is_new_group(idx) && idx > 0 {
                lines.push(self.render_group_separator(max_prefix_width));
            }
            lines.extend(self.render_row_with_padding(row, max_prefix_width));
        }

        lines
    }

    /// Render a subtle separator line between transaction groups
    fn render_group_separator(&self, prefix_padding: usize) -> Line<'static> {
        let col_width = self.graph.column_width;
        let col_spacing = self.graph.column_spacing;
        let total_cols = self.graph.columns.len();
        let center = col_width / 2;

        let mut spans = Vec::new();

        // Indicator column - fixed width with separator
        spans.push(Span::styled(
            " ".repeat(Self::SENDER_INDICATOR_WIDTH),
            Style::default().fg(Color::DarkGray),
        ));

        // Tree prefix padding
        if prefix_padding > 0 {
            spans.push(Span::raw(" ".repeat(prefix_padding)));
        }

        // Draw dotted line through columns
        for col_idx in 0..total_cols {
            if col_idx > 0 {
                spans.push(Span::styled(
                    "·".repeat(col_spacing),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            let col_content = format!(
                "{}┊{}",
                "·".repeat(center),
                "·".repeat(col_width.saturating_sub(center + 1))
            );
            spans.push(Span::styled(
                col_content,
                Style::default().fg(Color::DarkGray),
            ));
        }

        Line::from(spans)
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
                    // Each depth level adds 2 characters ("│ " or "  " + "├─" or "└─")
                    // Must match generate_tree_prefix()
                    row.depth * 2
                }
            })
            .max()
            .unwrap_or(0)
    }

    /// Width of the sender indicator column (e.g., "PAY " or "··AXF ")
    /// Format: up to 2 depth dots + 3-char type + 1 space = 6 chars fixed
    const SENDER_INDICATOR_WIDTH: usize = 6;

    /// Circled numbers for column headers (①②③...)
    const CIRCLED_NUMBERS: [&'static str; 10] = ["①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨", "⑩"];

    /// Render column headers with consistent padding for tree prefix alignment
    fn render_headers_with_padding(&self, prefix_padding: usize) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let col_width = self.graph.column_width;
        let col_spacing = self.graph.column_spacing;

        // Circled numbers row
        let mut number_spans = Vec::new();
        number_spans.push(Span::raw(" ".repeat(Self::SENDER_INDICATOR_WIDTH)));
        if prefix_padding > 0 {
            number_spans.push(Span::raw(" ".repeat(prefix_padding)));
        }
        for (i, _col) in self.graph.columns.iter().enumerate() {
            if i > 0 {
                number_spans.push(Span::raw(" ".repeat(col_spacing)));
            }
            let num = if i < Self::CIRCLED_NUMBERS.len() {
                Self::CIRCLED_NUMBERS[i]
            } else {
                "⓪"
            };
            // Center the circled number
            let num_len = 1; // Circled numbers are 1 char wide visually
            let padding_total = col_width.saturating_sub(num_len);
            let padding_left = padding_total / 2;
            let padding_right = padding_total - padding_left;
            let padded_num = format!(
                "{}{}{}",
                " ".repeat(padding_left),
                num,
                " ".repeat(padding_right)
            );
            number_spans.push(Span::styled(
                padded_num,
                Style::default().fg(Color::Rgb(122, 162, 247)), // Tokyo Night blue
            ));
        }
        lines.push(Line::from(number_spans));

        // Header labels row - TYP + entity labels
        let mut header_spans = Vec::new();

        // Sender indicator column header (fixed width, left-aligned)
        header_spans.push(Span::styled(
            format!("{:<width$}", "TYP", width = Self::SENDER_INDICATOR_WIDTH),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        ));

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

        // Entity type subtitle row (Account, App, Asset)
        let mut subtitle_spans = Vec::new();
        subtitle_spans.push(Span::raw(" ".repeat(Self::SENDER_INDICATOR_WIDTH)));
        if prefix_padding > 0 {
            subtitle_spans.push(Span::raw(" ".repeat(prefix_padding)));
        }
        for (i, col) in self.graph.columns.iter().enumerate() {
            if i > 0 {
                subtitle_spans.push(Span::raw(" ".repeat(col_spacing)));
            }
            let type_label = col.entity_type.type_label();
            let label_len = type_label.chars().count();
            let padding_total = col_width.saturating_sub(label_len);
            let padding_left = padding_total / 2;
            let padding_right = padding_total - padding_left;
            let padded_type = format!(
                "{}{}{}",
                " ".repeat(padding_left),
                type_label,
                " ".repeat(padding_right)
            );
            subtitle_spans.push(Span::styled(
                padded_type,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ));
        }
        lines.push(Line::from(subtitle_spans));

        // Underline with column markers (┬)
        let mut underline_spans = Vec::new();

        // Indicator column underline
        underline_spans.push(Span::styled(
            "─".repeat(Self::SENDER_INDICATOR_WIDTH),
            Style::default().fg(Color::DarkGray),
        ));

        // Add padding to match tree prefix width
        if prefix_padding > 0 {
            underline_spans.push(Span::styled(
                "─".repeat(prefix_padding),
                Style::default().fg(Color::DarkGray),
            ));
        }

        for (i, _col) in self.graph.columns.iter().enumerate() {
            if i > 0 {
                underline_spans.push(Span::styled(
                    "─".repeat(col_spacing),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            // Create column marker with lines
            let marker_padding = col_width / 2;
            let marker = format!(
                "{}┬{}",
                "─".repeat(marker_padding),
                "─".repeat(col_width.saturating_sub(marker_padding + 1))
            );
            underline_spans.push(Span::styled(marker, Style::default().fg(Color::DarkGray)));
        }
        lines.push(Line::from(underline_spans));

        // Vertical connectors row
        let mut connector_spans = Vec::new();
        connector_spans.push(Span::raw(" ".repeat(Self::SENDER_INDICATOR_WIDTH)));

        if prefix_padding > 0 {
            connector_spans.push(Span::raw(" ".repeat(prefix_padding)));
        }

        for (i, _col) in self.graph.columns.iter().enumerate() {
            if i > 0 {
                connector_spans.push(Span::raw(" ".repeat(col_spacing)));
            }
            let marker_padding = col_width / 2;
            let marker = format!(
                "{}│{}",
                " ".repeat(marker_padding),
                " ".repeat(col_width.saturating_sub(marker_padding + 1))
            );
            connector_spans.push(Span::styled(marker, Style::default().fg(Color::DarkGray)));
        }
        lines.push(Line::from(connector_spans));

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

        // Create sender indicator: show transaction type abbreviation on the left
        let sender_indicator = self.create_sender_indicator(row);

        match row.representation {
            GraphRepresentation::Vector => {
                if let (Some(from_col), Some(to_col)) = (row.from_col, row.to_col) {
                    lines.push(self.render_vector_line(
                        from_col,
                        to_col,
                        color,
                        &row.label,
                        &full_prefix,
                        &sender_indicator,
                    ));
                }
            }
            GraphRepresentation::SelfLoop => {
                if let Some(col) = row.from_col {
                    lines.push(self.render_self_loop_line(
                        col,
                        color,
                        &row.label,
                        &full_prefix,
                        &sender_indicator,
                    ));
                }
            }
            GraphRepresentation::Point => {
                if let Some(col) = row.from_col {
                    lines.push(self.render_point_line(
                        col,
                        color,
                        &row.label,
                        &full_prefix,
                        &sender_indicator,
                    ));
                }
            }
        }

        // No spacing between rows - compact layout
        lines
    }

    /// Create a left-side sender indicator showing transaction type
    /// Returns a fixed-width string (SENDER_INDICATOR_WIDTH chars)
    fn create_sender_indicator(&self, row: &GraphRow) -> String {
        let abbrev = match row.txn_type {
            crate::domain::TxnType::Payment => "PAY",
            crate::domain::TxnType::AssetTransfer => "AXF",
            crate::domain::TxnType::AppCall => "APP",
            crate::domain::TxnType::AssetConfig => "ACF",
            crate::domain::TxnType::AssetFreeze => "AFZ",
            crate::domain::TxnType::KeyReg => "KEY",
            crate::domain::TxnType::StateProof => "SPF",
            crate::domain::TxnType::Heartbeat => "HBT",
            crate::domain::TxnType::Unknown => "???",
        };

        // Add depth indicator for inner transactions (up to 2 dots)
        // Format: "  PAY " or " ·PAY " or "··PAY " (always 6 chars total)
        let depth_dots = row.depth.min(2);
        let leading_spaces = 2 - depth_dots;
        format!(
            "{}{}{} ",
            " ".repeat(leading_spaces),
            "·".repeat(depth_dots),
            abbrev
        )
    }

    /// Check if this row starts a new transaction group (depth 0 after depth > 0)
    fn is_new_group(&self, row_idx: usize) -> bool {
        if row_idx == 0 {
            return true;
        }
        let current_row = &self.graph.rows[row_idx];
        let prev_row = &self.graph.rows[row_idx - 1];

        // New group when going from nested (depth > 0) back to top level (depth 0)
        current_row.depth == 0 && prev_row.depth > 0
    }

    /// Generate tree prefix characters for inner transaction nesting
    fn generate_tree_prefix(&self, row: &GraphRow) -> String {
        if row.depth == 0 {
            return String::new();
        }

        let mut prefix = String::new();

        // Build prefix based on ancestry - check siblings at each depth level
        for d in 1..row.depth {
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
            if let Some(idx) = ancestor_idx
                && let Some(ancestor) = self.graph.rows.get(idx)
            {
                ancestor_idx = ancestor.parent_index;
                current_depth -= 1;
            } else {
                break;
            }
        }

        // Check if ancestor has more children after this row's branch
        if let Some(idx) = ancestor_idx {
            for (i, r) in self.graph.rows.iter().enumerate() {
                if i > row.index && r.parent_index == Some(idx) {
                    return true;
                }
            }
        }

        false
    }

    /// Render a vector (arrow) between two columns
    fn render_vector_line(
        &self,
        from_col: usize,
        to_col: usize,
        color: Color,
        label: &str,
        tree_prefix: &str,
        sender_indicator: &str,
    ) -> Line<'static> {
        let col_width = self.graph.column_width;
        let col_spacing = self.graph.column_spacing;
        let total_cols = self.graph.columns.len();

        let mut spans = Vec::new();

        // Add sender indicator (transaction type) - already fixed width
        spans.push(Span::styled(
            sender_indicator.to_string(),
            Style::default().fg(color).add_modifier(Modifier::DIM),
        ));

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

        // Determine where to place the label (middle of the arrow span)
        let label_col = if right_col > left_col + 1 {
            // Place label in a middle column
            Some(left_col + (right_col - left_col) / 2)
        } else {
            None // Arrow is too short for inline label
        };

        // Truncate label to fit in column width
        let truncated_label = if label.len() > col_width {
            format!("{}…", &label[..col_width.saturating_sub(1)])
        } else {
            label.to_string()
        };

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
            } else if Some(col_idx) == label_col && self.show_labels && !label.is_empty() {
                // Middle column with label - center the label text
                let label_len = truncated_label.chars().count();
                let padding_total = col_width.saturating_sub(label_len);
                let padding_left = padding_total / 2;
                let padding_right = padding_total - padding_left;
                let col_content = format!(
                    "{}{}{}",
                    "─".repeat(padding_left),
                    truncated_label,
                    "─".repeat(padding_right)
                );
                spans.push(Span::styled(
                    col_content,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ));
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

        // Add label at end if arrow was too short for inline label
        if label_col.is_none() && self.show_labels && !label.is_empty() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                truncated_label,
                Style::default().fg(color).add_modifier(Modifier::DIM),
            ));
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
        sender_indicator: &str,
    ) -> Line<'static> {
        let col_width = self.graph.column_width;
        let col_spacing = self.graph.column_spacing;
        let total_cols = self.graph.columns.len();

        let mut spans = Vec::new();

        // Add sender indicator (transaction type) - already fixed width
        spans.push(Span::styled(
            sender_indicator.to_string(),
            Style::default().fg(color).add_modifier(Modifier::DIM),
        ));

        // Add tree prefix if present
        if !tree_prefix.is_empty() {
            spans.push(Span::styled(
                tree_prefix.to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let center = col_width / 2;

        // Truncate label for compact display
        let truncated_label = if label.len() > 6 {
            format!("{}…", &label[..5])
        } else {
            label.to_string()
        };

        for col_idx in 0..total_cols {
            if col_idx > 0 {
                spans.push(Span::raw(" ".repeat(col_spacing)));
            }

            if col_idx == col {
                // Self-loop marker: ↺ with label next to it
                let marker_with_label = if self.show_labels && !label.is_empty() {
                    format!("↺{}", truncated_label)
                } else {
                    "↺".to_string()
                };
                let content_len = marker_with_label.chars().count();
                let left_pad = center.saturating_sub(content_len / 2);
                let right_pad = col_width.saturating_sub(left_pad + content_len);
                let col_content = format!(
                    "{}{}{}",
                    " ".repeat(left_pad),
                    marker_with_label,
                    " ".repeat(right_pad)
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

        Line::from(spans)
    }

    /// Render a point (single marker)
    fn render_point_line(
        &self,
        col: usize,
        color: Color,
        label: &str,
        tree_prefix: &str,
        sender_indicator: &str,
    ) -> Line<'static> {
        let col_width = self.graph.column_width;
        let col_spacing = self.graph.column_spacing;
        let total_cols = self.graph.columns.len();

        let mut spans = Vec::new();

        // Add sender indicator (transaction type) - already fixed width
        spans.push(Span::styled(
            sender_indicator.to_string(),
            Style::default().fg(color).add_modifier(Modifier::DIM),
        ));

        // Add tree prefix if present
        if !tree_prefix.is_empty() {
            spans.push(Span::styled(
                tree_prefix.to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let center = col_width / 2;

        // Truncate label for compact display
        let truncated_label = if label.len() > 6 {
            format!("{}…", &label[..5])
        } else {
            label.to_string()
        };

        for col_idx in 0..total_cols {
            if col_idx > 0 {
                spans.push(Span::raw(" ".repeat(col_spacing)));
            }

            if col_idx == col {
                // Point marker: ◉ with label next to it
                let marker_with_label = if self.show_labels && !label.is_empty() {
                    format!("◉{}", truncated_label)
                } else {
                    "◉".to_string()
                };
                let content_len = marker_with_label.chars().count();
                let left_pad = center.saturating_sub(content_len / 2);
                let right_pad = col_width.saturating_sub(left_pad + content_len);
                let col_content = format!(
                    "{}{}{}",
                    " ".repeat(left_pad),
                    marker_with_label,
                    " ".repeat(right_pad)
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

    /// Calculate required width for rendering by measuring actual line content
    /// Returns the width of the widest line (excluding trailing whitespace)
    #[must_use]
    pub fn required_width(&self) -> usize {
        if self.graph.columns.is_empty() {
            return 0;
        }

        // Generate lines and measure actual visual width
        let lines = self.to_lines();
        lines
            .iter()
            .map(|line| {
                // Concatenate all spans into a single string
                let full_line: String = line
                    .spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect();

                // Return length without trailing whitespace
                full_line.trim_end().chars().count()
            })
            .max()
            .unwrap_or(0)
    }
}

impl Widget for TxnGraphWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.to_lines();
        let total_width = self.required_width();

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
