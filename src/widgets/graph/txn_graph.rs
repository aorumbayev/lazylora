//! Transaction graph data structure and core logic.
//!
//! This module provides the `TxnGraph` struct which represents a complete
//! transaction visualization graph. Building logic is in the `builders` module.

use super::types::{GraphColumn, GraphRow};

// ============================================================================
// TxnGraph
// ============================================================================

/// Complete transaction graph structure.
///
/// This struct represents the full graph visualization data for one or more
/// transactions, including:
/// - Column definitions (entities like accounts, apps, assets)
/// - Row definitions (transactions with their visual representations)
/// - Layout configuration (column width, spacing)
///
/// # Example
///
/// ```ignore
/// use crate::widgets::graph::TxnGraph;
///
/// let graph = TxnGraph::from_transaction(&txn);
/// let svg = graph.to_svg();
/// ```
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
    /// Column width fits truncated addresses (8 chars: "ABCD..XY")
    pub const DEFAULT_COLUMN_WIDTH: usize = 8;
    /// Spacing fits arrow decoration (" → " = 3 chars)
    pub const DEFAULT_COLUMN_SPACING: usize = 3;

    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            column_width: Self::DEFAULT_COLUMN_WIDTH,
            column_spacing: Self::DEFAULT_COLUMN_SPACING,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_column_width(mut self, width: usize) -> Self {
        self.column_width = width;
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_column_spacing(mut self, spacing: usize) -> Self {
        self.column_spacing = spacing;
        self
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn total_width(&self) -> usize {
        if self.columns.is_empty() {
            return 0;
        }
        let num_cols = self.columns.len();
        num_cols * self.column_width + (num_cols.saturating_sub(1)) * self.column_spacing
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn column_center_x(&self, col_index: usize) -> usize {
        col_index * (self.column_width + self.column_spacing) + self.column_width / 2
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn column_start_x(&self, col_index: usize) -> usize {
        col_index * (self.column_width + self.column_spacing)
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty() || self.rows.is_empty()
    }

    #[allow(dead_code)]
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
        const REKEY_COLOR: &str = "#e0af68";

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
  <marker id="arrowhead-rekey" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
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
            REKEY_COLOR,
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
            let type_label = col.entity_type.type_label();
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
            self.render_svg_element(&mut svg, row, y, LABEL_WIDTH, COL_WIDTH, COL_SPACING);

            // Draw rekey indicator if present (dashed yellow line with key symbol)
            if let Some(rekey_col) = row.rekey_col {
                let from_col = row.from_col.unwrap_or(0);
                let x1 = LABEL_WIDTH + from_col * (COL_WIDTH + COL_SPACING) + COL_WIDTH / 2;
                let x2 = LABEL_WIDTH + rekey_col * (COL_WIDTH + COL_SPACING) + COL_WIDTH / 2;
                let rekey_y = y + 12; // Offset below main arrow

                // Dashed line
                svg.push_str(&format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="2" stroke-dasharray="4,2" marker-end="url(#arrowhead-rekey)"/>"#,
                    x1, rekey_y, x2, rekey_y, REKEY_COLOR
                ));
                svg.push('\n');

                // Key symbol (rekey indicator)
                let key_x = (x1 + x2) / 2;
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" fill="{}" font-family="monospace" font-size="10" text-anchor="middle" font-weight="bold">KEY</text>"#,
                    key_x, rekey_y - 2, REKEY_COLOR
                ));
                svg.push('\n');
            }
        }

        // Close SVG
        svg.push_str("</svg>\n");
        svg
    }

    /// Render SVG element for a single row
    fn render_svg_element(
        &self,
        svg: &mut String,
        row: &GraphRow,
        y: usize,
        label_width: usize,
        col_width: usize,
        col_spacing: usize,
    ) {
        use super::types::GraphRepresentation;
        use crate::domain::TxnType;

        const ARROW_PAYMENT: &str = "#9ece6a";
        const ARROW_ASSET: &str = "#bb9af7";
        const ARROW_APPCALL: &str = "#7dcfff";
        const POINT_COLOR: &str = "#f7768e";

        match row.representation {
            GraphRepresentation::Vector => {
                if let (Some(from), Some(to)) = (row.from_col, row.to_col) {
                    let x1 = label_width + from * (col_width + col_spacing) + col_width / 2;
                    let x2 = label_width + to * (col_width + col_spacing) + col_width / 2;

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
                    let cx = label_width + col * (col_width + col_spacing) + col_width / 2;
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
                    let cx = label_width + col * (col_width + col_spacing) + col_width / 2;
                    svg.push_str(&format!(
                        r#"<circle cx="{}" cy="{}" r="6" fill="{}"/>"#,
                        cx, y, POINT_COLOR
                    ));
                    svg.push('\n');
                }
            }
        }
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
