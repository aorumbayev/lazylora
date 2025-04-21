use arboard::Clipboard;
use color_eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::algorand::{AlgoBlock, AlgoClient, Network, SearchResultItem, Transaction};
use crate::ui;

// Global state for storing pending popup messages
lazy_static! {
    static ref PENDING_POPUPS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

/// Focus area in the application
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Blocks,
    Transactions,
    Sidebar,
}

/// State for popups
#[derive(Debug, Clone, PartialEq)]
pub enum PopupState {
    None,
    NetworkSelect(usize),               // Index of the selected network
    SearchWithType(String, SearchType), // Search query with explicit search type
    Message(String),                    // A message to display to the user
    SearchResults(Vec<(usize, SearchResultItem)>), // Search results with original indices
}

/// Search type for explicit search selector
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

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    pub network: Network,
    pub blocks: Arc<Mutex<Vec<AlgoBlock>>>,
    pub transactions: Arc<Mutex<Vec<Transaction>>>,
    pub show_live: Arc<Mutex<bool>>,
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
    runtime: tokio::runtime::Runtime,
    client: AlgoClient,
    current_client: Arc<Mutex<AlgoClient>>,
}

impl App {
    /// Create a new application instance with default state
    pub fn new() -> Self {
        let blocks = Arc::new(Mutex::new(Vec::new()));
        let transactions = Arc::new(Mutex::new(Vec::new()));

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let network = Network::MainNet;
        let client = AlgoClient::new(network);
        let current_client = Arc::new(Mutex::new(client.clone()));

        Self {
            network,
            blocks,
            transactions,
            show_live: Arc::new(Mutex::new(true)),
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
            runtime,
            client,
            current_client,
        }
    }

    /// Run the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut crate::tui::Tui) -> Result<()> {
        self.start_data_fetching();
        self.initial_data_fetch();

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        while !self.exit {
            // Check for any pending popup messages
            if let Ok(mut pending_popups) = PENDING_POPUPS.lock() {
                if !pending_popups.is_empty() && self.popup_state == PopupState::None {
                    let msg = pending_popups.remove(0);
                    self.popup_state = PopupState::Message(msg);
                }
            }

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));

            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            self.handle_key_event(key)?;
                        }
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_input(mouse)?;
                    }
                    Event::Resize(_, _) => {
                        // Immediate redraw on terminal resize to update UI
                        terminal.draw(|frame| ui::render(self, frame))?;
                    }
                    _ => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                // Sync selected indexes with IDs before rendering
                self.sync_selections();

                terminal.draw(|frame| ui::render(self, frame))?;
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
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
                    self.switch_network(new_network);
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
                    self.search_transactions(&query_clone, search_type);
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
                        // Get the first (selected) result
                        let (_, item) = &results[0];

                        // Handle based on entity type
                        match item {
                            SearchResultItem::Transaction(txn) => {
                                // Store transaction ID and set flag to view details
                                self.selected_transaction_id = Some(txn.id.clone());
                                self.viewing_search_result = true;
                                self.popup_state = PopupState::None;
                                self.show_transaction_details = true;
                            }
                            SearchResultItem::Block(block_info) => {
                                // Display block info message
                                self.popup_state = PopupState::Message(format!(
                                    "Block #{}: {} - {} transactions",
                                    block_info.id, block_info.timestamp, block_info.txn_count
                                ));
                            }
                            SearchResultItem::Account(account_info) => {
                                // Display account info message
                                self.popup_state = PopupState::Message(format!(
                                    "Account: {}\nBalance: {} microAlgos\nStatus: {}",
                                    account_info.address, account_info.balance, account_info.status
                                ));
                            }
                            SearchResultItem::Asset(asset_info) => {
                                // Display asset info message
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
                        // Take the first item and move it to the end
                        let mut updated_results = results.clone();
                        let first = updated_results.remove(0);
                        updated_results.push(first);
                        self.popup_state = PopupState::SearchResults(updated_results);
                    }
                    Ok(())
                }
                KeyCode::Down => {
                    if results.len() > 1 {
                        // Take the last item and move it to the front
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
                            self.initial_data_fetch();
                            Ok(())
                        }
                        KeyCode::Char(' ') => {
                            let mut show_live = self.show_live.lock().unwrap();
                            let was_off = !*show_live;
                            *show_live = !*show_live;

                            // If we're turning live updates on, verify network connectivity
                            if was_off && *show_live {
                                // Update client to use current network
                                self.client = AlgoClient::new(self.network);

                                // Update the shared client used by the polling thread
                                let mut current_client = self.current_client.lock().unwrap();
                                *current_client = self.client.clone();

                                // Clone what we need for the async check
                                let runtime = self.runtime.handle().clone();
                                let client = self.client.clone();

                                // Launch a check in the background
                                thread::spawn(move || {
                                    runtime.block_on(async {
                                        // Check network status
                                        match client.get_network_status().await {
                                            Err(error_msg) => {
                                                // Store error message for the main thread to display
                                                if let Ok(mut pending_popups) =
                                                    PENDING_POPUPS.lock()
                                                {
                                                    pending_popups.push(error_msg);
                                                }
                                            }
                                            Ok(_) => {
                                                // Network is available, no need to do anything
                                            }
                                        }
                                    });
                                });

                                // Also trigger a fresh data fetch with the current network
                                self.initial_data_fetch();
                            }

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

    fn handle_mouse_input(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let popup_state = self.popup_state.clone();
                let has_popup = popup_state != PopupState::None;
                let popup_open =
                    self.show_block_details || self.show_transaction_details || has_popup;

                if popup_open {
                    if let PopupState::SearchResults(_results) = popup_state {
                        // TODO: Handle mouse clicks on search results properly based on item type.
                        // For now, disable the direct click action on results.
                        // if !results.is_empty() {
                        //     let (_, item) = &results[0];
                        //     // Handle click based on item type...
                        //     // This logic needs to be similar to the Enter key press handler
                        //     return Ok(());
                        // }
                    } else if let PopupState::SearchWithType(query, current_type) = popup_state {
                        // Check if click is on a search type button
                        // These are positioned horizontally in the UI at selector_y = input_area.y + 4
                        let row = mouse.row;
                        let selector_y = 9; // Estimated y position for the search type buttons

                        if row == selector_y {
                            // Determine which button was clicked based on x coordinate
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
                        // Check if click is on copy button
                        // The button is positioned in the UI at:
                        // y: inner_area.y + inner_area.height - button_height - 2
                        // Height: 3
                        // We'll use approximate values based on the UI
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

                // Updated mouse click handler to use row-based calculation
                let header_height = 3; // App header
                let title_height = 3; // Section title

                if mouse.row <= (header_height + title_height) {
                    return Ok(()); // Ignore clicks in the header/title area
                }

                // Determine if click is in left or right panel
                let column_percent = (mouse.column as f32 / 100.0) * 100.0;

                if column_percent < 50.0 {
                    // Left panel - Blocks
                    self.focus = Focus::Blocks;

                    // Calculate which block was clicked
                    let blocks_area_row = mouse.row - header_height - title_height;
                    let visible_index = blocks_area_row / BLOCK_HEIGHT;
                    let scroll_offset = self.block_scroll as usize / BLOCK_HEIGHT as usize;
                    let absolute_index = scroll_offset + visible_index as usize;

                    let blocks = self.blocks.lock().unwrap();
                    if absolute_index < blocks.len() {
                        self.selected_block_index = Some(absolute_index);
                        self.show_block_details = true;
                    }
                } else {
                    // Right panel - Transactions
                    self.focus = Focus::Transactions;

                    // Calculate which transaction was clicked
                    let txns_area_row = mouse.row - header_height - title_height;
                    let visible_index = txns_area_row / TXN_HEIGHT;
                    let scroll_offset = self.transaction_scroll as usize / TXN_HEIGHT as usize;
                    let absolute_index = scroll_offset + visible_index as usize;

                    let transactions = self.transactions.lock().unwrap();
                    if absolute_index < transactions.len() {
                        self.selected_transaction_index = Some(absolute_index);
                        self.show_transaction_details = true;
                    }
                }
            }
            MouseEventKind::ScrollDown => self.handle_scroll_down(),
            MouseEventKind::ScrollUp => self.handle_scroll_up(),
            _ => {}
        }
        Ok(())
    }

    fn handle_scroll_down(&mut self) {
        match self.focus {
            Focus::Blocks => {
                let blocks = self.blocks.lock().unwrap();
                let block_height = 3;
                let max_scroll = blocks.len().saturating_sub(1) as u16 * block_height;

                self.block_scroll = self.block_scroll.saturating_add(block_height);
                if self.block_scroll > max_scroll {
                    self.block_scroll = max_scroll;
                }
            }
            Focus::Transactions => {
                let transactions = self.transactions.lock().unwrap();
                let txn_height = 4;
                let max_scroll = transactions.len().saturating_sub(1) as u16 * txn_height;

                self.transaction_scroll = self.transaction_scroll.saturating_add(txn_height);
                if self.transaction_scroll > max_scroll {
                    self.transaction_scroll = max_scroll;
                }
            }
            _ => {}
        }
    }

    fn handle_scroll_up(&mut self) {
        match self.focus {
            Focus::Blocks => {
                let block_height = 3;
                self.block_scroll = self.block_scroll.saturating_sub(block_height);
            }
            Focus::Transactions => {
                let txn_height = 4;
                self.transaction_scroll = self.transaction_scroll.saturating_sub(txn_height);
            }
            _ => {}
        }
    }

    /// Sync the selected indexes with their corresponding IDs
    fn sync_selections(&mut self) {
        // Sync block selection
        if let Some(block_id) = self.selected_block_id {
            let blocks = self.blocks.lock().unwrap();
            if let Some((index, _)) = blocks.iter().enumerate().find(|(_, b)| b.id == block_id) {
                self.selected_block_index = Some(index);
            } else if !blocks.is_empty() {
                // If the block with the ID is not found, select the first one
                self.selected_block_index = Some(0);
                self.selected_block_id = blocks.first().map(|b| b.id);
            } else {
                self.selected_block_index = None;
                self.selected_block_id = None;
            }
        }

        // Sync transaction selection only if not viewing a search result
        if !self.viewing_search_result {
            if let Some(txn_id) = self.selected_transaction_id.clone() {
                let transactions = self.transactions.lock().unwrap();
                if let Some((index, _)) = transactions
                    .iter()
                    .enumerate()
                    .find(|(_, t)| t.id == txn_id)
                {
                    self.selected_transaction_index = Some(index);
                } else if !transactions.is_empty() {
                    // If the transaction with the ID is not found, select the first one
                    self.selected_transaction_index = Some(0);
                    self.selected_transaction_id = transactions.first().map(|t| t.id.clone());
                } else {
                    self.selected_transaction_index = None;
                    self.selected_transaction_id = None;
                }
            }
        }
    }

    fn initial_data_fetch(&self) {
        let runtime = self.runtime.handle().clone();
        let blocks_clone = Arc::clone(&self.blocks);
        let txns_clone = Arc::clone(&self.transactions);
        let client = self.client.clone();
        let network = self.network;
        let show_live = Arc::clone(&self.show_live);

        // Create a channel to receive errors
        let (tx, rx) = mpsc::channel::<Option<String>>();

        thread::spawn(move || {
            runtime.block_on(async {
                // First check if network is available with detailed error message
                match client.get_network_status().await {
                    Err(error_msg) => {
                        let _ = tx.send(Some(error_msg));
                        return;
                    }
                    Ok(()) => {
                        // Network is available, continue
                    }
                }

                // Then attempt to fetch data
                match client.get_latest_blocks(5).await {
                    Ok(new_blocks) => {
                        let mut blocks = blocks_clone.lock().unwrap();
                        *blocks = new_blocks;
                    }
                    Err(e) => {
                        let _ = tx.send(Some(format!("Failed to fetch blocks: {}", e)));
                        return;
                    }
                }

                match client.get_latest_transactions(5).await {
                    Ok(new_txns) => {
                        let mut txns = txns_clone.lock().unwrap();
                        *txns = new_txns;
                    }
                    Err(e) => {
                        let _ = tx.send(Some(format!("Failed to fetch transactions: {}", e)));
                        return;
                    }
                }

                // Signal success
                let _ = tx.send(None);
            });
        });

        // Handle response in the same thread
        std::thread::spawn(move || {
            // Wait for a response with timeout
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(Some(error_msg)) => {
                    // If we got an error, update shared state
                    *show_live.lock().unwrap() = false;

                    // Store the receiver in a global/static location to be checked
                    // during the next UI update cycle
                    if let Ok(mut pending_popups) = PENDING_POPUPS.lock() {
                        pending_popups.push(error_msg);
                    }
                }
                Ok(None) => {
                    // Success, do nothing
                }
                Err(_) => {
                    // Timeout occurred
                    let timeout_msg = format!(
                        "Connection to {} timed out. Please check your network.",
                        network.as_str()
                    );
                    *show_live.lock().unwrap() = false;

                    // Store the message for the main thread to pick up
                    if let Ok(mut pending_popups) = PENDING_POPUPS.lock() {
                        pending_popups.push(timeout_msg);
                    }
                }
            }
        });
    }

    fn start_data_fetching(&self) {
        let blocks_clone = Arc::clone(&self.blocks);
        let txns_clone = Arc::clone(&self.transactions);
        let current_client = Arc::clone(&self.current_client);
        let runtime = self.runtime.handle().clone();
        let show_live = Arc::clone(&self.show_live);

        // These will be used to track selected items by IDs
        let selected_block_id = Arc::new(Mutex::new(None::<u64>));
        let selected_txn_id = Arc::new(Mutex::new(None::<String>));

        thread::spawn(move || {
            let mut last_txn_fetch = Instant::now();
            let mut last_block_fetch = Instant::now();
            let mut last_network_check = Instant::now();
            let mut is_network_available = true;
            let mut network_error_shown = false;

            let block_interval = Duration::from_secs(5);
            let txn_interval = Duration::from_secs(5);
            let network_check_interval = Duration::from_secs(10); // Check network every 10 seconds

            loop {
                // Don't do anything if live updates are turned off
                if !*show_live.lock().unwrap() {
                    // Reset error flags when live updates are off
                    network_error_shown = false;
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }

                // Always get the latest client for every operation
                let client = current_client.lock().unwrap().clone();

                let now = Instant::now();

                // Periodically check network status
                if now.duration_since(last_network_check) >= network_check_interval {
                    last_network_check = now;

                    // Get detailed connection status
                    match runtime.block_on(async { client.get_network_status().await }) {
                        Ok(()) => {
                            is_network_available = true;
                            network_error_shown = false;
                        }
                        Err(error_msg) => {
                            // If this is the first error, show it
                            if !network_error_shown {
                                if let Ok(mut pending_popups) = PENDING_POPUPS.lock() {
                                    pending_popups.push(error_msg);
                                }
                                network_error_shown = true;
                            }
                            is_network_available = false;
                            thread::sleep(Duration::from_secs(1));
                            continue;
                        }
                    }
                }

                // Only try to fetch data if we believe the network is available
                if is_network_available {
                    if now.duration_since(last_block_fetch) >= block_interval {
                        last_block_fetch = now;

                        let blocks_clone = Arc::clone(&blocks_clone);
                        let selected_block_id_clone = Arc::clone(&selected_block_id);
                        let client_clone = client.clone();

                        runtime.block_on(async {
                            if let Ok(new_blocks) = client_clone.get_latest_blocks(5).await {
                                if !new_blocks.is_empty() {
                                    let mut blocks = blocks_clone.lock().unwrap();
                                    let mut selected_id = selected_block_id_clone.lock().unwrap();

                                    // Save the currently selected block ID if any
                                    if let Some(index) =
                                        blocks.iter().position(|b| *selected_id == Some(b.id))
                                    {
                                        *selected_id = Some(blocks[index].id);
                                    }

                                    let block_map: HashMap<u64, usize> = blocks
                                        .iter()
                                        .enumerate()
                                        .map(|(i, block)| (block.id, i))
                                        .collect();

                                    for new_block in new_blocks {
                                        if !block_map.contains_key(&new_block.id) {
                                            let pos =
                                                blocks.partition_point(|b| b.id > new_block.id);
                                            blocks.insert(pos, new_block);
                                        }
                                    }

                                    if blocks.len() > 100 {
                                        blocks.truncate(100);
                                    }
                                }
                            } else {
                                // Failed to fetch blocks, check network again soon
                                last_network_check = Instant::now()
                                    .checked_sub(network_check_interval * 2)
                                    .unwrap_or_else(Instant::now);
                            }
                        });
                    }

                    if now.duration_since(last_txn_fetch) >= txn_interval {
                        last_txn_fetch = now;

                        let txns_clone = Arc::clone(&txns_clone);
                        let selected_txn_id_clone = Arc::clone(&selected_txn_id);
                        let client_clone = client.clone();

                        runtime.block_on(async {
                            if let Ok(new_txns) = client_clone.get_latest_transactions(5).await {
                                if !new_txns.is_empty() {
                                    let mut txns = txns_clone.lock().unwrap();
                                    let mut selected_id = selected_txn_id_clone.lock().unwrap();

                                    // Save the currently selected transaction ID if any
                                    if let Some(index) =
                                        txns.iter().position(|t| *selected_id == Some(t.id.clone()))
                                    {
                                        *selected_id = Some(txns[index].id.clone());
                                    }

                                    let txn_ids: HashSet<String> =
                                        txns.iter().map(|txn| txn.id.clone()).collect();

                                    let mut updated_txns = Vec::with_capacity(100);

                                    for new_txn in new_txns {
                                        if !txn_ids.contains(&new_txn.id) {
                                            updated_txns.push(new_txn);
                                        }
                                    }

                                    for old_txn in txns.iter().cloned() {
                                        if updated_txns.len() >= 100 {
                                            break;
                                        }
                                        if !updated_txns.iter().any(|t| t.id == old_txn.id) {
                                            updated_txns.push(old_txn);
                                        }
                                    }

                                    *txns = updated_txns;
                                }
                            } else {
                                // Failed to fetch transactions, check network again soon
                                last_network_check = Instant::now()
                                    .checked_sub(network_check_interval * 2)
                                    .unwrap_or_else(Instant::now);
                            }
                        });
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn search_transactions(&mut self, query: &str, search_type: SearchType) {
        if query.is_empty() {
            self.popup_state = PopupState::Message("Please enter a search term".to_string());
            return;
        }

        let search_query = query.trim();

        // Show a loading message that explains we're searching the network
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

        // Clone the client and runtime handle for the async operation
        let client = self.client.clone();
        let runtime = self.runtime.handle().clone();
        let query_clone = search_query.to_string();
        let network = self.network;

        // Create a channel to receive the search results from the async operation
        let (tx, rx) = mpsc::channel::<Result<Vec<SearchResultItem>>>();

        // Spawn a new thread to perform the search asynchronously
        thread::spawn(move || {
            runtime.block_on(async {
                // First check if the network is available
                if !client.is_network_available().await {
                    let error = color_eyre::eyre::eyre!(
                        "{} is not available. Please check your connection.",
                        network.as_str()
                    );
                    let _ = tx.send(Err(error));
                    return;
                }

                match client.search_by_query(&query_clone, search_type).await {
                    Ok(items) => {
                        // Send the search results back through the channel
                        let _ = tx.send(Ok(items));
                    }
                    Err(e) => {
                        // Send the error back through the channel
                        let _ = tx.send(Err(e));
                    }
                }
            });
        });

        // Wait for the search operation to complete (with a timeout)
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(items)) => {
                if items.is_empty() {
                    let search_type_str = match search_type {
                        SearchType::Transaction => "transactions",
                        SearchType::Asset => "assets",
                        SearchType::Account => "accounts",
                        SearchType::Block => "blocks",
                    };

                    self.popup_state = PopupState::Message(format!(
                        "No matching data found in {}",
                        search_type_str
                    ));
                } else {
                    // Create result entries with index positions
                    let results_with_indices: Vec<(usize, SearchResultItem)> =
                        items.into_iter().enumerate().collect();

                    self.filtered_search_results = results_with_indices.clone();
                    self.popup_state = PopupState::SearchResults(results_with_indices);
                }
            }
            Ok(Err(e)) => {
                // Display the error as a user-friendly message
                let error_msg = e.to_string();
                self.popup_state = PopupState::Message(format!(
                    "Error querying the Algorand network: {}",
                    error_msg
                ));
            }
            Err(_) => {
                self.popup_state = PopupState::Message(format!(
                    "Search timed out. Please check your connection to {}.",
                    network.as_str()
                ));
            }
        }
    }

    fn switch_network(&mut self, network: Network) {
        self.network = network;
        self.client = AlgoClient::new(network);

        // Update the shared client used by the polling thread
        let mut current_client = self.current_client.lock().unwrap();
        *current_client = self.client.clone();

        // Clear existing data
        {
            let mut blocks = self.blocks.lock().unwrap();
            blocks.clear();
        }
        {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.clear();
        }

        self.block_scroll = 0;
        self.transaction_scroll = 0;
        self.selected_block_index = None;
        self.selected_transaction_index = None;
        self.selected_block_id = None;
        self.selected_transaction_id = None;
        self.filtered_search_results.clear();
        self.viewing_search_result = false;

        // Check if network is available
        let runtime = self.runtime.handle().clone();
        let client = self.client.clone();

        // Create a channel to receive the result
        let (tx, rx) = mpsc::channel::<Result<(), String>>();

        thread::spawn(move || {
            runtime.block_on(async {
                // Try to connect to the network
                let status = client.get_network_status().await;
                let _ = tx.send(status);
            });
        });

        // Wait for the result with a short timeout
        match rx.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => {
                // Network is available, fetch data
                self.initial_data_fetch();
                // Ensure live updates are enabled for the new network
                *self.show_live.lock().unwrap() = true;
            }
            Ok(Err(error_msg)) => {
                // Network is not available, show specific error message
                self.popup_state = PopupState::Message(error_msg);
                // Force show_live to false to prevent continuous polling
                *self.show_live.lock().unwrap() = false;
            }
            Err(_) => {
                // Timeout occurred
                self.popup_state = PopupState::Message(format!(
                    "Connection to {} timed out. Please check your network connection.",
                    network.as_str()
                ));
                // Force show_live to false to prevent continuous polling
                *self.show_live.lock().unwrap() = false;
            }
        }
    }

    pub fn move_selection_up(&mut self) {
        match self.focus {
            Focus::Blocks => {
                if let Some(index) = self.selected_block_index {
                    if index > 0 {
                        let blocks = self.blocks.lock().unwrap();
                        let new_index = index - 1;
                        self.selected_block_index = Some(new_index);
                        self.selected_block_id = blocks.get(new_index).map(|b| b.id);

                        let block_height = 3;
                        let block_scroll = new_index as u16 * block_height;
                        if block_scroll < self.block_scroll {
                            self.block_scroll = block_scroll;
                        }
                    }
                } else {
                    let blocks = self.blocks.lock().unwrap();
                    if !blocks.is_empty() {
                        self.selected_block_index = Some(0);
                        self.selected_block_id = blocks.first().map(|b| b.id);
                        self.block_scroll = 0;
                    }
                }
            }
            Focus::Transactions => {
                if let Some(index) = self.selected_transaction_index {
                    if index > 0 {
                        let transactions = self.transactions.lock().unwrap();
                        let new_index = index - 1;
                        self.selected_transaction_index = Some(new_index);
                        self.selected_transaction_id =
                            transactions.get(new_index).map(|t| t.id.clone());

                        let txn_height = 4;
                        let txn_scroll = new_index as u16 * txn_height;
                        if txn_scroll < self.transaction_scroll {
                            self.transaction_scroll = txn_scroll;
                        }
                    }
                } else {
                    let transactions = self.transactions.lock().unwrap();
                    if !transactions.is_empty() {
                        self.selected_transaction_index = Some(0);
                        self.selected_transaction_id = transactions.first().map(|t| t.id.clone());
                        self.transaction_scroll = 0;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn move_selection_down(&mut self) {
        match self.focus {
            Focus::Blocks => {
                let blocks = self.blocks.lock().unwrap();
                let max_index = blocks.len().saturating_sub(1);

                if let Some(index) = self.selected_block_index {
                    if index < max_index {
                        let new_index = index + 1;
                        self.selected_block_index = Some(new_index);
                        self.selected_block_id = blocks.get(new_index).map(|b| b.id);

                        let block_height = 3;
                        let block_display_height = 10; // Approximate visible blocks
                        let visible_end = self.block_scroll + (block_display_height * block_height);
                        let item_position = (new_index as u16) * block_height;

                        if item_position >= visible_end {
                            self.block_scroll = self.block_scroll.saturating_add(block_height);
                        }
                    }
                } else if !blocks.is_empty() {
                    self.selected_block_index = Some(0);
                    self.selected_block_id = blocks.first().map(|b| b.id);
                }
            }
            Focus::Transactions => {
                let transactions = self.transactions.lock().unwrap();
                let max_index = transactions.len().saturating_sub(1);

                if let Some(index) = self.selected_transaction_index {
                    if index < max_index {
                        let new_index = index + 1;
                        self.selected_transaction_index = Some(new_index);
                        self.selected_transaction_id =
                            transactions.get(new_index).map(|t| t.id.clone());

                        let txn_height = 4;
                        let txn_display_height = 7; // Approximate visible transactions
                        let visible_end =
                            self.transaction_scroll + (txn_display_height * txn_height);
                        let item_position = (new_index as u16) * txn_height;

                        if item_position >= visible_end {
                            self.transaction_scroll =
                                self.transaction_scroll.saturating_add(txn_height);
                        }
                    }
                } else if !transactions.is_empty() {
                    self.selected_transaction_index = Some(0);
                    self.selected_transaction_id = transactions.first().map(|t| t.id.clone());
                }
            }
            _ => {}
        }
    }

    pub fn show_details(&mut self) {
        match self.focus {
            Focus::Blocks => {
                if self.selected_block_index.is_some() || self.selected_block_id.is_some() {
                    self.show_block_details = true;
                }
            }
            Focus::Transactions => {
                if self.selected_transaction_index.is_some()
                    || (self.selected_transaction_id.is_some() && !self.viewing_search_result)
                {
                    self.show_transaction_details = true;
                    self.viewing_search_result = false; // Ensure we're viewing from main list
                }
            }
            _ => {}
        }
    }

    fn copy_transaction_id_to_clipboard(&mut self) {
        // First check if we have a transaction ID selected
        if let Some(txn_id) = &self.selected_transaction_id {
            // Get the transaction from the appropriate source
            let transaction = if self.viewing_search_result {
                // If viewing a search result, find it in filtered_search_results
                self.filtered_search_results.iter().find_map(|(_, item)| {
                    if let SearchResultItem::Transaction(t) = item {
                        if t.id == *txn_id {
                            Some(t.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            } else {
                // Otherwise get it from the main transaction list
                let txns = self.transactions.lock().unwrap();
                txns.iter().find(|t| t.id == *txn_id).cloned()
            };

            if let Some(txn) = transaction {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(&txn.id).is_err() {
                        self.popup_state =
                            PopupState::Message("Failed to copy to clipboard".to_string());
                    } else {
                        self.popup_state =
                            PopupState::Message("Transaction ID copied to clipboard".to_string());
                    }
                } else {
                    self.popup_state = PopupState::Message("Clipboard not available".to_string());
                }
            }
        } else if let Some(index) = self.selected_transaction_index {
            // Fallback to old behavior using index (only for main transaction list)
            let transactions = self.transactions.lock().unwrap();
            if let Some(txn) = transactions.get(index) {
                if let Ok(mut clipboard) = Clipboard::new() {
                    if clipboard.set_text(&txn.id).is_err() {
                        self.popup_state =
                            PopupState::Message("Failed to copy to clipboard".to_string());
                    } else {
                        self.popup_state =
                            PopupState::Message("Transaction ID copied to clipboard".to_string());
                    }
                } else {
                    self.popup_state = PopupState::Message("Clipboard not available".to_string());
                }
            }
        }
    }
}

// Add constants for UI element heights
const BLOCK_HEIGHT: u16 = 3;
const TXN_HEIGHT: u16 = 4;
