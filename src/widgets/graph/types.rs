//! Graph type definitions for transaction visualization.
//!
//! This module provides the core data types used for representing
//! transaction graphs, including entity types, columns, rows, and
//! visual representation styles.

use ratatui::style::Color;

use crate::domain::TxnType;
use crate::widgets::helpers::truncate_address;

// ============================================================================
// GraphEntityType
// ============================================================================

/// Entity type for graph columns - matches algokit-lora's Vertical type.
///
/// This enum represents the different types of entities that can appear
/// as columns in a transaction graph visualization.
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
    /// Get the column header prefix for this entity type.
    ///
    /// # Returns
    ///
    /// A static string prefix used when displaying the entity header
    #[allow(dead_code)]
    #[must_use]
    pub const fn header_prefix(&self) -> &'static str {
        match self {
            Self::Account => "",
            Self::Application => "App #",
            Self::Asset => "ASA #",
        }
    }

    /// Get a short type label for this entity type.
    ///
    /// Used in the subtitle row below column headers.
    ///
    /// # Returns
    ///
    /// A short label like "Account", "App", or "Asset"
    #[must_use]
    pub const fn type_label(&self) -> &'static str {
        match self {
            Self::Account => "Account",
            Self::Application => "App",
            Self::Asset => "Asset",
        }
    }

    /// Get the header color for this entity type.
    ///
    /// # Returns
    ///
    /// The ratatui Color to use for this entity type's header
    #[must_use]
    pub const fn header_color(&self) -> Color {
        match self {
            Self::Account => Color::Yellow,
            Self::Application => Color::Cyan,
            Self::Asset => Color::Magenta,
        }
    }
}

// ============================================================================
// GraphColumn
// ============================================================================

/// A vertical column in the graph representing an entity.
///
/// Each column represents a unique participant in a transaction (or group of
/// transactions), such as an account address, application, or asset.
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
    /// Create a new account column.
    ///
    /// # Arguments
    ///
    /// * `address` - The full Algorand address
    /// * `index` - Column index (0-based)
    /// * `label_width` - Maximum width for the label
    ///
    /// # Returns
    ///
    /// A new `GraphColumn` configured for an account
    #[must_use]
    pub fn account(address: &str, index: usize, label_width: usize) -> Self {
        Self {
            entity_type: GraphEntityType::Account,
            entity_id: address.to_string(),
            label: truncate_address(address, label_width),
            index,
        }
    }

    /// Create a new application column.
    ///
    /// # Arguments
    ///
    /// * `app_id` - The application ID
    /// * `index` - Column index (0-based)
    /// * `label_width` - Maximum width for the label
    ///
    /// # Returns
    ///
    /// A new `GraphColumn` configured for an application
    #[must_use]
    pub fn application(app_id: u64, index: usize, label_width: usize) -> Self {
        let full_label = format!("App #{}", app_id);
        let label = if full_label.len() > label_width {
            // Truncate to fit width
            let id_str = app_id.to_string();
            let available = label_width.saturating_sub(6); // "App #" + "…" = 6 chars
            if available >= 1 && id_str.len() > available {
                format!("App #{}…", &id_str[..available])
            } else if label_width >= 2 {
                // Very small: just show truncated number with #
                let chars_available = label_width.saturating_sub(1).min(id_str.len());
                format!("#{}", &id_str[..chars_available])
            } else {
                "#".to_string()
            }
        } else {
            full_label
        };
        Self {
            entity_type: GraphEntityType::Application,
            entity_id: app_id.to_string(),
            label,
            index,
        }
    }

    /// Create a new asset column.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The asset ID
    /// * `index` - Column index (0-based)
    /// * `label_width` - Maximum width for the label
    ///
    /// # Returns
    ///
    /// A new `GraphColumn` configured for an asset
    #[must_use]
    pub fn asset(asset_id: u64, index: usize, label_width: usize) -> Self {
        let full_label = format!("ASA #{}", asset_id);
        let label = if full_label.len() > label_width {
            // Truncate to fit width
            let id_str = asset_id.to_string();
            let available = label_width.saturating_sub(6); // "ASA #" + "…" = 6 chars
            if available >= 1 && id_str.len() > available {
                format!("ASA #{}…", &id_str[..available])
            } else if label_width >= 2 {
                // Very small: just show truncated number with #
                let chars_available = label_width.saturating_sub(1).min(id_str.len());
                format!("#{}", &id_str[..chars_available])
            } else {
                "#".to_string()
            }
        } else {
            full_label
        };
        Self {
            entity_type: GraphEntityType::Asset,
            entity_id: asset_id.to_string(),
            label,
            index,
        }
    }
}

// ============================================================================
// GraphRepresentation
// ============================================================================

/// Visual representation type for a transaction - matches algokit-lora.
///
/// This enum determines how a transaction is visually rendered in the graph:
/// as an arrow between columns, a self-loop, or a single point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphRepresentation {
    /// Arrow between two different columns (sender → receiver)
    Vector,
    /// Curved arrow when sender = receiver (e.g., opt-in)
    SelfLoop,
    /// Single point marker (KeyReg, StateProof, Heartbeat)
    Point,
}

// ============================================================================
// GraphRow
// ============================================================================

/// A horizontal row in the graph representing a transaction.
///
/// Each row corresponds to a single transaction and contains information
/// about its visual representation, including source/target columns,
/// nesting depth for inner transactions, and display label.
#[derive(Debug, Clone)]
pub struct GraphRow {
    /// Transaction ID
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub has_children: bool,
    /// Whether this is the last child in its parent group
    pub is_last_child: bool,
    /// Column index for rekey target (if transaction is a rekey)
    pub rekey_col: Option<usize>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::account(GraphEntityType::Account, "", Color::Yellow)]
    #[case::application(GraphEntityType::Application, "App #", Color::Cyan)]
    #[case::asset(GraphEntityType::Asset, "ASA #", Color::Magenta)]
    fn test_graph_entity_type_properties(
        #[case] entity: GraphEntityType,
        #[case] expected_prefix: &str,
        #[case] expected_color: Color,
    ) {
        assert_eq!(entity.header_prefix(), expected_prefix);
        assert_eq!(entity.header_color(), expected_color);
    }

    #[test]
    fn test_graph_column_account() {
        let col = GraphColumn::account("TESTADDRESS123456789", 0, 10);
        assert_eq!(col.entity_type, GraphEntityType::Account);
        assert_eq!(col.entity_id, "TESTADDRESS123456789");
        assert_eq!(col.index, 0);
        assert!(col.label.len() <= 10);
    }

    #[test]
    fn test_graph_column_application() {
        let col = GraphColumn::application(12345, 1, 12);
        assert_eq!(col.entity_type, GraphEntityType::Application);
        assert_eq!(col.entity_id, "12345");
        assert_eq!(col.index, 1);
        assert!(col.label.contains("12345") || col.label.contains("#"));
    }

    #[test]
    fn test_graph_column_asset() {
        let col = GraphColumn::asset(31566704, 2, 15);
        assert_eq!(col.entity_type, GraphEntityType::Asset);
        assert_eq!(col.entity_id, "31566704");
        assert_eq!(col.index, 2);
    }

    #[rstest]
    #[case::vector_eq_vector(GraphRepresentation::Vector, GraphRepresentation::Vector, true)]
    #[case::vector_ne_selfloop(GraphRepresentation::Vector, GraphRepresentation::SelfLoop, false)]
    #[case::selfloop_ne_point(GraphRepresentation::SelfLoop, GraphRepresentation::Point, false)]
    fn test_graph_representation_equality(
        #[case] left: GraphRepresentation,
        #[case] right: GraphRepresentation,
        #[case] expected_eq: bool,
    ) {
        assert_eq!(left == right, expected_eq);
    }
}
