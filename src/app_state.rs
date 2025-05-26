use arboard::Clipboard;
use color_eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};
use tokio::time::interval;

use crate::algorand::{AlgoBlock, AlgoClient, Network, SearchResultItem, Transaction};
use crate::ui;

// Configuration structure for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    network: Network,
    show_live: bool,
    // Add more settings as needed
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

#[derive(Debug, Clone)]
pub enum AppMessage {
    BlocksUpdated(Vec<AlgoBlock>),
    TransactionsUpdated(Vec<Transaction>),
    SearchCompleted(Result<Vec<SearchResultItem>, String>),
    NetworkError(String),
    NetworkConnected,
    NetworkSwitchComplete,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Blocks,
    Transactions,
    Sidebar,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopupState {
    None,
    NetworkSelect(usize),               // Index of the selected network
    SearchWithType(String, SearchType), // Search query with explicit search type
    Message(String),                    // A message to display to the user
    SearchResults(Vec<(usize, SearchResultItem)>), // Search results with original indices
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchType {
    Transaction, // Search only transactions (default)
    Asset,       // Search only assets
    Account,     // Search only accounts
    Block,       // Search only blocks
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
}

#[derive(Debug)]
pub struct App {
    pub network: Network,
    pub blocks: Vec<AlgoBlock>,
    pub transactions: Vec<Transaction>,
    pub show_live: bool,
    pub focus: Focus,
    pub exit: bool,
    pub block_scroll: u16,
    pub transaction_scroll: u16,
    pub selected_block_index: Option<usize>,
    pub selected_transaction_index: Option<usize>,
    pub selected_block_id: Option<u64>,
    pub selected_transaction_id: Option<String>,
    pub show_block_details: bool,
    pub show_transaction_details: bool,
    pub popup_state: PopupState,
    pub filtered_search_results: Vec<(usize, SearchResultItem)>,
    pub viewing_search_result: bool,

    message_tx: mpsc::UnboundedSender<AppMessage>,
    message_rx: mpsc::UnboundedReceiver<AppMessage>,

    live_updates_tx: watch::Sender<bool>,
    network_tx: watch::Sender<Network>,

    client: AlgoClient,
}

impl App {
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
            network,
            blocks: Vec::new(),
            transactions: Vec::new(),
            show_live,
            focus: Focus::Blocks,
            exit: false,
            block_scroll: 0,
            transaction_scroll: 0,
            selected_block_index: None,
            selected_transaction_index: None,
            selected_block_id: None,
            selected_transaction_id: None,
            show_block_details: false,
            show_transaction_details: false,
            popup_state: PopupState::None,
            filtered_search_results: Vec::new(),
            viewing_search_result: false,
            message_tx,
            message_rx,
            live_updates_tx,
            network_tx,
            client,
        })
    }

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
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            self.handle_key_event(key).await?;
                        }
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
                self.sync_selections();

                terminal.draw(|frame| ui::render(self, frame))?;
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

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
                    if results.is_empty() {
                        self.popup_state =
                            PopupState::Message("No matching data found".to_string());
                    } else {
                        let results_with_indices: Vec<(usize, SearchResultItem)> =
                            results.into_iter().enumerate().collect();
                        self.filtered_search_results = results_with_indices.clone();
                        self.popup_state = PopupState::SearchResults(results_with_indices);
                    }
                }
                AppMessage::SearchCompleted(Err(error)) => {
                    self.popup_state = PopupState::Message(format!("Search error: {}", error));
                }
                AppMessage::NetworkError(error) => {
                    if self.popup_state == PopupState::None {
                        self.popup_state = PopupState::Message(error);
                    }
                    self.show_live = false;
                }
                AppMessage::NetworkConnected => {}
                AppMessage::NetworkSwitchComplete => {
                    self.popup_state = PopupState::Message("Network switch completed".to_string());
                }
            }
        }
    }

    fn merge_blocks(&mut self, new_blocks: Vec<AlgoBlock>) {
        if new_blocks.is_empty() {
            return;
        }

        let existing_ids: HashSet<u64> = self.blocks.iter().map(|b| b.id).collect();

        for new_block in new_blocks {
            if !existing_ids.contains(&new_block.id) {
                let pos = self.blocks.partition_point(|b| b.id > new_block.id);
                self.blocks.insert(pos, new_block);
            }
        }

        if self.blocks.len() > 100 {
            self.blocks.truncate(100);
        }
    }

    fn merge_transactions(&mut self, new_transactions: Vec<Transaction>) {
        if new_transactions.is_empty() {
            return;
        }

        let existing_ids: HashSet<String> =
            self.transactions.iter().map(|t| t.id.clone()).collect();

        let mut updated_transactions = Vec::with_capacity(100);

        for new_txn in new_transactions {
            if !existing_ids.contains(&new_txn.id) {
                updated_transactions.push(new_txn);
            }
        }

        for old_txn in self.transactions.iter().cloned() {
            if updated_transactions.len() >= 100 {
                break;
            }
            if !updated_transactions.iter().any(|t| t.id == old_txn.id) {
                updated_transactions.push(old_txn);
            }
        }

        self.transactions = updated_transactions;
    }

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

                _ = live_updates_rx.changed() => {

                }


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
                    if *live_updates_rx.borrow() && is_network_available {
                        if let Ok(blocks) = client.get_latest_blocks(5).await {
                            let _ = message_tx.send(AppMessage::BlocksUpdated(blocks));
                        }
                    }
                }


                _ = transaction_interval.tick() => {
                    if *live_updates_rx.borrow() && is_network_available {
                        if let Ok(transactions) = client.get_latest_transactions(5).await {
                            let _ = message_tx.send(AppMessage::TransactionsUpdated(transactions));
                        }
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

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.popup_state.clone() {
            PopupState::NetworkSelect(index) => match key_event.code {
                KeyCode::Esc => {
                    self.popup_state = PopupState::None;
                    Ok(())
                }
                KeyCode::Up => {
                    let new_index = if index == 0 { 2 } else { index - 1 };
                    self.popup_state = PopupState::NetworkSelect(new_index);
                    Ok(())
                }
                KeyCode::Down => {
                    let new_index = if index == 2 { 0 } else { index + 1 };
                    self.popup_state = PopupState::NetworkSelect(new_index);
                    Ok(())
                }
                KeyCode::Enter => {
                    let new_network = match index {
                        0 => Network::MainNet,
                        1 => Network::TestNet,
                        2 => Network::LocalNet,
                        _ => Network::MainNet,
                    };
                    self.switch_network(new_network).await;
                    self.popup_state = PopupState::None;
                    Ok(())
                }
                _ => Ok(()),
            },
            PopupState::SearchWithType(query, search_type) => match key_event.code {
                KeyCode::Esc => {
                    self.popup_state = PopupState::None;
                    self.filtered_search_results.clear();
                    Ok(())
                }
                KeyCode::Enter => {
                    let query_clone = query.clone();
                    self.popup_state = PopupState::None;
                    self.search_transactions(&query_clone, search_type).await;
                    Ok(())
                }
                KeyCode::Tab => {
                    let new_search_type = match search_type {
                        SearchType::Transaction => SearchType::Block,
                        SearchType::Block => SearchType::Account,
                        SearchType::Account => SearchType::Asset,
                        SearchType::Asset => SearchType::Transaction,
                    };
                    self.popup_state = PopupState::SearchWithType(query, new_search_type);
                    Ok(())
                }
                KeyCode::Char(c) => {
                    let mut new_query = query.clone();
                    new_query.push(c);
                    self.popup_state = PopupState::SearchWithType(new_query, search_type);
                    Ok(())
                }
                KeyCode::Backspace => {
                    let mut new_query = query.clone();
                    new_query.pop();
                    self.popup_state = PopupState::SearchWithType(new_query, search_type);
                    Ok(())
                }
                _ => Ok(()),
            },
            PopupState::Message(_) => {
                if key_event.code == KeyCode::Esc {
                    self.popup_state = PopupState::None;
                }
                Ok(())
            }
            PopupState::SearchResults(results) => match key_event.code {
                KeyCode::Esc => {
                    self.popup_state = PopupState::None;
                    self.filtered_search_results.clear();
                    self.viewing_search_result = false;
                    Ok(())
                }
                KeyCode::Enter => {
                    if !results.is_empty() {
                        let (_, item) = &results[0];
                        match item {
                            SearchResultItem::Transaction(txn) => {
                                self.selected_transaction_id = Some(txn.id.clone());
                                self.viewing_search_result = true;
                                self.popup_state = PopupState::None;
                                self.show_transaction_details = true;
                            }
                            SearchResultItem::Block(block_info) => {
                                self.popup_state = PopupState::Message(format!(
                                    "Block #{}: {} - {} transactions",
                                    block_info.id, block_info.timestamp, block_info.txn_count
                                ));
                            }
                            SearchResultItem::Account(account_info) => {
                                self.popup_state = PopupState::Message(format!(
                                    "Account: {}\nBalance: {} microAlgos\nStatus: {}",
                                    account_info.address, account_info.balance, account_info.status
                                ));
                            }
                            SearchResultItem::Asset(asset_info) => {
                                self.popup_state = PopupState::Message(format!(
                                    "Asset #{}: {} ({})\nCreator: {}\nTotal: {}",
                                    asset_info.id,
                                    asset_info.name,
                                    asset_info.unit_name,
                                    asset_info.creator,
                                    asset_info.total
                                ));
                            }
                        }
                    }
                    Ok(())
                }
                KeyCode::Up => {
                    if results.len() > 1 {
                        let mut updated_results = results.clone();
                        let first = updated_results.remove(0);
                        updated_results.push(first);
                        self.popup_state = PopupState::SearchResults(updated_results);
                    }
                    Ok(())
                }
                KeyCode::Down => {
                    if results.len() > 1 {
                        let mut updated_results = results.clone();
                        let last = updated_results.pop().unwrap();
                        updated_results.insert(0, last);
                        self.popup_state = PopupState::SearchResults(updated_results);
                    }
                    Ok(())
                }
                _ => Ok(()),
            },
            PopupState::None => {
                if self.show_block_details || self.show_transaction_details {
                    match key_event.code {
                        KeyCode::Esc => {
                            self.show_block_details = false;
                            self.show_transaction_details = false;
                            self.viewing_search_result = false;
                            Ok(())
                        }
                        KeyCode::Char('c') => {
                            if self.show_transaction_details {
                                self.copy_transaction_id_to_clipboard();
                            }
                            Ok(())
                        }
                        _ => Ok(()),
                    }
                } else {
                    match key_event.code {
                        KeyCode::Char('q') => {
                            self.exit = true;
                            Ok(())
                        }
                        KeyCode::Char('r') => {
                            self.initial_data_fetch().await;
                            Ok(())
                        }
                        KeyCode::Char(' ') => {
                            self.show_live = !self.show_live;
                            let _ = self.live_updates_tx.send(self.show_live);
                            self.save_config();
                            Ok(())
                        }
                        KeyCode::Char('f') => {
                            self.popup_state =
                                PopupState::SearchWithType(String::new(), SearchType::Transaction);
                            Ok(())
                        }
                        KeyCode::Char('n') => {
                            let current_index = match self.network {
                                Network::MainNet => 0,
                                Network::TestNet => 1,
                                Network::LocalNet => 2,
                            };
                            self.popup_state = PopupState::NetworkSelect(current_index);
                            Ok(())
                        }
                        KeyCode::Tab => {
                            self.focus = match self.focus {
                                Focus::Blocks => Focus::Transactions,
                                Focus::Transactions => Focus::Sidebar,
                                Focus::Sidebar => Focus::Blocks,
                            };
                            Ok(())
                        }
                        KeyCode::Up => {
                            self.move_selection_up();
                            Ok(())
                        }
                        KeyCode::Down => {
                            self.move_selection_down();
                            Ok(())
                        }
                        KeyCode::Enter => {
                            self.show_details();
                            Ok(())
                        }
                        _ => Ok(()),
                    }
                }
            }
        }
    }

    async fn handle_mouse_input(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let popup_state = self.popup_state.clone();
                let has_popup = popup_state != PopupState::None;
                let popup_open =
                    self.show_block_details || self.show_transaction_details || has_popup;

                if popup_open {
                    if let PopupState::SearchWithType(query, current_type) = popup_state {
                        let row = mouse.row;
                        let selector_y = 9; // Estimated y position for the search type buttons

                        if row == selector_y {
                            let column = mouse.column;
                            let button_width = 12; // Estimated width of each button
                            let start_x = 15; // Estimated start x position of the first button

                            if column >= start_x && column < start_x + (4 * button_width) {
                                let button_index = (column - start_x) / button_width;
                                let new_type = match button_index {
                                    0 => SearchType::Transaction,
                                    1 => SearchType::Block,
                                    2 => SearchType::Account,
                                    3 => SearchType::Asset,
                                    _ => current_type,
                                };

                                self.popup_state = PopupState::SearchWithType(query, new_type);
                                return Ok(());
                            }
                        }
                    } else if self.show_transaction_details {
                        let row = mouse.row;
                        let button_y_range = (20, 23); // Estimated position of the button
                        let button_x_range = (33, 47); // Estimated position of the button

                        if row >= button_y_range.0
                            && row <= button_y_range.1
                            && mouse.column >= button_x_range.0
                            && mouse.column <= button_x_range.1
                        {
                            self.copy_transaction_id_to_clipboard();
                            return Ok(());
                        }
                    }
                    return Ok(());
                }

                let header_height = 3; // App header
                let title_height = 3; // Section title

                if mouse.row <= (header_height + title_height) {
                    return Ok(());
                }

                let content_start_row = header_height + title_height;
                let content_row = mouse.row.saturating_sub(content_start_row);

                let terminal_width = 100; // Approximate terminal width, could be dynamic
                let is_left_half = mouse.column <= terminal_width / 2;

                if is_left_half {
                    self.focus = Focus::Blocks;
                    let block_index = (content_row / 3) as usize; // Each block takes 3 rows
                    if block_index < self.blocks.len() {
                        self.selected_block_index = Some(block_index);
                        self.selected_block_id = self.blocks.get(block_index).map(|b| b.id);
                    }
                } else {
                    self.focus = Focus::Transactions;
                    let txn_index = (content_row / 4) as usize; // Each transaction takes 4 rows
                    if txn_index < self.transactions.len() {
                        self.selected_transaction_index = Some(txn_index);
                        self.selected_transaction_id =
                            self.transactions.get(txn_index).map(|t| t.id.clone());
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn sync_selections(&mut self) {
        if let Some(block_id) = self.selected_block_id {
            if let Some(index) = self.blocks.iter().position(|b| b.id == block_id) {
                self.selected_block_index = Some(index);
            } else {
                self.selected_block_index = None;
                self.selected_block_id = None;
            }
        }

        if let Some(ref txn_id) = self.selected_transaction_id {
            if let Some(index) = self.transactions.iter().position(|t| t.id == *txn_id) {
                self.selected_transaction_index = Some(index);
            } else {
                self.selected_transaction_index = None;
                self.selected_transaction_id = None;
            }
        }
    }

    async fn search_transactions(&mut self, query: &str, search_type: SearchType) {
        if query.is_empty() {
            self.popup_state = PopupState::Message("Please enter a search term".to_string());
            return;
        }

        let search_type_str = match search_type {
            SearchType::Transaction => "transactions",
            SearchType::Asset => "assets",
            SearchType::Account => "accounts",
            SearchType::Block => "blocks",
        };

        self.popup_state = PopupState::Message(format!(
            "Querying Algorand network APIs for {}...",
            search_type_str
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

    async fn switch_network(&mut self, network: Network) {
        // Show immediate feedback
        self.popup_state = PopupState::Message(format!(
            "Switching to {}... Clearing data and reconnecting.",
            network.as_str()
        ));

        self.network = network;
        self.client = AlgoClient::new(network);

        let _ = self.network_tx.send(network);

        // Save configuration
        self.save_config();

        // Clear all existing data
        self.blocks.clear();
        self.transactions.clear();
        self.block_scroll = 0;
        self.transaction_scroll = 0;
        self.selected_block_index = None;
        self.selected_transaction_index = None;
        self.selected_block_id = None;
        self.selected_transaction_id = None;
        self.filtered_search_results.clear();
        self.viewing_search_result = false;

        // Start initial data fetch
        self.initial_data_fetch().await;

        // Show success message briefly
        tokio::spawn({
            let message_tx = self.message_tx.clone();
            async move {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                let success_message = AppMessage::NetworkSwitchComplete;
                let _ = message_tx.send(success_message);
            }
        });
    }

    fn save_config(&self) {
        let config = AppConfig {
            network: self.network,
            show_live: self.show_live,
        };
        if let Err(e) = config.save() {
            // Log error but don't crash the application
            eprintln!("Failed to save configuration: {}", e);
        }
    }

    pub fn move_selection_up(&mut self) {
        match self.focus {
            Focus::Blocks => {
                if let Some(index) = self.selected_block_index {
                    if index > 0 {
                        let new_index = index - 1;
                        self.selected_block_index = Some(new_index);
                        self.selected_block_id = self.blocks.get(new_index).map(|b| b.id);

                        let block_height = 3;
                        let block_scroll = new_index as u16 * block_height;
                        if block_scroll < self.block_scroll {
                            self.block_scroll = block_scroll;
                        }
                    }
                } else if !self.blocks.is_empty() {
                    self.selected_block_index = Some(0);
                    self.selected_block_id = self.blocks.first().map(|b| b.id);
                    self.block_scroll = 0;
                }
            }
            Focus::Transactions => {
                if let Some(index) = self.selected_transaction_index {
                    if index > 0 {
                        let new_index = index - 1;
                        self.selected_transaction_index = Some(new_index);
                        self.selected_transaction_id =
                            self.transactions.get(new_index).map(|t| t.id.clone());

                        let txn_height = 4;
                        let txn_scroll = new_index as u16 * txn_height;
                        if txn_scroll < self.transaction_scroll {
                            self.transaction_scroll = txn_scroll;
                        }
                    }
                } else if !self.transactions.is_empty() {
                    self.selected_transaction_index = Some(0);
                    self.selected_transaction_id = self.transactions.first().map(|t| t.id.clone());
                    self.transaction_scroll = 0;
                }
            }
            _ => {}
        }
    }

    pub fn move_selection_down(&mut self) {
        match self.focus {
            Focus::Blocks => {
                let max_index = self.blocks.len().saturating_sub(1);

                if let Some(index) = self.selected_block_index {
                    if index < max_index {
                        let new_index = index + 1;
                        self.selected_block_index = Some(new_index);
                        self.selected_block_id = self.blocks.get(new_index).map(|b| b.id);

                        let block_height = 3;
                        let block_display_height = 10; // Approximate visible blocks
                        let visible_end = self.block_scroll + (block_display_height * block_height);
                        let item_position = (new_index as u16) * block_height;

                        if item_position >= visible_end {
                            self.block_scroll = self.block_scroll.saturating_add(block_height);
                        }
                    }
                } else if !self.blocks.is_empty() {
                    self.selected_block_index = Some(0);
                    self.selected_block_id = self.blocks.first().map(|b| b.id);
                }
            }
            Focus::Transactions => {
                let max_index = self.transactions.len().saturating_sub(1);

                if let Some(index) = self.selected_transaction_index {
                    if index < max_index {
                        let new_index = index + 1;
                        self.selected_transaction_index = Some(new_index);
                        self.selected_transaction_id =
                            self.transactions.get(new_index).map(|t| t.id.clone());

                        let txn_height = 4;
                        let txn_display_height = 10; // Approximate visible transactions
                        let visible_end =
                            self.transaction_scroll + (txn_display_height * txn_height);
                        let item_position = (new_index as u16) * txn_height;

                        if item_position >= visible_end {
                            self.transaction_scroll =
                                self.transaction_scroll.saturating_add(txn_height);
                        }
                    }
                } else if !self.transactions.is_empty() {
                    self.selected_transaction_index = Some(0);
                    self.selected_transaction_id = self.transactions.first().map(|t| t.id.clone());
                }
            }
            _ => {}
        }
    }

    pub fn show_details(&mut self) {
        match self.focus {
            Focus::Blocks => {
                if self.selected_block_index.is_some() {
                    self.show_block_details = true;
                }
            }
            Focus::Transactions => {
                if self.selected_transaction_index.is_some() {
                    self.show_transaction_details = true;
                }
            }
            _ => {}
        }
    }

    fn copy_transaction_id_to_clipboard(&mut self) {
        if let Some(ref txn_id) = self.selected_transaction_id {
            match Clipboard::new() {
                Ok(mut clipboard) => {
                    if clipboard.set_text(txn_id.clone()).is_ok() {
                        self.popup_state =
                            PopupState::Message("Transaction ID copied to clipboard!".to_string());
                    } else {
                        self.popup_state =
                            PopupState::Message("Failed to copy to clipboard".to_string());
                    }
                }
                Err(_) => {
                    self.popup_state = PopupState::Message("Clipboard not available".to_string());
                }
            }
        }
    }
}
