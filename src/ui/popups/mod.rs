//! Popup UI components for the LazyLora TUI.
//!
//! This module contains all popup rendering logic including network selection,
//! search, search results, and message popups. Popups are modal overlays that
//! appear on top of the main UI and require user interaction to dismiss.

#![allow(unused_imports)]

pub mod message;
pub mod network;
pub mod search;
pub mod search_results;

// Re-export popup rendering functions for convenience
pub use message::render as render_message_popup;
pub use network::render as render_network_selector;
pub use search::render as render_search_with_type_popup;
pub use search_results::render as render_search_results;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all popup functions are exported
        let _ = render_message_popup;
        let _ = render_network_selector;
        let _ = render_search_with_type_popup;
        let _ = render_search_results;
    }
}
