use arboard::Clipboard;
use color_eyre::Result;
use crossterm::event::{
    self, Event, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};
use tokio::time::interval;

use crate::algorand::{
    AlgoBlock, AlgoClient, BlockDetails, Network, SearchResultItem, Transaction,
};
use crate::commands::{AppCommand, KeyMapper};
use crate::ui;

// ============================================================================
// Configuration
// ============================================================================

/// Configuration structure for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    network: Network,
    show_live: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: Network::MainNet,
            show_live: true,
        }
    }
}

impl AppConfig {
    fn config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not find config directory"))?;
        path.push("lazylora");
        fs::create_dir_all(&path)?;
        path.push("config.json");
        Ok(path)
    }

    fn load() -> Self {
        Self::config_path()
            .and_then(|path| {
                let content = fs::read_to_string(&path)?;
                let config: AppConfig = serde_json::from_str(&content)?;
                Ok(config)
            })
            .unwrap_or_default()
    }

    fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

// ============================================================================
// Messages & Enums
// ============================================================================

/// Messages sent between async tasks and the main app loop.
#[derive(Debug, Clone)]
pub enum AppMessage {
    BlocksUpdated(Vec<AlgoBlock>),
    TransactionsUpdated(Vec<Transaction>),
    SearchCompleted(Result<Vec<SearchResultItem>, String>),
    NetworkError(String),
    NetworkConnected,
    NetworkSwitchComplete,
    BlockDetailsLoaded(BlockDetails),
    /// Transaction details loaded for viewing in popup
    TransactionDetailsLoaded(Box<Transaction>),
    /// Failed to load transaction details
    TransactionDetailsFailed(String),
    /// Account details loaded for viewing in popup
    AccountDetailsLoaded(Box<crate::algorand::AccountDetails>),
    /// Failed to load account details
    AccountDetailsFailed(String),
    /// Asset details loaded for viewing in popup
    AssetDetailsLoaded(Box<crate::algorand::AssetDetails>),
    /// Failed to load asset details
    AssetDetailsFailed(String),
}

/// Represents which UI panel currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Blocks,
    Transactions,
}

/// Represents the current popup/modal state.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PopupState {
    #[default]
    None,
    /// Network selection popup with the currently highlighted index.
    NetworkSelect(usize),
    /// Search popup with query text and search type.
    SearchWithType(String, SearchType),
    /// Message/notification popup.
    Message(String),
    /// Search results display with indexed items.
    SearchResults(Vec<(usize, SearchResultItem)>),
}

impl PopupState {
    /// Returns `true` if there is an active popup.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns the search query and type if in search mode.
    #[must_use]
    #[allow(dead_code)]
    pub fn as_search(&self) -> Option<(&str, SearchType)> {
        match self {
            Self::SearchWithType(query, search_type) => Some((query.as_str(), *search_type)),
            _ => None,
        }
    }

    /// Returns the search results if displaying results.
    #[must_use]
    #[allow(dead_code)]
    pub fn as_search_results(&self) -> Option<&[(usize, SearchResultItem)]> {
        match self {
            Self::SearchResults(results) => Some(results.as_slice()),
            _ => None,
        }
    }

    /// Returns the network select index if in network select mode.
    #[must_use]
    #[allow(dead_code)]
    pub const fn as_network_select(&self) -> Option<usize> {
        match self {
            Self::NetworkSelect(index) => Some(*index),
            _ => None,
        }
    }
}

/// The type of search to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchType {
    #[default]
    Transaction,
    Asset,
    Account,
    Block,
}

/// The view mode for transaction/block details popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailViewMode {
    #[default]
    Table,
    Visual,
}

/// The tab in the block details popup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockDetailTab {
    #[default]
    Info,
    Transactions,
}

impl SearchType {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Transaction => "Transaction",
            Self::Asset => "Asset",
            Self::Account => "Account",
            Self::Block => "Block",
        }
    }

    /// Cycles to the next search type.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Transaction => Self::Block,
            Self::Block => Self::Account,
            Self::Account => Self::Asset,
            Self::Asset => Self::Transaction,
        }
    }
}

// ============================================================================
// State Sub-Structures
// ============================================================================

/// Navigation state: selection indices, scroll positions, and detail view flags.
#[derive(Debug, Default)]
pub struct NavigationState {
    /// Scroll position for blocks list (in pixels/rows).
    pub block_scroll: u16,
    /// Scroll position for transactions list (in pixels/rows).
    pub transaction_scroll: u16,
    /// Currently selected block index in the blocks list.
    pub selected_block_index: Option<usize>,
    /// Currently selected transaction index in the transactions list.
    pub selected_transaction_index: Option<usize>,
    /// The block ID of the currently selected block (for stable selection across updates).
    pub selected_block_id: Option<u64>,
    /// The transaction ID of the currently selected transaction (for stable selection).
    pub selected_transaction_id: Option<String>,
    /// Whether the block details popup is shown.
    pub show_block_details: bool,
    /// Whether the transaction details popup is shown.
    pub show_transaction_details: bool,
    /// Whether the account details popup is shown.
    pub show_account_details: bool,
    /// Whether the asset details popup is shown.
    pub show_asset_details: bool,
    /// Current tab in block details popup
    pub block_detail_tab: BlockDetailTab,
    /// Selected transaction index within block details
    pub block_txn_index: Option<usize>,
    /// Scroll position for block transactions list
    pub block_txn_scroll: u16,
    /// Horizontal scroll offset for transaction graph view
    pub graph_scroll_x: u16,
    /// Vertical scroll offset for transaction graph view
    pub graph_scroll_y: u16,
}

impl NavigationState {
    /// Creates a new `NavigationState` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets all navigation state (useful when switching networks).
    pub fn reset(&mut self) {
        self.block_scroll = 0;
        self.transaction_scroll = 0;
        self.selected_block_index = None;
        self.selected_transaction_index = None;
        self.selected_block_id = None;
        self.selected_transaction_id = None;
        self.show_block_details = false;
        self.show_transaction_details = false;
        self.show_account_details = false;
        self.show_asset_details = false;
        self.block_detail_tab = BlockDetailTab::default();
        self.block_txn_index = None;
        self.block_txn_scroll = 0;
        self.graph_scroll_x = 0;
        self.graph_scroll_y = 0;
    }

    /// Returns `true` if any detail view is currently shown.
    #[must_use]
    pub const fn is_showing_details(&self) -> bool {
        self.show_block_details
            || self.show_transaction_details
            || self.show_account_details
            || self.show_asset_details
    }

    /// Closes all detail views.
    pub fn close_details(&mut self) {
        self.show_block_details = false;
        self.show_transaction_details = false;
        self.show_account_details = false;
        self.show_asset_details = false;
    }

    /// Selects a block by index and updates the stable ID.
    pub fn select_block(&mut self, index: usize, blocks: &[AlgoBlock]) {
        self.selected_block_index = Some(index);
        self.selected_block_id = blocks.get(index).map(|b| b.id);
    }

    /// Selects a transaction by index and updates the stable ID.
    pub fn select_transaction(&mut self, index: usize, transactions: &[Transaction]) {
        self.selected_transaction_index = Some(index);
        self.selected_transaction_id = transactions.get(index).map(|t| t.id.clone());
    }

    /// Clears block selection.
    pub fn clear_block_selection(&mut self) {
        self.selected_block_index = None;
        self.selected_block_id = None;
    }

    /// Clears transaction selection.
    pub fn clear_transaction_selection(&mut self) {
        self.selected_transaction_index = None;
        self.selected_transaction_id = None;
    }

    /// Cycles the block detail tab between Info and Transactions.
    pub fn cycle_block_detail_tab(&mut self) {
        self.block_detail_tab = match self.block_detail_tab {
            BlockDetailTab::Info => BlockDetailTab::Transactions,
            BlockDetailTab::Transactions => BlockDetailTab::Info,
        };
    }

    /// Selects a transaction within the block details view.
    #[allow(dead_code)]
    pub fn select_block_txn(&mut self, index: usize) {
        self.block_txn_index = Some(index);
    }

    /// Moves the block transaction selection up.
    pub fn move_block_txn_up(&mut self) {
        if let Some(idx) = self.block_txn_index
            && idx > 0
        {
            self.block_txn_index = Some(idx - 1);
            // Adjust scroll if needed (each txn item is 2 lines)
            let item_height: u16 = 2;
            let new_pos = (idx - 1) as u16 * item_height;
            if new_pos < self.block_txn_scroll {
                self.block_txn_scroll = new_pos;
            }
        }
    }

    /// Moves the block transaction selection down.
    pub fn move_block_txn_down(&mut self, max: usize, visible_height: u16) {
        let item_height: u16 = 2;
        if let Some(idx) = self.block_txn_index {
            if idx < max {
                self.block_txn_index = Some(idx + 1);
                // Adjust scroll if needed
                let new_pos = (idx + 1) as u16 * item_height;
                let visible_end = self.block_txn_scroll + visible_height;
                if new_pos + item_height > visible_end {
                    self.block_txn_scroll = (new_pos + item_height).saturating_sub(visible_height);
                }
            }
        } else if max > 0 {
            self.block_txn_index = Some(0);
            self.block_txn_scroll = 0;
        }
    }
}

/// Data state: blocks, transactions, and search results.
#[derive(Debug, Default)]
pub struct DataState {
    /// List of recent blocks.
    pub blocks: Vec<AlgoBlock>,
    /// List of recent transactions.
    pub transactions: Vec<Transaction>,
    /// Filtered search results with their original indices.
    pub filtered_search_results: Vec<(usize, SearchResultItem)>,
    /// Currently loaded block details (for block details popup)
    pub block_details: Option<BlockDetails>,
    /// Currently viewed transaction details (for transaction details popup)
    pub viewed_transaction: Option<Transaction>,
    /// Currently viewed account details (for account details popup)
    pub viewed_account: Option<crate::algorand::AccountDetails>,
    /// Currently viewed asset details (for asset details popup)
    pub viewed_asset: Option<crate::algorand::AssetDetails>,
}

impl DataState {
    /// Creates a new `DataState` with empty collections.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all data (useful when switching networks).
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.transactions.clear();
        self.filtered_search_results.clear();
        self.block_details = None;
        self.viewed_transaction = None;
        self.viewed_account = None;
        self.viewed_asset = None;
    }

    /// Returns `true` if there are no blocks.
    #[must_use]
    #[allow(dead_code)]
    pub fn has_no_blocks(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Returns `true` if there are no transactions.
    #[must_use]
    #[allow(dead_code)]
    pub fn has_no_transactions(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Finds a block index by its ID.
    #[must_use]
    pub fn find_block_index(&self, block_id: u64) -> Option<usize> {
        self.blocks.iter().position(|b| b.id == block_id)
    }

    /// Finds a transaction index by its ID.
    #[must_use]
    pub fn find_transaction_index(&self, txn_id: &str) -> Option<usize> {
        self.transactions.iter().position(|t| t.id == txn_id)
    }

    /// Gets a transaction by ID from search results.
    #[must_use]
    #[allow(dead_code)]
    pub fn find_search_result_transaction(&self, txn_id: &str) -> Option<&Transaction> {
        self.filtered_search_results
            .iter()
            .find_map(|(_, item)| match item {
                SearchResultItem::Transaction(t) if t.id == txn_id => Some(t.as_ref()),
                _ => None,
            })
    }
}

/// UI state: focus, popup state, and viewing flags.
#[derive(Debug, Default)]
pub struct UiState {
    /// Which panel currently has focus.
    pub focus: Focus,
    /// Current popup/modal state.
    pub popup_state: PopupState,
    /// Whether we're currently viewing a search result (affects transaction details display).
    pub viewing_search_result: bool,
    /// The view mode for detail popups (Visual or Table).
    pub detail_view_mode: DetailViewMode,
    /// Set of expanded section names in transaction details (e.g., "app_args", "accounts").
    pub expanded_sections: HashSet<String>,
    /// Currently focused expandable section index in transaction details.
    pub detail_section_index: Option<usize>,
    /// Whether the detail popup is in fullscreen mode.
    pub detail_fullscreen: bool,
    /// Toast notification message and remaining ticks (non-blocking overlay).
    pub toast: Option<(String, u8)>,
}

impl UiState {
    /// Creates a new `UiState` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Cycles focus between Blocks and Transactions panels only.
    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Blocks => Focus::Transactions,
            Focus::Transactions => Focus::Blocks,
        };
    }

    /// Returns `true` if the popup is active.
    #[must_use]
    pub fn has_active_popup(&self) -> bool {
        self.popup_state.is_active()
    }

    /// Dismisses the current popup.
    pub fn dismiss_popup(&mut self) {
        self.popup_state = PopupState::None;
    }

    /// Shows a message popup.
    pub fn show_message(&mut self, message: impl Into<String>) {
        self.popup_state = PopupState::Message(message.into());
    }

    /// Shows a toast notification (non-blocking overlay that auto-dismisses).
    /// Duration is in ticks (each tick is ~100ms in the main loop).
    pub fn show_toast(&mut self, message: impl Into<String>, ticks: u8) {
        self.toast = Some((message.into(), ticks));
    }

    /// Decrements the toast countdown. Returns true if toast should be removed.
    pub fn tick_toast(&mut self) -> bool {
        if let Some((_, ref mut ticks)) = self.toast {
            if *ticks > 1 {
                *ticks -= 1;
                false
            } else {
                self.toast = None;
                true
            }
        } else {
            false
        }
    }

    /// Opens the search popup.
    pub fn open_search(&mut self) {
        self.popup_state = PopupState::SearchWithType(String::new(), SearchType::Transaction);
    }

    /// Opens the network selection popup with the given current index.
    pub fn open_network_select(&mut self, current_index: usize) {
        self.popup_state = PopupState::NetworkSelect(current_index);
    }

    /// Updates the search query text.
    pub fn update_search_query(&mut self, new_query: String, search_type: SearchType) {
        self.popup_state = PopupState::SearchWithType(new_query, search_type);
    }

    /// Switches to the next search type while preserving the query.
    pub fn cycle_search_type(&mut self, query: String, current_type: SearchType) {
        self.popup_state = PopupState::SearchWithType(query, current_type.next());
    }

    /// Updates the network selection index.
    pub fn update_network_selection(&mut self, index: usize) {
        self.popup_state = PopupState::NetworkSelect(index);
    }

    /// Sets search results popup.
    pub fn show_search_results(&mut self, results: Vec<(usize, SearchResultItem)>) {
        self.popup_state = PopupState::SearchResults(results);
    }

    /// Toggles the detail view mode between Visual and Table.
    pub fn toggle_detail_view_mode(&mut self) {
        self.detail_view_mode = match self.detail_view_mode {
            DetailViewMode::Visual => DetailViewMode::Table,
            DetailViewMode::Table => DetailViewMode::Visual,
        };
    }

    /// Toggles whether a section is expanded in transaction details.
    pub fn toggle_section(&mut self, section_name: &str) {
        if self.expanded_sections.contains(section_name) {
            self.expanded_sections.remove(section_name);
        } else {
            self.expanded_sections.insert(section_name.to_string());
        }
    }

    /// Returns whether a section is expanded.
    pub fn is_section_expanded(&self, section_name: &str) -> bool {
        self.expanded_sections.contains(section_name)
    }

    /// Resets expanded sections and fullscreen state (call when closing details).
    pub fn reset_expanded_sections(&mut self) {
        self.expanded_sections.clear();
        self.detail_section_index = None;
        self.detail_fullscreen = false;
    }

    /// Toggles fullscreen mode for detail popups.
    #[allow(dead_code)]
    pub fn toggle_fullscreen(&mut self) {
        self.detail_fullscreen = !self.detail_fullscreen;
    }
}

// ============================================================================
// Main App Structure
// ============================================================================

/// The main application state, composed of focused sub-states.
#[derive(Debug)]
pub struct App {
    // === Sub-states ===
    /// Navigation state (selections, scroll positions, detail views).
    pub nav: NavigationState,
    /// Data state (blocks, transactions, search results).
    pub data: DataState,
    /// UI state (focus, popups, viewing flags).
    pub ui: UiState,

    // === App-level state ===
    /// Current network.
    pub network: Network,
    /// Whether live updates are enabled.
    pub show_live: bool,
    /// Whether the app should exit.
    pub exit: bool,
    /// Animation tick counter for UI animations (e.g., logo shimmer).
    pub animation_tick: u64,

    // === Channels ===
    message_tx: mpsc::UnboundedSender<AppMessage>,
    message_rx: mpsc::UnboundedReceiver<AppMessage>,
    live_updates_tx: watch::Sender<bool>,
    network_tx: watch::Sender<Network>,

    // === Client ===
    client: AlgoClient,
}

impl App {
    /// Creates a new App instance, loading configuration from disk.
    ///
    /// # Errors
    /// Returns an error if initialization fails.
    pub async fn new() -> Result<Self> {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (live_updates_tx, _live_updates_rx) = watch::channel(true);
        let (network_tx, _network_rx) = watch::channel(Network::MainNet);

        // Load configuration
        let config = AppConfig::load();
        let network = config.network;
        let show_live = config.show_live;
        let client = AlgoClient::new(network);

        // Set initial state from config
        let _ = live_updates_tx.send(show_live);
        let _ = network_tx.send(network);

        Ok(Self {
            nav: NavigationState::new(),
            data: DataState::new(),
            ui: UiState::new(),
            network,
            show_live,
            exit: false,
            animation_tick: 0,
            message_tx,
            message_rx,
            live_updates_tx,
            network_tx,
            client,
        })
    }

    /// Runs the main application loop.
    ///
    /// # Errors
    /// Returns an error if the terminal operations fail.
    pub async fn run(&mut self, terminal: &mut crate::tui::Tui) -> Result<()> {
        self.start_background_tasks().await;
        self.initial_data_fetch().await;

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        while !self.exit {
            self.process_messages().await;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));

            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        self.handle_key_event(key).await?;
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_input(mouse).await?;
                    }
                    Event::Resize(_, _) => {
                        terminal.draw(|frame| ui::render(self, frame))?;
                    }
                    _ => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.animation_tick = self.animation_tick.wrapping_add(1);
                self.sync_selections();
                self.tick_timed_message_countdown();
                terminal.draw(|frame| ui::render(self, frame))?;
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    // ========================================================================
    // Message Processing
    // ========================================================================

    async fn process_messages(&mut self) {
        while let Ok(message) = self.message_rx.try_recv() {
            match message {
                AppMessage::BlocksUpdated(new_blocks) => {
                    self.merge_blocks(new_blocks);
                }
                AppMessage::TransactionsUpdated(new_transactions) => {
                    self.merge_transactions(new_transactions);
                }
                AppMessage::SearchCompleted(Ok(results)) => {
                    self.handle_search_results(results);
                }
                AppMessage::SearchCompleted(Err(error)) => {
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
            }
        }
    }

    fn handle_search_results(&mut self, results: Vec<SearchResultItem>) {
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

    fn merge_blocks(&mut self, new_blocks: Vec<AlgoBlock>) {
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

    fn merge_transactions(&mut self, new_transactions: Vec<Transaction>) {
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

    // ========================================================================
    // Background Tasks
    // ========================================================================

    async fn start_background_tasks(&self) {
        let message_tx = self.message_tx.clone();
        let live_updates_rx = self.live_updates_tx.subscribe();
        let network_rx = self.network_tx.subscribe();
        let client = self.client.clone();

        tokio::spawn(async move {
            Self::data_fetching_task(message_tx, live_updates_rx, network_rx, client).await;
        });
    }

    async fn data_fetching_task(
        message_tx: mpsc::UnboundedSender<AppMessage>,
        mut live_updates_rx: watch::Receiver<bool>,
        mut network_rx: watch::Receiver<Network>,
        mut client: AlgoClient,
    ) {
        let mut block_interval = interval(Duration::from_secs(5));
        let mut transaction_interval = interval(Duration::from_secs(5));
        let mut network_check_interval = interval(Duration::from_secs(10));

        let mut is_network_available = true;
        let mut network_error_shown = false;

        loop {
            tokio::select! {
                _ = live_updates_rx.changed() => {}

                _ = network_rx.changed() => {
                    let new_network = *network_rx.borrow_and_update();
                    client = AlgoClient::new(new_network);
                    is_network_available = true;
                    network_error_shown = false;
                }

                _ = network_check_interval.tick() => {
                    if *live_updates_rx.borrow() {
                        match client.get_network_status().await {
                            Ok(()) => {
                                if !is_network_available {
                                    let _ = message_tx.send(AppMessage::NetworkConnected);
                                }
                                is_network_available = true;
                                network_error_shown = false;
                            }
                            Err(error_msg) => {
                                if !network_error_shown {
                                    let _ = message_tx.send(AppMessage::NetworkError(error_msg));
                                    network_error_shown = true;
                                }
                                is_network_available = false;
                            }
                        }
                    }
                }

                _ = block_interval.tick() => {
                    if *live_updates_rx.borrow() && is_network_available
                        && let Ok(blocks) = client.get_latest_blocks(5).await
                    {
                        let _ = message_tx.send(AppMessage::BlocksUpdated(blocks));
                    }
                }

                _ = transaction_interval.tick() => {
                    if *live_updates_rx.borrow() && is_network_available
                        && let Ok(transactions) = client.get_latest_transactions(5).await
                    {
                        let _ = message_tx.send(AppMessage::TransactionsUpdated(transactions));
                    }
                }
            }
        }
    }

    async fn initial_data_fetch(&self) {
        let message_tx = self.message_tx.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            match client.get_network_status().await {
                Err(error_msg) => {
                    let _ = message_tx.send(AppMessage::NetworkError(error_msg));
                    return;
                }
                Ok(()) => {
                    let _ = message_tx.send(AppMessage::NetworkConnected);
                }
            }

            if let Ok(blocks) = client.get_latest_blocks(5).await {
                let _ = message_tx.send(AppMessage::BlocksUpdated(blocks));
            }

            if let Ok(transactions) = client.get_latest_transactions(5).await {
                let _ = message_tx.send(AppMessage::TransactionsUpdated(transactions));
            }
        });
    }

    // ========================================================================
    // Input Handling
    // ========================================================================

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let context = self.get_input_context();
        let command = KeyMapper::map_key(key_event, &context);
        self.execute_command(command).await
    }

    /// Executes an application command.
    ///
    /// This method handles all `AppCommand` variants and performs the corresponding
    /// application state mutations.
    async fn execute_command(&mut self, command: AppCommand) -> Result<()> {
        match command {
            // === Application Control ===
            AppCommand::Quit => {
                self.exit = true;
            }
            AppCommand::Refresh => {
                self.initial_data_fetch().await;
            }
            AppCommand::ToggleLive => {
                self.toggle_live_updates();
            }

            // === Popup/Modal Control ===
            AppCommand::OpenSearch => {
                self.ui.open_search();
            }
            AppCommand::OpenNetworkSelect => {
                let current_index = match self.network {
                    Network::MainNet => 0,
                    Network::TestNet => 1,
                    Network::LocalNet => 2,
                };
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

            // === Detail View Actions ===
            AppCommand::CopyToClipboard => {
                if self.nav.show_transaction_details {
                    self.copy_transaction_id_to_clipboard();
                }
            }
            AppCommand::ToggleDetailViewMode => {
                if self.nav.is_showing_details() {
                    self.ui.toggle_detail_view_mode();
                }
            }
            AppCommand::DetailSectionUp => {
                if self.nav.show_transaction_details {
                    self.move_detail_section_up();
                }
            }
            AppCommand::DetailSectionDown => {
                if self.nav.show_transaction_details {
                    self.move_detail_section_down();
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
                    self.nav.graph_scroll_y = self.nav.graph_scroll_y.saturating_sub(1);
                }
            }
            AppCommand::GraphScrollDown => {
                if self.nav.show_transaction_details {
                    self.nav.graph_scroll_y = self.nav.graph_scroll_y.saturating_add(1);
                }
            }
            AppCommand::ExportSvg => {
                if self.nav.show_transaction_details {
                    self.export_transaction_svg();
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

            // === Search Input Actions ===
            AppCommand::TypeChar(c) => {
                if let PopupState::SearchWithType(query, search_type) = &self.ui.popup_state {
                    let mut new_query = query.clone();
                    new_query.push(c);
                    let search_type = *search_type;
                    self.ui.update_search_query(new_query, search_type);
                }
            }
            AppCommand::Backspace => {
                if let PopupState::SearchWithType(query, search_type) = &self.ui.popup_state {
                    let mut new_query = query.clone();
                    new_query.pop();
                    let search_type = *search_type;
                    self.ui.update_search_query(new_query, search_type);
                }
            }
            AppCommand::CycleSearchType => {
                if let PopupState::SearchWithType(query, search_type) = &self.ui.popup_state {
                    let query = query.clone();
                    let search_type = *search_type;
                    self.ui.cycle_search_type(query, search_type);
                }
            }
            AppCommand::SubmitSearch => {
                if let PopupState::SearchWithType(query, search_type) = &self.ui.popup_state {
                    let query = query.clone();
                    let search_type = *search_type;
                    self.ui.dismiss_popup();
                    self.search_transactions(&query, search_type).await;
                }
            }

            // === Network Selection Actions ===
            AppCommand::NetworkUp => {
                if let PopupState::NetworkSelect(index) = &self.ui.popup_state {
                    let new_index = if *index == 0 { 2 } else { index - 1 };
                    self.ui.update_network_selection(new_index);
                }
            }
            AppCommand::NetworkDown => {
                if let PopupState::NetworkSelect(index) = &self.ui.popup_state {
                    let new_index = if *index == 2 { 0 } else { index + 1 };
                    self.ui.update_network_selection(new_index);
                }
            }
            AppCommand::SelectNetwork => {
                if let PopupState::NetworkSelect(index) = &self.ui.popup_state {
                    let new_network = match index {
                        0 => Network::MainNet,
                        1 => Network::TestNet,
                        2 => Network::LocalNet,
                        _ => Network::MainNet,
                    };
                    self.ui.dismiss_popup();
                    self.switch_network(new_network).await;
                }
            }

            // === Search Results Actions ===
            AppCommand::PreviousResult => {
                if let PopupState::SearchResults(results) = &self.ui.popup_state
                    && results.len() > 1
                {
                    let mut updated_results = results.clone();
                    let first = updated_results.remove(0);
                    updated_results.push(first);
                    self.ui.show_search_results(updated_results);
                }
            }
            AppCommand::NextResult => {
                if let PopupState::SearchResults(results) = &self.ui.popup_state
                    && results.len() > 1
                {
                    let mut updated_results = results.clone();
                    if let Some(last) = updated_results.pop() {
                        updated_results.insert(0, last);
                    }
                    self.ui.show_search_results(updated_results);
                }
            }
            AppCommand::SelectResult => {
                self.handle_select_result();
            }

            // === No Operation ===
            AppCommand::Noop => {}
        }
        Ok(())
    }

    /// Handles the Dismiss command based on current context.
    fn handle_dismiss(&mut self) {
        if self.nav.is_showing_details() {
            self.nav.close_details();
            self.ui.viewing_search_result = false;
            self.ui.reset_expanded_sections();
            self.data.viewed_transaction = None;
            self.data.viewed_account = None;
            self.data.viewed_asset = None;
            // Reset graph scroll position
            self.nav.graph_scroll_x = 0;
            self.nav.graph_scroll_y = 0;
        } else {
            match &self.ui.popup_state {
                PopupState::SearchWithType(_, _) | PopupState::SearchResults(_) => {
                    self.ui.dismiss_popup();
                    self.data.filtered_search_results.clear();
                    self.ui.viewing_search_result = false;
                }
                PopupState::NetworkSelect(_) | PopupState::Message(_) => {
                    self.ui.dismiss_popup();
                }
                PopupState::None => {}
            }
        }
    }

    /// Handles selecting a search result.
    fn handle_select_result(&mut self) {
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
            }
        }
    }

    /// Loads account details asynchronously
    fn load_account_details(&self, address: &str) {
        let message_tx = self.message_tx.clone();
        let client = self.client.clone();
        let address = address.to_string();

        tokio::spawn(async move {
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
    fn load_asset_details(&self, asset_id: u64) {
        let message_tx = self.message_tx.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
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

    async fn handle_mouse_input(&mut self, mouse: MouseEvent) -> Result<()> {
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
    fn handle_mouse_scroll_up(&mut self) -> Result<()> {
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
    fn handle_mouse_scroll_down(&mut self) -> Result<()> {
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

    fn handle_popup_mouse_click(&mut self, mouse: MouseEvent) -> Result<()> {
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

    fn handle_main_mouse_click(&mut self, mouse: MouseEvent) -> Result<()> {
        let header_height = 3;
        let title_height = 3;

        if mouse.row <= (header_height + title_height) {
            return Ok(());
        }

        let content_start_row = header_height + title_height;
        let content_row = mouse.row.saturating_sub(content_start_row);

        let terminal_width = 100;
        let is_left_half = mouse.column <= terminal_width / 2;

        if is_left_half {
            self.ui.focus = Focus::Blocks;
            let block_index = (content_row / 3) as usize;
            if block_index < self.data.blocks.len() {
                self.nav.select_block(block_index, &self.data.blocks);
            }
        } else {
            self.ui.focus = Focus::Transactions;
            let txn_index = (content_row / 4) as usize;
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

    fn sync_selections(&mut self) {
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
    fn tick_timed_message_countdown(&mut self) {
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

    fn move_block_selection_up(&mut self) {
        if let Some(index) = self.nav.selected_block_index {
            if index > 0 {
                let new_index = index - 1;
                self.nav.select_block(new_index, &self.data.blocks);

                let block_height = 3;
                let block_scroll = new_index as u16 * block_height;
                if block_scroll < self.nav.block_scroll {
                    self.nav.block_scroll = block_scroll;
                }
            }
        } else if !self.data.blocks.is_empty() {
            self.nav.select_block(0, &self.data.blocks);
            self.nav.block_scroll = 0;
        }
    }

    fn move_transaction_selection_up(&mut self) {
        if let Some(index) = self.nav.selected_transaction_index {
            if index > 0 {
                let new_index = index - 1;
                self.nav
                    .select_transaction(new_index, &self.data.transactions);

                let txn_height = 4;
                let txn_scroll = new_index as u16 * txn_height;
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

    fn move_block_selection_down(&mut self) {
        let max_index = self.data.blocks.len().saturating_sub(1);

        if let Some(index) = self.nav.selected_block_index {
            if index < max_index {
                let new_index = index + 1;
                self.nav.select_block(new_index, &self.data.blocks);

                let block_height = 3;
                let block_display_height = 10;
                let visible_end = self.nav.block_scroll + (block_display_height * block_height);
                let item_position = (new_index as u16) * block_height;

                if item_position >= visible_end {
                    self.nav.block_scroll = self.nav.block_scroll.saturating_add(block_height);
                }
            }
        } else if !self.data.blocks.is_empty() {
            self.nav.select_block(0, &self.data.blocks);
        }
    }

    fn move_transaction_selection_down(&mut self) {
        let max_index = self.data.transactions.len().saturating_sub(1);

        if let Some(index) = self.nav.selected_transaction_index {
            if index < max_index {
                let new_index = index + 1;
                self.nav
                    .select_transaction(new_index, &self.data.transactions);

                let txn_height = 4;
                let txn_display_height = 10;
                let visible_end = self.nav.transaction_scroll + (txn_display_height * txn_height);
                let item_position = (new_index as u16) * txn_height;

                if item_position >= visible_end {
                    self.nav.transaction_scroll =
                        self.nav.transaction_scroll.saturating_add(txn_height);
                }
            }
        } else if !self.data.transactions.is_empty() {
            self.nav.select_transaction(0, &self.data.transactions);
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
                // Reset graph scroll position when opening transaction details
                self.nav.graph_scroll_x = 0;
                self.nav.graph_scroll_y = 0;
                self.nav.show_transaction_details = true;
            }
            _ => {}
        }
    }

    /// Loads block details asynchronously.
    fn load_block_details(&self, round: u64) {
        let client = self.client.clone();
        let message_tx = self.message_tx.clone();

        tokio::spawn(async move {
            match client.get_block_details(round).await {
                Ok(Some(details)) => {
                    let _ = message_tx.send(AppMessage::BlockDetailsLoaded(details));
                }
                Ok(None) => {
                    let _ =
                        message_tx.send(AppMessage::NetworkError("Block not found".to_string()));
                }
                Err(e) => {
                    let _ = message_tx.send(AppMessage::NetworkError(e.to_string()));
                }
            }
        });
    }

    /// Loads transaction details asynchronously by transaction ID.
    async fn load_transaction_details(&self, txn_id: &str) {
        let client = self.client.clone();
        let message_tx = self.message_tx.clone();
        let txn_id = txn_id.to_string();

        tokio::spawn(async move {
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
    // ========================================================================

    async fn search_transactions(&mut self, query: &str, search_type: SearchType) {
        if query.is_empty() {
            self.ui.show_message("Please enter a search term");
            return;
        }

        let search_type_str = match search_type {
            SearchType::Transaction => "transactions",
            SearchType::Asset => "assets",
            SearchType::Account => "accounts",
            SearchType::Block => "blocks",
        };

        self.ui.show_message(format!(
            "Querying Algorand network APIs for {search_type_str}..."
        ));

        let client = self.client.clone();
        let query_clone = query.to_string();
        let message_tx = self.message_tx.clone();

        tokio::spawn(async move {
            let result = client.search_by_query(&query_clone, search_type).await;
            let message = match result {
                Ok(items) => AppMessage::SearchCompleted(Ok(items)),
                Err(e) => AppMessage::SearchCompleted(Err(e.to_string())),
            };
            let _ = message_tx.send(message);
        });
    }

    // ========================================================================
    // Network & Config
    // ========================================================================

    async fn switch_network(&mut self, network: Network) {
        self.ui
            .show_toast(format!("Switching to {}...", network.as_str()), 20);

        self.network = network;
        self.client = AlgoClient::new(network);
        let _ = self.network_tx.send(network);

        self.save_config();
        self.data.clear();
        self.nav.reset();
        self.ui.viewing_search_result = false;

        self.initial_data_fetch().await;

        tokio::spawn({
            let message_tx = self.message_tx.clone();
            async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let _ = message_tx.send(AppMessage::NetworkSwitchComplete);
            }
        });
    }

    fn toggle_live_updates(&mut self) {
        self.show_live = !self.show_live;
        let _ = self.live_updates_tx.send(self.show_live);
        self.save_config();
    }

    fn save_config(&self) {
        let config = AppConfig {
            network: self.network,
            show_live: self.show_live,
        };
        if let Err(e) = config.save() {
            eprintln!("Failed to save configuration: {e}");
        }
    }

    // ========================================================================
    // Clipboard
    // ========================================================================

    fn copy_transaction_id_to_clipboard(&mut self) {
        // Try to get transaction ID from selected_transaction_id first,
        // then fall back to the current transaction being viewed
        let txn_id = self
            .nav
            .selected_transaction_id
            .clone()
            .or_else(|| self.get_current_transaction().map(|t| t.id.clone()));

        let Some(txn_id) = txn_id else {
            self.ui.show_toast("✗ No transaction selected", 20);
            return;
        };

        // On Linux, try to use external clipboard tools first as they persist
        // the clipboard content even after the application exits
        #[cfg(target_os = "linux")]
        {
            if self.try_copy_with_external_tool(&txn_id) {
                self.ui.show_toast("✓ Transaction ID copied!", 20);
                return;
            }
            // Fall back to arboard if external tools fail
        }

        match Clipboard::new() {
            Ok(mut clipboard) => {
                if clipboard.set_text(txn_id.clone()).is_ok() {
                    self.ui.show_toast("✓ Transaction ID copied!", 20);
                } else {
                    self.ui.show_toast("✗ Failed to copy", 20);
                }
            }
            Err(_) => {
                self.ui.show_toast("✗ Clipboard not available", 20);
            }
        }
    }

    /// Try to copy text using external clipboard tools (Linux only).
    /// Returns true if successful.
    #[cfg(target_os = "linux")]
    fn try_copy_with_external_tool(&self, text: &str) -> bool {
        use std::io::Write;
        use std::process::{Command, Stdio};

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

    /// Export the current transaction graph to SVG file.
    fn export_transaction_svg(&mut self) {
        use crate::widgets::TxnGraph;

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
    fn get_expandable_sections(&self) -> Vec<&'static str> {
        let txn_opt = self.get_current_transaction();

        let Some(txn) = txn_opt else {
            return vec![];
        };

        use crate::algorand::TransactionDetails;
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
    fn get_current_transaction(&self) -> Option<Transaction> {
        if self.ui.viewing_search_result {
            self.nav
                .selected_transaction_id
                .as_ref()
                .and_then(|txn_id| {
                    self.data
                        .filtered_search_results
                        .iter()
                        .find_map(|(_, item)| match item {
                            crate::algorand::SearchResultItem::Transaction(t)
                                if &t.id == txn_id =>
                            {
                                Some((**t).clone())
                            }
                            _ => None,
                        })
                })
        } else {
            self.nav
                .selected_transaction_index
                .and_then(|index| self.data.transactions.get(index).cloned())
        }
    }

    /// Move to the previous expandable section.
    fn move_detail_section_up(&mut self) {
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
    fn move_detail_section_down(&mut self) {
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
    fn toggle_current_detail_section(&mut self) {
        let sections = self.get_expandable_sections();
        if sections.is_empty() {
            return;
        }

        if let Some(idx) = self.ui.detail_section_index
            && let Some(section_name) = sections.get(idx)
        {
            self.ui.toggle_section(section_name);
        }
    }
}
