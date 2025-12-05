//! Transaction graph data structure and construction logic.
//!
//! This module provides the `TxnGraph` struct which represents a complete
//! transaction visualization graph, including methods for building graphs
//! from transactions and calculating layout dimensions.

#![allow(dead_code)]

use std::collections::HashMap;

use crate::domain::{Transaction, TransactionDetails, TxnType};

use super::types::{GraphColumn, GraphEntityType, GraphRepresentation, GraphRow};

// ============================================================================
// Constants
// ============================================================================

/// Number of microAlgos per Algo
const MICROALGOS_PER_ALGO: f64 = 1_000_000.0;

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
    /// Default column width (compact to fit more columns)
    pub const DEFAULT_COLUMN_WIDTH: usize = 8;
    /// Default spacing between columns (reduced for compact layout)
    pub const DEFAULT_COLUMN_SPACING: usize = 3;

    /// Create a new empty graph.
    ///
    /// # Returns
    ///
    /// A new `TxnGraph` with default settings and no data
    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            column_width: Self::DEFAULT_COLUMN_WIDTH,
            column_spacing: Self::DEFAULT_COLUMN_SPACING,
        }
    }

    /// Set column width.
    ///
    /// # Arguments
    ///
    /// * `width` - The desired column width in characters
    ///
    /// # Returns
    ///
    /// Self for method chaining
    #[must_use]
    pub const fn with_column_width(mut self, width: usize) -> Self {
        self.column_width = width;
        self
    }

    /// Set column spacing.
    ///
    /// # Arguments
    ///
    /// * `spacing` - The desired spacing between columns
    ///
    /// # Returns
    ///
    /// Self for method chaining
    #[must_use]
    pub const fn with_column_spacing(mut self, spacing: usize) -> Self {
        self.column_spacing = spacing;
        self
    }

    /// Build a graph from a single transaction (including inner transactions).
    ///
    /// This method creates a complete graph representation for a transaction
    /// and all its inner transactions (if any).
    ///
    /// # Arguments
    ///
    /// * `txn` - The transaction to visualize
    ///
    /// # Returns
    ///
    /// A new `TxnGraph` containing the transaction visualization
    #[must_use]
    pub fn from_transaction(txn: &Transaction) -> Self {
        let mut graph = Self::new();
        graph.add_transaction_recursive(txn, 0, None, false);
        graph.finalize_tree_structure();
        graph
    }

    /// Build a graph from multiple transactions (e.g., inner transactions).
    ///
    /// # Arguments
    ///
    /// * `transactions` - Slice of transactions to visualize
    ///
    /// # Returns
    ///
    /// A new `TxnGraph` containing all transactions
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

    /// Add a transaction to the graph (legacy method for backward compatibility).
    ///
    /// # Arguments
    ///
    /// * `txn` - The transaction to add
    /// * `row_index` - The row index for this transaction
    /// * `parent_index` - Optional parent row index for inner transactions
    pub fn add_transaction(
        &mut self,
        txn: &Transaction,
        row_index: usize,
        parent_index: Option<usize>,
    ) {
        self.add_transaction_recursive(txn, row_index, parent_index, false);
    }

    /// Add a transaction and its inner transactions recursively to the graph.
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

        // Handle rekey_to - create column for rekey target if present
        let rekey_col = txn
            .rekey_to
            .as_ref()
            .map(|rekey_addr| self.get_or_create_account_column(rekey_addr));

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
            rekey_col,
        };

        self.rows.push(row);

        // Recursively add inner transactions
        let inner_count = txn.inner_transactions.len();
        for (i, inner_txn) in txn.inner_transactions.iter().enumerate() {
            let inner_is_last = i == inner_count - 1;
            self.add_transaction_recursive(inner_txn, i, Some(current_row_index), inner_is_last);
        }
    }

    /// Finalize tree structure by updating is_last_child flags based on siblings.
    fn finalize_tree_structure(&mut self) {
        // Group rows by parent_index
        let mut children_by_parent: HashMap<Option<usize>, Vec<usize>> = HashMap::new();

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

    /// Determine visual representation and column indices for a transaction.
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

    /// Get or create an account column, returning its index.
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

    /// Get or create an application column, returning its index.
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
        self.columns
            .push(GraphColumn::application(app_id, index, self.column_width));
        index
    }

    /// Get or create an asset column, returning its index.
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
        self.columns
            .push(GraphColumn::asset(asset_id, index, self.column_width));
        index
    }

    /// Create a display label for a transaction row.
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

    /// Calculate total width needed for the graph.
    ///
    /// # Returns
    ///
    /// The total width in characters needed to render all columns
    #[must_use]
    pub fn total_width(&self) -> usize {
        if self.columns.is_empty() {
            return 0;
        }
        let num_cols = self.columns.len();
        num_cols * self.column_width + (num_cols.saturating_sub(1)) * self.column_spacing
    }

    /// Calculate the x position for a column center.
    ///
    /// # Arguments
    ///
    /// * `col_index` - The column index
    ///
    /// # Returns
    ///
    /// The x coordinate of the column center
    #[must_use]
    pub fn column_center_x(&self, col_index: usize) -> usize {
        col_index * (self.column_width + self.column_spacing) + self.column_width / 2
    }

    /// Calculate the x position for a column start.
    ///
    /// # Arguments
    ///
    /// * `col_index` - The column index
    ///
    /// # Returns
    ///
    /// The x coordinate of the column start
    #[must_use]
    pub fn column_start_x(&self, col_index: usize) -> usize {
        col_index * (self.column_width + self.column_spacing)
    }

    /// Check if the graph is empty.
    ///
    /// # Returns
    ///
    /// `true` if the graph has no columns or rows
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty() || self.rows.is_empty()
    }

    /// Export the graph to SVG format
    ///
    /// # Returns
    ///
    /// A complete SVG document as a string
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
                            TxnType::AssetTransfer
                            | TxnType::AssetConfig
                            | TxnType::AssetFreeze => ARROW_ASSET,
                            TxnType::AppCall => ARROW_APPCALL,
                            _ => ARROW_PAYMENT,
                        };

                        let marker_id = match row.txn_type {
                            TxnType::AssetTransfer
                            | TxnType::AssetConfig
                            | TxnType::AssetFreeze => "arrowhead-asset",
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

                // Key symbol
                let key_x = (x1 + x2) / 2;
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" fill="{}" font-family="sans-serif" font-size="10" text-anchor="middle">🔑</text>"#,
                    key_x, rekey_y - 2, REKEY_COLOR
                ));
                svg.push('\n');
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TransactionDetails;

    fn create_test_payment() -> Transaction {
        Transaction {
            id: "TEST123".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER_ADDRESS".to_string(),
            to: "RECEIVER_ADDRESS".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 5_000_000,
            asset_id: None,
            rekey_to: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        }
    }

    fn create_test_app_call() -> Transaction {
        Transaction {
            id: "APP123".to_string(),
            txn_type: TxnType::AppCall,
            from: "CALLER_ADDRESS".to_string(),
            to: "12345".to_string(), // App ID
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 0,
            asset_id: None,
            rekey_to: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        }
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
    fn test_txn_graph_default() {
        let graph = TxnGraph::default();
        assert!(graph.is_empty());
    }

    #[test]
    fn test_txn_graph_with_column_width() {
        let graph = TxnGraph::new().with_column_width(12);
        assert_eq!(graph.column_width, 12);
    }

    #[test]
    fn test_txn_graph_with_column_spacing() {
        let graph = TxnGraph::new().with_column_spacing(5);
        assert_eq!(graph.column_spacing, 5);
    }

    #[test]
    fn test_txn_graph_from_payment() {
        let txn = create_test_payment();
        let graph = TxnGraph::from_transaction(&txn);

        // Should have 2 columns (sender and receiver)
        assert_eq!(graph.columns.len(), 2);
        // Should have 1 row
        assert_eq!(graph.rows.len(), 1);

        // Check row representation
        assert_eq!(graph.rows[0].representation, GraphRepresentation::Vector);
        assert_eq!(graph.rows[0].txn_type, TxnType::Payment);
    }

    #[test]
    fn test_txn_graph_from_app_call() {
        let txn = create_test_app_call();
        let graph = TxnGraph::from_transaction(&txn);

        // Should have 2 columns (account and app)
        assert_eq!(graph.columns.len(), 2);
        assert_eq!(graph.rows.len(), 1);

        // Check column types
        assert_eq!(graph.columns[0].entity_type, GraphEntityType::Account);
        assert_eq!(graph.columns[1].entity_type, GraphEntityType::Application);
    }

    #[test]
    fn test_txn_graph_from_multiple_transactions() {
        let transactions = vec![create_test_payment(), create_test_app_call()];
        let graph = TxnGraph::from_transactions(&transactions);

        // Should have rows for both transactions
        assert_eq!(graph.rows.len(), 2);
    }

    #[test]
    fn test_txn_graph_total_width() {
        let graph = TxnGraph::new().with_column_width(10).with_column_spacing(5);

        // Empty graph
        assert_eq!(graph.total_width(), 0);

        // Graph with columns
        let txn = create_test_payment();
        let graph = TxnGraph::from_transaction(&txn);
        let expected_width = 2 * graph.column_width + graph.column_spacing;
        assert_eq!(graph.total_width(), expected_width);
    }

    #[test]
    fn test_txn_graph_column_center_x() {
        let graph = TxnGraph::new().with_column_width(10).with_column_spacing(5);

        // First column center
        assert_eq!(graph.column_center_x(0), 5); // 10/2
        // Second column center
        assert_eq!(graph.column_center_x(1), 20); // (10+5) + 10/2
    }

    #[test]
    fn test_txn_graph_column_start_x() {
        let graph = TxnGraph::new().with_column_width(10).with_column_spacing(5);

        assert_eq!(graph.column_start_x(0), 0);
        assert_eq!(graph.column_start_x(1), 15); // 10+5
        assert_eq!(graph.column_start_x(2), 30); // 2*(10+5)
    }

    #[test]
    fn test_txn_graph_self_transfer() {
        let mut txn = create_test_payment();
        txn.to = txn.from.clone();

        let graph = TxnGraph::from_transaction(&txn);

        // Should have only 1 column (same address)
        assert_eq!(graph.columns.len(), 1);
        // Should be a self-loop
        assert_eq!(graph.rows[0].representation, GraphRepresentation::SelfLoop);
    }

    #[test]
    fn test_txn_graph_keyreg() {
        let txn = Transaction {
            id: "KEYREG123".to_string(),
            txn_type: TxnType::KeyReg,
            from: "ACCOUNT_ADDRESS".to_string(),
            to: "".to_string(),
            timestamp: "2024-01-01".to_string(),
            block: 12345,
            fee: 1000,
            note: "".to_string(),
            amount: 0,
            asset_id: None,
            rekey_to: None,
            details: TransactionDetails::default(),
            inner_transactions: Vec::new(),
        };

        let graph = TxnGraph::from_transaction(&txn);

        // Should have 1 column
        assert_eq!(graph.columns.len(), 1);
        // Should be a point
        assert_eq!(graph.rows[0].representation, GraphRepresentation::Point);
    }

    #[test]
    fn test_txn_graph_with_inner_transactions() {
        let inner_txn = create_test_payment();
        let mut outer_txn = create_test_app_call();
        outer_txn.inner_transactions = vec![inner_txn];

        let graph = TxnGraph::from_transaction(&outer_txn);

        // Should have 2 rows (outer + inner)
        assert_eq!(graph.rows.len(), 2);
        // Inner transaction should have depth 1
        assert_eq!(graph.rows[1].depth, 1);
        assert_eq!(graph.rows[1].parent_index, Some(0));
    }

    #[test]
    fn test_txn_graph_with_rekey() {
        let mut txn = create_test_payment();
        txn.rekey_to = Some("NEW_AUTH_ADDRESS".to_string());

        let graph = TxnGraph::from_transaction(&txn);

        // Should have 3 columns (sender, receiver, rekey target)
        assert_eq!(graph.columns.len(), 3);
        // Row should have rekey_col set
        assert!(graph.rows[0].rekey_col.is_some());
    }
}
