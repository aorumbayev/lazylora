//! Application lifecycle management.
//!
//! This module contains the core lifecycle methods for the `App`:
//! - `new()` - Creates a new application instance
//! - `run()` - Main event loop
//! - Background task management
//! - Initial data fetching

use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::client::AlgoClient;
use crate::domain::NetworkConfig;
use crate::tui::Tui;
use crate::ui;

use super::{App, AppConfig, AppMessage, NavigationState, StartupOptions, StartupSearch};

// ============================================================================
// Lifecycle Methods
// ============================================================================

impl App {
    /// Creates a new App instance, loading configuration from disk.
    ///
    /// # Errors
    /// Returns an error if initialization fails.
    pub async fn new(startup_options: StartupOptions) -> Result<Self> {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (live_updates_tx, _live_updates_rx) = tokio::sync::watch::channel(true);
        let (network_tx, _network_rx) =
            tokio::sync::watch::channel(NetworkConfig::BuiltIn(crate::domain::Network::MainNet));

        // Load configuration
        let config = AppConfig::load();

        // Build network config from startup options or config
        let network_config = startup_options
            .network
            .map(NetworkConfig::BuiltIn)
            .unwrap_or_else(|| config.network.clone());

        // Get built-in Network for client (custom networks use their own URLs)
        let network = match &network_config {
            NetworkConfig::BuiltIn(net) => *net,
            NetworkConfig::Custom(_) => crate::domain::Network::MainNet, // Placeholder for legacy display
        };

        let show_live = config.show_live;
        let client = AlgoClient::from_config(&network_config)?;

        // Cache available networks
        let available_networks = config.get_all_networks();

        // Set initial state from config
        // Watch channel sends: receivers subscribe later, ok if no subscribers yet
        let _ = live_updates_tx.send(show_live);
        let _ = network_tx.send(network_config.clone());

        Ok(Self {
            nav: NavigationState::new(),
            data: super::DataState::new(),
            ui: super::UiState::new(),
            network,
            network_config,
            available_networks,
            show_live,
            exit: false,
            animation_tick: 0,
            message_tx,
            message_rx,
            live_updates_tx,
            network_tx,
            client,
            startup_options: Some(startup_options),
        })
    }

    /// Runs the main application loop.
    ///
    /// # Errors
    /// Returns an error if the terminal operations fail.
    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        self.start_background_tasks().await;
        self.initial_data_fetch().await;

        // Process startup search if provided
        self.process_startup_search().await;

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        while !self.exit {
            self.process_messages().await;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));

            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key)
                        if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
                    {
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
    // Background Tasks
    // ========================================================================

    pub(super) async fn start_background_tasks(&self) {
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
        mut live_updates_rx: tokio::sync::watch::Receiver<bool>,
        mut network_rx: tokio::sync::watch::Receiver<NetworkConfig>,
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
                    let new_config = network_rx.borrow_and_update().clone();
                    match AlgoClient::from_config(&new_config) {
                        Ok(new_client) => {
                            client = new_client;
                            is_network_available = true;
                            network_error_shown = false;
                        }
                        Err(e) => {
                            let _ = message_tx.send(AppMessage::NetworkError(e.to_string()));
                            is_network_available = false;
                            network_error_shown = true;
                        }
                    }
                }

                _ = network_check_interval.tick() => {
                    if *live_updates_rx.borrow() {
                        match client.get_network_status().await {
                            Ok(()) => {
                                if !is_network_available {
                                    // Receiver may be dropped during shutdown - safe to ignore
                                    let _ = message_tx.send(AppMessage::NetworkConnected);
                                }
                                is_network_available = true;
                                network_error_shown = false;
                            }
                            Err(error_msg) => {
                                if !network_error_shown {
                                    // Receiver may be dropped during shutdown - safe to ignore
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
                        match client.get_latest_blocks(5).await {
                            Ok(blocks) => {
                                network_error_shown = false;
                                // Receiver may be dropped during shutdown - safe to ignore
                                let _ = message_tx.send(AppMessage::BlocksUpdated(blocks));
                            }
                            Err(err) => {
                                if !network_error_shown {
                                    // Receiver may be dropped during shutdown - safe to ignore
                                    let _ = message_tx.send(AppMessage::NetworkError(err.to_string()));
                                    network_error_shown = true;
                                }
                                is_network_available = false;
                            }
                        }
                    }
                }

                _ = transaction_interval.tick() => {
                    if *live_updates_rx.borrow() && is_network_available {
                        match client.get_latest_transactions(5).await {
                            Ok(transactions) => {
                                network_error_shown = false;
                                // Receiver may be dropped during shutdown - safe to ignore
                                let _ = message_tx.send(AppMessage::TransactionsUpdated(transactions));
                            }
                            Err(err) => {
                                if !network_error_shown {
                                    // Receiver may be dropped during shutdown - safe to ignore
                                    let _ = message_tx.send(AppMessage::NetworkError(err.to_string()));
                                    network_error_shown = true;
                                }
                                is_network_available = false;
                            }
                        }
                    }
                }
            }
        }
    }

    pub(super) async fn initial_data_fetch(&self) {
        let message_tx = self.message_tx.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            // Channel sends below: receiver may be dropped during shutdown - safe to ignore
            match client.get_network_status().await {
                Err(error_msg) => {
                    let _ = message_tx.send(AppMessage::NetworkError(error_msg));
                    return;
                }
                Ok(()) => {
                    let _ = message_tx.send(AppMessage::NetworkConnected);
                }
            }

            // Fetch blocks and transactions in parallel
            let (blocks_result, transactions_result) = tokio::join!(
                client.get_latest_blocks(5),
                client.get_latest_transactions(5)
            );

            match blocks_result {
                Ok(blocks) => {
                    let _ = message_tx.send(AppMessage::BlocksUpdated(blocks));
                }
                Err(err) => {
                    let _ = message_tx.send(AppMessage::NetworkError(err.to_string()));
                }
            }

            match transactions_result {
                Ok(transactions) => {
                    let _ = message_tx.send(AppMessage::TransactionsUpdated(transactions));
                }
                Err(err) => {
                    let _ = message_tx.send(AppMessage::NetworkError(err.to_string()));
                }
            }
        });
    }

    /// Process startup search options (transaction, account, block, or asset lookup).
    async fn process_startup_search(&mut self) {
        let startup_options = match self.startup_options.take() {
            Some(opts) => opts,
            None => return,
        };

        let Some(search) = startup_options.search else {
            return;
        };

        let graph_view = startup_options.graph_view;

        match search {
            StartupSearch::Transaction(txn_id) => {
                // Set graph view mode if requested
                if graph_view {
                    self.ui.detail_view_mode = super::DetailViewMode::Visual;
                }
                self.load_transaction_details(&txn_id).await;
            }
            StartupSearch::Account(address) => {
                self.load_account_details(&address);
                self.nav.show_account_details = true;
            }
            StartupSearch::Block(block_num) => {
                self.load_block_details_by_query(block_num);
            }
            StartupSearch::Asset(asset_id) => {
                self.load_asset_details_by_query(asset_id);
            }
        }
    }

    /// Loads block details by block number (for startup query).
    fn load_block_details_by_query(&self, round: u64) {
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

    /// Loads asset details by asset ID (for startup query).
    fn load_asset_details_by_query(&self, asset_id: u64) {
        let message_tx = self.message_tx.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            // Channel sends below: receiver may be dropped during shutdown - safe to ignore
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
}
