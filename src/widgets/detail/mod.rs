//! Transaction detail visualization widgets.
//!
//! This module provides widgets for displaying detailed transaction information,
//! including flow diagrams and visual cards that combine multiple pieces of
//! transaction data into a cohesive view.
//!
//! # Module Structure
//!
//! - [`flow_diagram`]: ASCII art flow diagram showing transaction sender/receiver
//! - [`visual_card`]: Comprehensive transaction card with type-specific details
//!
//! # Example Usage
//!
//! ```ignore
//! use crate::widgets::detail::{TxnFlowDiagram, TxnVisualCard};
//!
//! // Create a flow diagram for a transaction
//! let diagram = TxnFlowDiagram::new(&transaction);
//! let lines = diagram.to_lines();
//!
//! // Create a visual card with all transaction details
//! let card = TxnVisualCard::new(&transaction);
//! frame.render_widget(card, area);
//! ```

pub mod flow_diagram;
pub mod visual_card;

// Re-export main types at module level
pub use flow_diagram::TxnFlowDiagram;
pub use visual_card::TxnVisualCard;
