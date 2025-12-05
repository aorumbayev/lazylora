//! Modular widget components for the LazyLora TUI.
//!
//! This module provides a refactored, modular structure for reusable widget components
//! that render rich ASCII visualizations for Algorand blockchain data.
//!
//! # Module Structure
//!
//! - [`helpers`]: Utility functions for formatting addresses, amounts, and transaction data
//! - [`common`]: Common reusable widgets (badges, amount displays, address displays)
//! - [`list`]: List widgets for displaying blocks and transactions with selection/scrolling
//! - [`graph`]: Transaction graph visualization (columns and flow arrows)
//! - [`detail`]: Detailed transaction views (flow diagrams, visual cards)
//!
//! # Example Usage
//!
//! ```ignore
//! use crate::widgets::{
//!     helpers::{truncate_address, format_algo_amount},
//!     common::{TxnTypeBadge, AmountDisplay, AddressDisplay},
//!     list::{BlockListWidget, BlockListState, TransactionListWidget, TransactionListState},
//!     graph::{TxnGraph, GraphColumn, GraphRow},
//!     detail::{TxnFlowDiagram, TxnVisualCard},
//! };
//! ```

#![allow(unused_imports)] // Re-exports not yet used by main codebase

pub mod common;
pub mod detail;
pub mod graph;
pub mod helpers;
pub mod list;

// Re-export commonly used items at the module root for convenience
pub use common::{AddressDisplay, AmountDisplay, TxnTypeBadge};
pub use detail::{TxnFlowDiagram, TxnVisualCard};
pub use graph::{
    GraphColumn, GraphEntityType, GraphRepresentation, GraphRow, TxnGraph, TxnGraphWidget,
};
pub use helpers::{
    format_algo_amount, format_asset_amount, truncate_address, txn_type_code, txn_type_icon,
};
pub use list::{BlockListState, BlockListWidget, TransactionListState, TransactionListWidget};
