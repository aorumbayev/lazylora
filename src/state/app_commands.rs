//! Command execution and input handling for the LazyLora application.
//!
//! This module handles keyboard and mouse input, mapping them to commands,
//! and executing those commands to update application state.

use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use super::{
    AccountDetailTab, App, AppConfig, AppDetailTab, AppMessage, BlockDetailTab, DetailViewMode,
    Focus, PopupState, SearchType, navigation::DetailPopupType,
};
use crate::commands::{AppCommand, InputContext, map_key};
use crate::constants::{
    BLOCK_HEIGHT, DEFAULT_TERMINAL_WIDTH, HEADER_HEIGHT, SEARCH_BAR_HEIGHT, TXN_HEIGHT,
};
use crate::domain::{NetworkConfig, SearchResultItem};
use crate::ui;

impl App {
    pub(crate) async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let context = self.get_input_context();
        let command = map_key(key_event, &context);
        self.execute_command(command).await
    }

    /// Determines the current input context based on application state.
    ///
    /// This method examines the current popup state and navigation state
    /// to determine which keybindings should be active.
    #[must_use]
    pub fn get_input_context(&self) -> InputContext {
        // Check if help popup is open first (highest priority)
        if self.ui.show_help {
            return InputContext::HelpPopup;
        }

        // Check popup state (popups take precedence)
        match &self.ui.popup_state {
            PopupState::ConfirmQuit => InputContext::ConfirmQuit,
            PopupState::NetworkSelect(_) => InputContext::NetworkSelect,
            PopupState::NetworkForm(_) => InputContext::NetworkForm,
            PopupState::SearchWithType(_, _) => InputContext::SearchInput,
            PopupState::SearchResults(_) => InputContext::SearchResults,
            PopupState::Message(_) => InputContext::MessagePopup,
            PopupState::None => {
                // Check if inline search is focused
                if self.ui.is_search_focused() {
                    return InputContext::InlineSearch;
                }
                // No popup - check if showing details
                if self.nav.show_block_details {
                    InputContext::BlockDetailView
                } else if self.nav.show_account_details {
                    InputContext::AccountDetailView
                } else if self.nav.show_application_details {
                    InputContext::AppDetailView
                } else if self.nav.show_transaction_details {
                    // Use different context based on view mode
                    if self.ui.detail_view_mode == DetailViewMode::Table {
                        InputContext::TxnDetailViewTable
                    } else {
                        InputContext::DetailView
                    }
                } else if self.nav.show_asset_details {
                    InputContext::DetailView
                } else {
                    InputContext::Main
                }
            }
        }
    }

    /// Executes an application command.
    ///
    /// This method handles all `AppCommand` variants and performs the corresponding
    /// application state mutations.
    pub(crate) async fn execute_command(&mut self, command: AppCommand) -> Result<()> {
        match command {
            // === Application Control ===
            AppCommand::RequestQuit => {
                self.ui.open_confirm_quit();
            }
            AppCommand::ConfirmQuit => {
                self.exit = true;
            }
            AppCommand::Refresh => {
                self.initial_data_fetch().await;
            }
            AppCommand::ToggleLive => {
                self.toggle_live_updates();
            }
            AppCommand::ToggleHelp => {
                self.ui.toggle_help();
            }

            // === Popup/Modal Control ===
            AppCommand::FocusInlineSearch => {
                self.ui.focus_search();
            }
            AppCommand::OpenNetworkSelect => {
                let current_index = self.current_network_index();
                self.ui.open_network_select(current_index);
            }
            AppCommand::Dismiss => {
                self.handle_dismiss();
            }

            // === Navigation ===
            AppCommand::CycleFocus => {
                self.ui.cycle_focus();
            }
            AppCommand::MoveUp => {
                self.move_selection_up();
            }
            AppCommand::MoveDown => {
                self.move_selection_down();
            }
            AppCommand::Select => {
                self.show_details().await;
            }
            AppCommand::GoToTop => {
                self.go_to_top();
            }
            AppCommand::GoToBottom => {
                self.go_to_bottom();
            }

            // === Detail View Actions ===
            AppCommand::CopyToClipboard => {
                if self.nav.show_transaction_details {
                    self.copy_transaction_id_to_clipboard();
                }
            }
            AppCommand::CopyJson => {
                self.copy_json_to_clipboard().await;
            }
            AppCommand::OpenInBrowser => {
                self.open_in_browser();
            }
            AppCommand::ToggleDetailViewMode => {
                if self.nav.is_showing_details() {
                    self.ui.toggle_detail_view_mode();
                    // Rebuild detail table rows when switching to Table mode
                    if self.nav.show_transaction_details
                        && self.ui.detail_view_mode == DetailViewMode::Table
                    {
                        self.update_detail_table_rows();
                    }
                }
            }
            AppCommand::DetailSectionUp => {
                if self.nav.show_transaction_details {
                    // In Table mode, navigate rows; in Visual mode, navigate sections
                    if self.ui.detail_view_mode == DetailViewMode::Table {
                        self.nav.move_detail_row_up();
                    } else {
                        self.move_detail_section_up();
                    }
                }
            }
            AppCommand::DetailSectionDown => {
                if self.nav.show_transaction_details {
                    // In Table mode, navigate rows; in Visual mode, navigate sections
                    if self.ui.detail_view_mode == DetailViewMode::Table {
                        if let Some(txn) = self.get_transaction_for_details() {
                            let row_count =
                                ui::panels::details::transaction::get_flat_row_count(&txn);
                            // Visible height is approximate - table area is content area
                            self.nav
                                .move_detail_row_down(row_count.saturating_sub(1), 15);
                        }
                    } else {
                        self.move_detail_section_down();
                    }
                }
            }
            AppCommand::ToggleDetailSection => {
                if self.nav.show_transaction_details {
                    self.toggle_current_detail_section();
                }
            }
            AppCommand::GraphScrollLeft => {
                if self.nav.show_transaction_details {
                    self.nav.graph_scroll_x = self.nav.graph_scroll_x.saturating_sub(4);
                }
            }
            AppCommand::GraphScrollRight => {
                if self.nav.show_transaction_details {
                    self.nav.graph_scroll_x = self.nav.graph_scroll_x.saturating_add(4);
                }
            }
            AppCommand::GraphScrollUp => {
                if self.nav.show_transaction_details {
                    // In Table mode, arrow keys navigate sections; in Visual mode, scroll graph
                    if self.ui.detail_view_mode == DetailViewMode::Table {
                        self.nav.move_detail_row_up();
                    } else {
                        self.nav.graph_scroll_y = self.nav.graph_scroll_y.saturating_sub(1);
                    }
                }
            }
            AppCommand::GraphScrollDown => {
                if self.nav.show_transaction_details {
                    // In Table mode, arrow keys navigate sections; in Visual mode, scroll graph
                    if self.ui.detail_view_mode == DetailViewMode::Table {
                        if let Some(txn) = self.get_transaction_for_details() {
                            let row_count =
                                ui::panels::details::transaction::get_flat_row_count(&txn);
                            self.nav
                                .move_detail_row_down(row_count.saturating_sub(1), 15);
                        }
                    } else {
                        self.nav.graph_scroll_y = self.nav.graph_scroll_y.saturating_add(1);
                    }
                }
            }
            AppCommand::ExportSvg => {
                if self.nav.show_transaction_details {
                    self.export_transaction_svg();
                }
            }
            AppCommand::ToggleFullscreen => {
                if self.nav.is_showing_details() {
                    self.ui.toggle_fullscreen();
                }
            }

            // === Block Detail View Actions ===
            AppCommand::CycleBlockDetailTab => {
                self.nav.cycle_block_detail_tab();
                // When switching to Transactions tab, ensure first transaction is selected
                if self.nav.block_detail_tab == BlockDetailTab::Transactions
                    && self.nav.block_txn_index.is_none()
                    && let Some(block_details) = &self.data.block_details
                    && !block_details.transactions.is_empty()
                {
                    self.nav.block_txn_index = Some(0);
                    self.nav.block_txn_scroll = 0;
                }
            }
            AppCommand::MoveBlockTxnUp => {
                self.nav.move_block_txn_up();
            }
            AppCommand::MoveBlockTxnDown => {
                if let Some(block_details) = &self.data.block_details {
                    let max = block_details.transactions.len().saturating_sub(1);
                    // Use a reasonable default visible height (popup content area ~20 lines)
                    self.nav.move_block_txn_down(max, 20);
                }
            }
            AppCommand::SelectBlockTxn => {
                // If we have a selected transaction in block details, fetch and show its full details
                if self.nav.block_detail_tab == BlockDetailTab::Transactions
                    && let Some(block_details) = &self.data.block_details
                    && let Some(txn_idx) = self.nav.block_txn_index
                    && let Some(txn) = block_details.transactions.get(txn_idx)
                {
                    // Close the block details popup
                    self.nav.show_block_details = false;

                    // Fetch the full transaction details asynchronously
                    let txn_id = txn.id.clone();
                    self.load_transaction_details(&txn_id).await;
                }
            }

            // === Account Detail View Actions ===
            AppCommand::CycleAccountDetailTab => {
                self.nav.cycle_account_detail_tab();
            }
            AppCommand::MoveAccountItemUp => {
                self.nav.move_account_item_up();
            }
            AppCommand::MoveAccountItemDown => {
                if let Some(account) = &self.data.viewed_account {
                    let max = match self.nav.account_detail_tab {
                        AccountDetailTab::Assets => account.assets.len().saturating_sub(1),
                        AccountDetailTab::Apps => account.apps_local_state.len().saturating_sub(1),
                        AccountDetailTab::Info => 0,
                    };
                    // Reasonable default visible height (~8 items in the list area)
                    self.nav.move_account_item_down(max, 8);
                }
            }
            AppCommand::SelectAccountItem => {
                self.handle_select_account_item();
            }

            // === Application Detail View Actions ===
            AppCommand::CycleAppDetailTab => {
                self.nav.cycle_app_detail_tab();
            }
            AppCommand::MoveAppStateUp => {
                self.nav.move_app_state_up();
            }
            AppCommand::MoveAppStateDown => {
                if let Some(app) = &self.data.viewed_application {
                    let max = match self.nav.app_detail_tab {
                        AppDetailTab::State => app.global_state.len().saturating_sub(1),
                        AppDetailTab::Info | AppDetailTab::Programs => 0,
                    };
                    // Reasonable default visible height (~8 items in the list area)
                    self.nav.move_app_state_down(max, 8);
                }
            }

            // === Search Input Actions ===
            AppCommand::TypeChar(c) => {
                if matches!(self.ui.popup_state, PopupState::NetworkForm(_)) {
                    self.ui.network_form_type_char(c);
                } else if self.ui.is_search_focused() {
                    self.ui.search_type_char(c);
                } else if let PopupState::SearchWithType(query, search_type) = &self.ui.popup_state
                {
                    let mut new_query = query.clone();
                    new_query.push(c);
                    let search_type = *search_type;
                    self.ui.update_search_query(new_query, search_type);
                }
            }
            AppCommand::Backspace => {
                if matches!(self.ui.popup_state, PopupState::NetworkForm(_)) {
                    self.ui.network_form_backspace();
                } else if self.ui.is_search_focused() {
                    self.ui.search_backspace();
                } else if let PopupState::SearchWithType(query, search_type) = &self.ui.popup_state
                {
                    let mut new_query = query.clone();
                    new_query.pop();
                    let search_type = *search_type;
                    self.ui.update_search_query(new_query, search_type);
                }
            }
            AppCommand::CycleSearchType => {
                // Handle inline search first
                if self.ui.is_search_focused() {
                    self.ui.cycle_inline_search_type();
                } else if let PopupState::SearchWithType(query, search_type) = &self.ui.popup_state
                {
                    let query = query.clone();
                    let search_type = *search_type;
                    self.ui.cycle_search_type(query, search_type);
                }
            }
            AppCommand::SubmitSearch => {
                // Handle inline search first
                if self.ui.is_search_focused() {
                    let query = self.ui.search_query().to_string();
                    if let Some(search_type) = self.ui.get_effective_search_type() {
                        self.ui.add_to_search_history(&query);
                        self.ui.unfocus_search();
                        self.search_transactions(&query, search_type).await;
                    } else if !query.is_empty() {
                        // If we couldn't detect the type, show a message
                        self.ui.show_message(
                            "Could not determine search type. Try a complete ID or address.",
                        );
                    }
                } else if let PopupState::SearchWithType(query, search_type) = &self.ui.popup_state
                {
                    let query = query.clone();
                    let search_type = *search_type;
                    self.ui.dismiss_popup();
                    self.search_transactions(&query, search_type).await;
                }
            }
            AppCommand::SearchHistoryPrev => {
                if self.ui.is_search_focused() {
                    self.ui.search_history_prev();
                }
            }
            AppCommand::SearchHistoryNext => {
                if self.ui.is_search_focused() {
                    self.ui.search_history_next();
                }
            }
            AppCommand::SearchCursorLeft => {
                if self.ui.is_search_focused() {
                    self.ui.search_cursor_left();
                }
            }
            AppCommand::SearchCursorRight => {
                if self.ui.is_search_focused() {
                    self.ui.search_cursor_right();
                }
            }

            // === Network Selection Actions ===
            AppCommand::NetworkUp => {
                if let PopupState::NetworkSelect(index) = &self.ui.popup_state {
                    let max_index = self.available_networks.len().saturating_sub(1);
                    let new_index = if *index == 0 { max_index } else { index - 1 };
                    self.ui.update_network_selection(new_index);
                }
            }
            AppCommand::NetworkDown => {
                if let PopupState::NetworkSelect(index) = &self.ui.popup_state {
                    let max_index = self.available_networks.len().saturating_sub(1);
                    let new_index = if *index >= max_index { 0 } else { index + 1 };
                    self.ui.update_network_selection(new_index);
                }
            }
            AppCommand::SelectNetwork => {
                if let PopupState::NetworkSelect(index) = &self.ui.popup_state
                    && let Some(selected) = self.available_networks.get(*index)
                {
                    let selected = selected.clone();
                    self.ui.dismiss_popup();
                    self.switch_network_config(selected).await;
                }
            }
            AppCommand::NetworkFormNextField => {
                self.ui.network_form_next_field();
            }
            AppCommand::NetworkFormPrevField => {
                self.ui.network_form_prev_field();
            }
            AppCommand::SubmitNetworkForm => {
                self.handle_submit_network_form().await?;
            }
            AppCommand::AddNetwork => {
                let return_index = match &self.ui.popup_state {
                    PopupState::NetworkSelect(index) => *index,
                    _ => 0,
                };
                self.ui.open_network_form(return_index);
            }
            AppCommand::DeleteNetwork => {
                self.handle_delete_network();
            }

            // === Search Results Actions ===
            AppCommand::PreviousResult => {
                self.ui.rotate_search_results_forward();
            }
            AppCommand::NextResult => {
                self.ui.rotate_search_results_backward();
            }
            AppCommand::SelectResult => {
                self.handle_select_result();
            }

            // === Help Popup Actions ===
            AppCommand::ScrollHelpUp => {
                self.ui.scroll_help_up();
            }
            AppCommand::ScrollHelpDown => {
                self.ui.scroll_help_down();
            }

            // === No Operation ===
            AppCommand::Noop => {}
        }
        Ok(())
    }

    /// Handles deleting a custom network from the network selection popup.
    pub(crate) fn handle_delete_network(&mut self) {
        // Extract index from popup state
        let index = match &self.ui.popup_state {
            PopupState::NetworkSelect(i) => *i,
            _ => return,
        };

        // Get selected network
        let Some(selected) = self.available_networks.get(index).cloned() else {
            return;
        };

        match selected {
            NetworkConfig::Custom(custom) => {
                // Don't allow deleting the current network
                if self.network_config == NetworkConfig::Custom(custom.clone()) {
                    self.ui
                        .show_toast("Cannot delete the currently active network".to_string(), 30);
                    return;
                }

                let name = custom.name.clone();
                // Remove from available networks
                self.available_networks.remove(index);
                // Remove from config and save
                let mut config = AppConfig::load();
                if let Err(e) = config.delete_custom_network(&name) {
                    self.ui.show_toast(format!("Failed to delete: {e}"), 30);
                } else {
                    self.available_networks = config.get_all_networks();
                    self.ui.show_toast(format!("Deleted network '{name}'"), 20);
                }
                // Adjust selection if needed
                let max_index = self.available_networks.len().saturating_sub(1);
                if index > max_index {
                    self.ui.update_network_selection(max_index);
                }
            }
            NetworkConfig::BuiltIn(_) => {
                self.ui
                    .show_toast("Cannot delete built-in networks".to_string(), 20);
            }
        }
    }

    /// Handles the Dismiss command based on current context.
    pub(crate) fn handle_dismiss(&mut self) {
        // Check if help popup is open first
        if self.ui.show_help {
            self.ui.show_help = false;
            self.ui.help_scroll_offset = 0;
            return;
        }

        // Check if inline search is focused first
        if self.ui.is_search_focused() {
            self.ui.unfocus_search();
            self.ui.clear_search();
            return;
        }

        if self.nav.is_showing_details() {
            // Check if we have a saved popup state in the stack (nested navigation)
            if self.nav.has_popup_stack() {
                // Close the current detail popup
                self.nav.close_details();
                self.data.viewed_asset = None;
                self.data.viewed_application = None;

                // Pop and restore the parent popup state
                if let Some(saved) = self.nav.pop_popup_state()
                    && saved.popup_type == DetailPopupType::Account
                {
                    // Reload account details (we still have the address)
                    self.load_account_details(&saved.entity_id);
                    // Restore navigation state
                    self.nav.restore_account_state(&saved);
                }
                return;
            }

            // No stack - close all details normally
            self.nav.close_details();
            self.ui.viewing_search_result = false;
            self.ui.reset_expanded_sections();
            self.data.viewed_transaction = None;
            self.data.viewed_account = None;
            self.data.viewed_asset = None;
            self.data.viewed_application = None;
            // Reset graph scroll position
            self.nav.graph_scroll_x = 0;
            self.nav.graph_scroll_y = 0;
            // Reset account detail view state
            self.nav.reset_account_detail();
            // Reset app detail view state
            self.nav.reset_app_detail();
        } else {
            match &self.ui.popup_state {
                PopupState::SearchWithType(_, _) | PopupState::SearchResults(_) => {
                    self.ui.dismiss_popup();
                    self.data.filtered_search_results.clear();
                    self.ui.viewing_search_result = false;
                }
                PopupState::NetworkForm(form) => {
                    let max_index = self.available_networks.len().saturating_sub(1);
                    let return_index = form.return_to_index.min(max_index);
                    self.ui.popup_state = PopupState::NetworkSelect(return_index);
                }
                PopupState::NetworkSelect(_) | PopupState::Message(_) | PopupState::ConfirmQuit => {
                    self.ui.dismiss_popup();
                }
                PopupState::None => {}
            }
        }
    }

    /// Handles selecting a search result.
    pub(crate) fn handle_select_result(&mut self) {
        let result_item = if let PopupState::SearchResults(results) = &self.ui.popup_state {
            results.first().map(|(_, item)| item.clone())
        } else {
            None
        };

        if let Some(item) = result_item {
            match item {
                SearchResultItem::Transaction(txn) => {
                    // Store the transaction for display
                    self.data.viewed_transaction = Some(*txn.clone());
                    self.nav.selected_transaction_id = Some(txn.id.clone());
                    self.ui.viewing_search_result = true;
                    self.ui.dismiss_popup();
                    self.nav.show_transaction_details = true;
                }
                SearchResultItem::Block(block_info) => {
                    // Load block details and show block details popup
                    let block_id = block_info.id;
                    self.ui.viewing_search_result = true;
                    self.ui.dismiss_popup();
                    self.load_block_details(block_id);
                    self.nav.show_block_details = true;
                }
                SearchResultItem::Account(account_info) => {
                    // Load account details and show account details popup
                    let address = account_info.address.clone();
                    self.ui.viewing_search_result = true;
                    self.ui.dismiss_popup();
                    self.load_account_details(&address);
                    self.nav.show_account_details = true;
                }
                SearchResultItem::Asset(asset_info) => {
                    // Load asset details and show asset details popup
                    let asset_id = asset_info.id;
                    self.ui.viewing_search_result = true;
                    self.ui.dismiss_popup();
                    self.load_asset_details(asset_id);
                    self.nav.show_asset_details = true;
                }
                SearchResultItem::Application(app_info) => {
                    // Load application details and show application details popup
                    let app_id = app_info.app_id;
                    self.ui.viewing_search_result = true;
                    self.ui.dismiss_popup();
                    self.load_application_details(app_id);
                    self.nav.show_application_details = true;
                }
            }
        }
    }

    /// Handles selecting an asset or app from account details.
    ///
    /// When pressing Enter in the Assets or Apps tab of account details,
    /// this opens the selected asset or application details popup while
    /// saving the account popup state for stack-based navigation.
    pub(crate) fn handle_select_account_item(&mut self) {
        let Some(account) = &self.data.viewed_account else {
            return;
        };

        let Some(item_index) = self.nav.account_item_index else {
            return;
        };

        match self.nav.account_detail_tab {
            AccountDetailTab::Assets => {
                // Get the selected asset
                if let Some(asset_holding) = account.assets.get(item_index) {
                    let asset_id = asset_holding.asset_id;

                    // Save current account popup state to stack
                    self.nav.push_account_state(&account.address);

                    // Close account details and open asset details
                    self.nav.show_account_details = false;
                    self.load_asset_details(asset_id);
                }
            }
            AccountDetailTab::Apps => {
                // Get the selected app
                if let Some(app_state) = account.apps_local_state.get(item_index) {
                    let app_id = app_state.app_id;

                    // Save current account popup state to stack
                    self.nav.push_account_state(&account.address);

                    // Close account details and open application details
                    self.nav.show_account_details = false;
                    self.load_application_details(app_id);
                }
            }
            AccountDetailTab::Info => {
                // Nothing to select in Info tab
            }
        }
    }

    /// Loads account details asynchronously
    pub(crate) fn load_account_details(&self, address: &str) {
        let message_tx = self.message_tx.clone();
        let client = self.client.clone();
        let address = address.to_string();

        tokio::spawn(async move {
            // Channel sends below: receiver may be dropped during shutdown - safe to ignore
            match client.get_account_details(&address).await {
                Ok(details) => {
                    let _ = message_tx.send(AppMessage::AccountDetailsLoaded(Box::new(details)));
                }
                Err(e) => {
                    let _ = message_tx.send(AppMessage::AccountDetailsFailed(e.to_string()));
                }
            }
        });
    }

    /// Loads asset details asynchronously
    pub(crate) fn load_asset_details(&self, asset_id: u64) {
        let message_tx = self.message_tx.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            // Channel sends below: receiver may be dropped during shutdown - safe to ignore
            match client.get_asset_details(asset_id).await {
                Ok(details) => {
                    let _ = message_tx.send(AppMessage::AssetDetailsLoaded(Box::new(details)));
                }
                Err(e) => {
                    let _ = message_tx.send(AppMessage::AssetDetailsFailed(e.to_string()));
                }
            }
        });
    }

    /// Loads application details asynchronously
    pub(crate) fn load_application_details(&self, app_id: u64) {
        let message_tx = self.message_tx.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            // Channel sends below: receiver may be dropped during shutdown - safe to ignore
            match client.get_application_details(app_id).await {
                Ok(details) => {
                    let _ =
                        message_tx.send(AppMessage::ApplicationDetailsLoaded(Box::new(details)));
                }
                Err(e) => {
                    let _ = message_tx.send(AppMessage::ApplicationDetailsFailed(e.to_string()));
                }
            }
        });
    }

    pub(crate) async fn handle_mouse_input(&mut self, mouse: MouseEvent) -> Result<()> {
        // Handle scroll events for the focused panel
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                return self.handle_mouse_scroll_up();
            }
            MouseEventKind::ScrollDown => {
                return self.handle_mouse_scroll_down();
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // Continue with click handling below
            }
            _ => {
                return Ok(());
            }
        }

        let popup_open = self.nav.is_showing_details() || self.ui.has_active_popup();

        if popup_open {
            return self.handle_popup_mouse_click(mouse);
        }

        self.handle_main_mouse_click(mouse)
    }

    /// Handles mouse scroll up event based on focused panel.
    pub(crate) fn handle_mouse_scroll_up(&mut self) -> Result<()> {
        // Don't handle scroll when popup is open
        if self.nav.is_showing_details() || self.ui.has_active_popup() {
            return Ok(());
        }

        match self.ui.focus {
            Focus::Blocks => {
                self.move_block_selection_up();
            }
            Focus::Transactions => {
                self.move_transaction_selection_up();
            }
        }
        Ok(())
    }

    /// Handles mouse scroll down event based on focused panel.
    pub(crate) fn handle_mouse_scroll_down(&mut self) -> Result<()> {
        // Don't handle scroll when popup is open
        if self.nav.is_showing_details() || self.ui.has_active_popup() {
            return Ok(());
        }

        match self.ui.focus {
            Focus::Blocks => {
                self.move_block_selection_down();
            }
            Focus::Transactions => {
                self.move_transaction_selection_down();
            }
        }
        Ok(())
    }

    pub(crate) fn handle_popup_mouse_click(&mut self, mouse: MouseEvent) -> Result<()> {
        if let Some((query, current_type)) = self.ui.popup_state.as_search() {
            let row = mouse.row;
            let selector_y = 9;

            if row == selector_y {
                let column = mouse.column;
                let button_width = 12;
                let start_x = 15;

                if column >= start_x && column < start_x + (4 * button_width) {
                    let button_index = (column - start_x) / button_width;
                    let new_type = match button_index {
                        0 => SearchType::Transaction,
                        1 => SearchType::Block,
                        2 => SearchType::Account,
                        3 => SearchType::Asset,
                        _ => current_type,
                    };
                    self.ui.update_search_query(query.to_string(), new_type);
                }
            }
        } else if self.nav.show_transaction_details {
            let row = mouse.row;
            let button_y_range = (20, 23);
            let button_x_range = (33, 47);

            if row >= button_y_range.0
                && row <= button_y_range.1
                && mouse.column >= button_x_range.0
                && mouse.column <= button_x_range.1
            {
                self.copy_transaction_id_to_clipboard();
            }
        }
        Ok(())
    }

    pub(crate) fn handle_main_mouse_click(&mut self, mouse: MouseEvent) -> Result<()> {
        // Content starts after header + search bar
        let top_area_height = HEADER_HEIGHT + SEARCH_BAR_HEIGHT;
        if mouse.row <= top_area_height {
            return Ok(());
        }

        let content_start_row = top_area_height;
        let content_row = mouse.row.saturating_sub(content_start_row);

        let terminal_width = DEFAULT_TERMINAL_WIDTH;
        let is_left_half = mouse.column <= terminal_width / 2;

        if is_left_half {
            self.ui.focus = Focus::Blocks;
            let block_index = (content_row / BLOCK_HEIGHT) as usize;
            if block_index < self.data.blocks.len() {
                self.nav.select_block(block_index, &self.data.blocks);
            }
        } else {
            self.ui.focus = Focus::Transactions;
            let txn_index = (content_row / TXN_HEIGHT) as usize;
            if txn_index < self.data.transactions.len() {
                self.nav
                    .select_transaction(txn_index, &self.data.transactions);
            }
        }
        Ok(())
    }

    // ========================================================================
    // Selection & Navigation
    // ========================================================================
}
