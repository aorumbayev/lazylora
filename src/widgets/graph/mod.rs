//! Transaction graph visualization widgets.
//!
//! This module provides components for visualizing transactions as flow graphs,
//! inspired by AlgoKit-Lora's web UI. The graph shows entities (accounts, apps, assets)
//! as columns and transactions as arrows between them.
//!
//! # Module Structure
//!
//! - [`types`]: Core type definitions (`GraphEntityType`, `GraphColumn`, `GraphRow`, `GraphRepresentation`)
//! - [`txn_graph`]: Graph data structure and construction logic
//!
//! # Example Usage
//!
//! ```ignore
//! use crate::widgets::graph::{TxnGraph, TxnGraphWidget};
//!
//! // Build a graph from a transaction
//! let graph = TxnGraph::from_transaction(&txn);
//!
//! // Render as ASCII art in the terminal
//! let widget = TxnGraphWidget::new(&graph);
//! frame.render_widget(widget, area);
//!
//! // Export to SVG
//! let svg = graph.to_svg();
//! ```

#![allow(dead_code)] // Transitional phase - items will be used after integration

pub mod renderer;
pub mod txn_graph;
pub mod types;

// Re-export main types at module level
pub use renderer::TxnGraphWidget;
pub use txn_graph::TxnGraph;
pub use types::{GraphColumn, GraphEntityType, GraphRepresentation, GraphRow};
