//! Popup UI components for the LazyLora TUI.
//!
//! This module contains all popup rendering logic including network selection,
//! search, search results, and message popups. Popups are modal overlays that
//! appear on top of the main UI and require user interaction to dismiss.

pub mod confirm;
pub mod help;
pub mod message;
pub mod network;
pub mod network_form;
pub mod search;
pub mod search_results;

// Re-export popup rendering functions for external API convenience.
// Used by library consumers who prefer `popups::render_*` over `popups::module::render`.
#[allow(unused_imports)]
pub use confirm::render as render_confirm_quit;
pub use help::render as render_help_popup;
#[allow(unused_imports)]
pub use message::render as render_message_popup;
#[allow(unused_imports)]
pub use network::render as render_network_selector;
#[allow(unused_imports)]
pub use network_form::render as render_network_form;
#[allow(unused_imports)]
pub use search::render as render_search_with_type_popup;
#[allow(unused_imports)]
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
        let _ = render_confirm_quit;
        let _ = render_help_popup;
        let _ = render_message_popup;
        let _ = render_network_selector;
        let _ = render_network_form;
        let _ = render_search_with_type_popup;
        let _ = render_search_results;
    }
}
