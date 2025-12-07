//! Message processing for the LazyLora application.
//!
//! This module handles async message processing, background tasks,
//! and data fetching/merging logic.

use std::collections::HashSet;

use super::{App, AppMessage};
use crate::domain::{AlgoBlock, SearchResultItem, Transaction};

impl App {
    pub(crate) async fn process_messages(&mut self) {
        while let Ok(message) = self.message_rx.try_recv() {
            match message {
                AppMessage::BlocksUpdated(new_blocks) => {
                    self.merge_blocks(new_blocks);
                }
                AppMessage::TransactionsUpdated(new_transactions) => {
                    self.merge_transactions(new_transactions);
                }
                AppMessage::SearchCompleted(Ok(results)) => {
                    self.ui.set_search_loading(false);
                    self.handle_search_results(results);
                }
                AppMessage::SearchCompleted(Err(error)) => {
                    self.ui.set_search_loading(false);
                    self.ui.show_message(format!("Search error: {error}"));
                }
                AppMessage::NetworkError(error) => {
                    if !self.ui.has_active_popup() {
                        self.ui.show_message(error);
                    }
                    self.show_live = false;
                }
                AppMessage::NetworkConnected => {}
                AppMessage::NetworkSwitchComplete => {
                    self.ui.show_toast("Network switched", 20);
                }
                AppMessage::BlockDetailsLoaded(details) => {
                    // Auto-select first transaction if there are any
                    if !details.transactions.is_empty() {
                        self.nav.block_txn_index = Some(0);
                        self.nav.block_txn_scroll = 0;
                    }
                    self.data.block_details = Some(details);
                }
                AppMessage::TransactionDetailsLoaded(txn) => {
                    // Store the transaction for viewing
                    self.data.viewed_transaction = Some(*txn);
                    self.nav.show_transaction_details = true;
                    // Build detail table rows for copy functionality
                    self.update_detail_table_rows();
                }
                AppMessage::TransactionDetailsFailed(error) => {
                    self.ui
                        .show_message(format!("Failed to load transaction: {}", error));
                }
                AppMessage::AccountDetailsLoaded(details) => {
                    self.data.viewed_account = Some(*details);
                    self.nav.show_account_details = true;
                }
                AppMessage::AccountDetailsFailed(error) => {
                    self.nav.show_account_details = false;
                    self.ui
                        .show_message(format!("Failed to load account: {}", error));
                }
                AppMessage::AssetDetailsLoaded(details) => {
                    self.data.viewed_asset = Some(*details);
                    self.nav.show_asset_details = true;
                }
                AppMessage::AssetDetailsFailed(error) => {
                    self.nav.show_asset_details = false;
                    self.ui
                        .show_message(format!("Failed to load asset: {}", error));
                }
                AppMessage::ApplicationDetailsLoaded(details) => {
                    self.data.viewed_application = Some(*details);
                    self.nav.show_application_details = true;
                }
                AppMessage::ApplicationDetailsFailed(error) => {
                    self.nav.show_application_details = false;
                    self.ui
                        .show_message(format!("Failed to load application: {}", error));
                }
            }
        }
    }

    pub(crate) fn handle_search_results(&mut self, results: Vec<SearchResultItem>) {
        if results.is_empty() {
            self.ui.show_message("No matching data found");
        } else {
            let results_with_indices: Vec<(usize, SearchResultItem)> =
                results.into_iter().enumerate().collect();
            self.data
                .filtered_search_results
                .clone_from(&results_with_indices);
            self.ui.show_search_results(results_with_indices);
        }
    }

    // ========================================================================
    // Data Merging
    // ========================================================================

    pub(crate) fn merge_blocks(&mut self, new_blocks: Vec<AlgoBlock>) {
        if new_blocks.is_empty() {
            return;
        }

        let existing_ids: HashSet<u64> = self.data.blocks.iter().map(|b| b.id).collect();

        for new_block in new_blocks {
            if !existing_ids.contains(&new_block.id) {
                let pos = self.data.blocks.partition_point(|b| b.id > new_block.id);
                self.data.blocks.insert(pos, new_block);
            }
        }

        if self.data.blocks.len() > 100 {
            self.data.blocks.truncate(100);
        }
    }

    pub(crate) fn merge_transactions(&mut self, new_transactions: Vec<Transaction>) {
        if new_transactions.is_empty() {
            return;
        }

        let existing_ids: HashSet<&str> = self
            .data
            .transactions
            .iter()
            .map(|t| t.id.as_str())
            .collect();

        let mut updated_transactions = Vec::with_capacity(100);

        for new_txn in new_transactions {
            if !existing_ids.contains(new_txn.id.as_str()) {
                updated_transactions.push(new_txn);
            }
        }

        for old_txn in &self.data.transactions {
            if updated_transactions.len() >= 100 {
                break;
            }
            if !updated_transactions.iter().any(|t| t.id == old_txn.id) {
                updated_transactions.push(old_txn.clone());
            }
        }

        self.data.transactions = updated_transactions;
    }
}
