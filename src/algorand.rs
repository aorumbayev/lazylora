use crate::app::SearchType;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use color_eyre::Result;
use ratatui::style::Color;
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

// Network types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum Network {
    MainNet,
    TestNet,
    LocalNet,
    Custom {
        name: String,
        algod_url: String,
        indexer_url: String,
        algod_token: Option<String>,
    },
}

impl Network {
    pub fn as_str(&self) -> &str {
        match self {
            Self::MainNet => "MainNet",
            Self::TestNet => "TestNet",
            Self::LocalNet => "LocalNet",
            Self::Custom { name, .. } => name,
        }
    }

    pub fn indexer_url(&self) -> &str {
        match self {
            Self::MainNet => "https://mainnet-idx.algonode.cloud",
            Self::TestNet => "https://testnet-idx.algonode.cloud",
            Self::LocalNet => "http://localhost:8980",
            Self::Custom { indexer_url, .. } => indexer_url,
        }
    }

    pub fn algod_url(&self) -> &str {
        match self {
            Self::MainNet => "https://mainnet-api.algonode.cloud",
            Self::TestNet => "https://testnet-api.algonode.cloud",
            Self::LocalNet => "http://localhost:4001",
            Self::Custom { algod_url, .. } => algod_url,
        }
    }

    pub fn algod_token(&self) -> Option<&str> {
        match self {
            Self::Custom { algod_token, .. } => algod_token.as_deref(),
            _ => None,
        }
    }
}

// API Client
#[derive(Debug, Clone)]
pub struct AlgoClient {
    network: Network,
    client: Client,
}

impl AlgoClient {
    pub fn new(network: Network) -> Self {
        let mut headers = HeaderMap::new();
        if let Some(token) = network.algod_token() {
            if let Ok(mut header_value) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                header_value.set_sensitive(true);
                headers.insert(AUTHORIZATION, header_value);
            } else if let Ok(mut header_value) = HeaderValue::from_str(token) {
                header_value.set_sensitive(true);
                headers.insert("X-Algo-API-Token", header_value);
            }
        }

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { network, client }
    }

    // Remove unused function
    // pub async fn is_network_available(&self) -> bool {
    //     let algod_url = format!("{}/health", self.network.algod_url());
    //     let indexer_url = format!("{}/health", self.network.indexer_url());
    //
    //     let algod_result = self
    //         .client
    //         .get(&algod_url)
    //         .timeout(std::time::Duration::from_secs(2))
    //         .send()
    //         .await;
    //
    //     let indexer_result = self
    //         .client
    //         .get(&indexer_url)
    //         .timeout(std::time::Duration::from_secs(2))
    //         .send()
    //         .await;
    //
    //     match self.network {
    //         Network::LocalNet => algod_result.is_ok() && indexer_result.is_ok(),
    //         _ => algod_result.is_ok() || indexer_result.is_ok(),
    //     }
    // }

    pub async fn get_network_status(&self) -> Result<()> {
        let algod_url = format!("{}/health", self.network.algod_url());
        let indexer_url = format!("{}/health", self.network.indexer_url());

        let algod_check = self
            .client
            .get(&algod_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;

        let indexer_check = self
            .client
            .get(&indexer_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;

        // Check algod first, as it's often primary
        if let Err(e) = algod_check {
            return Err(color_eyre::eyre::eyre!(
                "Unable to connect to algod at {}: {}",
                self.network.algod_url(),
                e
            ));
        }

        // Special handling for LocalNet (requires both) and Custom (warn if indexer fails)
        match self.network {
            Network::LocalNet => {
                if let Err(e) = indexer_check {
                    return Err(color_eyre::eyre::eyre!(
                        "LocalNet: Algod OK, but unable to connect to indexer at {}: {}",
                        self.network.indexer_url(),
                        e
                    ));
                }
            }
            Network::Custom { .. } => {
                if let Err(e) = indexer_check {
                    // Log warning for custom networks, but don't fail the check
                    eprintln!(
                        "Warning: Unable to connect to custom indexer at {}: {}. Proceeding with algod.",
                        self.network.indexer_url(),
                        e
                    );
                }
            }
            // MainNet, TestNet: Indexer is optional for basic status check if algod is ok.
            _ => {
                if let Err(e) = indexer_check {
                    // Log non-critical warning if indexer fails for public networks
                    eprintln!(
                        "Warning: Unable to connect to indexer at {}: {}",
                        self.network.indexer_url(),
                        e
                    );
                }
            }
        }

        Ok(()) // If algod check passed and specific network checks passed/warned
    }

    #[allow(dead_code)]
    pub async fn get_transaction_by_id(&self, txid: &str) -> Result<Option<Transaction>> {
        let url = format!("{}/v2/transactions/{}", self.network.indexer_url(), txid);
        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch transaction {}: {}", txid, e))?;

        if !response.status().is_success() {
            // Consider logging the status code or returning a more specific error
            return Ok(None);
        }

        let json: Value = response.json().await.map_err(|e| {
            color_eyre::eyre::eyre!("Failed to parse transaction JSON for {}: {}", txid, e)
        })?;

        let txn_json = match json.get("transaction") {
            Some(txn) => txn,
            None => return Ok(None), // No 'transaction' field in the response
        };

        // Use the extracted parsing function
        parse_transaction_from_json(txn_json).map(Some)
    }

    pub async fn get_latest_blocks(&self, limit: usize) -> Result<Vec<AlgoBlock>> {
        let status_url = format!("{}/v2/status", self.network.algod_url());
        let status_response = self
            .client
            .get(&status_url)
            .timeout(Duration::from_secs(5))
            .header("accept", "application/json")
            .send()
            .await?;

        let status: Value = status_response.json().await?;
        let latest_round = status["last-round"].as_u64().unwrap_or(0);

        if latest_round == 0 {
            return Ok(Vec::new());
        }

        let mut blocks = Vec::with_capacity(limit);

        for i in 0..limit {
            if i >= latest_round as usize {
                break;
            }

            let round = latest_round - i as u64;
            let block_url = format!("{}/v2/blocks/{}", self.network.algod_url(), round);

            let response = match self
                .client
                .get(&block_url)
                .timeout(Duration::from_secs(5))
                .header("accept", "application/json")
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => resp,
                _ => continue,
            };

            let block_data: Value = match response.json().await {
                Ok(data) => data,
                Err(_) => continue,
            };

            let block = block_data.get("block").unwrap_or(&block_data);
            let timestamp_secs = block["ts"].as_u64().unwrap_or(0);
            let formatted_time = format_timestamp(timestamp_secs);
            let txn_count = count_transactions(block);

            blocks.push(AlgoBlock {
                id: round,
                txn_count,
                timestamp: formatted_time,
            });
        }

        Ok(blocks)
    }

    pub async fn get_latest_transactions(&self, limit: usize) -> Result<Vec<Transaction>> {
        let status_url = format!("{}/v2/status", self.network.algod_url());
        let status_response = self
            .client
            .get(&status_url)
            .timeout(Duration::from_secs(5))
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch network status: {}", e))?;

        if !status_response.status().is_success() {
            return Err(color_eyre::eyre::eyre!(
                "Failed to get network status: {}",
                status_response.status()
            ));
        }

        let status: Value = status_response
            .json()
            .await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to parse network status JSON: {}", e))?;

        let latest_round = status["last-round"].as_u64().unwrap_or(0);
        if latest_round == 0 {
            return Ok(Vec::new()); // No blocks yet, valid empty case
        }

        let min_round = latest_round.saturating_sub(20);
        let url = format!(
            "{}/v2/transactions?limit={}&min-round={}&max-round={}&order=desc",
            self.network.indexer_url(),
            limit,
            min_round,
            latest_round
        );

        // Delegate fetching and parsing to fetch_transactions_from_url
        self.fetch_transactions_from_url(&url).await
    }

    /// Search for blocks, assets, accounts, and transactions based on the input query
    pub async fn search_by_query(
        &self,
        query: &str,
        search_type: SearchType,
    ) -> Result<Vec<SearchResultItem>> {
        match search_type {
            SearchType::Transaction => self.search_transaction(query).await,
            SearchType::Account => self
                .search_address(query)
                .await
                .map(|opt_item| opt_item.map_or_else(Vec::new, |item| vec![item])),
            SearchType::Block => self
                .search_block(query)
                .await
                .map(|opt_item| opt_item.map_or_else(Vec::new, |item| vec![item])),
            SearchType::Asset => self
                .search_asset(query)
                .await
                .map(|opt_item| opt_item.map_or_else(Vec::new, |item| vec![item])),
        }
    }

    // Updated to return Result<Vec<SearchResultItem>>
    async fn search_transaction(&self, txid: &str) -> Result<Vec<SearchResultItem>> {
        // Direct transaction lookup
        let direct_url = format!("{}/v2/transactions/{}", self.network.indexer_url(), txid);
        let mut results = Vec::new();

        match self
            .client
            .get(&direct_url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(json) = response.json::<Value>().await {
                    if let Some(txn_json) = json.get("transaction") {
                        match parse_transaction_from_json(txn_json) {
                            Ok(txn) => results.push(SearchResultItem::Transaction(txn)), // Wrap here
                            Err(e) => {
                                eprintln!("Error parsing found transaction {}: {}", txid, e);
                                // Decide if error here should prevent secondary search
                            }
                        }
                    }
                } else {
                    eprintln!("Error parsing JSON for direct txn lookup {}", txid);
                }
            }
            Ok(response) => {
                eprintln!(
                    "Direct lookup for txn {} failed with status: {}",
                    txid,
                    response.status()
                );
            }
            Err(e) => {
                eprintln!("Network error during direct lookup for txn {}: {}", txid, e);
                // Optionally return the error here if direct lookup network error is critical
                // return Err(color_eyre::eyre::eyre!("Network error during direct lookup: {}", e));
            }
        }

        // If direct lookup didn't yield a result (or maybe even if it did, depending on desired behavior),
        // perform secondary search. Current logic performs it regardless of direct lookup outcome.
        let search_url = format!(
            "{}/v2/transactions?tx-id={}&limit=10",
            self.network.indexer_url(),
            txid
        );

        // Fetch transactions (which internally uses parse_transaction_from_json)
        match self.fetch_transactions_from_url(&search_url).await {
            Ok(fetched_txns) => {
                for txn in fetched_txns {
                    // Avoid adding duplicates if direct lookup already found it
                    if !results.iter().any(
                        |item| matches!(item, SearchResultItem::Transaction(t) if t.id == txn.id),
                    ) {
                        results.push(SearchResultItem::Transaction(txn)); // Wrap here
                    }
                }
            }
            Err(e) => {
                // Log error from secondary search, but potentially return results from direct lookup if any
                eprintln!(
                    "Error during secondary transaction search for {}: {}",
                    txid, e
                );
                // Decide: return partial results or propagate error? Returning partial for now.
            }
        }

        Ok(results)
    }

    // Updated to return Result<Option<SearchResultItem>>
    async fn search_block(&self, round_str: &str) -> Result<Option<SearchResultItem>> {
        let round = match round_str.parse::<u64>() {
            Ok(r) => r,
            Err(_) => return Ok(None), // Not a valid round number
        };

        let block_url = format!("{}/v2/blocks/{}", self.network.algod_url(), round);

        match self
            .client
            .get(&block_url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                // Parse block info
                let block_data: Value = response.json().await.map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to parse block JSON for round {}: {}", round, e)
                })?;
                let block_val = block_data.get("block").unwrap_or(&block_data); // Assume block exists if status is success
                let txn_count = count_transactions(block_val);
                let timestamp_secs = block_val["ts"]
                    .as_u64()
                    .ok_or_else(|| color_eyre::eyre::eyre!("Missing 'ts' in block {}", round))?;
                let formatted_time = format_timestamp(timestamp_secs);
                let proposer = block_val
                    .get("cert")
                    .and_then(|c| c.get("prop"))
                    .and_then(|p| p.get("addr"))
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string();
                let seed = block_val["seed"]
                    .as_str()
                    .ok_or_else(|| color_eyre::eyre::eyre!("Missing 'seed' in block {}", round))?
                    .to_string();

                // Wrap BlockInfo in SearchResultItem::Block
                Ok(Some(SearchResultItem::Block(BlockInfo {
                    id: round,
                    timestamp: formatted_time,
                    txn_count,
                    proposer,
                    seed,
                })))
            }
            Ok(response) if response.status() == reqwest::StatusCode::NOT_FOUND => Ok(None), // Block not found
            Ok(response) => Err(color_eyre::eyre::eyre!(
                "Failed to fetch block {}: Status {}",
                round,
                response.status()
            )),
            Err(e) => Err(color_eyre::eyre::eyre!(
                "Network error fetching block {}: {}",
                round,
                e
            )),
        }
    }

    // Updated to return Result<Option<SearchResultItem>>
    async fn search_asset(&self, asset_id_str: &str) -> Result<Option<SearchResultItem>> {
        let asset_id = match asset_id_str.parse::<u64>() {
            Ok(id) => id,
            Err(_) => return Ok(None), // Not a valid asset ID
        };

        let asset_url = format!("{}/v2/assets/{}", self.network.indexer_url(), asset_id);

        match self
            .client
            .get(&asset_url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let asset_data: Value = response.json().await.map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to parse asset JSON for id {}: {}", asset_id, e)
                })?;
                let params = asset_data
                    .get("asset")
                    .and_then(|a| a.get("params"))
                    .ok_or_else(|| {
                        color_eyre::eyre::eyre!("Missing 'asset.params' in asset {}", asset_id)
                    })?;

                let name = params["name"].as_str().unwrap_or("").to_string();
                let unit_name = params["unit-name"].as_str().unwrap_or("").to_string();
                let creator = params["creator"]
                    .as_str()
                    .ok_or_else(|| {
                        color_eyre::eyre::eyre!("Missing 'creator' in asset {}", asset_id)
                    })?
                    .to_string();
                let total = params["total"].as_u64().ok_or_else(|| {
                    color_eyre::eyre::eyre!("Missing 'total' in asset {}", asset_id)
                })?;
                let decimals = params["decimals"].as_u64().ok_or_else(|| {
                    color_eyre::eyre::eyre!("Missing 'decimals' in asset {}", asset_id)
                })?;
                let url = params["url"].as_str().unwrap_or("").to_string();

                // Wrap AssetInfo in SearchResultItem::Asset
                Ok(Some(SearchResultItem::Asset(AssetInfo {
                    id: asset_id,
                    name,
                    unit_name,
                    creator,
                    total,
                    decimals,
                    url,
                })))
            }
            Ok(response) if response.status() == reqwest::StatusCode::NOT_FOUND => Ok(None), // Asset not found
            Ok(response) => Err(color_eyre::eyre::eyre!(
                "Failed to fetch asset {}: Status {}",
                asset_id,
                response.status()
            )),
            Err(e) => Err(color_eyre::eyre::eyre!(
                "Network error fetching asset {}: {}",
                asset_id,
                e
            )),
        }
    }

    // Updated to return Result<Option<SearchResultItem>>
    async fn search_address(&self, address: &str) -> Result<Option<SearchResultItem>> {
        // Basic address validation (length)
        if address.len() != 58 {
            // Consider more robust validation if needed
            return Ok(None); // Invalid address format
        }

        let account_url = format!("{}/v2/accounts/{}", self.network.indexer_url(), address);

        match self
            .client
            .get(&account_url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let account_data: Value = response.json().await.map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to parse account JSON for {}: {}", address, e)
                })?;
                let account = account_data.get("account").ok_or_else(|| {
                    color_eyre::eyre::eyre!("Missing 'account' field for {}", address)
                })?;

                let balance = account["amount"].as_u64().ok_or_else(|| {
                    color_eyre::eyre::eyre!("Missing 'amount' for account {}", address)
                })?;
                let pending_rewards = account["pending-rewards"].as_u64().ok_or_else(|| {
                    color_eyre::eyre::eyre!("Missing 'pending-rewards' for account {}", address)
                })?;
                let reward_base = account["reward-base"].as_u64().ok_or_else(|| {
                    color_eyre::eyre::eyre!("Missing 'reward-base' for account {}", address)
                })?;
                let status = account["status"]
                    .as_str()
                    .ok_or_else(|| {
                        color_eyre::eyre::eyre!("Missing 'status' for account {}", address)
                    })?
                    .to_string();

                let assets_count = account
                    .get("assets")
                    .and_then(Value::as_array)
                    .map_or(0, |a| a.len());
                let created_assets_count = account
                    .get("created-assets")
                    .and_then(Value::as_array)
                    .map_or(0, |a| a.len());

                // Wrap AccountInfo in SearchResultItem::Account
                Ok(Some(SearchResultItem::Account(AccountInfo {
                    address: address.to_string(),
                    balance,
                    pending_rewards,
                    reward_base,
                    status,
                    assets_count,
                    created_assets_count,
                })))
            }
            Ok(response) if response.status() == reqwest::StatusCode::NOT_FOUND => Ok(None), // Account not found
            Ok(response) => Err(color_eyre::eyre::eyre!(
                "Failed to fetch account {}: Status {}",
                address,
                response.status()
            )),
            Err(e) => Err(color_eyre::eyre::eyre!(
                "Network error fetching account {}: {}",
                address,
                e
            )),
        }
    }

    // Helper function to fetch transactions from a URL
    async fn fetch_transactions_from_url(&self, url: &str) -> Result<Vec<Transaction>> {
        let response = self
            .client
            .get(url)
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to fetch transactions from {}: {}", url, e)
            })?;

        if !response.status().is_success() {
            return Err(color_eyre::eyre::eyre!(
                "Failed to fetch transactions from {}: Status {}",
                url,
                response.status()
            ));
        }

        let json: Value = response.json().await.map_err(|e| {
            color_eyre::eyre::eyre!("Failed to parse transactions JSON from {}: {}", url, e)
        })?;

        let empty_vec = Vec::new();
        let transactions_array = json
            .get("transactions")
            .and_then(Value::as_array)
            .unwrap_or(&empty_vec);
        let mut transactions = Vec::with_capacity(transactions_array.len());

        for (index, txn_json) in transactions_array.iter().enumerate() {
            match parse_transaction_from_json(txn_json) {
                Ok(txn) => transactions.push(txn),
                Err(e) => {
                    // Log the error and the index/content if possible, but continue parsing others
                    eprintln!(
                        "Error parsing transaction at index {} from {}: {}. Skipping.",
                        index, url, e
                    );
                    // Consider adding partial results or more detailed error reporting later
                }
            }
        }

        Ok(transactions)
    }
}

// Extracted transaction parsing logic
fn parse_transaction_from_json(txn_json: &Value) -> Result<Transaction> {
    let id = txn_json["id"]
        .as_str()
        .ok_or_else(|| color_eyre::eyre::eyre!("Missing or invalid 'id'"))?
        .to_string();
    let txn_type = determine_transaction_type(txn_json);
    let from = txn_json["sender"]
        .as_str()
        .ok_or_else(|| color_eyre::eyre::eyre!("Missing or invalid 'sender'"))?
        .to_string();
    let to = extract_receiver(txn_json, &txn_type)?; // Updated extract_receiver to return Result

    let timestamp = txn_json["round-time"]
        .as_u64()
        .map(format_timestamp) // format_timestamp might need update if it can fail
        .unwrap_or_else(|| "Unknown".to_string()); // Keep fallback for now, revisit format_timestamp later

    let block = txn_json["confirmed-round"]
        .as_u64()
        .ok_or_else(|| color_eyre::eyre::eyre!("Missing or invalid 'confirmed-round'"))?;
    let fee = txn_json["fee"]
        .as_u64()
        .ok_or_else(|| color_eyre::eyre::eyre!("Missing or invalid 'fee'"))?;

    // Handle note: might be string or bytes array
    let note = txn_json["note"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| {
            txn_json["note"].as_array().and_then(|arr| {
                // Attempt to decode base64 first, then format bytes as fallback
                if arr.is_empty() {
                    Some(String::new())
                } else if let Some(first) = arr.first().and_then(Value::as_str) {
                    // Use Engine::decode instead of deprecated function
                    BASE64_STANDARD
                        .decode(first)
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                        // Fallback: format the raw bytes if not valid base64/utf8
                        .or_else(|| Some(format!("{:?}", arr))) // Or just format the raw array if needed
                } else {
                    Some(format!("{:?}", arr)) // Format if not string array
                }
            })
        })
        .unwrap_or_else(|| "None".to_string()); // Default if note field is missing or null

    // Extract amount and asset_id based on transaction type
    let (amount, asset_id) = match txn_type {
        TxnType::Payment => {
            let payment_txn = txn_json.get("payment-transaction").ok_or_else(|| {
                color_eyre::eyre::eyre!("Missing 'payment-transaction' for Payment")
            })?;
            let amount = payment_txn["amount"]
                .as_u64()
                .ok_or_else(|| color_eyre::eyre::eyre!("Missing or invalid 'amount' in payment"))?;
            (amount, None)
        }
        TxnType::AssetTransfer => {
            let axfer_txn = txn_json.get("asset-transfer-transaction").ok_or_else(|| {
                color_eyre::eyre::eyre!("Missing 'asset-transfer-transaction' for AssetTransfer")
            })?;
            let amount = axfer_txn["amount"]
                .as_u64()
                .ok_or_else(|| color_eyre::eyre::eyre!("Missing or invalid 'amount' in axfer"))?;
            let asset_id = axfer_txn["asset-id"]
                .as_u64()
                .ok_or_else(|| color_eyre::eyre::eyre!("Missing or invalid 'asset-id' in axfer"))?;
            (amount, Some(asset_id))
        }
        // Handle other types if they have amounts/assets, otherwise default
        _ => (0, None),
    };

    Ok(Transaction {
        id,
        txn_type,
        from,
        to,
        timestamp,
        block,
        fee,
        note,
        amount,
        asset_id,
    })
}

fn determine_transaction_type(txn_json: &Value) -> TxnType {
    if txn_json["payment-transaction"].is_object() {
        TxnType::Payment
    } else if txn_json["application-transaction"].is_object() {
        TxnType::AppCall
    } else if txn_json["asset-transfer-transaction"].is_object() {
        TxnType::AssetTransfer
    } else if txn_json["asset-config-transaction"].is_object() {
        TxnType::AssetConfig
    } else if txn_json["asset-freeze-transaction"].is_object() {
        TxnType::AssetFreeze
    } else if txn_json["keyreg-transaction"].is_object() {
        TxnType::KeyReg
    } else if txn_json["state-proof-transaction"].is_object() {
        TxnType::StateProof
    } else if txn_json["heartbeat-transaction"].is_object() {
        TxnType::Heartbeat
    } else {
        TxnType::Unknown
    }
}

// Update extract_receiver to return Result for better error handling
fn extract_receiver(txn_json: &Value, txn_type: &TxnType) -> Result<String> {
    match txn_type {
        TxnType::Payment => Ok(txn_json["payment-transaction"]["receiver"]
            .as_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Missing receiver in payment tx"))?
            .to_string()),
        TxnType::AssetTransfer => Ok(txn_json["asset-transfer-transaction"]["receiver"]
            .as_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Missing receiver in asset transfer tx"))?
            .to_string()),
        TxnType::AssetConfig => {
            // Asset config might not have a simple 'receiver' equivalent, using manager as placeholder
            Ok(txn_json
                .get("asset-config-transaction")
                .and_then(|acfg| acfg.get("params"))
                .and_then(|params| params.get("manager"))
                .and_then(Value::as_str)
                .unwrap_or("N/A") // Or some other indicator
                .to_string())
        }
        TxnType::AssetFreeze => Ok(txn_json["asset-freeze-transaction"]["address"]
            .as_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Missing address in asset freeze tx"))?
            .to_string()),
        TxnType::AppCall => {
            // Use application ID or created index as the 'target'
            let app_txn = txn_json
                .get("application-transaction")
                .ok_or_else(|| color_eyre::eyre::eyre!("Missing application-transaction field"))?;
            let app_id = app_txn["application-id"].as_u64().unwrap_or(0);
            if app_id > 0 {
                Ok(app_id.to_string())
            } else {
                Ok(txn_json["created-application-index"]
                    .as_u64()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "AppCreate".to_string())) // Indicate creation if ID is 0
            }
        }
        // Provide a default/placeholder for types without a clear receiver
        _ => Ok("N/A".to_string()),
    }
}

fn format_timestamp(timestamp_secs: u64) -> String {
    if timestamp_secs == 0 {
        return "N/A".to_string(); // Or "Timestamp Missing"
    }

    match chrono::DateTime::from_timestamp(timestamp_secs as i64, 0) {
        Some(datetime) => datetime.format("%a, %d %b %Y %H:%M:%S UTC").to_string(),
        None => {
            eprintln!(
                "Warning: Failed to parse timestamp from seconds: {}",
                timestamp_secs
            );
            "Invalid Timestamp".to_string()
        }
    }
}

fn count_transactions(block: &Value) -> u16 {
    if let Some(txns) = block.get("txns") {
        if let Some(arr) = txns.as_array() {
            return arr.len() as u16;
        } else if let Some(obj) = txns.as_object() {
            if let Some(transactions) = obj.get("transactions") {
                if let Some(arr) = transactions.as_array() {
                    return arr.len() as u16;
                }
            }
        }
    }
    0
}

/// Basic block information
#[derive(Debug, Clone, PartialEq)]
pub struct AlgoBlock {
    pub id: u64,
    pub txn_count: u16,
    pub timestamp: String,
}

/// Transaction type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TxnType {
    Payment,
    AppCall,
    AssetTransfer,
    AssetConfig,
    AssetFreeze,
    KeyReg,
    StateProof,
    Heartbeat,
    Unknown,
}

impl TxnType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Payment => "Payment",
            Self::AppCall => "App Call",
            Self::AssetTransfer => "Asset Transfer",
            Self::AssetConfig => "Asset Config",
            Self::AssetFreeze => "Asset Freeze",
            Self::KeyReg => "Key Registration",
            Self::StateProof => "State Proof",
            Self::Heartbeat => "Heartbeat",
            Self::Unknown => "Unknown",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Payment => Color::Green,
            Self::AppCall => Color::Blue,
            Self::AssetTransfer => Color::Yellow,
            Self::AssetConfig => Color::Cyan,
            Self::AssetFreeze => Color::Magenta,
            Self::KeyReg => Color::Red,
            Self::StateProof => Color::Gray,
            Self::Heartbeat => Color::White,
            Self::Unknown => Color::DarkGray,
        }
    }
}

/// Basic transaction information
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub txn_type: TxnType,
    pub from: String,
    pub to: String,
    pub timestamp: String,
    pub block: u64,
    pub fee: u64,
    pub note: String,
    pub amount: u64,
    pub asset_id: Option<u64>,
}

/// Detailed Block Information
#[derive(Debug, Clone, PartialEq)]
pub struct BlockInfo {
    pub id: u64,
    pub timestamp: String,
    pub txn_count: u16,
    pub proposer: String,
    pub seed: String,
}

/// Detailed Account Information
#[derive(Debug, Clone, PartialEq)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u64,         // In microAlgos
    pub pending_rewards: u64, // In microAlgos
    pub reward_base: u64,
    pub status: String,              // e.g., "Offline", "Online"
    pub assets_count: usize,         // Number of assets the account holds
    pub created_assets_count: usize, // Number of assets created by the account
}

/// Detailed Asset Information
#[derive(Debug, Clone, PartialEq)]
pub struct AssetInfo {
    pub id: u64,
    pub name: String,
    pub unit_name: String,
    pub creator: String,
    pub total: u64,    // Total supply
    pub decimals: u64, // For display formatting
    pub url: String,   // Optional URL for metadata
}

/// Enum to hold different types of search results
#[derive(Debug, Clone, PartialEq)]
pub enum SearchResultItem {
    Transaction(Transaction),
    Block(BlockInfo),
    Account(AccountInfo),
    Asset(AssetInfo),
}
