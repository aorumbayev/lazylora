use arboard::Clipboard;
use color_eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::algorand::{AlgoBlock, AlgoClient, Network, Transaction};
use crate::ui;

/// Focus area in the application
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Blocks,
    Transactions,
    Sidebar,
}

/// Search type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchType {
    All,
    TxnID,
    AssetID,
    Address,
    Block,
}

impl SearchType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::All => "All",
            Self::TxnID => "Transaction ID",
            Self::AssetID => "Asset ID",
            Self::Address => "Address",
            Self::Block => "Block",
        }
    }

    pub fn cycle_next(&self) -> Self {
        match self {
            Self::All => Self::TxnID,
            Self::TxnID => Self::AssetID,
            Self::AssetID => Self::Address,
            Self::Address => Self::Block,
            Self::Block => Self::All,
        }
    }

    pub fn cycle_prev(&self) -> Self {
        match self {
            Self::All => Self::Block,
            Self::TxnID => Self::All,
            Self::AssetID => Self::TxnID,
            Self::Address => Self::AssetID,
            Self::Block => Self::Address,
        }
    }
}

/// State for popups
#[derive(Debug, Clone, PartialEq)]
pub enum PopupState {
    None,
    NetworkSelect(usize),                     // Index of the selected network
    Search(String, SearchType),               // Current search query and type
    Message(String),                          // A message to display to the user
    SearchResults(Vec<(usize, Transaction)>), // Search results with original indices
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
    pub filtered_transactions: Vec<(usize, Transaction)>,
    runtime: tokio::runtime::Runtime,
    client: AlgoClient,
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
            filtered_transactions: Vec::new(),
            runtime,
            client,
        }
    }

    /// Run the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut crate::tui::Tui) -> Result<()> {
        self.start_data_fetching();
        self.initial_data_fetch();

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        while !self.exit {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key)?;
                    }
                } else if let Event::Mouse(mouse) = event::read()? {
                    self.handle_mouse_input(mouse)?;
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

        // Sync transaction selection
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

    fn initial_data_fetch(&self) {
        let runtime = self.runtime.handle().clone();
        let blocks_clone = Arc::clone(&self.blocks);
        let txns_clone = Arc::clone(&self.transactions);
        let client = self.client.clone();

        thread::spawn(move || {
            runtime.block_on(async {
                if let Ok(new_blocks) = client.get_latest_blocks(5).await {
                    let mut blocks = blocks_clone.lock().unwrap();
                    *blocks = new_blocks;
                }

                if let Ok(new_txns) = client.get_latest_transactions(5).await {
                    let mut txns = txns_clone.lock().unwrap();
                    *txns = new_txns;
                }
            });
        });
    }

    fn start_data_fetching(&self) {
        let blocks_clone = Arc::clone(&self.blocks);
        let txns_clone = Arc::clone(&self.transactions);
        let client = self.client.clone();
        let runtime = self.runtime.handle().clone();
        let show_live = Arc::clone(&self.show_live);

        // These will be used to track selected items by IDs
        let selected_block_id = Arc::new(Mutex::new(None::<u64>));
        let selected_txn_id = Arc::new(Mutex::new(None::<String>));

        thread::spawn(move || {
            let mut last_txn_fetch = Instant::now();
            let mut last_block_fetch = Instant::now();

            let block_interval = Duration::from_secs(5);
            let txn_interval = Duration::from_secs(5);

            loop {
                if !*show_live.lock().unwrap() {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }

                let now = Instant::now();

                if now.duration_since(last_block_fetch) >= block_interval {
                    last_block_fetch = now;

                    let blocks_clone = Arc::clone(&blocks_clone);
                    let selected_block_id_clone = Arc::clone(&selected_block_id);
                    runtime.block_on(async {
                        if let Ok(new_blocks) = client.get_latest_blocks(5).await {
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
                                        let pos = blocks.partition_point(|b| b.id > new_block.id);
                                        blocks.insert(pos, new_block);
                                    }
                                }

                                if blocks.len() > 100 {
                                    blocks.truncate(100);
                                }
                            }
                        }
                    });
                }

                if now.duration_since(last_txn_fetch) >= txn_interval {
                    last_txn_fetch = now;

                    let txns_clone = Arc::clone(&txns_clone);
                    let selected_txn_id_clone = Arc::clone(&selected_txn_id);
                    runtime.block_on(async {
                        if let Ok(new_txns) = client.get_latest_transactions(5).await {
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
                        }
                    });
                }

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.popup_state.clone() {
            PopupState::NetworkSelect(index) => match key_event.code {
                KeyCode::Esc => self.popup_state = PopupState::None,
                KeyCode::Up => {
                    let new_index = if index == 0 { 2 } else { index - 1 };
                    self.popup_state = PopupState::NetworkSelect(new_index);
                }
                KeyCode::Down => {
                    let new_index = if index == 2 { 0 } else { index + 1 };
                    self.popup_state = PopupState::NetworkSelect(new_index);
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
                }
                _ => {}
            },
            PopupState::Search(query, search_type) => match key_event.code {
                KeyCode::Esc => {
                    self.popup_state = PopupState::None;
                    self.filtered_transactions.clear();
                }
                KeyCode::Enter => {
                    let query_clone = query.clone();
                    self.popup_state = PopupState::None;
                    self.search_transactions(&query_clone, search_type);
                }
                KeyCode::Char(c) => {
                    let mut new_query = query.clone();
                    new_query.push(c);
                    self.popup_state = PopupState::Search(new_query, search_type);
                }
                KeyCode::Backspace => {
                    let mut new_query = query.clone();
                    new_query.pop();
                    self.popup_state = PopupState::Search(new_query, search_type);
                }
                KeyCode::Tab => {
                    self.popup_state = PopupState::Search(query.clone(), search_type.cycle_next());
                }
                KeyCode::BackTab => {
                    self.popup_state = PopupState::Search(query.clone(), search_type.cycle_prev());
                }
                _ => {}
            },
            PopupState::Message(_) => {
                if key_event.code == KeyCode::Esc {
                    self.popup_state = PopupState::None;
                }
            }
            PopupState::SearchResults(results) => match key_event.code {
                KeyCode::Esc => {
                    self.popup_state = PopupState::None;
                    self.filtered_transactions.clear();
                }
                KeyCode::Enter => {
                    if !results.is_empty() {
                        let (orig_index, _) = &results[0];
                        self.selected_transaction_index = Some(*orig_index);
                        self.popup_state = PopupState::None;
                        self.show_transaction_details = true;
                    }
                }
                _ => {}
            },
            PopupState::None => {
                if self.show_block_details || self.show_transaction_details {
                    match key_event.code {
                        KeyCode::Esc => {
                            self.show_block_details = false;
                            self.show_transaction_details = false;
                        }
                        KeyCode::Char('c') => {
                            if self.show_transaction_details {
                                self.copy_transaction_id_to_clipboard();
                            }
                        }
                        _ => {}
                    }
                } else {
                    match key_event.code {
                        KeyCode::Char('q') => self.exit(),
                        KeyCode::Char('r') => self.refresh_data(),
                        KeyCode::Char(' ') => self.toggle_show_live(),
                        KeyCode::Char('f') => self.open_search(),
                        KeyCode::Char('n') => self.open_network_selector(),
                        KeyCode::Tab => self.cycle_focus(),
                        KeyCode::Up => self.move_selection_up(),
                        KeyCode::Down => self.move_selection_down(),
                        KeyCode::Enter => self.show_details(),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_mouse_input(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let popup_state = self.popup_state.clone();
                let has_popup = popup_state != PopupState::None;
                let popup_open =
                    self.show_block_details || self.show_transaction_details || has_popup;

                if popup_open {
                    if let PopupState::SearchResults(results) = popup_state {
                        if !results.is_empty() {
                            let (orig_index, _) = &results[0];
                            self.selected_transaction_index = Some(*orig_index);
                            self.popup_state = PopupState::None;
                            self.show_transaction_details = true;
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

    pub fn open_network_selector(&mut self) {
        let current_index = match self.network {
            Network::MainNet => 0,
            Network::TestNet => 1,
            Network::LocalNet => 2,
        };
        self.popup_state = PopupState::NetworkSelect(current_index);
    }

    pub fn open_search(&mut self) {
        self.popup_state = PopupState::Search(String::new(), SearchType::All);
    }

    fn search_transactions(&mut self, query: &str, search_type: SearchType) {
        if query.is_empty() {
            self.popup_state = PopupState::Message("Please enter a search term".to_string());
            return;
        }

        let search_query = query.to_lowercase();
        let transactions = self.transactions.lock().unwrap();

        let mut results = Vec::new();
        for (i, txn) in transactions.iter().enumerate() {
            match search_type {
                SearchType::All => {
                    if txn.id.to_lowercase().contains(&search_query)
                        || txn.from.to_lowercase().contains(&search_query)
                        || txn.to.to_lowercase().contains(&search_query)
                        || (txn.asset_id.is_some()
                            && txn.asset_id.unwrap().to_string().contains(&search_query))
                        || txn.block.to_string().contains(&search_query)
                    {
                        results.push((i, txn.clone()));
                    }
                }
                SearchType::TxnID => {
                    if txn.id.to_lowercase().contains(&search_query) {
                        results.push((i, txn.clone()));
                    }
                }
                SearchType::AssetID => {
                    if let Some(asset_id) = txn.asset_id {
                        if asset_id.to_string().contains(&search_query) {
                            results.push((i, txn.clone()));
                        }
                    }
                }
                SearchType::Address => {
                    if txn.from.to_lowercase().contains(&search_query)
                        || txn.to.to_lowercase().contains(&search_query)
                    {
                        results.push((i, txn.clone()));
                    }
                }
                SearchType::Block => {
                    if txn.block.to_string().contains(&search_query) {
                        results.push((i, txn.clone()));
                    }
                }
            }
        }

        if results.is_empty() {
            self.popup_state = PopupState::Message("No matching transactions found".to_string());
        } else {
            self.filtered_transactions = results.clone();
            self.popup_state = PopupState::SearchResults(results);
        }
    }

    fn switch_network(&mut self, network: Network) {
        self.network = network;
        self.client = AlgoClient::new(network);

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
        self.filtered_transactions.clear();

        // Fetch new data
        self.initial_data_fetch();
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn toggle_show_live(&mut self) {
        let mut show_live = self.show_live.lock().unwrap();
        *show_live = !*show_live;
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Blocks => Focus::Transactions,
            Focus::Transactions => Focus::Sidebar,
            Focus::Sidebar => Focus::Blocks,
        };
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
                    || self.selected_transaction_id.is_some()
                {
                    self.show_transaction_details = true;
                }
            }
            _ => {}
        }
    }

    pub fn refresh_data(&mut self) {
        self.initial_data_fetch();
    }

    fn copy_transaction_id_to_clipboard(&mut self) {
        if let Some(index) = self.selected_transaction_index {
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
