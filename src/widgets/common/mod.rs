//! Common reusable widget components.
//!
//! This module contains small, reusable widgets that can be composed into larger views:
//!
//! - [`TxnTypeBadge`]: A colored badge showing transaction type with an icon
//! - [`AmountDisplay`]: Formatted display for ALGO or ASA amounts
//! - [`AddressDisplay`]: Truncated address display with optional labels

mod address;
mod amount;
mod badge;

pub use address::AddressDisplay;
pub use amount::AmountDisplay;
pub use badge::TxnTypeBadge;
