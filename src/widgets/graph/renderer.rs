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

    /// Calculate required width for rendering (including tree prefix and labels)
    #[must_use]
    pub fn required_width(&self) -> usize {
        if self.graph.columns.is_empty() {
            return 0;
        }

        // Base width from columns
        let base_width = self.graph.total_width();

        // Tree prefix width
        let max_prefix_width = self.calculate_max_prefix_width();

        // Find maximum label width if labels are shown
        let max_label_width = if self.show_labels {
            self.graph
                .rows
                .iter()
                .map(|row| row.label.chars().count())
                .max()
                .unwrap_or(0)
        } else {
            0
        };

        // Total: prefix + columns + spacing before label + label + right margin
        let label_spacing = if max_label_width > 0 { 2 } else { 0 };
        let right_margin = 2; // Extra space to prevent cutoff
        max_prefix_width + base_width + label_spacing + max_label_width + right_margin
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
