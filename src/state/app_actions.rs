//! Application actions for LazyLora.
//!
//! This module handles high-level actions like searching, network switching,
//! clipboard operations, browser integration, and SVG export.

use arboard::Clipboard;
use color_eyre::Result;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::{App, AppConfig, AppMessage, DetailViewMode, PopupState, SearchType};
use crate::client::AlgoClient;
use crate::domain::{Network, NetworkConfig, SearchResultItem, Transaction, TransactionDetails};
use crate::ui;
use crate::widgets::TxnGraph;

impl App {
    pub(crate) async fn search_transactions(&mut self, query: &str, search_type: SearchType) {
        if query.is_empty() {
            self.ui.show_message("Please enter a search term");
            return;
        }

        // Set loading state
        self.ui.set_search_loading(true);

        let client = self.client.clone();
        let query_clone = query.to_string();
        let message_tx = self.message_tx.clone();

        tokio::spawn(async move {
            let result = client.search_by_query(&query_clone, search_type).await;
            let message = match result {
                Ok(items) => AppMessage::SearchCompleted(Ok(items)),
                Err(e) => AppMessage::SearchCompleted(Err(e.to_string())),
            };
            // Receiver may be dropped during shutdown - safe to ignore
            let _ = message_tx.send(message);
        });
    }

    // ========================================================================
    // Network & Config
    // ========================================================================

    pub(crate) async fn handle_submit_network_form(&mut self) -> Result<()> {
        let PopupState::NetworkForm(form) = &self.ui.popup_state else {
            return Ok(());
        };

        let name = form.name.trim().to_string();
        let indexer_url = form.indexer_url.trim().to_string();
        let algod_url = form.algod_url.trim().to_string();
        let indexer_port = form.indexer_port.trim();
        let algod_port = form.algod_port.trim();
        let indexer_token = form.indexer_token.trim();
        let algod_token = form.algod_token.trim();

        if name.is_empty() || indexer_url.is_empty() || algod_url.is_empty() {
            self.ui
                .show_toast("Name, Indexer URL, and Algod URL are required", 30);
            return Ok(());
        }

        let indexer_port: u16 = match indexer_port.parse() {
            Ok(port) if port > 0 => port,
            _ => {
                self.ui.show_toast("Indexer port must be a number", 30);
                return Ok(());
            }
        };

        let algod_port: u16 = match algod_port.parse() {
            Ok(port) if port > 0 => port,
            _ => {
                self.ui.show_toast("Algod port must be a number", 30);
                return Ok(());
            }
        };

        if !indexer_url.starts_with("http") || !algod_url.starts_with("http") {
            self.ui.show_toast("URLs should start with http/https", 30);
            return Ok(());
        }

        let indexer_full = Self::append_port_if_missing(&indexer_url, indexer_port);
        let algod_full = Self::append_port_if_missing(&algod_url, algod_port);

        let custom = crate::domain::CustomNetwork {
            name,
            indexer_url: indexer_full,
            algod_url: algod_full,
            indexer_token: (!indexer_token.is_empty()).then(|| indexer_token.to_string()),
            algod_token: (!algod_token.is_empty()).then(|| algod_token.to_string()),
            nfd_api_url: None,
        };

        let mut config = AppConfig::load();
        if let Err(e) = config.add_custom_network(custom.clone()) {
            self.ui.show_toast(format!("Failed to save: {e}"), 30);
            return Ok(());
        }

        self.available_networks = config.get_all_networks();
        self.ui.dismiss_popup();
        self.switch_network_config(NetworkConfig::Custom(custom))
            .await;
        Ok(())
    }

    pub(crate) fn append_port_if_missing(url: &str, port: u16) -> String {
        let port_str = port.to_string();
        if let Some((scheme, rest)) = url.split_once("://") {
            if rest.contains(':') {
                url.to_string()
            } else {
                format!("{scheme}://{rest}:{port_str}")
            }
        } else if url.contains(':') {
            url.to_string()
        } else {
            format!("{url}:{port_str}")
        }
    }

    /// Switches to the given network configuration.
    pub(crate) async fn switch_network_config(&mut self, config: NetworkConfig) {
        let name = config.as_str().to_string();
        self.ui.show_toast(format!("Switching to {name}..."), 20);

        // Create client from config (supports both built-in and custom)
        let new_client = match AlgoClient::from_config(&config) {
            Ok(client) => client,
            Err(e) => {
                self.ui
                    .show_toast(format!("Failed to switch network: {e}"), 50);
                return;
            }
        };

        // Watch channel send: subscribers react to network changes (clone for channel)
        let _ = self.network_tx.send(config.clone());
        // Update config (move original to field)
        self.network = match &config {
            NetworkConfig::BuiltIn(network) => *network,
            NetworkConfig::Custom(_) => Network::MainNet,
        };

        self.client = new_client;

        // Move config to field after all borrows complete
        self.network_config = config;

        self.save_config();
        self.data.clear();
        self.nav.reset();
        self.ui.viewing_search_result = false;

        self.initial_data_fetch().await;

        tokio::spawn({
            let message_tx = self.message_tx.clone();
            async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                // Receiver may be dropped during shutdown - safe to ignore
                let _ = message_tx.send(AppMessage::NetworkSwitchComplete);
            }
        });
    }

    pub(crate) fn toggle_live_updates(&mut self) {
        self.show_live = !self.show_live;
        // Watch channel send: background task reacts to toggle
        let _ = self.live_updates_tx.send(self.show_live);
        self.save_config();
    }

    pub(crate) fn save_config(&self) {
        let mut config = AppConfig::load();
        config.network = self.network_config.clone();
        config.show_live = self.show_live;
        if let Err(e) = config.save() {
            eprintln!("Failed to save configuration: {e}");
        }
    }

    // ========================================================================
    // Clipboard
    // ========================================================================

    pub(crate) fn copy_transaction_id_to_clipboard(&mut self) {
        // In Table mode with a selected row, copy the row value from flat list
        if self.nav.show_transaction_details
            && self.ui.detail_view_mode == DetailViewMode::Table
            && let Some(row_idx) = self.nav.detail_row_index
        {
            // Get the value from the flat row list
            if let Some(txn) = self.get_transaction_for_details() {
                let flat_rows =
                    ui::panels::details::transaction::build_flat_row_list_for_copy(&txn);

                if let Some((label, value)) = flat_rows.get(row_idx) {
                    let text_to_copy = value.clone();
                    let label_short = label.trim_end_matches(':').trim();

                    #[cfg(target_os = "linux")]
                    {
                        if self.try_copy_with_external_tool(&text_to_copy) {
                            self.ui
                                .show_toast(format!("[+] {} copied!", label_short), 20);
                            return;
                        }
                    }

                    match Clipboard::new() {
                        Ok(mut clipboard) => {
                            if clipboard.set_text(text_to_copy).is_ok() {
                                self.ui
                                    .show_toast(format!("[+] {} copied!", label_short), 20);
                            } else {
                                self.ui.show_toast("[x] Failed to copy", 20);
                            }
                        }
                        Err(_) => {
                            self.ui.show_toast("[x] Clipboard not available", 20);
                        }
                    }
                    return;
                }
            }
        }

        // Default: copy transaction ID
        let txn_id = self
            .nav
            .selected_transaction_id
            .clone()
            .or_else(|| self.get_current_transaction().map(|t| t.id.clone()));

        let Some(txn_id) = txn_id else {
            self.ui.show_toast("[x] No transaction selected", 20);
            return;
        };

        // On Linux, try to use external clipboard tools first as they persist
        // the clipboard content even after the application exits
        #[cfg(target_os = "linux")]
        {
            if self.try_copy_with_external_tool(&txn_id) {
                self.ui.show_toast("[+] Transaction ID copied!", 20);
                return;
            }
            // Fall back to arboard if external tools fail
        }

        match Clipboard::new() {
            Ok(mut clipboard) => {
                if clipboard.set_text(txn_id).is_ok() {
                    self.ui.show_toast("[+] Transaction ID copied!", 20);
                } else {
                    self.ui.show_toast("[x] Failed to copy", 20);
                }
            }
            Err(_) => {
                self.ui.show_toast("[x] Clipboard not available", 20);
            }
        }
    }

    /// Try to copy text using external clipboard tools (Linux only).
    /// Returns true if successful.
    #[cfg(target_os = "linux")]
    pub(crate) fn try_copy_with_external_tool(&self, text: &str) -> bool {
        // Try wl-copy first (Wayland)
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            && let Some(mut stdin) = child.stdin.take()
            && stdin.write_all(text.as_bytes()).is_ok()
        {
            drop(stdin);
            if child.wait().map(|s| s.success()).unwrap_or(false) {
                return true;
            }
        }

        // Try xclip (X11)
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            && let Some(mut stdin) = child.stdin.take()
            && stdin.write_all(text.as_bytes()).is_ok()
        {
            drop(stdin);
            if child.wait().map(|s| s.success()).unwrap_or(false) {
                return true;
            }
        }

        // Try xsel (X11 alternative)
        if let Ok(mut child) = Command::new("xsel")
            .args(["--clipboard", "--input"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            && let Some(mut stdin) = child.stdin.take()
            && stdin.write_all(text.as_bytes()).is_ok()
        {
            drop(stdin);
            if child.wait().map(|s| s.success()).unwrap_or(false) {
                return true;
            }
        }

        false
    }

    /// Copy raw JSON to clipboard for the currently viewed entity.
    ///
    /// Fetches the raw JSON from the indexer/algod and copies it to clipboard.
    /// Works for transactions, blocks, accounts, assets, and applications.
    pub(crate) async fn copy_json_to_clipboard(&mut self) {
        // Determine what entity we're viewing and fetch its JSON
        if self.nav.show_transaction_details {
            if let Some(txn) = self.get_current_transaction() {
                self.fetch_and_copy_transaction_json(&txn.id).await;
            } else {
                self.ui.show_toast("[x] No transaction selected", 20);
            }
        } else if self.nav.show_block_details {
            if let Some(block_id) = self.nav.selected_block_id {
                self.fetch_and_copy_block_json(block_id).await;
            } else {
                self.ui.show_toast("[x] No block selected", 20);
            }
        } else if self.nav.show_account_details {
            if let Some(account) = &self.data.viewed_account {
                self.fetch_and_copy_account_json(&account.address.clone())
                    .await;
            } else {
                self.ui.show_toast("[x] No account selected", 20);
            }
        } else if self.nav.show_asset_details {
            if let Some(asset) = &self.data.viewed_asset {
                self.fetch_and_copy_asset_json(asset.id).await;
            } else {
                self.ui.show_toast("[x] No asset selected", 20);
            }
        } else if self.nav.show_application_details {
            if let Some(application) = &self.data.viewed_application {
                self.fetch_and_copy_application_json(application.app_id)
                    .await;
            } else {
                self.ui.show_toast("[x] No application selected", 20);
            }
        } else {
            self.ui.show_toast("[x] No detail view open", 20);
        }
    }

    /// Fetch transaction JSON from indexer and copy to clipboard.
    pub(crate) async fn fetch_and_copy_transaction_json(&mut self, txn_id: &str) {
        let url = format!(
            "{}/v2/transactions/{}",
            self.network_config.indexer_url(),
            txn_id
        );
        self.fetch_and_copy_json(&url, "Transaction").await;
    }

    /// Fetch block JSON from algod and copy to clipboard.
    pub(crate) async fn fetch_and_copy_block_json(&mut self, round: u64) {
        let url = format!("{}/v2/blocks/{}", self.network_config.algod_url(), round);
        self.fetch_and_copy_json(&url, "Block").await;
    }

    /// Fetch account JSON from algod and copy to clipboard.
    pub(crate) async fn fetch_and_copy_account_json(&mut self, address: &str) {
        let url = format!(
            "{}/v2/accounts/{}",
            self.network_config.algod_url(),
            address
        );
        self.fetch_and_copy_json(&url, "Account").await;
    }

    /// Fetch asset JSON from indexer and copy to clipboard.
    pub(crate) async fn fetch_and_copy_asset_json(&mut self, asset_id: u64) {
        let url = format!(
            "{}/v2/assets/{}",
            self.network_config.indexer_url(),
            asset_id
        );
        self.fetch_and_copy_json(&url, "Asset").await;
    }

    /// Fetch application JSON from indexer and copy to clipboard.
    pub(crate) async fn fetch_and_copy_application_json(&mut self, app_id: u64) {
        let url = format!(
            "{}/v2/applications/{}",
            self.network_config.indexer_url(),
            app_id
        );
        self.fetch_and_copy_json(&url, "Application").await;
    }

    /// Generic helper to fetch JSON from a URL and copy to clipboard.
    pub(crate) async fn fetch_and_copy_json(&mut self, url: &str, entity_name: &str) {
        self.ui
            .show_toast(format!("Fetching {} JSON...", entity_name), 10);

        let client = reqwest::Client::new();
        let mut request = client.get(url).header("accept", "application/json");

        let is_algod = url.contains(self.network_config.algod_url());
        let token = if is_algod {
            self.network_config.algod_token()
        } else {
            self.network_config.indexer_token()
        };

        if let Some(token) = token {
            let header = if is_algod {
                "X-Algo-API-Token"
            } else {
                "X-Indexer-API-Token"
            };
            request = request.header(header, token);
        } else if matches!(
            self.network_config,
            NetworkConfig::BuiltIn(Network::LocalNet)
        ) {
            let header = if is_algod {
                "X-Algo-API-Token"
            } else {
                "X-Indexer-API-Token"
            };
            request = request.header(
                header,
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            );
        }

        match request.send().await {
            Ok(response) if response.status().is_success() => {
                match response.text().await {
                    Ok(json_text) => {
                        // Pretty-print the JSON
                        let pretty_json =
                            match serde_json::from_str::<serde_json::Value>(&json_text) {
                                Ok(value) => {
                                    serde_json::to_string_pretty(&value).unwrap_or(json_text)
                                }
                                Err(_) => json_text,
                            };
                        self.copy_text_to_clipboard(&pretty_json, &format!("{} JSON", entity_name));
                    }
                    Err(_) => {
                        self.ui.show_toast("[x] Failed to read response", 20);
                    }
                }
            }
            Ok(response) => {
                self.ui
                    .show_toast(format!("[x] HTTP {}", response.status()), 20);
            }
            Err(e) => {
                self.ui.show_toast(format!("[x] Network error: {}", e), 20);
            }
        }
    }

    /// Copy text to clipboard with platform-specific handling.
    pub(crate) fn copy_text_to_clipboard(&mut self, text: &str, description: &str) {
        #[cfg(target_os = "linux")]
        {
            if self.try_copy_with_external_tool(text) {
                self.ui
                    .show_toast(format!("[+] {} copied!", description), 20);
                return;
            }
        }

        match Clipboard::new() {
            Ok(mut clipboard) => {
                if clipboard.set_text(text).is_ok() {
                    self.ui
                        .show_toast(format!("[+] {} copied!", description), 20);
                } else {
                    self.ui.show_toast("[x] Failed to copy", 20);
                }
            }
            Err(_) => {
                self.ui.show_toast("[x] Clipboard not available", 20);
            }
        }
    }

    /// Open the current entity in the web browser (Lora explorer).
    pub(crate) fn open_in_browser(&mut self) {
        // Only built-in networks have Lora URLs
        if !matches!(self.network_config, NetworkConfig::BuiltIn(_)) {
            self.ui.show_toast(
                "Opening in browser is only available on built-in networks",
                30,
            );
            return;
        }

        let url = if self.nav.show_transaction_details {
            self.get_current_transaction()
                .map(|txn| self.network_config.transaction_url(&txn.id))
                .filter(|url| !url.is_empty())
        } else if self.nav.show_block_details {
            self.nav
                .selected_block_id
                .map(|block_id| self.network_config.block_url(block_id))
                .filter(|url| !url.is_empty())
        } else if self.nav.show_account_details {
            self.data
                .viewed_account
                .as_ref()
                .map(|account| self.network_config.account_url(&account.address))
                .filter(|url| !url.is_empty())
        } else if self.nav.show_asset_details {
            self.data
                .viewed_asset
                .as_ref()
                .map(|asset| self.network_config.asset_url(asset.id))
                .filter(|url| !url.is_empty())
        } else if self.nav.show_application_details {
            self.data
                .viewed_application
                .as_ref()
                .map(|app| self.network_config.application_url(app.app_id))
                .filter(|url| !url.is_empty())
        } else {
            None
        };

        match url {
            Some(url) => match open::that(&url) {
                Ok(()) => {
                    self.ui.show_toast("[+] Opened in browser", 20);
                }
                Err(e) => {
                    self.ui
                        .show_toast(format!("[x] Failed to open browser: {}", e), 30);
                }
            },
            None => {
                self.ui.show_toast("[x] Explorer link unavailable", 20);
            }
        }
    }

    /// Export the current transaction graph to SVG file.
    pub(crate) fn export_transaction_svg(&mut self) {
        // Get the current transaction
        let txn = self.get_current_transaction();
        let Some(txn) = txn else {
            self.ui.show_toast("No transaction selected", 20);
            return;
        };

        // Build the graph and export to SVG
        let graph = TxnGraph::from_transaction(&txn);
        let svg_content = graph.to_svg();

        // Create filename based on transaction ID (truncated)
        let txn_id = &txn.id;
        let short_id = if txn_id.len() > 12 {
            format!("{}_{}", &txn_id[..6], &txn_id[txn_id.len() - 6..])
        } else {
            txn_id.clone()
        };
        let filename = format!("lazylora_{}.svg", short_id);

        // Try to save to current directory or home directory
        let path = std::path::Path::new(&filename);
        match std::fs::write(path, &svg_content) {
            Ok(()) => {
                self.ui.show_toast(format!("Saved {}", filename), 30);
            }
            Err(e) => {
                // Try home directory as fallback
                if let Some(home) = dirs::home_dir() {
                    let home_path = home.join(&filename);
                    match std::fs::write(&home_path, &svg_content) {
                        Ok(()) => {
                            self.ui.show_toast(format!("Saved ~/{}", filename), 30);
                        }
                        Err(_) => {
                            self.ui.show_toast(format!("Export failed: {}", e), 30);
                        }
                    }
                } else {
                    self.ui.show_toast(format!("Export failed: {}", e), 30);
                }
            }
        }
    }

    // ========================================================================
    // Expandable Sections
    // ========================================================================

    /// Returns the list of expandable section names for the current transaction.
    pub(crate) fn get_expandable_sections(&self) -> Vec<&'static str> {
        let txn_opt = self.get_current_transaction();

        let Some(txn) = txn_opt else {
            return vec![];
        };

        match &txn.details {
            TransactionDetails::AppCall(app_details) => {
                let mut sections = Vec::new();
                if !app_details.app_args.is_empty() {
                    sections.push("app_args");
                }
                if !app_details.accounts.is_empty() {
                    sections.push("accounts");
                }
                if !app_details.foreign_apps.is_empty() {
                    sections.push("foreign_apps");
                }
                if !app_details.foreign_assets.is_empty() {
                    sections.push("foreign_assets");
                }
                if !app_details.boxes.is_empty() {
                    sections.push("boxes");
                }
                sections
            }
            _ => vec![],
        }
    }

    /// Gets the current transaction being viewed.
    pub(crate) fn get_current_transaction(&self) -> Option<Transaction> {
        // First check if we have a directly viewed transaction (from search or block details)
        if let Some(txn) = &self.data.viewed_transaction {
            return Some(txn.clone());
        }

        // Then check if viewing a search result
        if self.ui.viewing_search_result {
            return self
                .nav
                .selected_transaction_id
                .as_ref()
                .and_then(|txn_id| {
                    self.data
                        .filtered_search_results
                        .iter()
                        .find_map(|(_, item)| match item {
                            SearchResultItem::Transaction(t) if &t.id == txn_id => {
                                Some((**t).clone())
                            }
                            _ => None,
                        })
                });
        }

        // Finally fall back to selected transaction from main list
        self.nav
            .selected_transaction_index
            .and_then(|index| self.data.transactions.get(index).cloned())
    }

    /// Move to the previous expandable section.
    pub(crate) fn move_detail_section_up(&mut self) {
        let sections = self.get_expandable_sections();
        if sections.is_empty() {
            return;
        }

        if let Some(idx) = self.ui.detail_section_index {
            if idx > 0 {
                self.ui.detail_section_index = Some(idx - 1);
            }
        } else {
            // Start from the last section when pressing up with no selection
            self.ui.detail_section_index = Some(sections.len() - 1);
        }
    }

    /// Move to the next expandable section.
    pub(crate) fn move_detail_section_down(&mut self) {
        let sections = self.get_expandable_sections();
        if sections.is_empty() {
            return;
        }

        if let Some(idx) = self.ui.detail_section_index {
            if idx < sections.len() - 1 {
                self.ui.detail_section_index = Some(idx + 1);
            }
        } else {
            self.ui.detail_section_index = Some(0);
        }
    }

    /// Toggle the currently selected expandable section.
    /// In Table mode, uses the current row position; in Visual mode, uses section index.
    pub(crate) fn toggle_current_detail_section(&mut self) {
        let sections = self.get_expandable_sections();
        if sections.is_empty() {
            return;
        }

        // In Table mode, detect section from current row label
        if self.ui.detail_view_mode == DetailViewMode::Table {
            if let Some(row_idx) = self.nav.detail_row_index
                && let Some((label, _)) = self.ui.detail_table_rows.get(row_idx)
            {
                // Check if this row is an expandable section header (contains ▶ or ▼)
                if let Some(section_name) = self.detect_section_from_row_label(label) {
                    self.ui.toggle_section(&section_name);
                    // Rebuild rows to reflect expanded/collapsed state
                    self.update_detail_table_rows();
                }
            }
        } else {
            // Visual mode: use section index
            if let Some(idx) = self.ui.detail_section_index
                && let Some(section_name) = sections.get(idx)
            {
                self.ui.toggle_section(section_name);
            }
        }
    }

    /// Detects which expandable section a row label belongs to.
    /// Returns the section name if the row is an expandable section header.
    pub(crate) fn detect_section_from_row_label(&self, label: &str) -> Option<String> {
        // Expandable section headers contain ▶ or ▼ followed by the section label
        // Format: "  ▶ App Args:" or "→ ▼ Accounts:" etc.
        let section_mappings = [
            ("App Args", "app_args"),
            ("Accounts", "accounts"),
            ("Foreign Apps", "foreign_apps"),
            ("Foreign Assets", "foreign_assets"),
            ("Box Refs", "boxes"),
        ];

        for (display_label, section_id) in section_mappings {
            if label.contains(display_label) && (label.contains('▶') || label.contains('▼')) {
                return Some(section_id.to_string());
            }
        }
        None
    }

    /// Updates the detail table rows based on the current transaction.
    /// Called when opening transaction details or when the transaction changes.
    pub(crate) fn update_detail_table_rows(&mut self) {
        let txn_opt = self.get_transaction_for_details();
        if let Some(txn) = txn_opt {
            self.ui.detail_table_rows =
                ui::panels::details::transaction::build_transaction_details(&txn, self);
            // Initialize row selection at first row
            self.nav
                .init_detail_row_if_needed(self.ui.detail_table_rows.len());
        } else {
            self.ui.detail_table_rows.clear();
            self.nav.reset_detail_row();
        }
    }

    /// Gets the transaction to display in details (from various sources).
    pub(crate) fn get_transaction_for_details(&self) -> Option<Transaction> {
        // First check for directly viewed transaction (from block details)
        if let Some(txn) = &self.data.viewed_transaction {
            return Some(txn.clone());
        }

        // Check if viewing a search result
        if self.ui.viewing_search_result {
            return self
                .nav
                .selected_transaction_id
                .as_ref()
                .and_then(|txn_id| {
                    self.data
                        .filtered_search_results
                        .iter()
                        .find_map(|(_, item)| match item {
                            SearchResultItem::Transaction(t) if &t.id == txn_id => {
                                Some((**t).clone())
                            }
                            _ => None,
                        })
                });
        }

        // Fall back to selected transaction from list
        self.nav
            .selected_transaction_index
            .and_then(|index| self.data.transactions.get(index).cloned())
    }
}
