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
//! let command = KeyMapper::map_key(key_event, &context);
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

impl InputContext {
    /// Returns `true` if this context represents a popup/overlay state.
    #[must_use]
    #[allow(dead_code)]
    pub const fn is_popup(&self) -> bool {
        !matches!(self, Self::Main)
    }

    /// Returns `true` if this context accepts text input.
    #[must_use]
    #[allow(dead_code)]
    pub const fn accepts_text_input(&self) -> bool {
        matches!(self, Self::SearchInput)
    }
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

impl AppCommand {
    /// Returns `true` if this command would modify application state.
    #[must_use]
    #[allow(dead_code)]
    pub const fn is_mutating(&self) -> bool {
        !matches!(self, Self::Noop)
    }

    /// Returns `true` if this command would exit the application.
    #[must_use]
    #[allow(dead_code)]
    pub const fn is_exit(&self) -> bool {
        matches!(self, Self::Quit)
    }

    /// Returns `true` if this command is a navigation action.
    #[must_use]
    #[allow(dead_code)]
    pub const fn is_navigation(&self) -> bool {
        matches!(
            self,
            Self::MoveUp
                | Self::MoveDown
                | Self::CycleFocus
                | Self::NetworkUp
                | Self::NetworkDown
                | Self::PreviousResult
                | Self::NextResult
                | Self::MoveBlockTxnUp
                | Self::MoveBlockTxnDown
                | Self::GraphScrollLeft
                | Self::GraphScrollRight
                | Self::GraphScrollUp
                | Self::GraphScrollDown
        )
    }
}

// ============================================================================
// Key Mapper
// ============================================================================

/// Maps key events to application commands based on the current input context.
///
/// The `KeyMapper` provides a centralized place for all keybinding logic,
/// making it easy to understand, test, and modify the application's key handling.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyMapper;

impl KeyMapper {
    /// Creates a new `KeyMapper` instance.
    #[must_use]
    #[allow(dead_code)]
    pub const fn new() -> Self {
        Self
    }

    /// Maps a key event to an application command based on the current context.
    ///
    /// This is a pure function with no side effects - it simply translates
    /// input events to semantic commands.
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
            InputContext::Main => Self::map_main_keys(key),
            InputContext::DetailView => Self::map_detail_view_keys(key),
            InputContext::BlockDetailView => Self::map_block_detail_view_keys(key),
            InputContext::NetworkSelect => Self::map_network_select_keys(key),
            InputContext::SearchInput => Self::map_search_input_keys(key),
            InputContext::SearchResults => Self::map_search_results_keys(key),
            InputContext::MessagePopup => Self::map_message_popup_keys(key),
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

    mod input_context_tests {
        use super::*;

        #[test]
        fn test_main_is_not_popup() {
            assert!(!InputContext::Main.is_popup());
        }

        #[test]
        fn test_detail_view_is_popup() {
            assert!(InputContext::DetailView.is_popup());
        }

        #[test]
        fn test_network_select_is_popup() {
            assert!(InputContext::NetworkSelect.is_popup());
        }

        #[test]
        fn test_search_input_is_popup() {
            assert!(InputContext::SearchInput.is_popup());
        }

        #[test]
        fn test_search_results_is_popup() {
            assert!(InputContext::SearchResults.is_popup());
        }

        #[test]
        fn test_message_popup_is_popup() {
            assert!(InputContext::MessagePopup.is_popup());
        }

        #[test]
        fn test_only_search_input_accepts_text() {
            assert!(!InputContext::Main.accepts_text_input());
            assert!(!InputContext::DetailView.accepts_text_input());
            assert!(!InputContext::BlockDetailView.accepts_text_input());
            assert!(!InputContext::NetworkSelect.accepts_text_input());
            assert!(InputContext::SearchInput.accepts_text_input());
            assert!(!InputContext::SearchResults.accepts_text_input());
            assert!(!InputContext::MessagePopup.accepts_text_input());
        }

        #[test]
        fn test_block_detail_view_is_popup() {
            assert!(InputContext::BlockDetailView.is_popup());
        }
    }

    mod app_command_tests {
        use super::*;

        #[test]
        fn test_noop_is_not_mutating() {
            assert!(!AppCommand::Noop.is_mutating());
        }

        #[test]
        fn test_quit_is_mutating() {
            assert!(AppCommand::Quit.is_mutating());
        }

        #[test]
        fn test_quit_is_exit() {
            assert!(AppCommand::Quit.is_exit());
            assert!(!AppCommand::Refresh.is_exit());
            assert!(!AppCommand::Noop.is_exit());
        }

        #[test]
        fn test_navigation_commands() {
            assert!(AppCommand::MoveUp.is_navigation());
            assert!(AppCommand::MoveDown.is_navigation());
            assert!(AppCommand::CycleFocus.is_navigation());
            assert!(AppCommand::NetworkUp.is_navigation());
            assert!(AppCommand::NetworkDown.is_navigation());
            assert!(AppCommand::PreviousResult.is_navigation());
            assert!(AppCommand::NextResult.is_navigation());
            assert!(AppCommand::MoveBlockTxnUp.is_navigation());
            assert!(AppCommand::MoveBlockTxnDown.is_navigation());

            assert!(!AppCommand::Quit.is_navigation());
            assert!(!AppCommand::Select.is_navigation());
            assert!(!AppCommand::Noop.is_navigation());
        }
    }

    mod main_context_mapping_tests {
        use super::*;

        #[test]
        fn test_q_quits() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char('q')), &InputContext::Main);
            assert_eq!(cmd, AppCommand::Quit);
        }

        #[test]
        fn test_r_refreshes() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char('r')), &InputContext::Main);
            assert_eq!(cmd, AppCommand::Refresh);
        }

        #[test]
        fn test_space_toggles_live() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char(' ')), &InputContext::Main);
            assert_eq!(cmd, AppCommand::ToggleLive);
        }

        #[test]
        fn test_f_opens_search() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char('f')), &InputContext::Main);
            assert_eq!(cmd, AppCommand::OpenSearch);
        }

        #[test]
        fn test_n_opens_network_select() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char('n')), &InputContext::Main);
            assert_eq!(cmd, AppCommand::OpenNetworkSelect);
        }

        #[test]
        fn test_tab_cycles_focus() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Tab), &InputContext::Main);
            assert_eq!(cmd, AppCommand::CycleFocus);
        }

        #[test]
        fn test_up_moves_up() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Up), &InputContext::Main);
            assert_eq!(cmd, AppCommand::MoveUp);
        }

        #[test]
        fn test_down_moves_down() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Down), &InputContext::Main);
            assert_eq!(cmd, AppCommand::MoveDown);
        }

        #[test]
        fn test_enter_selects() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Enter), &InputContext::Main);
            assert_eq!(cmd, AppCommand::Select);
        }

        #[test]
        fn test_unknown_key_is_noop() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::F(1)), &InputContext::Main);
            assert_eq!(cmd, AppCommand::Noop);
        }
    }

    mod detail_view_mapping_tests {
        use super::*;

        #[test]
        fn test_esc_dismisses() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Esc), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::Dismiss);
        }

        #[test]
        fn test_c_copies_to_clipboard() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char('c')), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::CopyToClipboard);
        }

        #[test]
        fn test_q_quits_from_detail() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char('q')), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::Quit);
        }

        #[test]
        fn test_tab_toggles_detail_view_mode() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Tab), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::ToggleDetailViewMode);
        }

        #[test]
        fn test_up_scrolls_graph_up() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Up), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::GraphScrollUp);
        }

        #[test]
        fn test_down_scrolls_graph_down() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Down), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::GraphScrollDown);
        }

        #[test]
        fn test_left_scrolls_graph_left() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Left), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::GraphScrollLeft);
        }

        #[test]
        fn test_right_scrolls_graph_right() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Right), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::GraphScrollRight);
        }

        #[test]
        fn test_j_moves_section_down() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char('j')), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::DetailSectionDown);
        }

        #[test]
        fn test_k_moves_section_up() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char('k')), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::DetailSectionUp);
        }

        #[test]
        fn test_s_exports_svg() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char('s')), &InputContext::DetailView);
            assert_eq!(cmd, AppCommand::ExportSvg);
        }
    }

    mod network_select_mapping_tests {
        use super::*;

        #[test]
        fn test_esc_dismisses() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Esc), &InputContext::NetworkSelect);
            assert_eq!(cmd, AppCommand::Dismiss);
        }

        #[test]
        fn test_up_moves_up() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Up), &InputContext::NetworkSelect);
            assert_eq!(cmd, AppCommand::NetworkUp);
        }

        #[test]
        fn test_down_moves_down() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Down), &InputContext::NetworkSelect);
            assert_eq!(cmd, AppCommand::NetworkDown);
        }

        #[test]
        fn test_enter_selects_network() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Enter), &InputContext::NetworkSelect);
            assert_eq!(cmd, AppCommand::SelectNetwork);
        }
    }

    mod search_input_mapping_tests {
        use super::*;

        #[test]
        fn test_esc_dismisses() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Esc), &InputContext::SearchInput);
            assert_eq!(cmd, AppCommand::Dismiss);
        }

        #[test]
        fn test_enter_submits_search() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Enter), &InputContext::SearchInput);
            assert_eq!(cmd, AppCommand::SubmitSearch);
        }

        #[test]
        fn test_tab_cycles_search_type() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Tab), &InputContext::SearchInput);
            assert_eq!(cmd, AppCommand::CycleSearchType);
        }

        #[test]
        fn test_backspace_deletes() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Backspace), &InputContext::SearchInput);
            assert_eq!(cmd, AppCommand::Backspace);
        }

        #[test]
        fn test_char_types() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Char('a')), &InputContext::SearchInput);
            assert_eq!(cmd, AppCommand::TypeChar('a'));
        }

        #[test]
        fn test_ctrl_c_dismisses() {
            let cmd = KeyMapper::map_key(
                key_event_with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL),
                &InputContext::SearchInput,
            );
            assert_eq!(cmd, AppCommand::Dismiss);
        }
    }

    mod search_results_mapping_tests {
        use super::*;

        #[test]
        fn test_esc_dismisses() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Esc), &InputContext::SearchResults);
            assert_eq!(cmd, AppCommand::Dismiss);
        }

        #[test]
        fn test_up_goes_to_previous() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Up), &InputContext::SearchResults);
            assert_eq!(cmd, AppCommand::PreviousResult);
        }

        #[test]
        fn test_down_goes_to_next() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Down), &InputContext::SearchResults);
            assert_eq!(cmd, AppCommand::NextResult);
        }

        #[test]
        fn test_enter_selects_result() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Enter), &InputContext::SearchResults);
            assert_eq!(cmd, AppCommand::SelectResult);
        }
    }

    mod message_popup_mapping_tests {
        use super::*;

        #[test]
        fn test_esc_dismisses() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Esc), &InputContext::MessagePopup);
            assert_eq!(cmd, AppCommand::Dismiss);
        }

        #[test]
        fn test_enter_dismisses() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Enter), &InputContext::MessagePopup);
            assert_eq!(cmd, AppCommand::Dismiss);
        }

        #[test]
        fn test_space_dismisses() {
            let cmd =
                KeyMapper::map_key(key_event(KeyCode::Char(' ')), &InputContext::MessagePopup);
            assert_eq!(cmd, AppCommand::Dismiss);
        }

        #[test]
        fn test_q_quits() {
            let cmd =
                KeyMapper::map_key(key_event(KeyCode::Char('q')), &InputContext::MessagePopup);
            assert_eq!(cmd, AppCommand::Quit);
        }
    }

    mod block_detail_view_mapping_tests {
        use super::*;

        #[test]
        fn test_esc_dismisses() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Esc), &InputContext::BlockDetailView);
            assert_eq!(cmd, AppCommand::Dismiss);
        }

        #[test]
        fn test_tab_cycles_block_detail_tab() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Tab), &InputContext::BlockDetailView);
            assert_eq!(cmd, AppCommand::CycleBlockDetailTab);
        }

        #[test]
        fn test_up_moves_block_txn_up() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Up), &InputContext::BlockDetailView);
            assert_eq!(cmd, AppCommand::MoveBlockTxnUp);
        }

        #[test]
        fn test_down_moves_block_txn_down() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Down), &InputContext::BlockDetailView);
            assert_eq!(cmd, AppCommand::MoveBlockTxnDown);
        }

        #[test]
        fn test_enter_selects_block_txn() {
            let cmd = KeyMapper::map_key(key_event(KeyCode::Enter), &InputContext::BlockDetailView);
            assert_eq!(cmd, AppCommand::SelectBlockTxn);
        }

        #[test]
        fn test_q_quits_from_block_detail() {
            let cmd = KeyMapper::map_key(
                key_event(KeyCode::Char('q')),
                &InputContext::BlockDetailView,
            );
            assert_eq!(cmd, AppCommand::Quit);
        }

        #[test]
        fn test_unknown_key_is_noop() {
            let cmd = KeyMapper::map_key(
                key_event(KeyCode::Char('x')),
                &InputContext::BlockDetailView,
            );
            assert_eq!(cmd, AppCommand::Noop);
        }
    }
}
