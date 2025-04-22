use crate::algorand::{AlgoBlock, AlgoClient, Network, Transaction};
use crate::app::SearchType;
use crate::constants::{
    BLOCK_FETCH_INTERVAL, MAX_BLOCKS_TO_KEEP, MAX_TXNS_TO_KEEP, NETWORK_CHECK_INTERVAL,
    TXN_FETCH_INTERVAL,
};
use crate::event::NetworkUpdateEvent;
use color_eyre::Result;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, mpsc};
use tokio::time::sleep;

/// Manages background network tasks.
pub struct NetworkManager {
    client: Arc<Mutex<AlgoClient>>,
    show_live: Arc<Mutex<bool>>,
    blocks: Arc<Mutex<Vec<AlgoBlock>>>,
    transactions: Arc<Mutex<Vec<Transaction>>>,
    runtime: tokio::runtime::Handle,
    network_event_sender: mpsc::Sender<NetworkUpdateEvent>,
}

impl NetworkManager {
    /// Creates a new NetworkManager.
    pub fn new(
        client: Arc<Mutex<AlgoClient>>,
        show_live: Arc<Mutex<bool>>,
        blocks: Arc<Mutex<Vec<AlgoBlock>>>,
        transactions: Arc<Mutex<Vec<Transaction>>>,
        runtime: tokio::runtime::Handle,
        network_event_sender: mpsc::Sender<NetworkUpdateEvent>,
    ) -> Self {
        Self {
            client,
            show_live,
            blocks,
            transactions,
            runtime,
            network_event_sender,
        }
    }

    /// Starts the main background loop for fetching data and checking status.
    pub fn start_background_loop(&self) {
        let client = Arc::clone(&self.client);
        let show_live = Arc::clone(&self.show_live);
        let blocks = Arc::clone(&self.blocks);
        let transactions = Arc::clone(&self.transactions);
        let runtime = self.runtime.clone();
        let sender = self.network_event_sender.clone();

        runtime.spawn(async move {
            let mut last_txn_fetch = Instant::now();
            let mut last_block_fetch = Instant::now();
            let mut last_network_check = Instant::now();
            let mut is_network_available = true;
            let mut network_error_shown = false;

            loop {
                if !*show_live.lock().await {
                    network_error_shown = false;
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }

                let current_client = client.lock().await.clone();
                let now = Instant::now();

                if now.duration_since(last_network_check) >= NETWORK_CHECK_INTERVAL {
                    last_network_check = now;
                    let status_result = current_client.get_network_status().await;

                    let event_payload = match status_result {
                        Ok(_) => Ok(()),
                        Err(ref e) => Err(format!("{}", e)),
                    };

                    if event_payload.is_ok() {
                        if !is_network_available {
                            let _ = sender
                                .send(NetworkUpdateEvent::StatusUpdate(event_payload))
                                .await;
                        }
                        is_network_available = true;
                        network_error_shown = false;
                    } else if !network_error_shown {
                        let _ = sender
                            .send(NetworkUpdateEvent::StatusUpdate(event_payload))
                            .await;
                        network_error_shown = true;
                        is_network_available = false;
                    }

                    if !is_network_available {
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }

                if is_network_available
                    && now.duration_since(last_block_fetch) >= BLOCK_FETCH_INTERVAL
                {
                    last_block_fetch = now;
                    let blocks_result = fetch_and_update_blocks(
                        &current_client,
                        Arc::clone(&blocks),
                        MAX_BLOCKS_TO_KEEP,
                    )
                    .await;

                    if blocks_result.is_err() {
                        last_network_check = Instant::now()
                            .checked_sub(NETWORK_CHECK_INTERVAL * 2)
                            .unwrap_or_else(Instant::now);
                    }
                    let event_payload = blocks_result.map_err(|e| format!("{}", e));
                    let _ = sender
                        .send(NetworkUpdateEvent::BlocksFetched(event_payload))
                        .await;
                }

                if is_network_available && now.duration_since(last_txn_fetch) >= TXN_FETCH_INTERVAL
                {
                    last_txn_fetch = now;
                    let txns_result = fetch_and_update_transactions(
                        &current_client,
                        Arc::clone(&transactions),
                        MAX_TXNS_TO_KEEP,
                    )
                    .await;

                    if txns_result.is_err() {
                        last_network_check = Instant::now()
                            .checked_sub(NETWORK_CHECK_INTERVAL * 2)
                            .unwrap_or_else(Instant::now);
                    }
                    let event_payload = txns_result.map_err(|e| format!("{}", e));
                    let _ = sender
                        .send(NetworkUpdateEvent::TransactionsFetched(event_payload))
                        .await;
                }

                sleep(crate::constants::TICK_RATE / 2).await;
            }
        });
    }

    /// Fetches initial data (blocks and transactions) when the app starts or network changes.
    pub fn fetch_initial_data(&self, _network_name: String) {
        let client = Arc::clone(&self.client);
        let blocks = Arc::clone(&self.blocks);
        let transactions = Arc::clone(&self.transactions);
        let runtime = self.runtime.clone();
        let sender = self.network_event_sender.clone();

        runtime.spawn(async move {
            let client_lock = client.lock().await;
            let current_client = client_lock.clone();
            drop(client_lock);

            let status_result = current_client.get_network_status().await;
            let event_payload = match status_result {
                Ok(_) => Ok(()),
                Err(ref e) => Err(format!("{}", e)),
            };
            let _ = sender
                .send(NetworkUpdateEvent::StatusUpdate(event_payload.clone()))
                .await;

            if event_payload.is_ok() {
                let blocks_result = fetch_and_update_blocks(
                    &current_client,
                    Arc::clone(&blocks),
                    MAX_BLOCKS_TO_KEEP,
                )
                .await;
                let blocks_event_payload = blocks_result.map_err(|e| format!("{}", e));
                let _ = sender
                    .send(NetworkUpdateEvent::BlocksFetched(blocks_event_payload))
                    .await;

                let txns_result = fetch_and_update_transactions(
                    &current_client,
                    Arc::clone(&transactions),
                    MAX_TXNS_TO_KEEP,
                )
                .await;
                let txns_event_payload = txns_result.map_err(|e| format!("{}", e));
                let _ = sender
                    .send(NetworkUpdateEvent::TransactionsFetched(txns_event_payload))
                    .await;
            }
        });
    }

    /// Performs a search based on the query and type.
    pub fn search(&self, query: String, search_type: SearchType) {
        let client = Arc::clone(&self.client);
        let runtime = self.runtime.clone();
        let sender = self.network_event_sender.clone();

        runtime.spawn(async move {
            let client_lock = client.lock().await;
            let current_client = client_lock.clone();
            drop(client_lock);

            let search_result = current_client.search_by_query(&query, search_type).await;
            let event_payload = search_result.map_err(|e| format!("{}", e));
            let _ = sender
                .send(NetworkUpdateEvent::SearchResults(event_payload))
                .await;
        });
    }

    /// Updates the internally held client when the network switches.
    pub fn switch_network(
        &self,
        new_network: Network,
    ) -> impl std::future::Future<Output = ()> + Send + 'static {
        // Clone Arcs needed for the async block
        let client_arc = Arc::clone(&self.client);
        let blocks_arc = Arc::clone(&self.blocks);
        let transactions_arc = Arc::clone(&self.transactions);
        let sender = self.network_event_sender.clone();

        // Return an async move block which is implicitly 'static
        async move {
            let new_client = AlgoClient::new(new_network.clone());
            // Update the shared client
            {
                let mut client_lock = client_arc.lock().await;
                *client_lock = new_client.clone(); // Clone new_client for subsequent fetches
            }

            // Clear existing data (locks are held briefly inside)
            let blocks_arc_for_fetch = Arc::clone(&blocks_arc);
            let transactions_arc_for_fetch = Arc::clone(&transactions_arc);
            clear_data(blocks_arc, transactions_arc).await;

            // Fetch initial data for the new network
            fetch_initial_data_after_switch(
                new_client,                 // Use the cloned client
                blocks_arc_for_fetch,       // Use the clone made for fetching
                transactions_arc_for_fetch, // Use the clone made for fetching
                sender,
                new_network.as_str().to_string(),
            )
            .await;
        }
    }

    /// Spawns a future onto the NetworkManager's runtime.
    pub fn spawn_task<F>(&self, future: F)
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future);
    }
}

/// Fetches the latest blocks and updates the shared state.
async fn fetch_and_update_blocks(
    client: &AlgoClient,
    blocks_arc: Arc<Mutex<Vec<AlgoBlock>>>,
    max_blocks: usize,
) -> Result<Vec<AlgoBlock>> {
    let new_blocks = client.get_latest_blocks(5).await?;
    if new_blocks.is_empty() {
        return Ok(Vec::new()); // Return empty vec, not an error
    }

    let mut blocks = blocks_arc.lock().await;

    // Efficiently merge new blocks while maintaining sort order and capacity
    let existing_block_ids: HashSet<u64> = blocks.iter().map(|b| b.id).collect();
    let mut added_blocks = Vec::new();

    for new_block in new_blocks {
        if !existing_block_ids.contains(&new_block.id) {
            let pos = blocks.partition_point(|b| b.id > new_block.id);
            blocks.insert(pos, new_block.clone());
            added_blocks.push(new_block);
        }
    }

    if blocks.len() > max_blocks {
        blocks.truncate(max_blocks);
    }

    Ok(added_blocks) // Return only the newly added blocks for the event
}

/// Fetches the latest transactions and updates the shared state.
async fn fetch_and_update_transactions(
    client: &AlgoClient,
    transactions_arc: Arc<Mutex<Vec<Transaction>>>,
    max_txns: usize,
) -> Result<Vec<Transaction>> {
    let new_txns = client.get_latest_transactions(5).await?;
    if new_txns.is_empty() {
        return Ok(Vec::new());
    }

    let mut txns = transactions_arc.lock().await;

    let existing_txn_ids: HashSet<String> = txns.iter().map(|t| t.id.clone()).collect();
    let mut added_txns = Vec::new();

    // Prepend new, unique transactions
    for new_txn in new_txns.into_iter().rev() {
        // Reverse to prepend in correct order
        if !existing_txn_ids.contains(&new_txn.id) {
            txns.insert(0, new_txn.clone());
            added_txns.push(new_txn);
        }
    }

    // Ensure capacity
    if txns.len() > max_txns {
        txns.truncate(max_txns);
    }

    Ok(added_txns.into_iter().rev().collect()) // Return added transactions in descending order
}

// Helper function to clear data, keeps locking minimal
async fn clear_data(
    blocks_arc: Arc<Mutex<Vec<AlgoBlock>>>,
    transactions_arc: Arc<Mutex<Vec<Transaction>>>,
) {
    {
        let mut blocks = blocks_arc.lock().await;
        blocks.clear();
    }
    {
        let mut transactions = transactions_arc.lock().await;
        transactions.clear();
    }
}

// Helper function similar to fetch_initial_data but takes client directly
async fn fetch_initial_data_after_switch(
    client: AlgoClient, // Takes ownership/clone
    blocks_arc: Arc<Mutex<Vec<AlgoBlock>>>,
    transactions_arc: Arc<Mutex<Vec<Transaction>>>,
    sender: mpsc::Sender<NetworkUpdateEvent>,
    _network_name: String, // Keep signature consistent for now
) {
    // Send status update first
    let status_result = client.get_network_status().await;
    let event_payload = match status_result {
        Ok(_) => Ok(()),
        Err(ref e) => Err(format!("{}", e)),
    };
    let _ = sender
        .send(NetworkUpdateEvent::StatusUpdate(event_payload.clone()))
        .await;

    // Proceed only if network is ok
    if event_payload.is_ok() {
        // Fetch initial blocks
        let blocks_result = fetch_and_update_blocks(&client, blocks_arc, MAX_BLOCKS_TO_KEEP).await;
        let blocks_event_payload = blocks_result.map_err(|e| format!("{}", e));
        let _ = sender
            .send(NetworkUpdateEvent::BlocksFetched(blocks_event_payload))
            .await;

        // Fetch initial transactions
        let txns_result =
            fetch_and_update_transactions(&client, transactions_arc, MAX_TXNS_TO_KEEP).await;
        let txns_event_payload = txns_result.map_err(|e| format!("{}", e));
        let _ = sender
            .send(NetworkUpdateEvent::TransactionsFetched(txns_event_payload))
            .await;
    }
}

// Tokio requires a Duration type for sleep
use tokio::time::Duration;
