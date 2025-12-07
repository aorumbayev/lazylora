//! Graph building logic for transaction visualization.
//!
//! This module contains all the methods for constructing a `TxnGraph` from
//! transactions, including:
//! - Adding transactions recursively (with inner transactions)
//! - Determining visual representation (Vector, SelfLoop, Point)
//! - Creating and managing columns for entities
//! - Generating row labels

use std::collections::HashMap;

use crate::constants::MICROALGOS_PER_ALGO;
use crate::domain::{Transaction, TransactionDetails, TxnType};

use super::txn_graph::TxnGraph;
use super::types::{GraphColumn, GraphEntityType, GraphRepresentation, GraphRow};

// ============================================================================
// TxnGraph Builder Methods
// ============================================================================

impl TxnGraph {
    /// Build a graph from a single transaction (including inner transactions).
    #[must_use]
    pub fn from_transaction(txn: &Transaction) -> Self {
        let mut graph = Self::new();
        graph.add_transaction_recursive(txn, 0, None, false);
        graph.finalize_tree_structure();
        graph
    }

    /// Build a graph from multiple transactions (e.g., a transaction group).
    #[allow(dead_code)]
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

    /// Add a transaction and its inner transactions recursively to the graph.
    pub(super) fn add_transaction_recursive(
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
    pub(super) fn finalize_tree_structure(&mut self) {
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

    /// Chooses graph representation based on transaction semantics.
    ///
    /// - Point: Single-entity operations (KeyReg, StateProof) or creations
    /// - Vector: Transfers between distinct entities (payments, app calls)
    /// - SelfLoop: Operations where sender equals receiver (self-transfers)
    ///
    /// App calls show Account→Application flow to visualize contract interactions.
    /// Asset operations show the asset as a node to track asset lifecycle.
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
    pub(super) fn get_or_create_account_column(&mut self, address: &str) -> usize {
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
    pub(super) fn get_or_create_app_column(&mut self, app_id: u64) -> usize {
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
    pub(super) fn get_or_create_asset_column(&mut self, asset_id: u64) -> usize {
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
}
