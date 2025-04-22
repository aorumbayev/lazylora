use arboard::Clipboard;
use color_eyre::Result;
use ratatui::widgets::ListState;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    algorand::{AlgoBlock, Network, SearchResultItem, Transaction},
    config::{self, AppSettings},
    event::Action,
    network::NetworkManager,
};

/// Focus area in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Blocks,
    Transactions,
}

/// Fields in the Add Custom Network popup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomNetworkField {
    Name,
    AlgodUrl,
    IndexerUrl,
    Token,
}

impl CustomNetworkField {
    fn next(self) -> Self {
        match self {
            Self::Name => Self::AlgodUrl,
            Self::AlgodUrl => Self::IndexerUrl,
            Self::IndexerUrl => Self::Token,
            Self::Token => Self::Name,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Name => Self::Token,
            Self::AlgodUrl => Self::Name,
            Self::IndexerUrl => Self::AlgodUrl,
            Self::Token => Self::IndexerUrl,
        }
    }

    pub fn as_index(self) -> usize {
        match self {
            Self::Name => 0,
            Self::AlgodUrl => 1,
            Self::IndexerUrl => 2,
            Self::Token => 3,
        }
    }
}

/// State for the Add Custom Network popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddCustomNetworkState {
    pub name: String,
    pub algod_url: String,
    pub indexer_url: String,
    pub algod_token: String,
    pub focused_field: CustomNetworkField,
}

impl AddCustomNetworkState {
    fn input_char(&mut self, c: char) {
        match self.focused_field {
            CustomNetworkField::Name => self.name.push(c),
            CustomNetworkField::AlgodUrl => self.algod_url.push(c),
            CustomNetworkField::IndexerUrl => self.indexer_url.push(c),
            CustomNetworkField::Token => self.algod_token.push(c),
        }
    }

    fn backspace(&mut self) {
        match self.focused_field {
            CustomNetworkField::Name => {
                self.name.pop();
            }
            CustomNetworkField::AlgodUrl => {
                self.algod_url.pop();
            }
            CustomNetworkField::IndexerUrl => {
                self.indexer_url.pop();
            }
            CustomNetworkField::Token => {
                self.algod_token.pop();
            }
        }
    }

    fn focus_next(&mut self) {
        self.focused_field = self.focused_field.next();
    }

    fn focus_prev(&mut self) {
        self.focused_field = self.focused_field.prev();
    }

    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Network Name cannot be empty.".to_string());
        }
        if self.algod_url.trim().is_empty() || !self.algod_url.starts_with("http") {
            return Err("Invalid Algod URL format.".to_string());
        }
        if self.indexer_url.trim().is_empty() || !self.indexer_url.starts_with("http") {
            return Err("Invalid Indexer URL format.".to_string());
        }
        Ok(())
    }

    fn get_final_token(&self) -> Option<String> {
        if self.algod_token.trim().is_empty() {
            None
        } else {
            Some(self.algod_token.trim().to_string())
        }
    }
}

impl Default for AddCustomNetworkState {
    fn default() -> Self {
        Self {
            name: String::new(),
            algod_url: String::new(),
            indexer_url: String::new(),
            algod_token: String::new(),
            focused_field: CustomNetworkField::Name,
        }
    }
}

/// State for the Search Results popup.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResultsState {
    pub results: Vec<(usize, SearchResultItem)>,
    pub selected_index: usize,
}

impl SearchResultsState {
    fn new(items: Vec<SearchResultItem>) -> Self {
        let results_with_indices = items.into_iter().enumerate().collect();
        Self {
            results: results_with_indices,
            selected_index: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    fn len(&self) -> usize {
        self.results.len()
    }

    fn select_next(&mut self) {
        if !self.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.len();
        }
    }

    fn select_prev(&mut self) {
        if !self.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn get_selected_item(&self) -> Option<&SearchResultItem> {
        self.results.get(self.selected_index).map(|(_, item)| item)
    }
}

/// State for popups
#[derive(Debug, Clone, PartialEq)]
pub enum PopupState {
    None,
    NetworkSelect {
        available_networks: Vec<Network>,
        selected_index: usize,
    },
    AddCustomNetwork(AddCustomNetworkState),
    SearchWithType {
        query: String,
        search_type: SearchType,
    },
    Message(String),
    SearchResults(SearchResultsState),
}

/// Search type for explicit search selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    Transaction,
    Asset,
    Account,
    Block,
}

impl SearchType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Transaction => "Transaction",
            Self::Asset => "Asset",
            Self::Account => "Account",
            Self::Block => "Block",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Transaction => Self::Block,
            Self::Block => Self::Account,
            Self::Account => Self::Asset,
            Self::Asset => Self::Transaction,
        }
    }
}

/// The main application struct holding the state.
pub struct App {
    pub settings: AppSettings,

    pub focus: Focus,
    pub show_live: Arc<Mutex<bool>>,
    pub exit: bool,
    pub terminal_size: (u16, u16),

    pub blocks: Arc<Mutex<Vec<AlgoBlock>>>,
    pub transactions: Arc<Mutex<Vec<Transaction>>>,

    pub block_list_state: ListState,
    pub transaction_list_state: ListState,

    pub show_block_details: bool,
    pub show_transaction_details: bool,
    pub popup_state: PopupState,

    pub viewing_search_result_details: bool,
    pub detailed_search_result: Option<SearchResultItem>,

    clipboard: Option<Clipboard>,
}

impl App {
    /// Creates a new App instance.
    pub fn new() -> Self {
        let settings = config::load_settings().unwrap_or_default();
        let show_live = Arc::new(Mutex::new(true));
        let blocks = Arc::new(Mutex::new(Vec::new()));
        let transactions = Arc::new(Mutex::new(Vec::new()));

        // Try initializing clipboard, but don't panic if it fails
        let clipboard = Clipboard::new().ok();

        // Initialize ListState
        let block_list_state = ListState::default();
        let transaction_list_state = ListState::default();

        Self {
            settings,
            show_live,
            blocks,
            transactions,
            focus: Focus::Blocks,
            exit: false,
            terminal_size: (0, 0), // Will be updated on first resize event
            block_list_state,
            transaction_list_state,
            show_block_details: false,
            show_transaction_details: false,
            popup_state: PopupState::None,
            viewing_search_result_details: false,
            detailed_search_result: None, // Initialize new field
            clipboard,
        }
    }

    /// Updates the application state based on the received action.
    /// This is the core logic function, dispatching to helper methods.
    pub fn update(&mut self, action: Action, network_manager: &NetworkManager) -> Result<()> {
        // Store selected IDs *before* potentially updating data and changing indices
        let mut selected_block_id: Option<u64> = None;
        // Get selected index from ListState
        if let Some(index) = self.block_list_state.selected() {
            if let Ok(blocks) = self.blocks.try_lock() {
                selected_block_id = blocks.get(index).map(|b| b.id);
            }
        }
        let mut selected_txn_id: Option<String> = None;
        if !self.viewing_search_result_details {
            // Only track main list selection
            // Get selected index from ListState
            if let Some(index) = self.transaction_list_state.selected() {
                if let Ok(transactions) = self.transactions.try_lock() {
                    selected_txn_id = transactions.get(index).map(|t| t.id.clone());
                }
            }
        }

        match action {
            // App Lifecycle & Control
            Action::Quit => self.handle_quit(),
            Action::ToggleLiveUpdates => self.handle_toggle_live_updates(network_manager),
            Action::RefreshData => self.handle_refresh_data(network_manager),
            Action::CloseDetailsOrPopup => self.handle_close_details_or_popup(),
            Action::ClearPopup => self.popup_state = PopupState::None, // Simple enough to keep inline

            // Focus & Selection
            Action::SwitchFocus => self.handle_switch_focus(),
            Action::MoveSelectionUp => self.move_selection_up(),
            Action::MoveSelectionDown => self.move_selection_down(),
            Action::HandleScrollUp => self.handle_scroll_up(),
            Action::HandleScrollDown => self.handle_scroll_down(),
            Action::ScrollPageUp => self.handle_scroll_page_up(),
            Action::ScrollPageDown => self.handle_scroll_page_down(),
            Action::ShowDetails => self.show_details(),

            // Network Selection Popup
            Action::OpenNetworkSelector => self.handle_open_network_selector(),
            Action::SelectNetworkOption(idx) => self.handle_select_network_option(idx),
            Action::SwitchToNetwork(network) => {
                self.handle_switch_to_network(network, network_manager)
            }

            // Custom Network Popup
            Action::OpenAddCustomNetwork => self.handle_open_add_custom_network(),
            Action::AddCustomNetworkInput(c, _field_index) => {
                self.handle_add_custom_network_input(c, _field_index)
            }
            Action::AddCustomNetworkBackspace(field_idx) => {
                self.handle_add_custom_network_backspace(field_idx)
            }
            Action::AddCustomNetworkFocusNext => self.handle_add_custom_network_focus_next(),
            Action::AddCustomNetworkFocusPrev => self.handle_add_custom_network_focus_prev(),
            Action::SaveCustomNetwork { .. } => self.handle_save_custom_network(),

            // Search Popup & Results
            Action::OpenSearchPopup => self.handle_open_search_popup(),
            Action::SearchInput(c) => self.handle_search_input(c),
            Action::SearchBackspace => self.handle_search_backspace(),
            Action::SearchSwitchType => self.handle_search_switch_type(),
            Action::PerformSearch(query, search_type) => {
                self.handle_perform_search(query, search_type, network_manager)
            }
            Action::SearchResultSelectNext => self.handle_search_result_select_next(),
            Action::SearchResultSelectPrev => self.handle_search_result_select_prev(),
            Action::SearchResultShowSelected => self.handle_search_result_show_selected(),

            // Clipboard
            Action::CopySelectedTxnId => self.copy_selected_transaction_id_to_clipboard(),

            // Network Update Handling (from NetworkManager)
            Action::UpdateNetworkStatus(res) => self.handle_network_status_update(res),
            Action::UpdateBlocks(blocks_result) => {
                // Pass stored ID to handler
                self.handle_blocks_update(blocks_result, selected_block_id);
                // No longer need manual ensure_visible call, ListState handles offset
            }
            Action::UpdateTransactions(txns_result) => {
                // Pass stored ID to handler
                self.handle_transactions_update(txns_result, selected_txn_id);
                // No longer need manual ensure_visible call, ListState handles offset
            }
            Action::UpdateSearchResults(search_result) => {
                self.handle_search_results_update(search_result);
            }
            Action::HandleNetworkError(msg) => self.handle_network_error(msg),

            // Utility
            Action::ShowMessage(msg) => self.show_message(msg),
        }
        Ok(())
    }

    // --- Private Helper Methods for Actions ---

    fn handle_quit(&mut self) {
        self.exit = true;
    }

    fn handle_toggle_live_updates(&mut self, network_manager: &NetworkManager) {
        let new_state = match self.show_live.try_lock() {
            Ok(show) => !*show,
            Err(_) => return, // If we can't get the lock, do nothing
        };

        // Try again to set the new state
        if let Ok(mut show) = self.show_live.try_lock() {
            *show = new_state;
        } else {
            return; // If we can't get the lock, do nothing
        }

        if new_state {
            // If turning live ON, refresh data immediately
            network_manager.fetch_initial_data(self.settings.selected_network.as_str().to_string());
            self.popup_state = PopupState::None; // Clear any potential error message
        } else {
            // Optionally show a message when turning off
            self.show_message("Live updates paused.".to_string());
        }
    }

    fn handle_refresh_data(&mut self, network_manager: &NetworkManager) {
        // Trigger initial data fetch via NetworkManager
        network_manager.fetch_initial_data(self.settings.selected_network.as_str().to_string());
        self.show_message("Refreshing data...".to_string());
    }

    fn handle_close_details_or_popup(&mut self) {
        if self.show_block_details || self.show_transaction_details {
            self.show_block_details = false;
            self.show_transaction_details = false;
            self.viewing_search_result_details = false;
            self.detailed_search_result = None; // Clear stored search result
        } else {
            self.popup_state = PopupState::None;
        }
    }

    fn handle_switch_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Blocks => Focus::Transactions,
            Focus::Transactions => Focus::Blocks,
        };
    }

    // --- Network Selection Popup Handlers ---
    fn handle_open_network_selector(&mut self) {
        let available_networks = config::get_available_networks(&self.settings);
        let current_index = available_networks
            .iter()
            .position(|n| *n == self.settings.selected_network)
            .unwrap_or(0);
        self.popup_state = PopupState::NetworkSelect {
            available_networks,
            selected_index: current_index,
        };
    }

    fn handle_select_network_option(&mut self, target_index: usize) {
        if let PopupState::NetworkSelect {
            available_networks,
            selected_index,
        } = &mut self.popup_state
        {
            let num_options = available_networks.len() + 1; // +1 for Add Custom
            // Handle wrapping
            *selected_index = if target_index >= num_options {
                0
            } else {
                target_index
            };
        }
    }

    fn handle_switch_to_network(
        &mut self,
        network_to_switch: Network,
        network_manager: &NetworkManager,
    ) {
        self.switch_network(network_to_switch.clone(), network_manager);
    }

    // --- Custom Network Popup Handlers ---
    fn handle_open_add_custom_network(&mut self) {
        // Initialize the state struct and set the popup state
        self.popup_state = PopupState::AddCustomNetwork(AddCustomNetworkState::default());
    }

    fn handle_add_custom_network_input(&mut self, c: char, _field_index: usize) {
        // The action still carries the index, but the state knows the focused field
        if let PopupState::AddCustomNetwork(state) = &mut self.popup_state {
            // We now call methods on the state struct
            state.input_char(c);
        }
        // TODO: Ensure UI sends input action ONLY for the currently focused field.
        // Alternatively, change Action to not include index and rely solely on state.focused_field.
    }

    fn handle_add_custom_network_backspace(&mut self, _field_index: usize) {
        if let PopupState::AddCustomNetwork(state) = &mut self.popup_state {
            state.backspace();
        }
        // TODO: Same as above re: field_index in Action.
    }

    fn handle_add_custom_network_focus_next(&mut self) {
        if let PopupState::AddCustomNetwork(state) = &mut self.popup_state {
            state.focus_next();
        }
    }

    fn handle_add_custom_network_focus_prev(&mut self) {
        if let PopupState::AddCustomNetwork(state) = &mut self.popup_state {
            state.focus_prev();
        }
    }

    fn handle_save_custom_network(&mut self) {
        if let PopupState::AddCustomNetwork(state) = self.popup_state.clone() {
            // Use validation method from state
            if let Err(validation_msg) = state.validate() {
                self.show_message(validation_msg);
                return;
            }

            let final_token = state.get_final_token();

            match config::add_custom_network(
                &mut self.settings,
                state.name.trim().to_string(),
                state.algod_url.trim().to_string(),
                state.indexer_url.trim().to_string(),
                final_token,
            ) {
                Ok(_) => {
                    self.show_message("Custom Network Added!".to_string());
                    self.handle_open_network_selector(); // Re-open selector
                }
                Err(e) => {
                    self.show_error_message(format!("Failed to add network: {}", e));
                }
            }
        }
    }

    // --- Search Popup & Results Handlers ---
    fn handle_open_search_popup(&mut self) {
        self.popup_state = PopupState::SearchWithType {
            query: String::new(),
            search_type: SearchType::Transaction,
        };
    }

    fn handle_search_input(&mut self, c: char) {
        if let PopupState::SearchWithType { query, .. } = &mut self.popup_state {
            query.push(c);
        }
    }

    fn handle_search_backspace(&mut self) {
        if let PopupState::SearchWithType { query, .. } = &mut self.popup_state {
            query.pop();
        }
    }

    fn handle_search_switch_type(&mut self) {
        if let PopupState::SearchWithType { search_type, .. } = &mut self.popup_state {
            *search_type = search_type.next();
        }
    }

    fn handle_perform_search(
        &mut self,
        query: String,
        search_type: SearchType,
        network_manager: &NetworkManager,
    ) {
        if query.trim().is_empty() {
            self.show_message("Please enter a search term".to_string());
        } else {
            self.show_message(format!("Searching for {}...", search_type.as_str()));
            network_manager.search(query.trim().to_string(), search_type);
        }
    }

    fn handle_search_result_select_next(&mut self) {
        if let PopupState::SearchResults(state) = &mut self.popup_state {
            state.select_next();
        }
    }

    fn handle_search_result_select_prev(&mut self) {
        if let PopupState::SearchResults(state) = &mut self.popup_state {
            state.select_prev();
        }
    }

    fn handle_search_result_show_selected(&mut self) {
        if let PopupState::SearchResults(state) = self.popup_state.clone() {
            if let Some(item) = state.get_selected_item() {
                // Clear main list selections when showing details from search
                self.block_list_state.select(None);
                self.transaction_list_state.select(None);
                self.viewing_search_result_details = true; // Set flag
                self.detailed_search_result = Some(item.clone()); // Store the item
                self.popup_state = PopupState::None; // Close search results popup immediately

                // Set the correct detail view flag based on the *stored* item type
                match self.detailed_search_result.as_ref().unwrap() {
                    // Use stored item
                    SearchResultItem::Transaction(txn) => {
                        self.show_message(format!("Showing Txn: {}...", &txn.id[..8])); // Keep brief message
                        self.show_transaction_details = true;
                        self.show_block_details = false;
                    }
                    SearchResultItem::Block(block_info) => {
                        self.show_message(format!(
                            "Block #{}: {} txns",
                            block_info.id, block_info.txn_count
                        ));
                        // TODO: Set self.show_block_details = true; when implemented
                        self.show_block_details = false; // Explicitly false for now
                        self.show_transaction_details = false;
                    }
                    SearchResultItem::Account(account_info) => {
                        self.show_message(format!("Account: {}...", &account_info.address[..8]));
                        // TODO: Set self.show_account_details = true; when implemented
                        self.show_block_details = false;
                        self.show_transaction_details = false;
                    }
                    SearchResultItem::Asset(asset_info) => {
                        self.show_message(format!("Asset #{}: {}", asset_info.id, asset_info.name));
                        // TODO: Set self.show_asset_details = true; when implemented
                        self.show_block_details = false;
                        self.show_transaction_details = false;
                    }
                }
            }
        }
    }

    // --- Network Update Handlers ---

    fn handle_network_status_update(&mut self, status_result: Result<(), String>) {
        match status_result {
            Ok(_) => {
                // Network is back online or switch completed successfully.
                // Clear any message popup (e.g., "Switching...", previous errors).
                self.popup_state = PopupState::None;

                // Re-enable live updates if they were off due to error
                if let Ok(mut live) = self.show_live.try_lock() {
                    *live = true;
                }
            }
            Err(e) => {
                self.show_error_message(e); // Show specific error from status check
                // Disable live updates on error
                if let Ok(mut live) = self.show_live.try_lock() {
                    *live = false;
                }
            }
        }
    }

    fn handle_blocks_update(
        &mut self,
        blocks_result: Result<Vec<AlgoBlock>, String>,
        prev_selected_id: Option<u64>,
    ) {
        if let Err(e) = blocks_result {
            self.show_error_message(format!("Failed to update blocks: {}", e));
            return;
        }
        // Data was updated by NetworkManager before this event was sent.
        // Now, find the new index of the previously selected block.
        self.sync_block_selection(prev_selected_id);
    }

    fn handle_transactions_update(
        &mut self,
        txns_result: Result<Vec<Transaction>, String>,
        prev_selected_id: Option<String>,
    ) {
        if let Err(e) = txns_result {
            self.show_error_message(format!("Failed to update transactions: {}", e));
            return;
        }
        // Data was updated by NetworkManager. Find new index.
        self.sync_transaction_selection(prev_selected_id);
    }

    fn handle_search_results_update(
        &mut self,
        search_result: Result<Vec<SearchResultItem>, String>,
    ) {
        match search_result {
            Ok(items) => {
                if items.is_empty() {
                    self.show_message("No results found.".to_string());
                } else {
                    self.popup_state = PopupState::SearchResults(SearchResultsState::new(items));
                }
            }
            Err(e) => {
                self.show_error_message(format!("Search failed: {}", e));
            }
        }
    }

    fn handle_network_error(&mut self, error_msg: String) {
        self.show_error_message(format!("Network Error: {}", error_msg));
    }

    // --- Utility Helpers ---

    /// Sets the popup state to show a message.
    fn show_message(&mut self, msg: String) {
        self.popup_state = if msg.is_empty() {
            PopupState::None
        } else {
            PopupState::Message(msg)
        };
    }

    /// Sets the popup state to show an error message.
    fn show_error_message(&mut self, error_msg: String) {
        // Consider prefixing or styling error messages differently
        self.popup_state = PopupState::Message(format!("Error: {}", error_msg));
    }

    // --- Sync Selection After Data Update (using ListState) ---

    // Update selection logic to use ListState
    fn sync_block_selection(&mut self, prev_selected_id: Option<u64>) {
        if let Ok(blocks) = self.blocks.try_lock() {
            let new_index = if let Some(prev_id) = prev_selected_id {
                blocks.iter().position(|b| b.id == prev_id)
            } else {
                None
            };
            // Select the new index in ListState, or None if not found/no prev selection
            self.block_list_state.select(new_index);

            // If no previous selection or item not found, select the first item if list isn't empty
            if new_index.is_none() && !blocks.is_empty() {
                self.block_list_state.select(Some(0));
            } else if blocks.is_empty() {
                self.block_list_state.select(None); // Clear selection if empty
            }
        }
    }

    // Update selection logic to use ListState
    fn sync_transaction_selection(&mut self, prev_selected_id: Option<String>) {
        if let Ok(transactions) = self.transactions.try_lock() {
            let new_index = if let Some(ref prev_id) = prev_selected_id {
                transactions.iter().position(|t| t.id == *prev_id)
            } else {
                None
            };
            // Select the new index in ListState, or None if not found/no prev selection
            self.transaction_list_state.select(new_index);

            // If no previous selection or item not found, select the first item if list isn't empty
            if new_index.is_none() && !transactions.is_empty() {
                self.transaction_list_state.select(Some(0));
            } else if transactions.is_empty() {
                self.transaction_list_state.select(None); // Clear selection if empty
            }
        }
    }

    // --- Scrolling & Selection (Refactored using ListState) ---

    // Move selection up (using ListState)
    fn move_selection_up(&mut self) {
        match self.focus {
            Focus::Blocks => {
                if let Ok(blocks) = self.blocks.try_lock() {
                    let list_len = blocks.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.block_list_state.selected().unwrap_or(0);
                    let new_index = if current_index == 0 {
                        list_len - 1
                    } else {
                        current_index - 1
                    };
                    self.block_list_state.select(Some(new_index));
                }
            }
            Focus::Transactions => {
                if let Ok(transactions) = self.transactions.try_lock() {
                    let list_len = transactions.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.transaction_list_state.selected().unwrap_or(0);
                    let new_index = if current_index == 0 {
                        list_len - 1
                    } else {
                        current_index - 1
                    };
                    self.transaction_list_state.select(Some(new_index));
                }
            }
        }
    }

    // Move selection down (using ListState)
    fn move_selection_down(&mut self) {
        match self.focus {
            Focus::Blocks => {
                if let Ok(blocks) = self.blocks.try_lock() {
                    let list_len = blocks.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.block_list_state.selected().unwrap_or(0);
                    let new_index = (current_index + 1) % list_len;
                    self.block_list_state.select(Some(new_index));
                }
            }
            Focus::Transactions => {
                if let Ok(transactions) = self.transactions.try_lock() {
                    let list_len = transactions.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.transaction_list_state.selected().unwrap_or(0);
                    let new_index = (current_index + 1) % list_len;
                    self.transaction_list_state.select(Some(new_index));
                }
            }
        }
    }

    // Get items per page (still needed for page scroll logic)
    fn get_items_per_page(&self, item_height: u16) -> usize {
        let list_height = self.get_list_pane_height();
        if list_height > 0 && item_height > 0 {
            (list_height / item_height) as usize
        } else {
            1 // Avoid division by zero, assume at least 1 fits
        }
    }

    // Scroll down (moves selection, ListState handles offset)
    fn handle_scroll_down(&mut self) {
        match self.focus {
            Focus::Blocks => {
                if let Ok(blocks) = self.blocks.try_lock() {
                    let list_len = blocks.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.block_list_state.selected().unwrap_or(0);
                    let new_index = (current_index + 1) % list_len;
                    self.block_list_state.select(Some(new_index));
                }
            }
            Focus::Transactions => {
                if let Ok(transactions) = self.transactions.try_lock() {
                    let list_len = transactions.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.transaction_list_state.selected().unwrap_or(0);
                    let new_index = (current_index + 1) % list_len;
                    self.transaction_list_state.select(Some(new_index));
                }
            }
        }
    }

    // Scroll up (moves selection, ListState handles offset)
    fn handle_scroll_up(&mut self) {
        match self.focus {
            Focus::Blocks => {
                if let Ok(blocks) = self.blocks.try_lock() {
                    let list_len = blocks.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.block_list_state.selected().unwrap_or(0);
                    let new_index = if current_index == 0 {
                        list_len - 1
                    } else {
                        current_index - 1
                    };
                    self.block_list_state.select(Some(new_index));
                }
            }
            Focus::Transactions => {
                if let Ok(transactions) = self.transactions.try_lock() {
                    let list_len = transactions.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.transaction_list_state.selected().unwrap_or(0);
                    let new_index = if current_index == 0 {
                        list_len - 1
                    } else {
                        current_index - 1
                    };
                    self.transaction_list_state.select(Some(new_index));
                }
            }
        }
    }

    // Page Down (using ListState)
    fn handle_scroll_page_down(&mut self) {
        // Use the correct constants based on focus
        let page_size = match self.focus {
            Focus::Blocks => self.get_items_per_page(crate::constants::BLOCK_ITEM_HEIGHT),
            Focus::Transactions => self.get_items_per_page(crate::constants::TXN_ITEM_HEIGHT),
        };

        match self.focus {
            Focus::Blocks => {
                if let Ok(blocks) = self.blocks.try_lock() {
                    let list_len = blocks.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.block_list_state.selected().unwrap_or(0);
                    let new_index = (current_index + page_size).min(list_len - 1);
                    self.block_list_state.select(Some(new_index));
                }
            }
            Focus::Transactions => {
                if let Ok(transactions) = self.transactions.try_lock() {
                    let list_len = transactions.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.transaction_list_state.selected().unwrap_or(0);
                    let new_index = (current_index + page_size).min(list_len - 1);
                    self.transaction_list_state.select(Some(new_index));
                }
            }
        }
    }

    // Page Up (using ListState)
    fn handle_scroll_page_up(&mut self) {
        // Use the correct constants based on focus
        let page_size = match self.focus {
            Focus::Blocks => self.get_items_per_page(crate::constants::BLOCK_ITEM_HEIGHT),
            Focus::Transactions => self.get_items_per_page(crate::constants::TXN_ITEM_HEIGHT),
        };

        match self.focus {
            Focus::Blocks => {
                if let Ok(blocks) = self.blocks.try_lock() {
                    let list_len = blocks.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.block_list_state.selected().unwrap_or(0);
                    let new_index = current_index.saturating_sub(page_size);
                    self.block_list_state.select(Some(new_index));
                }
            }
            Focus::Transactions => {
                if let Ok(transactions) = self.transactions.try_lock() {
                    let list_len = transactions.len();
                    if list_len == 0 {
                        return;
                    }
                    let current_index = self.transaction_list_state.selected().unwrap_or(0);
                    let new_index = current_index.saturating_sub(page_size);
                    self.transaction_list_state.select(Some(new_index));
                }
            }
        }
    }

    // --- Show Details ---
    fn show_details(&mut self) {
        match self.focus {
            // Use ListState to get selected index
            Focus::Blocks => {
                if self.block_list_state.selected().is_some() {
                    self.show_block_details = true;
                    self.show_transaction_details = false; // Ensure only one detail view is active
                    self.viewing_search_result_details = false;
                }
            }
            // Use ListState to get selected index
            Focus::Transactions => {
                if self.transaction_list_state.selected().is_some() {
                    self.show_transaction_details = true;
                    self.show_block_details = false; // Ensure only one detail view is active
                    self.viewing_search_result_details = false;
                }
            }
        }
    }

    /// Switches the active network.
    fn switch_network(&mut self, network: Network, network_manager: &NetworkManager) {
        self.show_message(format!("Switching to {}...", network.as_str()));

        if let Err(e) = config::set_selected_network(&mut self.settings, network.clone()) {
            self.popup_state =
                PopupState::Message(format!("Failed to save network setting: {}", e));
            return;
        }

        // Clear existing data
        if let Ok(mut blocks) = self.blocks.try_lock() {
            blocks.clear();
        }
        if let Ok(mut transactions) = self.transactions.try_lock() {
            transactions.clear();
        }

        self.block_list_state.select(None);
        self.transaction_list_state.select(None);
        self.show_block_details = false;
        self.show_transaction_details = false;
        self.viewing_search_result_details = false;
        self.detailed_search_result = None; // Clear stored search result
        // Clear search results popup if it was open
        if matches!(self.popup_state, PopupState::SearchResults { .. }) {
            self.popup_state = PopupState::None;
        }

        // Tell NetworkManager to use the new client and fetch data (async)
        // No need to block here, NetworkManager handles the async internally.
        let switch_future = network_manager.switch_network(network.clone());
        // Use the public spawn_task method
        network_manager.spawn_task(switch_future);

        // The PopupState::Message will eventually be replaced by fetch status updates
        // triggered by the NetworkManager's actions.
    }

    /// Copies the selected transaction ID to the clipboard.
    fn copy_selected_transaction_id_to_clipboard(&mut self) {
        let txn_id_to_copy = match self.focus {
            Focus::Transactions => {
                // Use ListState to get selected index
                if let Some(index) = self.transaction_list_state.selected() {
                    if let Ok(transactions) = self.transactions.try_lock() {
                        transactions.get(index).map(|t| t.id.clone())
                    } else {
                        None
                    } // Could not lock
                } else {
                    None
                } // Nothing selected
            }
            Focus::Blocks => {
                // Find the selected block, then find its first transaction ID (if any)
                if let Some(block_index) = self.block_list_state.selected() {
                    if let Ok(blocks) = self.blocks.try_lock() {
                        blocks.get(block_index).map(|b| b.id.to_string()) // Convert block ID to string
                    } else {
                        None
                    } // Could not lock blocks
                } else {
                    None
                } // No block selected
            }
        };

        // Copy to clipboard if we found an ID
        if let Some(id) = txn_id_to_copy {
            if let Some(clipboard) = self.clipboard.as_mut() {
                match clipboard.set_text(id.clone()) {
                    Ok(_) => self.show_message(format!("Copied: {}", id)), // More generic message
                    Err(e) => self.show_error_message(format!("Clipboard Error: {}", e)),
                }
            } else {
                self.show_error_message("Clipboard not available".to_string());
            }
        } else {
            self.show_message("No item selected to copy.".to_string()); // More generic message
        }
    }

    /// Updates the stored terminal size.
    pub fn update_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
        // TODO: Potentially adjust scroll positions here if needed based on new size
    }

    // Re-add the method to calculate list pane height
    fn get_list_pane_height(&self) -> u16 {
        self.terminal_size
            .1
            .saturating_sub(crate::constants::HEADER_HEIGHT)
            .saturating_sub(crate::constants::FOOTER_HEIGHT)
            .saturating_sub(crate::constants::TITLE_HEIGHT)
            .saturating_sub(2) // Account for top/bottom borders of the list pane itself
    }
}
