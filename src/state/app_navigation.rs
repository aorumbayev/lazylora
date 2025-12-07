//! Navigation helpers and selection management for the LazyLora application.
//!
//! This module handles cursor movement, selection synchronization,
//! and loading detail views for blocks, transactions, accounts, etc.

use super::{AccountDetailTab, AppDetailTab, BlockDetailTab};
use super::{App, AppMessage, Focus, PopupState};
use crate::constants::{
    BLOCK_HEIGHT, DEFAULT_VISIBLE_BLOCKS, DEFAULT_VISIBLE_TRANSACTIONS, TXN_HEIGHT,
};

impl App {
    pub(crate) fn sync_selections(&mut self) {
        if let Some(block_id) = self.nav.selected_block_id {
            if let Some(index) = self.data.find_block_index(block_id) {
                self.nav.selected_block_index = Some(index);
            } else {
                self.nav.clear_block_selection();
            }
        }

        if let Some(ref txn_id) = self.nav.selected_transaction_id {
            if let Some(index) = self.data.find_transaction_index(txn_id) {
                self.nav.selected_transaction_index = Some(index);
            } else {
                self.nav.clear_transaction_selection();
            }
        }
    }

    /// Handles countdown ticking for toast notifications.
    pub(crate) fn tick_timed_message_countdown(&mut self) {
        // Tick toast notifications (they tick every ~100ms with the main loop)
        self.ui.tick_toast();
    }

    /// Moves the selection up in the currently focused list.
    pub fn move_selection_up(&mut self) {
        match self.ui.focus {
            Focus::Blocks => self.move_block_selection_up(),
            Focus::Transactions => self.move_transaction_selection_up(),
        }
    }

    pub(crate) fn move_block_selection_up(&mut self) {
        if let Some(index) = self.nav.selected_block_index {
            if index > 0 {
                let new_index = index - 1;
                self.nav.select_block(new_index, &self.data.blocks);

                let block_scroll = new_index as u16 * BLOCK_HEIGHT;
                if block_scroll < self.nav.block_scroll {
                    self.nav.block_scroll = block_scroll;
                }
            }
        } else if !self.data.blocks.is_empty() {
            self.nav.select_block(0, &self.data.blocks);
            self.nav.block_scroll = 0;
        }
    }

    pub(crate) fn move_transaction_selection_up(&mut self) {
        if let Some(index) = self.nav.selected_transaction_index {
            if index > 0 {
                let new_index = index - 1;
                self.nav
                    .select_transaction(new_index, &self.data.transactions);

                let txn_scroll = new_index as u16 * TXN_HEIGHT;
                if txn_scroll < self.nav.transaction_scroll {
                    self.nav.transaction_scroll = txn_scroll;
                }
            }
        } else if !self.data.transactions.is_empty() {
            self.nav.select_transaction(0, &self.data.transactions);
            self.nav.transaction_scroll = 0;
        }
    }

    /// Moves the selection down in the currently focused list.
    pub fn move_selection_down(&mut self) {
        match self.ui.focus {
            Focus::Blocks => self.move_block_selection_down(),
            Focus::Transactions => self.move_transaction_selection_down(),
        }
    }

    pub(crate) fn move_block_selection_down(&mut self) {
        let max_index = self.data.blocks.len().saturating_sub(1);

        if let Some(index) = self.nav.selected_block_index {
            if index < max_index {
                let new_index = index + 1;
                self.nav.select_block(new_index, &self.data.blocks);

                let visible_end = self.nav.block_scroll + (DEFAULT_VISIBLE_BLOCKS * BLOCK_HEIGHT);
                let item_position = (new_index as u16) * BLOCK_HEIGHT;

                if item_position >= visible_end {
                    self.nav.block_scroll = self.nav.block_scroll.saturating_add(BLOCK_HEIGHT);
                }
            }
        } else if !self.data.blocks.is_empty() {
            self.nav.select_block(0, &self.data.blocks);
        }
    }

    pub(crate) fn move_transaction_selection_down(&mut self) {
        let max_index = self.data.transactions.len().saturating_sub(1);

        if let Some(index) = self.nav.selected_transaction_index {
            if index < max_index {
                let new_index = index + 1;
                self.nav
                    .select_transaction(new_index, &self.data.transactions);

                let visible_end =
                    self.nav.transaction_scroll + (DEFAULT_VISIBLE_TRANSACTIONS * TXN_HEIGHT);
                let item_position = (new_index as u16) * TXN_HEIGHT;

                if item_position >= visible_end {
                    self.nav.transaction_scroll =
                        self.nav.transaction_scroll.saturating_add(TXN_HEIGHT);
                }
            }
        } else if !self.data.transactions.is_empty() {
            self.nav.select_transaction(0, &self.data.transactions);
        }
    }

    /// Jumps to the top of the currently focused list.
    pub fn go_to_top(&mut self) {
        // Check if help popup is open
        if self.ui.show_help {
            self.ui.help_scroll_offset = 0;
            return;
        }

        // Check if in detail views
        if self.nav.show_block_details {
            if self.nav.block_detail_tab == BlockDetailTab::Transactions
                && let Some(block_details) = &self.data.block_details
                && !block_details.transactions.is_empty()
            {
                self.nav.block_txn_index = Some(0);
                self.nav.block_txn_scroll = 0;
            }
            return;
        }

        if self.nav.show_account_details {
            if let Some(account) = &self.data.viewed_account {
                let has_items = match self.nav.account_detail_tab {
                    AccountDetailTab::Assets => !account.assets.is_empty(),
                    AccountDetailTab::Apps => !account.apps_local_state.is_empty(),
                    AccountDetailTab::Info => false,
                };
                if has_items {
                    self.nav.account_item_index = Some(0);
                    self.nav.account_item_scroll = 0;
                }
            }
            return;
        }

        if self.nav.show_application_details {
            if let Some(app) = &self.data.viewed_application
                && self.nav.app_detail_tab == AppDetailTab::State
                && !app.global_state.is_empty()
            {
                self.nav.app_state_index = Some(0);
                self.nav.app_state_scroll = 0;
            }
            return;
        }

        // Check popup state
        match &self.ui.popup_state {
            PopupState::NetworkSelect(_) => {
                self.ui.update_network_selection(0);
                return;
            }
            PopupState::SearchResults(results) if results.len() > 1 => {
                // Rotate to show first result (already at front)
                return;
            }
            _ => {}
        }

        // Main context navigation
        match self.ui.focus {
            Focus::Blocks => {
                if !self.data.blocks.is_empty() {
                    self.nav.select_block(0, &self.data.blocks);
                    self.nav.block_scroll = 0;
                }
            }
            Focus::Transactions => {
                if !self.data.transactions.is_empty() {
                    self.nav.select_transaction(0, &self.data.transactions);
                    self.nav.transaction_scroll = 0;
                }
            }
        }
    }

    /// Jumps to the bottom of the currently focused list.
    pub fn go_to_bottom(&mut self) {
        // Check if help popup is open
        if self.ui.show_help {
            // Set to a large value - it will be clamped by the help render function
            self.ui.help_scroll_offset = 1000;
            return;
        }

        // Check if in detail views
        if self.nav.show_block_details {
            if self.nav.block_detail_tab == BlockDetailTab::Transactions
                && let Some(block_details) = &self.data.block_details
            {
                let max_index = block_details.transactions.len().saturating_sub(1);
                if !block_details.transactions.is_empty() {
                    self.nav.block_txn_index = Some(max_index);
                    // Scroll to show the last item
                    let item_height: u16 = 2;
                    let total_height = block_details.transactions.len() as u16 * item_height;
                    let visible_height: u16 = 20; // Approximate visible height
                    self.nav.block_txn_scroll = total_height.saturating_sub(visible_height);
                }
            }
            return;
        }

        if self.nav.show_account_details {
            if let Some(account) = &self.data.viewed_account {
                let max = match self.nav.account_detail_tab {
                    AccountDetailTab::Assets => account.assets.len().saturating_sub(1),
                    AccountDetailTab::Apps => account.apps_local_state.len().saturating_sub(1),
                    AccountDetailTab::Info => return,
                };
                if max > 0 {
                    self.nav.account_item_index = Some(max);
                    // Scroll to show the last item
                    let visible_height: u16 = 8;
                    self.nav.account_item_scroll = (max as u16).saturating_sub(visible_height) + 1;
                }
            }
            return;
        }

        if self.nav.show_application_details {
            if let Some(app) = &self.data.viewed_application
                && self.nav.app_detail_tab == AppDetailTab::State
                && !app.global_state.is_empty()
            {
                let max = app.global_state.len().saturating_sub(1);
                self.nav.app_state_index = Some(max);
                // Scroll to show the last item
                let visible_height: u16 = 8;
                self.nav.app_state_scroll = (max as u16).saturating_sub(visible_height) + 1;
            }
            return;
        }

        // Check popup state
        match &self.ui.popup_state {
            PopupState::NetworkSelect(_) => {
                self.ui.update_network_selection(2); // 3 networks: 0, 1, 2
                return;
            }
            PopupState::SearchResults(results) if results.len() > 1 => {
                // Rotate to show last result - it's already a ring buffer, no-op is fine
                return;
            }
            _ => {}
        }

        // Main context navigation
        match self.ui.focus {
            Focus::Blocks => {
                let max_index = self.data.blocks.len().saturating_sub(1);
                if !self.data.blocks.is_empty() {
                    self.nav.select_block(max_index, &self.data.blocks);
                    // Scroll to show the last item
                    let total_height = self.data.blocks.len() as u16 * BLOCK_HEIGHT;
                    let visible_height = DEFAULT_VISIBLE_BLOCKS * BLOCK_HEIGHT;
                    self.nav.block_scroll = total_height.saturating_sub(visible_height);
                }
            }
            Focus::Transactions => {
                let max_index = self.data.transactions.len().saturating_sub(1);
                if !self.data.transactions.is_empty() {
                    self.nav
                        .select_transaction(max_index, &self.data.transactions);
                    // Scroll to show the last item
                    let total_height = self.data.transactions.len() as u16 * TXN_HEIGHT;
                    let visible_height = DEFAULT_VISIBLE_TRANSACTIONS * TXN_HEIGHT;
                    self.nav.transaction_scroll = total_height.saturating_sub(visible_height);
                }
            }
        }
    }

    /// Shows details for the currently selected item.
    pub async fn show_details(&mut self) {
        match self.ui.focus {
            Focus::Blocks if self.nav.selected_block_index.is_some() => {
                // Reset block detail state
                self.nav.block_detail_tab = BlockDetailTab::default();
                self.nav.block_txn_index = None;
                self.data.block_details = None;
                self.nav.show_block_details = true;

                // Load block details asynchronously
                if let Some(block_id) = self.nav.selected_block_id {
                    self.load_block_details(block_id);
                }
            }
            Focus::Transactions if self.nav.selected_transaction_index.is_some() => {
                // Reset graph scroll position and bounds when opening transaction details
                self.nav.graph_scroll_x = 0;
                self.nav.graph_scroll_y = 0;
                self.nav.graph_max_scroll_x = 0;
                self.nav.graph_max_scroll_y = 0;
                self.nav.show_transaction_details = true;
                // Build detail table rows for copy functionality
                self.update_detail_table_rows();
            }
            _ => {}
        }
    }

    /// Loads block details asynchronously.
    pub(crate) fn load_block_details(&self, round: u64) {
        let client = self.client.clone();
        let message_tx = self.message_tx.clone();

        tokio::spawn(async move {
            // Channel sends below: receiver may be dropped during shutdown - safe to ignore
            match client.get_block_details(round).await {
                Ok(Some(details)) => {
                    let _ = message_tx.send(AppMessage::BlockDetailsLoaded(details));
                }
                Ok(None) => {
                    let _ = message_tx.send(AppMessage::NetworkError(format!(
                        "Block {} not found",
                        round
                    )));
                }
                Err(e) => {
                    let _ = message_tx.send(AppMessage::NetworkError(e.to_string()));
                }
            }
        });
    }

    /// Loads transaction details asynchronously by transaction ID.
    pub(crate) async fn load_transaction_details(&self, txn_id: &str) {
        let client = self.client.clone();
        let message_tx = self.message_tx.clone();
        let txn_id = txn_id.to_string();

        tokio::spawn(async move {
            // Channel sends below: receiver may be dropped during shutdown - safe to ignore
            match client.get_transaction_by_id(&txn_id).await {
                Ok(Some(txn)) => {
                    let _ = message_tx.send(AppMessage::TransactionDetailsLoaded(Box::new(txn)));
                }
                Ok(None) => {
                    let _ = message_tx.send(AppMessage::TransactionDetailsFailed(
                        "Transaction not found".to_string(),
                    ));
                }
                Err(e) => {
                    let _ = message_tx.send(AppMessage::TransactionDetailsFailed(e.to_string()));
                }
            }
        });
    }

    // ========================================================================
    // Search
}
