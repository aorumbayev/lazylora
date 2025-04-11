use color_eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::algorand::{AlgoBlock, AlgoClient, Network, Transaction};
use crate::ui;

/// Tab options for the sidebar
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Back,
    Explore,
}

impl Tab {
    pub fn as_str(&self) -> &str {
        match self {
            Tab::Back => "← Back",
            Tab::Explore => "Explore",
        }
    }
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
    NetworkSelect(usize),                     // Index of the selected network
    Search(String),                           // Current search query
    Message(String),                          // A message to display to the user
    SearchResults(Vec<(usize, Transaction)>), // Search results with original indices
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    pub network: Network,
    pub active_tab: Tab,
    pub blocks: Arc<Mutex<Vec<AlgoBlock>>>,
    pub transactions: Arc<Mutex<Vec<Transaction>>>,
    pub show_live: Arc<Mutex<bool>>,
    pub focus: Focus,
    pub exit: bool,
    pub block_scroll: u16,
    pub transaction_scroll: u16,
    pub selected_block_index: Option<usize>,
    pub selected_transaction_index: Option<usize>,
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
            active_tab: Tab::Explore,
            blocks,
            transactions,
            show_live: Arc::new(Mutex::new(true)),
            focus: Focus::Blocks,
            exit: false,
            block_scroll: 0,
            transaction_scroll: 0,
            selected_block_index: None,
            selected_transaction_index: None,
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

        let runtime = self.runtime.handle().clone();
        let blocks_clone = Arc::clone(&self.blocks);
        let txns_clone = Arc::clone(&self.transactions);
        let client = self.client.clone();

        std::thread::spawn(move || {
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

        let tick_rate = std::time::Duration::from_millis(100);
        let mut last_tick = std::time::Instant::now();

        while !self.exit {
            let now = std::time::Instant::now();
            let timeout = tick_rate
                .checked_sub(now.duration_since(last_tick))
                .unwrap_or(std::time::Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            self.handle_key_event(key)?;
                        }
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_input(mouse)?;
                    }
                    _ => {}
                }
            }

            let now = std::time::Instant::now();
            if now.duration_since(last_tick) >= tick_rate {
                terminal.draw(|frame| ui::render(self, frame))?;
                last_tick = now;
            }
        }

        Ok(())
    }

    fn start_data_fetching(&self) {
        let blocks_clone = Arc::clone(&self.blocks);
        let txns_clone = Arc::clone(&self.transactions);
        let client = self.client.clone();
        let runtime = self.runtime.handle().clone();
        let show_live = Arc::clone(&self.show_live);

        thread::spawn(move || {
            let mut last_txn_fetch = std::time::Instant::now();
            let mut last_block_fetch = std::time::Instant::now();

            let block_interval = Duration::from_secs(5);
            let txn_interval = Duration::from_secs(5);

            loop {
                if !*show_live.lock().unwrap() {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }

                let now = std::time::Instant::now();

                if now.duration_since(last_block_fetch) >= block_interval {
                    last_block_fetch = now;

                    let blocks_clone = Arc::clone(&blocks_clone);
                    let _ = runtime.block_on(async {
                        match client.get_latest_blocks(5).await {
                            Ok(new_blocks) => {
                                if !new_blocks.is_empty() {
                                    let mut blocks = blocks_clone.lock().unwrap();
                                    let block_map: std::collections::HashMap<u64, usize> = blocks
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

                                    true
                                } else {
                                    false
                                }
                            }
                            Err(err) => {
                                eprintln!("Error fetching blocks: {}", err);
                                false
                            }
                        }
                    });
                }

                if now.duration_since(last_txn_fetch) >= txn_interval {
                    last_txn_fetch = now;

                    let txns_clone = Arc::clone(&txns_clone);
                    let _ = runtime.block_on(async {
                        match client.get_latest_transactions(5).await {
                            Ok(new_txns) => {
                                if !new_txns.is_empty() {
                                    let mut txns = txns_clone.lock().unwrap();
                                    let txn_ids: std::collections::HashSet<String> =
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

                                    true
                                } else {
                                    false
                                }
                            }
                            Err(err) => {
                                eprintln!("Error fetching transactions: {}", err);
                                false
                            }
                        }
                    });
                }

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        if let PopupState::NetworkSelect(index) = &self.popup_state {
            match key_event.code {
                KeyCode::Esc => self.popup_state = PopupState::None,
                KeyCode::Up => {
                    let new_index = if *index == 0 { 2 } else { index - 1 };
                    self.popup_state = PopupState::NetworkSelect(new_index);
                }
                KeyCode::Down => {
                    let new_index = if *index == 2 { 0 } else { index + 1 };
                    self.popup_state = PopupState::NetworkSelect(new_index);
                }
                KeyCode::Enter => {
                    let new_network = match *index {
                        0 => Network::MainNet,
                        1 => Network::TestNet,
                        2 => Network::LocalNet,
                        _ => Network::MainNet,
                    };
                    self.switch_network(new_network);
                    self.popup_state = PopupState::None;
                }
                _ => {}
            }
            return Ok(());
        } else if let PopupState::Search(query) = &self.popup_state {
            match key_event.code {
                KeyCode::Esc => {
                    self.popup_state = PopupState::None;
                    self.filtered_transactions.clear();
                }
                KeyCode::Enter => {
                    let query_clone = query.clone();
                    self.popup_state = PopupState::None;
                    self.search_transactions(&query_clone);
                }
                KeyCode::Char(c) => {
                    let mut new_query = query.clone();
                    new_query.push(c);
                    self.popup_state = PopupState::Search(new_query);
                }
                KeyCode::Backspace => {
                    let mut new_query = query.clone();
                    new_query.pop();
                    self.popup_state = PopupState::Search(new_query);
                }
                _ => {}
            }
            return Ok(());
        } else if let PopupState::Message(_) = &self.popup_state {
            if key_event.code == KeyCode::Esc {
                self.popup_state = PopupState::None;
            }
            return Ok(());
        } else if let PopupState::SearchResults(results) = &self.popup_state {
            match key_event.code {
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
            }
            return Ok(());
        } else if self.show_block_details || self.show_transaction_details {
            if key_event.code == KeyCode::Esc {
                self.show_block_details = false;
                self.show_transaction_details = false;
            }
            return Ok(());
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
        Ok(())
    }

    fn handle_mouse_input(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let has_popup = self.popup_state != PopupState::None;
                let popup_open =
                    self.show_block_details || self.show_transaction_details || has_popup;

                if popup_open {
                    if let PopupState::SearchResults(results) = &self.popup_state {
                        if !results.is_empty() {
                            let (orig_index, _) = &results[0];
                            self.selected_transaction_index = Some(*orig_index);
                            self.popup_state = PopupState::None;
                            self.show_transaction_details = true;
                            return Ok(());
                        }
                    }
                    return Ok(());
                }

                let col_percent = (mouse.column as f32 / 100.0) * 100.0;

                if col_percent < 25.0 {
                    self.focus = Focus::Sidebar;
                } else if col_percent < 60.0 {
                    self.focus = Focus::Blocks;

                    let header_height = 3;
                    let title_height = 3;
                    let block_height = 3;

                    if mouse.row as u16 > (header_height + title_height) {
                        let blocks_area_row = mouse.row as u16 - header_height - title_height;
                        let visible_index = blocks_area_row / block_height;
                        let scroll_offset = self.block_scroll as usize / block_height as usize;
                        let absolute_index = scroll_offset + visible_index as usize;

                        let blocks = self.blocks.lock().unwrap();

                        if absolute_index < blocks.len() {
                            self.selected_block_index = Some(absolute_index);
                            self.show_block_details = true;
                        }
                    }
                } else {
                    self.focus = Focus::Transactions;

                    let header_height = 3;
                    let title_height = 3;
                    let txn_height = 4;

                    if mouse.row as u16 > (header_height + title_height) {
                        let txns_area_row = mouse.row as u16 - header_height - title_height;
                        let visible_index = txns_area_row / txn_height;
                        let scroll_offset = self.transaction_scroll as usize / txn_height as usize;
                        let absolute_index = scroll_offset + visible_index as usize;

                        let transactions = self.transactions.lock().unwrap();

                        if absolute_index < transactions.len() {
                            self.selected_transaction_index = Some(absolute_index);
                            self.show_transaction_details = true;
                        }
                    }
                }
            }
            MouseEventKind::ScrollDown => match self.focus {
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
            },
            MouseEventKind::ScrollUp => match self.focus {
                Focus::Blocks => {
                    let block_height = 3;
                    self.block_scroll = self.block_scroll.saturating_sub(block_height);
                }
                Focus::Transactions => {
                    let txn_height = 4;
                    self.transaction_scroll = self.transaction_scroll.saturating_sub(txn_height);
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
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
        self.popup_state = PopupState::Search(String::new());
    }

    fn search_transactions(&mut self, query: &str) {
        if query.is_empty() {
            self.popup_state = PopupState::Message("Please enter a search term".to_string());
            return;
        }

        let search_query = query.to_lowercase();
        let transactions = self.transactions.lock().unwrap();

        let mut results = Vec::new();
        for (i, txn) in transactions.iter().enumerate() {
            if txn.id.to_lowercase().contains(&search_query)
                || txn.from.to_lowercase().contains(&search_query)
                || txn.to.to_lowercase().contains(&search_query)
            {
                results.push((i, txn.clone()));
            }
        }

        if !results.is_empty() {
            self.popup_state = PopupState::SearchResults(results);
        } else {
            self.popup_state = PopupState::Message("No matching transactions found".to_string());
        }
    }

    fn switch_network(&mut self, network: Network) {
        if self.network == network {
            return;
        }

        self.network = network;
        self.client = AlgoClient::new(network);

        {
            let mut blocks = self.blocks.lock().unwrap();
            blocks.clear();
        }
        {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.clear();
        }

        self.selected_block_index = None;
        self.selected_transaction_index = None;
        self.block_scroll = 0;
        self.transaction_scroll = 0;

        self.refresh_data();
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
                        self.selected_block_index = Some(index - 1);
                        let block_height = 3;
                        if index * block_height <= self.block_scroll as usize {
                            self.block_scroll =
                                self.block_scroll.saturating_sub(block_height as u16);
                        }
                    }
                } else if !self.blocks.lock().unwrap().is_empty() {
                    self.selected_block_index = Some(0);
                }
            }
            Focus::Transactions => {
                if let Some(index) = self.selected_transaction_index {
                    if index > 0 {
                        self.selected_transaction_index = Some(index - 1);
                        let txn_height = 4;
                        if index * txn_height <= self.transaction_scroll as usize {
                            self.transaction_scroll =
                                self.transaction_scroll.saturating_sub(txn_height as u16);
                        }
                    }
                } else if !self.transactions.lock().unwrap().is_empty() {
                    self.selected_transaction_index = Some(0);
                }
            }
            _ => {}
        }
    }

    pub fn move_selection_down(&mut self) {
        match self.focus {
            Focus::Blocks => {
                let blocks = self.blocks.lock().unwrap();
                if let Some(index) = self.selected_block_index {
                    if index < blocks.len() - 1 {
                        self.selected_block_index = Some(index + 1);

                        let block_height = 3;
                        let visible_rows = 15;
                        let visible_blocks = visible_rows / block_height;
                        let top_visible_index = self.block_scroll as usize / block_height;

                        if index + 1 >= top_visible_index + visible_blocks {
                            self.block_scroll =
                                self.block_scroll.saturating_add(block_height as u16);
                        }
                    }
                } else if !blocks.is_empty() {
                    self.selected_block_index = Some(0);
                }
            }
            Focus::Transactions => {
                let txns = self.transactions.lock().unwrap();
                if let Some(index) = self.selected_transaction_index {
                    if index < txns.len() - 1 {
                        self.selected_transaction_index = Some(index + 1);

                        let txn_height = 4;
                        let visible_rows = 15;
                        let visible_txns = visible_rows / txn_height;
                        let top_visible_index = self.transaction_scroll as usize / txn_height;

                        if index + 1 >= top_visible_index + visible_txns {
                            self.transaction_scroll =
                                self.transaction_scroll.saturating_add(txn_height as u16);
                        }
                    }
                } else if !txns.is_empty() {
                    self.selected_transaction_index = Some(0);
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

    pub fn refresh_data(&mut self) {
        let runtime = &self.runtime;
        let blocks_clone = Arc::clone(&self.blocks);
        let client = &self.client;

        runtime.block_on(async {
            if let Ok(new_blocks) = client.get_latest_blocks(5).await {
                let mut blocks = blocks_clone.lock().unwrap();
                let block_map: std::collections::HashMap<u64, usize> = blocks
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
        });

        let txns_clone = Arc::clone(&self.transactions);
        runtime.block_on(async {
            if let Ok(new_txns) = client.get_latest_transactions(5).await {
                let mut txns = txns_clone.lock().unwrap();
                let txn_ids: std::collections::HashSet<String> =
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
        });
    }
}
