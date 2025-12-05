//! Command pattern for key event handling in the TUI application.
//!
//! This module provides a clean separation between key input and application actions,
//! making it easy to:
//! - Test key mappings in isolation
//! - Add new keybindings
//! - Support future keybinding customization
//!
//! # Example
//!
//! ```ignore
//! let context = app.get_input_context();
//! let command = map_key(key_event, &context);
//!
//! match command {
//!     AppCommand::Quit => app.exit = true,
//!     AppCommand::Refresh => app.initial_data_fetch().await,
//!     // ...
//! }
//! ```

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// ============================================================================
// Input Context
// ============================================================================

/// Represents the current input context for key mapping.
///
/// The input context determines which keybindings are active and how
/// key events should be interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputContext {
    /// Normal browsing mode - viewing blocks and transactions lists.
    Main,
    /// Viewing transaction details overlay.
    DetailView,
    /// Viewing block details overlay with tabs (Info / Transactions).
    BlockDetailView,
    /// Network selection popup is open.
    NetworkSelect,
    /// Search popup with text input is open.
    SearchInput,
    /// Viewing search results list.
    SearchResults,
    /// Viewing a message/notification popup.
    MessagePopup,
}

// ============================================================================
// App Commands
// ============================================================================

/// All possible commands the application can execute.
///
/// Commands are the result of mapping key events to application actions.
/// This enum represents the "what" of user intent, decoupled from the "how"
/// of key input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    // === Application Control ===
    /// Exit the application.
    Quit,
    /// Refresh data from the network.
    Refresh,
    /// Toggle live updates on/off.
    ToggleLive,

    // === Popup/Modal Control ===
    /// Open the search popup.
    OpenSearch,
    /// Open the network selection popup.
    OpenNetworkSelect,
    /// Dismiss/close the current popup or detail view.
    Dismiss,

    // === Navigation ===
    /// Cycle focus between panels (blocks, transactions, sidebar).
    CycleFocus,
    /// Move selection up in the current list.
    MoveUp,
    /// Move selection down in the current list.
    MoveDown,
    /// Select/confirm the current item (open details, confirm selection).
    Select,

    // === Detail View Actions ===
    /// Copy the current transaction ID to clipboard.
    CopyToClipboard,
    /// Toggle between Visual and Table view modes in detail popup.
    ToggleDetailViewMode,
    /// Move to previous expandable section in detail view.
    DetailSectionUp,
    /// Move to next expandable section in detail view.
    DetailSectionDown,
    /// Toggle expand/collapse of the current section.
    ToggleDetailSection,
    /// Scroll graph view left.
    GraphScrollLeft,
    /// Scroll graph view right.
    GraphScrollRight,
    /// Scroll graph view up.
    GraphScrollUp,
    /// Scroll graph view down.
    GraphScrollDown,
    /// Export transaction graph as SVG file.
    ExportSvg,

    // === Search Input Actions ===
    /// Type a character in the search input.
    TypeChar(char),
    /// Delete the last character in the search input.
    Backspace,
    /// Cycle through search types (Transaction, Block, Account, Asset).
    CycleSearchType,
    /// Submit the current search query.
    SubmitSearch,

    // === Network Selection Actions ===
    /// Move up in the network selection list.
    NetworkUp,
    /// Move down in the network selection list.
    NetworkDown,
    /// Select the currently highlighted network.
    SelectNetwork,

    // === Search Results Actions ===
    /// Move to the previous search result.
    PreviousResult,
    /// Move to the next search result.
    NextResult,
    /// Select the current search result.
    SelectResult,

    // === Block Detail View Actions ===
    /// Cycle between block detail tabs (Info / Transactions).
    CycleBlockDetailTab,
    /// Move up in block transactions list.
    MoveBlockTxnUp,
    /// Move down in block transactions list.
    MoveBlockTxnDown,
    /// Select transaction from block details to view full details.
    SelectBlockTxn,

    // === No Operation ===
    /// No action to perform (unhandled key).
    Noop,
}

// ============================================================================
// Key Mapping Functions
// ============================================================================

/// Maps a key event to an application command based on the current context.
///
/// This is a pure function with no side effects - it simply translates
/// input events to semantic commands, providing a centralized place for
/// all keybinding logic.
///
/// # Arguments
///
/// * `key` - The key event to map
/// * `context` - The current input context
///
/// # Returns
///
/// The appropriate `AppCommand` for the given key and context.
#[must_use]
pub fn map_key(key: KeyEvent, context: &InputContext) -> AppCommand {
    match context {
        InputContext::Main => map_main_keys(key),
        InputContext::DetailView => map_detail_view_keys(key),
        InputContext::BlockDetailView => map_block_detail_view_keys(key),
        InputContext::NetworkSelect => map_network_select_keys(key),
        InputContext::SearchInput => map_search_input_keys(key),
        InputContext::SearchResults => map_search_results_keys(key),
        InputContext::MessagePopup => map_message_popup_keys(key),
    }
}

/// Maps keys in the main browsing context.
fn map_main_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Char('q') => AppCommand::Quit,
        KeyCode::Char('r') => AppCommand::Refresh,
        KeyCode::Char(' ') => AppCommand::ToggleLive,
        KeyCode::Char('f') => AppCommand::OpenSearch,
        KeyCode::Char('n') => AppCommand::OpenNetworkSelect,
        KeyCode::Tab => AppCommand::CycleFocus,
        KeyCode::Up => AppCommand::MoveUp,
        KeyCode::Down => AppCommand::MoveDown,
        KeyCode::Enter => AppCommand::Select,
        KeyCode::Esc => AppCommand::Dismiss,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the detail view context.
fn map_detail_view_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Char('c') => AppCommand::CopyToClipboard,
        KeyCode::Char('s') => AppCommand::ExportSvg,
        KeyCode::Char('q') => AppCommand::Quit,
        KeyCode::Tab => AppCommand::ToggleDetailViewMode,
        // Arrow keys for graph scrolling (Visual mode) and section navigation (Table mode)
        KeyCode::Up => AppCommand::GraphScrollUp,
        KeyCode::Down => AppCommand::GraphScrollDown,
        KeyCode::Left => AppCommand::GraphScrollLeft,
        KeyCode::Right => AppCommand::GraphScrollRight,
        // j/k for section navigation in Table mode
        KeyCode::Char('j') => AppCommand::DetailSectionDown,
        KeyCode::Char('k') => AppCommand::DetailSectionUp,
        KeyCode::Enter | KeyCode::Char(' ') => AppCommand::ToggleDetailSection,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the block detail view context.
fn map_block_detail_view_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Tab => AppCommand::CycleBlockDetailTab,
        KeyCode::Up => AppCommand::MoveBlockTxnUp,
        KeyCode::Down => AppCommand::MoveBlockTxnDown,
        KeyCode::Enter => AppCommand::SelectBlockTxn,
        KeyCode::Char('q') => AppCommand::Quit,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the network selection popup.
fn map_network_select_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Up => AppCommand::NetworkUp,
        KeyCode::Down => AppCommand::NetworkDown,
        KeyCode::Enter => AppCommand::SelectNetwork,
        KeyCode::Char('q') => AppCommand::Dismiss,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the search input popup.
fn map_search_input_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Enter => AppCommand::SubmitSearch,
        KeyCode::Tab => AppCommand::CycleSearchType,
        KeyCode::Backspace => AppCommand::Backspace,
        KeyCode::Char(c) => {
            // Handle Ctrl+C as quit in search mode too
            if c == 'c' && key.modifiers.contains(KeyModifiers::CONTROL) {
                AppCommand::Dismiss
            } else {
                AppCommand::TypeChar(c)
            }
        }
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the search results popup.
fn map_search_results_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Up => AppCommand::PreviousResult,
        KeyCode::Down => AppCommand::NextResult,
        KeyCode::Enter => AppCommand::SelectResult,
        KeyCode::Char('q') => AppCommand::Dismiss,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the message popup.
fn map_message_popup_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => AppCommand::Dismiss,
        KeyCode::Char('q') => AppCommand::Quit,
        _ => AppCommand::Noop,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    /// Helper to create a key event for testing.
    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    /// Helper to create a key event with modifiers.
    fn key_event_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn test_main_context_key_mappings() {
        let cases = [
            (KeyCode::Char('q'), AppCommand::Quit),
            (KeyCode::Char('r'), AppCommand::Refresh),
            (KeyCode::Char(' '), AppCommand::ToggleLive),
            (KeyCode::Char('f'), AppCommand::OpenSearch),
            (KeyCode::Char('n'), AppCommand::OpenNetworkSelect),
            (KeyCode::Tab, AppCommand::CycleFocus),
            (KeyCode::Up, AppCommand::MoveUp),
            (KeyCode::Down, AppCommand::MoveDown),
            (KeyCode::Enter, AppCommand::Select),
            (KeyCode::Esc, AppCommand::Dismiss),
            (KeyCode::F(1), AppCommand::Noop),
        ];

        for (key_code, expected) in cases {
            let result = map_key(key_event(key_code), &InputContext::Main);
            assert_eq!(result, expected, "Key {:?} in Main context", key_code);
        }
    }

    #[test]
    fn test_detail_view_key_mappings() {
        let cases = [
            (KeyCode::Esc, AppCommand::Dismiss),
            (KeyCode::Char('c'), AppCommand::CopyToClipboard),
            (KeyCode::Char('s'), AppCommand::ExportSvg),
            (KeyCode::Char('q'), AppCommand::Quit),
            (KeyCode::Tab, AppCommand::ToggleDetailViewMode),
            (KeyCode::Up, AppCommand::GraphScrollUp),
            (KeyCode::Down, AppCommand::GraphScrollDown),
            (KeyCode::Left, AppCommand::GraphScrollLeft),
            (KeyCode::Right, AppCommand::GraphScrollRight),
            (KeyCode::Char('j'), AppCommand::DetailSectionDown),
            (KeyCode::Char('k'), AppCommand::DetailSectionUp),
            (KeyCode::Enter, AppCommand::ToggleDetailSection),
            (KeyCode::Char(' '), AppCommand::ToggleDetailSection),
            (KeyCode::F(1), AppCommand::Noop),
        ];

        for (key_code, expected) in cases {
            let result = map_key(key_event(key_code), &InputContext::DetailView);
            assert_eq!(result, expected, "Key {:?} in DetailView context", key_code);
        }
    }

    #[test]
    fn test_block_detail_view_key_mappings() {
        let cases = [
            (KeyCode::Esc, AppCommand::Dismiss),
            (KeyCode::Tab, AppCommand::CycleBlockDetailTab),
            (KeyCode::Up, AppCommand::MoveBlockTxnUp),
            (KeyCode::Down, AppCommand::MoveBlockTxnDown),
            (KeyCode::Enter, AppCommand::SelectBlockTxn),
            (KeyCode::Char('q'), AppCommand::Quit),
            (KeyCode::Char('x'), AppCommand::Noop),
            (KeyCode::F(1), AppCommand::Noop),
        ];

        for (key_code, expected) in cases {
            let result = map_key(key_event(key_code), &InputContext::BlockDetailView);
            assert_eq!(
                result, expected,
                "Key {:?} in BlockDetailView context",
                key_code
            );
        }
    }

    #[test]
    fn test_network_select_key_mappings() {
        let cases = [
            (KeyCode::Esc, AppCommand::Dismiss),
            (KeyCode::Up, AppCommand::NetworkUp),
            (KeyCode::Down, AppCommand::NetworkDown),
            (KeyCode::Enter, AppCommand::SelectNetwork),
            (KeyCode::Char('q'), AppCommand::Dismiss),
            (KeyCode::F(1), AppCommand::Noop),
        ];

        for (key_code, expected) in cases {
            let result = map_key(key_event(key_code), &InputContext::NetworkSelect);
            assert_eq!(
                result, expected,
                "Key {:?} in NetworkSelect context",
                key_code
            );
        }
    }

    #[test]
    fn test_search_input_key_mappings() {
        // Test cases without modifiers
        let cases = [
            (KeyCode::Esc, AppCommand::Dismiss),
            (KeyCode::Enter, AppCommand::SubmitSearch),
            (KeyCode::Tab, AppCommand::CycleSearchType),
            (KeyCode::Backspace, AppCommand::Backspace),
            (KeyCode::Char('a'), AppCommand::TypeChar('a')),
            (KeyCode::Char('1'), AppCommand::TypeChar('1')),
            (KeyCode::F(1), AppCommand::Noop),
        ];

        for (key_code, expected) in cases {
            let result = map_key(key_event(key_code), &InputContext::SearchInput);
            assert_eq!(
                result, expected,
                "Key {:?} in SearchInput context",
                key_code
            );
        }

        // Test Ctrl+C special case
        let ctrl_c_result = map_key(
            key_event_with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &InputContext::SearchInput,
        );
        assert_eq!(
            ctrl_c_result,
            AppCommand::Dismiss,
            "Ctrl+C should dismiss in SearchInput context"
        );
    }

    #[test]
    fn test_search_results_key_mappings() {
        let cases = [
            (KeyCode::Esc, AppCommand::Dismiss),
            (KeyCode::Up, AppCommand::PreviousResult),
            (KeyCode::Down, AppCommand::NextResult),
            (KeyCode::Enter, AppCommand::SelectResult),
            (KeyCode::Char('q'), AppCommand::Dismiss),
            (KeyCode::F(1), AppCommand::Noop),
        ];

        for (key_code, expected) in cases {
            let result = map_key(key_event(key_code), &InputContext::SearchResults);
            assert_eq!(
                result, expected,
                "Key {:?} in SearchResults context",
                key_code
            );
        }
    }

    #[test]
    fn test_message_popup_key_mappings() {
        let cases = [
            (KeyCode::Esc, AppCommand::Dismiss),
            (KeyCode::Enter, AppCommand::Dismiss),
            (KeyCode::Char(' '), AppCommand::Dismiss),
            (KeyCode::Char('q'), AppCommand::Quit),
            (KeyCode::F(1), AppCommand::Noop),
        ];

        for (key_code, expected) in cases {
            let result = map_key(key_event(key_code), &InputContext::MessagePopup);
            assert_eq!(
                result, expected,
                "Key {:?} in MessagePopup context",
                key_code
            );
        }
    }
}
