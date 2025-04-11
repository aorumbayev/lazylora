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
    SearchResults(Vec<(usize, Transaction)>), // Search results with original indices and transaction data
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
    // New state for popups
    pub popup_state: PopupState,
    pub filtered_transactions: Vec<(usize, Transaction)>, // (original_index, transaction)
    // Tokio runtime for async operations
    runtime: tokio::runtime::Runtime,
    client: AlgoClient,
}

impl App {
    /// Create a new application instance with default state
    pub fn new() -> Self {
        let blocks = Arc::new(Mutex::new(Vec::new()));
        let transactions = Arc::new(Mutex::new(Vec::new()));

        // Create a tokio runtime
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
        // Start background data fetching thread immediately
        self.start_data_fetching();

        // Trigger data fetch asynchronously, don't wait for it
        let runtime = self.runtime.handle().clone();
        let blocks_clone = Arc::clone(&self.blocks);
        let txns_clone = Arc::clone(&self.transactions);
        let client = self.client.clone();

        // Perform initial data load in the background
        std::thread::spawn(move || {
            runtime.block_on(async {
                // Get some initial blocks
                if let Ok(new_blocks) = client.get_latest_blocks(5).await {
                    let mut blocks = blocks_clone.lock().unwrap();
                    *blocks = new_blocks;
                }

                // Get some initial transactions - fetch more initially to ensure we have a good selection
                if let Ok(new_txns) = client.get_latest_transactions(5).await {
                    let mut txns = txns_clone.lock().unwrap();
                    *txns = new_txns;
                }
            });
        });

        // Main event loop - with better timed redraw
        // We'll track time precisely for smoother UI updates
        let tick_rate = std::time::Duration::from_millis(100); // 10 FPS
        let mut last_tick = std::time::Instant::now();

        // Note: Terminal initialization is now handled in the tui module
        // We don't need to enable raw mode or mouse capture here

        while !self.exit {
            // Update based on time
            let now = std::time::Instant::now();
            let timeout = tick_rate
                .checked_sub(now.duration_since(last_tick))
                .unwrap_or(std::time::Duration::from_secs(0));

            // Only poll for events for a short time to maintain responsiveness
            if crossterm::event::poll(timeout)? {
                // An event is available - handle it
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

            // Check if it's time to redraw
            let now = std::time::Instant::now();
            if now.duration_since(last_tick) >= tick_rate {
                // Redraw the UI
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

        // Spawn a thread for data fetching
        thread::spawn(move || {
            let mut last_txn_fetch = std::time::Instant::now();
            let mut last_block_fetch = std::time::Instant::now();

            let block_interval = std::time::Duration::from_secs(5);
            let txn_interval = std::time::Duration::from_secs(5);

            loop {
                // Check if live updates are enabled
                if !*show_live.lock().unwrap() {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }

                let now = std::time::Instant::now();

                // Fetch latest blocks on interval
                if now.duration_since(last_block_fetch) >= block_interval {
                    last_block_fetch = now;

                    let blocks_clone = Arc::clone(&blocks_clone);
                    let _ = runtime.block_on(async {
                        match client.get_latest_blocks(5).await {
                            Ok(new_blocks) => {
                                if !new_blocks.is_empty() {
                                    let mut blocks = blocks_clone.lock().unwrap();

                                    // Create a map of existing blocks by ID for quick lookup
                                    let block_map: std::collections::HashMap<u64, usize> = blocks
                                        .iter()
                                        .enumerate()
                                        .map(|(i, block)| (block.id, i))
                                        .collect();

                                    // Add new blocks, avoiding duplicates and maintaining sorted order
                                    for new_block in new_blocks {
                                        if !block_map.contains_key(&new_block.id) {
                                            // Insert the new block at the right position to maintain descending order
                                            let pos =
                                                blocks.partition_point(|b| b.id > new_block.id);
                                            blocks.insert(pos, new_block);
                                        }
                                    }

                                    // Keep only the most recent 100 blocks to avoid unbounded growth
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

                // Fetch latest transactions on a more frequent interval
                if now.duration_since(last_txn_fetch) >= txn_interval {
                    last_txn_fetch = now;

                    // Fetch latest transactions
                    let txns_clone = Arc::clone(&txns_clone);
                    let _ = runtime.block_on(async {
                        match client.get_latest_transactions(5).await {
                            Ok(new_txns) => {
                                if !new_txns.is_empty() {
                                    let mut txns = txns_clone.lock().unwrap();

                                    // Create a set of existing transaction IDs for quick lookup
                                    let txn_ids: std::collections::HashSet<String> =
                                        txns.iter().map(|txn| txn.id.clone()).collect();

                                    // Add new transactions directly
                                    // Put new transactions first, they are already in desc order from API
                                    let mut updated_txns = Vec::with_capacity(100);

                                    // First add all new transactions we don't already have
                                    for new_txn in new_txns {
                                        if !txn_ids.contains(&new_txn.id) {
                                            updated_txns.push(new_txn);
                                        }
                                    }

                                    // Then add existing transactions we're keeping
                                    for old_txn in txns.iter().cloned() {
                                        if updated_txns.len() >= 100 {
                                            break; // Stop when we have 100 transactions
                                        }
                                        if !updated_txns.iter().any(|t| t.id == old_txn.id) {
                                            updated_txns.push(old_txn);
                                        }
                                    }

                                    // Replace the old transactions with our new ordered list
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

                // Sleep a short time before checking intervals again
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    /// Update the application based on user input
    // The handle_events method is now unused since we handle events directly in the run loop

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        // Handle popups first
        if let PopupState::NetworkSelect(index) = &self.popup_state {
            // Handle network selection popup
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
                    // Change network
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
            // Handle search popup
            match key_event.code {
                KeyCode::Esc => {
                    self.popup_state = PopupState::None;
                    self.filtered_transactions.clear();
                }
                KeyCode::Enter => {
                    // Apply the search
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
            // Handle message popup
            if key_event.code == KeyCode::Esc {
                self.popup_state = PopupState::None;
            }
            return Ok(());
        } else if let PopupState::SearchResults(results) = &self.popup_state {
            // Handle search results popup
            match key_event.code {
                KeyCode::Esc => {
                    // Close popup and clear results
                    self.popup_state = PopupState::None;
                    self.filtered_transactions.clear();
                }
                KeyCode::Enter => {
                    if !results.is_empty() {
                        // Select the first result for viewing in the detail popup
                        let (orig_index, _) = &results[0];

                        // Switch to transaction details view
                        self.selected_transaction_index = Some(*orig_index);

                        // First switch to regular view
                        self.popup_state = PopupState::None;

                        // Then show transaction details
                        self.show_transaction_details = true;
                    }
                }
                _ => {}
            }
            return Ok(());
        } else if self.show_block_details || self.show_transaction_details {
            // If detail popups are shown, only handle ESC to close them
            if key_event.code == KeyCode::Esc {
                self.show_block_details = false;
                self.show_transaction_details = false;
            }
            return Ok(());
        } else {
            // Normal key handling
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

    /// Handle mouse input events
    fn handle_mouse_input(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check if we have any popup open
                let has_popup = match &self.popup_state {
                    PopupState::None => false,
                    _ => true,
                };

                let popup_open =
                    self.show_block_details || self.show_transaction_details || has_popup;

                if popup_open {
                    // Special case for search results popup - click on a result to view details
                    match &self.popup_state {
                        PopupState::SearchResults(results) => {
                            if !results.is_empty() {
                                // This is a simplified implementation that just selects the first result
                                // In a more advanced version, you would calculate which result was clicked
                                let (orig_index, _) = &results[0];

                                // Switch to transaction details view
                                self.selected_transaction_index = Some(*orig_index);

                                // First switch to regular view
                                self.popup_state = PopupState::None;

                                // Then show transaction details
                                self.show_transaction_details = true;

                                return Ok(());
                            }
                        }
                        _ => {}
                    }

                    // For other popups, keep them open for all clicks
                    // This prevents the issue of closing popups when clicking inside them
                    // User can still close popups with ESC key
                    return Ok(());
                }

                // If no popup is open, handle normal UI interaction
                // We'll use basic area detection based on column position
                // Left area (0-25%) is sidebar, middle (25%-60%) is blocks, right (60%+) is transactions

                let col_percent = (mouse.column as f32 / 100.0) * 100.0;

                if col_percent < 25.0 {
                    // Left area - sidebar
                    self.focus = Focus::Sidebar;
                } else if col_percent < 60.0 {
                    // Middle area - blocks
                    self.focus = Focus::Blocks;

                    // Try to determine which block was clicked based on row position
                    // Each block takes 3 rows (corrected calculation)
                    // We need to account for header (3 rows) and title area (3 rows)
                    // Click position needs to be adjusted by these fixed header heights
                    let header_height = 3; // Header height
                    let title_height = 3; // Title area height
                    let block_height = 3; // Each block takes 3 rows

                    // Only process clicks in the blocks area
                    if mouse.row as u16 > (header_height + title_height) {
                        // Calculate the clicked row within the blocks area
                        let blocks_area_row = mouse.row as u16 - header_height - title_height;

                        // Calculate which visible block was clicked (0-based)
                        let visible_index = blocks_area_row / block_height;

                        // Add scroll offset to get the actual block index
                        let scroll_offset = self.block_scroll as usize / block_height as usize;
                        let absolute_index = scroll_offset + visible_index as usize;

                        let blocks = self.blocks.lock().unwrap();

                        // Check if the calculated index is valid
                        if absolute_index < blocks.len() {
                            self.selected_block_index = Some(absolute_index);
                            self.show_block_details = true;
                        }
                    }
                } else {
                    // Right area - transactions
                    self.focus = Focus::Transactions;

                    // Similar to blocks, but transactions take 4 rows each
                    let header_height = 3; // Header height
                    let title_height = 3; // Title area height
                    let txn_height = 4; // Each transaction takes 4 rows

                    // Only process clicks in the transactions area
                    if mouse.row as u16 > (header_height + title_height) {
                        // Calculate the clicked row within the transactions area
                        let txns_area_row = mouse.row as u16 - header_height - title_height;

                        // Calculate which visible transaction was clicked (0-based)
                        let visible_index = txns_area_row / txn_height;

                        // Add scroll offset to get the actual transaction index
                        let scroll_offset = self.transaction_scroll as usize / txn_height as usize;
                        let absolute_index = scroll_offset + visible_index as usize;

                        let transactions = self.transactions.lock().unwrap();

                        // Check if the calculated index is valid
                        if absolute_index < transactions.len() {
                            self.selected_transaction_index = Some(absolute_index);
                            self.show_transaction_details = true;
                        }
                    }
                }
            }
            // Handle other mouse events like scrolling
            MouseEventKind::ScrollDown => {
                match self.focus {
                    Focus::Blocks => {
                        // Get maximum scroll position to prevent overflow
                        let blocks = self.blocks.lock().unwrap();
                        let block_height = 3; // Each block takes 3 rows
                        let max_scroll = blocks.len().saturating_sub(1) as u16 * block_height;

                        // Use saturating_add to prevent overflow
                        self.block_scroll = self.block_scroll.saturating_add(block_height);

                        // Ensure we don't scroll beyond the end
                        if self.block_scroll > max_scroll {
                            self.block_scroll = max_scroll;
                        }
                    }
                    Focus::Transactions => {
                        // Get maximum scroll position to prevent overflow
                        let transactions = self.transactions.lock().unwrap();
                        let txn_height = 4; // Each transaction takes 4 rows
                        let max_scroll = transactions.len().saturating_sub(1) as u16 * txn_height;

                        // Use saturating_add to prevent overflow
                        self.transaction_scroll =
                            self.transaction_scroll.saturating_add(txn_height);

                        // Ensure we don't scroll beyond the end
                        if self.transaction_scroll > max_scroll {
                            self.transaction_scroll = max_scroll;
                        }
                    }
                    _ => {}
                }
            }
            MouseEventKind::ScrollUp => {
                match self.focus {
                    Focus::Blocks => {
                        let block_height = 3; // Each block takes 3 rows
                        // Use saturating_sub to prevent underflow
                        self.block_scroll = self.block_scroll.saturating_sub(block_height);
                    }
                    Focus::Transactions => {
                        let txn_height = 4; // Each transaction takes 4 rows
                        // Use saturating_sub to prevent underflow
                        self.transaction_scroll =
                            self.transaction_scroll.saturating_sub(txn_height);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Open the network selector popup
    pub fn open_network_selector(&mut self) {
        let current_index = match self.network {
            Network::MainNet => 0,
            Network::TestNet => 1,
            Network::LocalNet => 2,
        };
        self.popup_state = PopupState::NetworkSelect(current_index);
    }

    /// Open the search popup
    pub fn open_search(&mut self) {
        self.popup_state = PopupState::Search(String::new());
    }

    /// Search for transactions matching the query and display them in a popup
    fn search_transactions(&mut self, query: &str) {
        // If query is empty, don't filter
        if query.is_empty() {
            self.popup_state = PopupState::Message("Please enter a search term".to_string());
            return;
        }

        // First, get the transactions once and collect what we need
        let search_query = query.to_lowercase();
        let transactions = self.transactions.lock().unwrap();

        // Create a list of matching transactions
        let mut results = Vec::new();
        for (i, txn) in transactions.iter().enumerate() {
            if txn.id.to_lowercase().contains(&search_query)
                || txn.from.to_lowercase().contains(&search_query)
                || txn.to.to_lowercase().contains(&search_query)
            {
                results.push((i, txn.clone()));
            }
        }

        // Display search results or a message
        if !results.is_empty() {
            self.popup_state = PopupState::SearchResults(results);
        } else {
            self.popup_state = PopupState::Message("No matching transactions found".to_string());
        }
    }

    /// Switch the network
    fn switch_network(&mut self, network: Network) {
        if self.network == network {
            return;
        }

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

        // Reset selection state
        self.selected_block_index = None;
        self.selected_transaction_index = None;
        self.block_scroll = 0;
        self.transaction_scroll = 0;

        // Refresh data for the new network
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
                        // Adjust scroll if selection moves out of view
                        let block_height = 3; // Each block takes 3 rows
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
                        // Adjust scroll if selection moves out of view
                        let txn_height = 4; // Each transaction takes 4 rows
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

                        // Adjust scroll to keep selection in view
                        let block_height = 3; // Each block takes 3 rows (height)
                        // Approximate visible rows based on a typical terminal height
                        let visible_rows = 15; // Approximate visible area in rows
                        let visible_blocks = visible_rows / block_height;

                        // Calculate the top visible index based on current scroll
                        let top_visible_index = self.block_scroll as usize / block_height;

                        // If the newly selected index would be outside the visible area, scroll down
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

                        // Adjust scroll to keep selection in view
                        let txn_height = 4; // Each transaction takes 4 rows (height)
                        // Approximate visible rows based on a typical terminal height
                        let visible_rows = 15; // Approximate visible area in rows
                        let visible_txns = visible_rows / txn_height;

                        // Calculate the top visible index based on current scroll
                        let top_visible_index = self.transaction_scroll as usize / txn_height;

                        // If the newly selected index would be outside the visible area, scroll down
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
        // Manual refresh - useful for when we're not in live mode
        let runtime = &self.runtime;
        let blocks_clone = Arc::clone(&self.blocks);
        let client = &self.client;

        runtime.block_on(async {
            if let Ok(new_blocks) = client.get_latest_blocks(5).await {
                let mut blocks = blocks_clone.lock().unwrap();

                // Create a map of existing blocks by ID for quick lookup
                let block_map: std::collections::HashMap<u64, usize> = blocks
                    .iter()
                    .enumerate()
                    .map(|(i, block)| (block.id, i))
                    .collect();

                // Add new blocks, avoiding duplicates and maintaining sorted order
                for new_block in new_blocks {
                    if !block_map.contains_key(&new_block.id) {
                        // Insert the new block at the right position to maintain descending order
                        let pos = blocks.partition_point(|b| b.id > new_block.id);
                        blocks.insert(pos, new_block);
                    }
                }

                // Keep only the most recent 100 blocks to avoid unbounded growth
                if blocks.len() > 100 {
                    blocks.truncate(100);
                }
            }
        });

        let txns_clone = Arc::clone(&self.transactions);
        runtime.block_on(async {
            if let Ok(new_txns) = client.get_latest_transactions(5).await {
                let mut txns = txns_clone.lock().unwrap();

                // Create a set of existing transaction IDs for quick lookup
                let txn_ids: std::collections::HashSet<String> =
                    txns.iter().map(|txn| txn.id.clone()).collect();

                // Add new transactions directly (replacing the previous logic)
                // Put new transactions first, they are already in desc order from API
                let mut updated_txns = Vec::with_capacity(100);

                // First add all new transactions we don't already have
                for new_txn in new_txns {
                    if !txn_ids.contains(&new_txn.id) {
                        updated_txns.push(new_txn);
                    }
                }

                // Then add existing transactions we're keeping
                for old_txn in txns.iter().cloned() {
                    if updated_txns.len() >= 100 {
                        break; // Stop when we have 100 transactions
                    }
                    if !updated_txns.iter().any(|t| t.id == old_txn.id) {
                        updated_txns.push(old_txn);
                    }
                }

                // Replace the old transactions with our new ordered list
                *txns = updated_txns;
            }
        });
    }
}
