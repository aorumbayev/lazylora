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
//!     AppCommand::ConfirmQuit => app.exit = true,
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
    /// Viewing transaction details overlay in Visual/Graph mode.
    DetailView,
    /// Viewing transaction details overlay in Table mode (with tabs).
    TxnDetailViewTable,
    /// Viewing block details overlay with tabs (Info / Transactions).
    BlockDetailView,
    /// Viewing account details overlay with tabs (Info / Assets / Apps).
    AccountDetailView,
    /// Viewing application details overlay with tabs (Info / State / Programs).
    AppDetailView,
    /// Network selection popup is open.
    NetworkSelect,
    /// Search popup with text input is open.
    SearchInput,
    /// Inline search bar in header is focused.
    InlineSearch,
    /// Viewing search results list.
    SearchResults,
    /// Viewing a message/notification popup.
    MessagePopup,
    /// Viewing the help popup with keybindings.
    HelpPopup,
    /// Quit confirmation popup is open.
    ConfirmQuit,
    /// Adding or editing a custom network.
    NetworkForm,
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
    /// Request quit confirmation (show popup).
    RequestQuit,
    /// Confirm quit from the confirmation popup.
    ConfirmQuit,
    /// Refresh data from the network.
    Refresh,
    /// Toggle live updates on/off.
    ToggleLive,
    /// Toggle the help popup.
    ToggleHelp,

    // === Popup/Modal Control ===
    /// Focus the inline search bar in the header.
    FocusInlineSearch,
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
    /// Jump to the top of the current list.
    GoToTop,
    /// Jump to the bottom of the current list.
    GoToBottom,
    /// Select/confirm the current item (open details, confirm selection).
    Select,

    // === Detail View Actions ===
    /// Copy the current transaction ID to clipboard.
    CopyToClipboard,
    /// Copy raw JSON to clipboard.
    CopyJson,
    /// Open current entity in web browser (Lora).
    OpenInBrowser,
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
    /// Toggle fullscreen mode for detail popups.
    ToggleFullscreen,

    // === Search Input Actions ===
    /// Type a character in the search input.
    TypeChar(char),
    /// Delete the last character in the search input.
    Backspace,
    /// Cycle through search types (Transaction, Block, Account, Asset).
    CycleSearchType,
    /// Submit the current search query.
    SubmitSearch,
    /// Navigate to previous search in history.
    SearchHistoryPrev,
    /// Navigate to next search in history.
    SearchHistoryNext,
    /// Move cursor left in search input.
    SearchCursorLeft,
    /// Move cursor right in search input.
    SearchCursorRight,

    // === Network Selection Actions ===
    /// Move up in the network selection list.
    NetworkUp,
    /// Move down in the network selection list.
    NetworkDown,
    /// Select the currently highlighted network.
    SelectNetwork,
    /// Add a new custom network.
    AddNetwork,
    /// Delete the selected custom network.
    DeleteNetwork,

    // === Search Results Actions ===
    /// Move to the previous search result.
    PreviousResult,
    /// Move to the next search result.
    NextResult,
    /// Select the current search result.
    SelectResult,

    // === Help Popup Actions ===
    /// Scroll help popup up.
    ScrollHelpUp,
    /// Scroll help popup down.
    ScrollHelpDown,

    // === Block Detail View Actions ===
    /// Cycle between block detail tabs (Info / Transactions).
    CycleBlockDetailTab,
    /// Move up in block transactions list.
    MoveBlockTxnUp,
    /// Move down in block transactions list.
    MoveBlockTxnDown,
    /// Select transaction from block details to view full details.
    SelectBlockTxn,

    // === Account Detail View Actions ===
    /// Cycle between account detail tabs (Info / Assets / Apps).
    CycleAccountDetailTab,
    /// Move up in account item list (assets or apps).
    MoveAccountItemUp,
    /// Move down in account item list (assets or apps).
    MoveAccountItemDown,
    /// Select asset or app from account details to view full details.
    SelectAccountItem,

    // === Application Detail View Actions ===
    /// Cycle between app detail tabs (Info / State / Programs).
    CycleAppDetailTab,
    /// Move up in app state list.
    MoveAppStateUp,
    /// Move down in app state list.
    MoveAppStateDown,

    // === Network Form Actions ===
    /// Submit the custom network form.
    SubmitNetworkForm,
    /// Move to the next field in the network form.
    NetworkFormNextField,
    /// Move to the previous field in the network form.
    NetworkFormPrevField,

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
        InputContext::TxnDetailViewTable => map_txn_detail_view_table_keys(key),
        InputContext::BlockDetailView => map_block_detail_view_keys(key),
        InputContext::AccountDetailView => map_account_detail_view_keys(key),
        InputContext::AppDetailView => map_app_detail_view_keys(key),
        InputContext::NetworkSelect => map_network_select_keys(key),
        InputContext::SearchInput => map_search_input_keys(key),
        InputContext::InlineSearch => map_inline_search_keys(key),
        InputContext::SearchResults => map_search_results_keys(key),
        InputContext::MessagePopup => map_message_popup_keys(key),
        InputContext::HelpPopup => map_help_popup_keys(key),
        InputContext::ConfirmQuit => map_confirm_quit_keys(key),
        InputContext::NetworkForm => map_network_form_keys(key),
    }
}

/// Maps keys in the main browsing context.
fn map_main_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Char('q') => AppCommand::RequestQuit,
        KeyCode::Char('r') => AppCommand::Refresh,
        KeyCode::Char(' ') => AppCommand::ToggleLive,
        KeyCode::Char('?') => AppCommand::ToggleHelp,
        KeyCode::Char('f') => AppCommand::FocusInlineSearch,
        KeyCode::Char('n') => AppCommand::OpenNetworkSelect,
        KeyCode::Tab => AppCommand::CycleFocus,
        KeyCode::Char('k') | KeyCode::Up => AppCommand::MoveUp,
        KeyCode::Char('j') | KeyCode::Down => AppCommand::MoveDown,
        KeyCode::Char('g') => AppCommand::GoToTop,
        KeyCode::Char('G') => AppCommand::GoToBottom,
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
        KeyCode::Char('y') => AppCommand::CopyJson,
        KeyCode::Char('o') => AppCommand::OpenInBrowser,
        KeyCode::Char('s') => AppCommand::ExportSvg,
        KeyCode::Char('q') => AppCommand::RequestQuit,
        KeyCode::Char('f') => AppCommand::ToggleFullscreen,
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
        KeyCode::Char('k') | KeyCode::Up => AppCommand::MoveBlockTxnUp,
        KeyCode::Char('j') | KeyCode::Down => AppCommand::MoveBlockTxnDown,
        KeyCode::Char('g') => AppCommand::GoToTop,
        KeyCode::Char('G') => AppCommand::GoToBottom,
        KeyCode::Enter => AppCommand::SelectBlockTxn,
        KeyCode::Char('c') => AppCommand::CopyToClipboard,
        KeyCode::Char('y') => AppCommand::CopyJson,
        KeyCode::Char('o') => AppCommand::OpenInBrowser,
        KeyCode::Char('f') => AppCommand::ToggleFullscreen,
        KeyCode::Char('q') => AppCommand::RequestQuit,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the add custom network form.
fn map_network_form_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Enter => AppCommand::SubmitNetworkForm,
        KeyCode::Tab | KeyCode::Down => AppCommand::NetworkFormNextField,
        KeyCode::Up => AppCommand::NetworkFormPrevField,
        KeyCode::Backspace => AppCommand::Backspace,
        KeyCode::Char(c) => AppCommand::TypeChar(c),
        _ => AppCommand::Noop,
    }
}
/// Maps keys in the account detail view context.
fn map_account_detail_view_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Tab => AppCommand::CycleAccountDetailTab,
        KeyCode::Char('k') | KeyCode::Up => AppCommand::MoveAccountItemUp,
        KeyCode::Char('j') | KeyCode::Down => AppCommand::MoveAccountItemDown,
        KeyCode::Char('g') => AppCommand::GoToTop,
        KeyCode::Char('G') => AppCommand::GoToBottom,
        KeyCode::Enter => AppCommand::SelectAccountItem,
        KeyCode::Char('c') => AppCommand::CopyToClipboard,
        KeyCode::Char('y') => AppCommand::CopyJson,
        KeyCode::Char('o') => AppCommand::OpenInBrowser,
        KeyCode::Char('f') => AppCommand::ToggleFullscreen,
        KeyCode::Char('q') => AppCommand::RequestQuit,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the application detail view context.
fn map_app_detail_view_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Tab => AppCommand::CycleAppDetailTab,
        KeyCode::Char('k') | KeyCode::Up => AppCommand::MoveAppStateUp,
        KeyCode::Char('j') | KeyCode::Down => AppCommand::MoveAppStateDown,
        KeyCode::Char('g') => AppCommand::GoToTop,
        KeyCode::Char('G') => AppCommand::GoToBottom,
        KeyCode::Char('c') => AppCommand::CopyToClipboard,
        KeyCode::Char('y') => AppCommand::CopyJson,
        KeyCode::Char('o') => AppCommand::OpenInBrowser,
        KeyCode::Char('f') => AppCommand::ToggleFullscreen,
        KeyCode::Char('q') => AppCommand::RequestQuit,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the network selection popup.
fn map_network_select_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Char('k') | KeyCode::Up => AppCommand::NetworkUp,
        KeyCode::Char('j') | KeyCode::Down => AppCommand::NetworkDown,
        KeyCode::Char('g') => AppCommand::GoToTop,
        KeyCode::Char('G') => AppCommand::GoToBottom,
        KeyCode::Enter => AppCommand::SelectNetwork,
        KeyCode::Char('a') => AppCommand::AddNetwork,
        KeyCode::Char('d') => AppCommand::DeleteNetwork,
        KeyCode::Char('q') => AppCommand::RequestQuit,
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

/// Maps keys in the inline search bar context.
fn map_inline_search_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Enter => AppCommand::SubmitSearch,
        KeyCode::Backspace => AppCommand::Backspace,
        KeyCode::Tab => AppCommand::CycleSearchType,
        KeyCode::Up => AppCommand::SearchHistoryPrev,
        KeyCode::Down => AppCommand::SearchHistoryNext,
        KeyCode::Left => AppCommand::SearchCursorLeft,
        KeyCode::Right => AppCommand::SearchCursorRight,
        KeyCode::Char(c) => {
            // Handle Ctrl+C as dismiss
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
        KeyCode::Char('k') | KeyCode::Up => AppCommand::PreviousResult,
        KeyCode::Char('j') | KeyCode::Down => AppCommand::NextResult,
        KeyCode::Char('g') => AppCommand::GoToTop,
        KeyCode::Char('G') => AppCommand::GoToBottom,
        KeyCode::Enter => AppCommand::SelectResult,
        KeyCode::Char('q') => AppCommand::RequestQuit,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the message popup.
fn map_message_popup_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => AppCommand::Dismiss,
        KeyCode::Char('q') => AppCommand::RequestQuit,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the help popup.
fn map_help_popup_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?') => AppCommand::Dismiss,
        KeyCode::Char('k') | KeyCode::Up => AppCommand::ScrollHelpUp,
        KeyCode::Char('j') | KeyCode::Down => AppCommand::ScrollHelpDown,
        KeyCode::Char('g') => AppCommand::GoToTop,
        KeyCode::Char('G') => AppCommand::GoToBottom,
        KeyCode::Char('q') => AppCommand::RequestQuit,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the quit confirmation popup.
fn map_confirm_quit_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => AppCommand::ConfirmQuit,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => AppCommand::Dismiss,
        _ => AppCommand::Noop,
    }
}

/// Maps keys in the transaction detail view (Table mode) context.
fn map_txn_detail_view_table_keys(key: KeyEvent) -> AppCommand {
    match key.code {
        KeyCode::Esc => AppCommand::Dismiss,
        KeyCode::Tab => AppCommand::ToggleDetailViewMode,
        KeyCode::Char('k') | KeyCode::Up => AppCommand::DetailSectionUp,
        KeyCode::Char('j') | KeyCode::Down => AppCommand::DetailSectionDown,
        KeyCode::Char('g') => AppCommand::GoToTop,
        KeyCode::Char('G') => AppCommand::GoToBottom,
        KeyCode::Char('c') => AppCommand::CopyToClipboard,
        KeyCode::Char('y') => AppCommand::CopyJson,
        KeyCode::Char('o') => AppCommand::OpenInBrowser,
        KeyCode::Char('f') => AppCommand::ToggleFullscreen,
        KeyCode::Char('s') => AppCommand::ExportSvg,
        KeyCode::Char('q') => AppCommand::RequestQuit,
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
    use rstest::rstest;

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

    // ============================================================================
    // Consolidated Key Mapping Tests
    // ============================================================================

    /// Tests all key mappings for Main context.
    #[rstest]
    #[case::quit(KeyCode::Char('q'), AppCommand::RequestQuit)]
    #[case::refresh(KeyCode::Char('r'), AppCommand::Refresh)]
    #[case::toggle_live(KeyCode::Char(' '), AppCommand::ToggleLive)]
    #[case::toggle_help(KeyCode::Char('?'), AppCommand::ToggleHelp)]
    #[case::focus_search(KeyCode::Char('f'), AppCommand::FocusInlineSearch)]
    #[case::network_select(KeyCode::Char('n'), AppCommand::OpenNetworkSelect)]
    #[case::cycle_focus(KeyCode::Tab, AppCommand::CycleFocus)]
    #[case::move_up_arrow(KeyCode::Up, AppCommand::MoveUp)]
    #[case::move_up_vim(KeyCode::Char('k'), AppCommand::MoveUp)]
    #[case::move_down_arrow(KeyCode::Down, AppCommand::MoveDown)]
    #[case::move_down_vim(KeyCode::Char('j'), AppCommand::MoveDown)]
    #[case::go_top(KeyCode::Char('g'), AppCommand::GoToTop)]
    #[case::go_bottom(KeyCode::Char('G'), AppCommand::GoToBottom)]
    #[case::select(KeyCode::Enter, AppCommand::Select)]
    #[case::dismiss(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::noop(KeyCode::F(1), AppCommand::Noop)]
    fn test_main_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(map_key(key_event(key_code), &InputContext::Main), expected);
    }

    /// Tests all key mappings for DetailView context.
    #[rstest]
    #[case::dismiss(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::copy_clipboard(KeyCode::Char('c'), AppCommand::CopyToClipboard)]
    #[case::copy_json(KeyCode::Char('y'), AppCommand::CopyJson)]
    #[case::open_browser(KeyCode::Char('o'), AppCommand::OpenInBrowser)]
    #[case::export_svg(KeyCode::Char('s'), AppCommand::ExportSvg)]
    #[case::fullscreen(KeyCode::Char('f'), AppCommand::ToggleFullscreen)]
    #[case::quit(KeyCode::Char('q'), AppCommand::RequestQuit)]
    #[case::toggle_mode(KeyCode::Tab, AppCommand::ToggleDetailViewMode)]
    #[case::scroll_up(KeyCode::Up, AppCommand::GraphScrollUp)]
    #[case::scroll_down(KeyCode::Down, AppCommand::GraphScrollDown)]
    #[case::scroll_left(KeyCode::Left, AppCommand::GraphScrollLeft)]
    #[case::scroll_right(KeyCode::Right, AppCommand::GraphScrollRight)]
    #[case::section_down(KeyCode::Char('j'), AppCommand::DetailSectionDown)]
    #[case::section_up(KeyCode::Char('k'), AppCommand::DetailSectionUp)]
    #[case::toggle_section_enter(KeyCode::Enter, AppCommand::ToggleDetailSection)]
    #[case::toggle_section_space(KeyCode::Char(' '), AppCommand::ToggleDetailSection)]
    #[case::noop(KeyCode::F(1), AppCommand::Noop)]
    fn test_detail_view_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::DetailView),
            expected
        );
    }

    /// Tests all key mappings for BlockDetailView context.
    #[rstest]
    #[case::dismiss(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::cycle_tab(KeyCode::Tab, AppCommand::CycleBlockDetailTab)]
    #[case::move_up_arrow(KeyCode::Up, AppCommand::MoveBlockTxnUp)]
    #[case::move_up_vim(KeyCode::Char('k'), AppCommand::MoveBlockTxnUp)]
    #[case::move_down_arrow(KeyCode::Down, AppCommand::MoveBlockTxnDown)]
    #[case::move_down_vim(KeyCode::Char('j'), AppCommand::MoveBlockTxnDown)]
    #[case::go_top(KeyCode::Char('g'), AppCommand::GoToTop)]
    #[case::go_bottom(KeyCode::Char('G'), AppCommand::GoToBottom)]
    #[case::select(KeyCode::Enter, AppCommand::SelectBlockTxn)]
    #[case::copy_clipboard(KeyCode::Char('c'), AppCommand::CopyToClipboard)]
    #[case::copy_json(KeyCode::Char('y'), AppCommand::CopyJson)]
    #[case::open_browser(KeyCode::Char('o'), AppCommand::OpenInBrowser)]
    #[case::fullscreen(KeyCode::Char('f'), AppCommand::ToggleFullscreen)]
    #[case::quit(KeyCode::Char('q'), AppCommand::RequestQuit)]
    #[case::noop_x(KeyCode::Char('x'), AppCommand::Noop)]
    #[case::noop_f1(KeyCode::F(1), AppCommand::Noop)]
    fn test_block_detail_view_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::BlockDetailView),
            expected
        );
    }

    /// Tests all key mappings for AccountDetailView context.
    #[rstest]
    #[case::dismiss(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::cycle_tab(KeyCode::Tab, AppCommand::CycleAccountDetailTab)]
    #[case::move_up_arrow(KeyCode::Up, AppCommand::MoveAccountItemUp)]
    #[case::move_up_vim(KeyCode::Char('k'), AppCommand::MoveAccountItemUp)]
    #[case::move_down_arrow(KeyCode::Down, AppCommand::MoveAccountItemDown)]
    #[case::move_down_vim(KeyCode::Char('j'), AppCommand::MoveAccountItemDown)]
    #[case::go_top(KeyCode::Char('g'), AppCommand::GoToTop)]
    #[case::go_bottom(KeyCode::Char('G'), AppCommand::GoToBottom)]
    #[case::select(KeyCode::Enter, AppCommand::SelectAccountItem)]
    #[case::copy_clipboard(KeyCode::Char('c'), AppCommand::CopyToClipboard)]
    #[case::copy_json(KeyCode::Char('y'), AppCommand::CopyJson)]
    #[case::open_browser(KeyCode::Char('o'), AppCommand::OpenInBrowser)]
    #[case::fullscreen(KeyCode::Char('f'), AppCommand::ToggleFullscreen)]
    #[case::quit(KeyCode::Char('q'), AppCommand::RequestQuit)]
    #[case::noop_x(KeyCode::Char('x'), AppCommand::Noop)]
    #[case::noop_f1(KeyCode::F(1), AppCommand::Noop)]
    fn test_account_detail_view_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::AccountDetailView),
            expected
        );
    }

    /// Tests all key mappings for AppDetailView context.
    #[rstest]
    #[case::dismiss(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::cycle_tab(KeyCode::Tab, AppCommand::CycleAppDetailTab)]
    #[case::move_up_arrow(KeyCode::Up, AppCommand::MoveAppStateUp)]
    #[case::move_up_vim(KeyCode::Char('k'), AppCommand::MoveAppStateUp)]
    #[case::move_down_arrow(KeyCode::Down, AppCommand::MoveAppStateDown)]
    #[case::move_down_vim(KeyCode::Char('j'), AppCommand::MoveAppStateDown)]
    #[case::go_top(KeyCode::Char('g'), AppCommand::GoToTop)]
    #[case::go_bottom(KeyCode::Char('G'), AppCommand::GoToBottom)]
    #[case::copy_clipboard(KeyCode::Char('c'), AppCommand::CopyToClipboard)]
    #[case::copy_json(KeyCode::Char('y'), AppCommand::CopyJson)]
    #[case::open_browser(KeyCode::Char('o'), AppCommand::OpenInBrowser)]
    #[case::fullscreen(KeyCode::Char('f'), AppCommand::ToggleFullscreen)]
    #[case::quit(KeyCode::Char('q'), AppCommand::RequestQuit)]
    #[case::noop_x(KeyCode::Char('x'), AppCommand::Noop)]
    #[case::noop_f1(KeyCode::F(1), AppCommand::Noop)]
    fn test_app_detail_view_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::AppDetailView),
            expected
        );
    }

    /// Tests all key mappings for NetworkSelect context.
    #[rstest]
    #[case::dismiss(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::move_up_arrow(KeyCode::Up, AppCommand::NetworkUp)]
    #[case::move_up_vim(KeyCode::Char('k'), AppCommand::NetworkUp)]
    #[case::move_down_arrow(KeyCode::Down, AppCommand::NetworkDown)]
    #[case::move_down_vim(KeyCode::Char('j'), AppCommand::NetworkDown)]
    #[case::go_top(KeyCode::Char('g'), AppCommand::GoToTop)]
    #[case::go_bottom(KeyCode::Char('G'), AppCommand::GoToBottom)]
    #[case::select(KeyCode::Enter, AppCommand::SelectNetwork)]
    #[case::add_network(KeyCode::Char('a'), AppCommand::AddNetwork)]
    #[case::delete_network(KeyCode::Char('d'), AppCommand::DeleteNetwork)]
    #[case::quit_q(KeyCode::Char('q'), AppCommand::RequestQuit)]
    #[case::noop(KeyCode::F(1), AppCommand::Noop)]
    fn test_network_select_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::NetworkSelect),
            expected
        );
    }

    /// Tests key mappings for NetworkForm context.
    #[rstest]
    #[case::dismiss(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::submit(KeyCode::Enter, AppCommand::SubmitNetworkForm)]
    #[case::next_tab(KeyCode::Tab, AppCommand::NetworkFormNextField)]
    #[case::next_down(KeyCode::Down, AppCommand::NetworkFormNextField)]
    #[case::prev(KeyCode::Up, AppCommand::NetworkFormPrevField)]
    #[case::backspace(KeyCode::Backspace, AppCommand::Backspace)]
    #[case::type_char(KeyCode::Char('n'), AppCommand::TypeChar('n'))]
    fn test_network_form_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::NetworkForm),
            expected
        );
    }

    /// Tests all key mappings for SearchInput context.
    #[rstest]
    #[case::dismiss(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::submit(KeyCode::Enter, AppCommand::SubmitSearch)]
    #[case::cycle_type(KeyCode::Tab, AppCommand::CycleSearchType)]
    #[case::backspace(KeyCode::Backspace, AppCommand::Backspace)]
    #[case::type_a(KeyCode::Char('a'), AppCommand::TypeChar('a'))]
    #[case::type_1(KeyCode::Char('1'), AppCommand::TypeChar('1'))]
    #[case::noop(KeyCode::F(1), AppCommand::Noop)]
    fn test_search_input_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::SearchInput),
            expected
        );
    }

    /// Tests Ctrl+C special case in SearchInput context.
    #[test]
    fn test_search_input_ctrl_c() {
        assert_eq!(
            map_key(
                key_event_with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL),
                &InputContext::SearchInput
            ),
            AppCommand::Dismiss
        );
    }

    /// Tests all key mappings for InlineSearch context.
    #[rstest]
    #[case::dismiss(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::submit(KeyCode::Enter, AppCommand::SubmitSearch)]
    #[case::backspace(KeyCode::Backspace, AppCommand::Backspace)]
    #[case::cycle_type(KeyCode::Tab, AppCommand::CycleSearchType)]
    #[case::history_prev(KeyCode::Up, AppCommand::SearchHistoryPrev)]
    #[case::history_next(KeyCode::Down, AppCommand::SearchHistoryNext)]
    #[case::cursor_left(KeyCode::Left, AppCommand::SearchCursorLeft)]
    #[case::cursor_right(KeyCode::Right, AppCommand::SearchCursorRight)]
    #[case::type_a(KeyCode::Char('a'), AppCommand::TypeChar('a'))]
    #[case::type_1(KeyCode::Char('1'), AppCommand::TypeChar('1'))]
    #[case::noop(KeyCode::F(1), AppCommand::Noop)]
    fn test_inline_search_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::InlineSearch),
            expected
        );
    }

    /// Tests Ctrl+C special case in InlineSearch context.
    #[test]
    fn test_inline_search_ctrl_c() {
        assert_eq!(
            map_key(
                key_event_with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL),
                &InputContext::InlineSearch
            ),
            AppCommand::Dismiss
        );
    }

    /// Tests all key mappings for SearchResults context.
    #[rstest]
    #[case::dismiss(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::prev_arrow(KeyCode::Up, AppCommand::PreviousResult)]
    #[case::prev_vim(KeyCode::Char('k'), AppCommand::PreviousResult)]
    #[case::next_arrow(KeyCode::Down, AppCommand::NextResult)]
    #[case::next_vim(KeyCode::Char('j'), AppCommand::NextResult)]
    #[case::go_top(KeyCode::Char('g'), AppCommand::GoToTop)]
    #[case::go_bottom(KeyCode::Char('G'), AppCommand::GoToBottom)]
    #[case::select(KeyCode::Enter, AppCommand::SelectResult)]
    #[case::quit_q(KeyCode::Char('q'), AppCommand::RequestQuit)]
    #[case::noop(KeyCode::F(1), AppCommand::Noop)]
    fn test_search_results_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::SearchResults),
            expected
        );
    }

    /// Tests all key mappings for MessagePopup context.
    #[rstest]
    #[case::dismiss_esc(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::dismiss_enter(KeyCode::Enter, AppCommand::Dismiss)]
    #[case::dismiss_space(KeyCode::Char(' '), AppCommand::Dismiss)]
    #[case::quit(KeyCode::Char('q'), AppCommand::RequestQuit)]
    #[case::noop(KeyCode::F(1), AppCommand::Noop)]
    fn test_message_popup_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::MessagePopup),
            expected
        );
    }

    /// Tests all key mappings for HelpPopup context.
    #[rstest]
    #[case::dismiss_esc(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::quit_q(KeyCode::Char('q'), AppCommand::RequestQuit)]
    #[case::dismiss_question(KeyCode::Char('?'), AppCommand::Dismiss)]
    #[case::scroll_up_arrow(KeyCode::Up, AppCommand::ScrollHelpUp)]
    #[case::scroll_up_vim(KeyCode::Char('k'), AppCommand::ScrollHelpUp)]
    #[case::scroll_down_arrow(KeyCode::Down, AppCommand::ScrollHelpDown)]
    #[case::scroll_down_vim(KeyCode::Char('j'), AppCommand::ScrollHelpDown)]
    #[case::go_top(KeyCode::Char('g'), AppCommand::GoToTop)]
    #[case::go_bottom(KeyCode::Char('G'), AppCommand::GoToBottom)]
    #[case::noop(KeyCode::F(1), AppCommand::Noop)]
    fn test_help_popup_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::HelpPopup),
            expected
        );
    }

    /// Tests all key mappings for ConfirmQuit context.
    #[rstest]
    #[case::confirm_y(KeyCode::Char('y'), AppCommand::ConfirmQuit)]
    #[case::confirm_upper_y(KeyCode::Char('Y'), AppCommand::ConfirmQuit)]
    #[case::dismiss_n(KeyCode::Char('n'), AppCommand::Dismiss)]
    #[case::dismiss_upper_n(KeyCode::Char('N'), AppCommand::Dismiss)]
    #[case::dismiss_esc(KeyCode::Esc, AppCommand::Dismiss)]
    #[case::noop(KeyCode::F(1), AppCommand::Noop)]
    fn test_confirm_quit_context(#[case] key_code: KeyCode, #[case] expected: AppCommand) {
        assert_eq!(
            map_key(key_event(key_code), &InputContext::ConfirmQuit),
            expected
        );
    }
}
