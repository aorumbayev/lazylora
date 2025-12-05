//! List widgets for displaying blocks and transactions.
//!
//! This module contains stateful list widgets that support selection and scrolling:
//!
//! - [`BlockListWidget`]: Displays a list of blocks with selection highlighting
//! - [`TransactionListWidget`]: Displays a list of transactions with selection highlighting
//! - [`BlockListState`]: State management for block lists
//! - [`TransactionListState`]: State management for transaction lists

mod block_list;
mod state;
mod txn_list;

pub use block_list::BlockListWidget;
pub use state::{BlockListState, TransactionListState};
pub use txn_list::TransactionListWidget;

// Re-export the scrollbar helper for internal use
pub(crate) use state::render_list_scrollbar;
