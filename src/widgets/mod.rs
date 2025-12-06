//! Modular widget components for the LazyLora TUI.
//!
//! This module provides reusable widget components that render rich ASCII
//! visualizations for Algorand blockchain data.
//!
//! # Module Structure
//!
//! - [`helpers`]: Utility functions for formatting addresses and amounts
//! - [`graph`]: Transaction graph visualization (columns and flow arrows)
//! - [`detail`]: Detailed transaction views (visual cards)

pub mod detail;
pub mod graph;
pub mod helpers;

// Re-export commonly used items at the module root for convenience
// Only re-export types that are actually used outside this module
pub use detail::TxnVisualCard;
pub use graph::{TxnGraph, TxnGraphWidget};
