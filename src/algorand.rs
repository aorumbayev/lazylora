use crate::app_state::SearchType;
use color_eyre::Result;
use ratatui::style::Color;
use reqwest::Client;
use serde_json::Value;

// Network types
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Network {
    MainNet,
    TestNet,
    LocalNet,
}

impl Network {
    pub fn as_str(&self) -> &str {
        match self {
            Self::MainNet => "MainNet",
            Self::TestNet => "TestNet",
            Self::LocalNet => "LocalNet",
        }
    }

    pub fn indexer_url(&self) -> &str {
        match self {
            Self::MainNet => "https://mainnet-idx.algonode.cloud",
            Self::TestNet => "https://testnet-idx.algonode.cloud",
            Self::LocalNet => "http://localhost:8980",
        }
    }

    pub fn algod_url(&self) -> &str {
        match self {
            Self::MainNet => "https://mainnet-api.algonode.cloud",
            Self::TestNet => "https://testnet-api.algonode.cloud",
            Self::LocalNet => "http://localhost:8080",
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
        Self {
            network,
            client: Client::new(),
        }
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
            .map_err(|_| color_eyre::eyre::eyre!("Failed to fetch transaction"))?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let json: Value = response
            .json()
            .await
            .map_err(|_| color_eyre::eyre::eyre!("Failed to parse transaction JSON"))?;

        let txn_json = match json.get("transaction") {
            Some(txn) => txn,
            None => return Ok(None),
        };

        let id = txn_json["id"].as_str().unwrap_or("unknown").to_string();
        let txn_type = determine_transaction_type(txn_json);
        let from = txn_json["sender"].as_str().unwrap_or("unknown").to_string();
        let to = extract_receiver(txn_json, &txn_type);

        // Extract additional transaction details
        let timestamp = txn_json["round-time"]
            .as_u64()
            .map(format_timestamp)
            .unwrap_or_else(|| "Unknown".to_string());

        let block = txn_json["confirmed-round"].as_u64().unwrap_or(0);
        let fee = txn_json["fee"].as_u64().unwrap_or(0);

        let note = txn_json["note"]
            .as_str()
            .map(|n| n.to_string())
            .unwrap_or_else(|| {
                txn_json["note"]
                    .as_array()
                    .map(|bytes| format!("{:?}", bytes))
                    .unwrap_or_else(|| "None".to_string())
            });

        // Extract amount based on transaction type
        let (amount, asset_id) = match txn_type {
            TxnType::Payment => {
                let amount = txn_json["payment-transaction"]["amount"]
                    .as_u64()
                    .unwrap_or(0);
                (amount, None)
            }
            TxnType::AssetTransfer => {
                let amount = txn_json["asset-transfer-transaction"]["amount"]
                    .as_u64()
                    .unwrap_or(0);
                let asset_id = txn_json["asset-transfer-transaction"]["asset-id"].as_u64();
                (amount, asset_id)
            }
            _ => (0, None),
        };

        Ok(Some(Transaction {
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
        }))
    }

    pub async fn get_latest_blocks(&self, limit: usize) -> Result<Vec<AlgoBlock>> {
        let status_url = format!("{}/v2/status", self.network.algod_url());
        let status_response = self
            .client
            .get(&status_url)
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
        let status_response = match self
            .client
            .get(&status_url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => resp,
            _ => return Ok(Vec::new()),
        };

        let status: Value = match status_response.json().await {
            Ok(data) => data,
            Err(_) => return Ok(Vec::new()),
        };

        let latest_round = status["last-round"].as_u64().unwrap_or(0);
        if latest_round == 0 {
            return Ok(Vec::new());
        }

        let min_round = latest_round.saturating_sub(20);
        let url = format!(
            "{}/v2/transactions?limit={}&min-round={}&max-round={}&order=desc",
            self.network.indexer_url(),
            limit,
            min_round,
            latest_round
        );

        let response = match self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => resp,
            _ => return Ok(Vec::new()),
        };

        let json: Value = match response.json().await {
            Ok(data) => data,
            Err(_) => return Ok(Vec::new()),
        };

        let empty_vec = Vec::new();
        let transactions_array = json["transactions"].as_array().unwrap_or(&empty_vec);
        let mut transactions = Vec::with_capacity(transactions_array.len());

        for txn_json in transactions_array {
            let id = txn_json["id"].as_str().unwrap_or("unknown").to_string();
            let txn_type = determine_transaction_type(txn_json);
            let from = txn_json["sender"].as_str().unwrap_or("unknown").to_string();
            let to = extract_receiver(txn_json, &txn_type);

            // Extract additional transaction details
            let timestamp = txn_json["round-time"]
                .as_u64()
                .map(format_timestamp)
                .unwrap_or_else(|| "Unknown".to_string());

            let block = txn_json["confirmed-round"].as_u64().unwrap_or(0);
            let fee = txn_json["fee"].as_u64().unwrap_or(0);

            let note = txn_json["note"]
                .as_str()
                .map(|n| n.to_string())
                .unwrap_or_else(|| {
                    txn_json["note"]
                        .as_array()
                        .map(|bytes| format!("{:?}", bytes))
                        .unwrap_or_else(|| "None".to_string())
                });

            // Extract amount based on transaction type
            let (amount, asset_id) = match txn_type {
                TxnType::Payment => {
                    let amount = txn_json["payment-transaction"]["amount"]
                        .as_u64()
                        .unwrap_or(0);
                    (amount, None)
                }
                TxnType::AssetTransfer => {
                    let amount = txn_json["asset-transfer-transaction"]["amount"]
                        .as_u64()
                        .unwrap_or(0);
                    let asset_id = txn_json["asset-transfer-transaction"]["asset-id"].as_u64();
                    (amount, asset_id)
                }
                _ => (0, None),
            };

            transactions.push(Transaction {
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
            });
        }

        transactions.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(transactions)
    }

    /// Search for blocks, assets, accounts, and transactions based on the input query
    pub async fn search_by_query(
        &self,
        query: &str,
        search_type: SearchType,
    ) -> Result<Vec<SearchResultItem>> {
        // Execute the appropriate search based on search type
        let results = match search_type {
            SearchType::Transaction => {
                let txns = self.search_transaction(query).await?;
                // Wrap Transaction in SearchResultItem::Transaction
                txns.into_iter()
                    .map(SearchResultItem::Transaction)
                    .collect()
            }
            SearchType::Account => {
                self.search_address(query)
                    .await?
                    .map(SearchResultItem::Account) // Wrap AccountInfo
                    .map_or_else(
                        || Ok::<_, color_eyre::Report>(vec![]),
                        |item| Ok(vec![item]),
                    )?
            }
            SearchType::Block => {
                self.search_block(query)
                    .await?
                    .map(SearchResultItem::Block) // Wrap BlockInfo
                    .map_or_else(
                        || Ok::<_, color_eyre::Report>(vec![]),
                        |item| Ok(vec![item]),
                    )?
            }
            SearchType::Asset => {
                self.search_asset(query)
                    .await?
                    .map(SearchResultItem::Asset) // Wrap AssetInfo
                    .map_or_else(
                        || Ok::<_, color_eyre::Report>(vec![]),
                        |item| Ok(vec![item]),
                    )?
            }
        };

        // Deduplication might need adjustment depending on how we identify unique items across types
        // For now, we assume the search functions return unique primary entities.
        // If they return related transactions, that logic needs rethinking.
        // The current search functions seem to return a single "primary" entity
        // represented as a Transaction, which is what we are fixing.

        // Let's remove the old deduplication logic as it was based on Transaction IDs.
        // let mut unique_items = Vec::new();
        // let mut seen_ids = HashSet::new(); // This needs a more complex ID strategy for different types

        // for item in results {
        //     // Generate a unique ID based on item type and its primary identifier
        //     let unique_id = match &item {
        //         SearchResultItem::Transaction(t) => format!("txn-{}", t.id),
        //         SearchResultItem::Account(a) => format!("acc-{}", a.address),
        //         SearchResultItem::Block(b) => format!("blk-{}", b.id),
        //         SearchResultItem::Asset(a) => format!("asset-{}", a.id),
        //     };
        //     if seen_ids.insert(unique_id) {
        //         unique_items.push(item);
        //     }
        // }

        Ok(results) // Return the direct results for now
    }

    // Search for a specific transaction by ID
    async fn search_transaction(&self, txid: &str) -> Result<Vec<Transaction>> {
        // Direct transaction lookup without any format validation
        let url = format!("{}/v2/transactions/{}", self.network.indexer_url(), txid);

        match self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(json) = response.json::<Value>().await {
                    if let Some(txn_json) = json.get("transaction") {
                        let id = txn_json["id"].as_str().unwrap_or("unknown").to_string();
                        let txn_type = determine_transaction_type(txn_json);
                        let from = txn_json["sender"].as_str().unwrap_or("unknown").to_string();
                        let to = extract_receiver(txn_json, &txn_type);

                        let timestamp = txn_json["round-time"]
                            .as_u64()
                            .map(format_timestamp)
                            .unwrap_or_else(|| "Unknown".to_string());

                        let block = txn_json["confirmed-round"].as_u64().unwrap_or(0);
                        let fee = txn_json["fee"].as_u64().unwrap_or(0);

                        let note = txn_json["note"]
                            .as_str()
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| {
                                txn_json["note"]
                                    .as_array()
                                    .map(|bytes| format!("{:?}", bytes))
                                    .unwrap_or_else(|| "None".to_string())
                            });

                        let (amount, asset_id) = match txn_type {
                            TxnType::Payment => {
                                let amount = txn_json["payment-transaction"]["amount"]
                                    .as_u64()
                                    .unwrap_or(0);
                                (amount, None)
                            }
                            TxnType::AssetTransfer => {
                                let amount = txn_json["asset-transfer-transaction"]["amount"]
                                    .as_u64()
                                    .unwrap_or(0);
                                let asset_id =
                                    txn_json["asset-transfer-transaction"]["asset-id"].as_u64();
                                (amount, asset_id)
                            }
                            _ => (0, None),
                        };

                        return Ok(vec![Transaction {
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
                        }]);
                    }
                }
            }
            _ => {}
        }

        // Secondary search for transactions
        let search_url = format!(
            "{}/v2/transactions?tx-id={}&limit=10",
            self.network.indexer_url(),
            txid
        );

        let search_results = self.fetch_transactions_from_url(&search_url).await?;

        Ok(search_results)
    }

    // Search for a block
    async fn search_block(&self, round_str: &str) -> Result<Option<BlockInfo>> {
        if let Ok(round) = round_str.parse::<u64>() {
            // Check if the block exists
            let block_url = format!("{}/v2/blocks/{}", self.network.algod_url(), round);

            match self
                .client
                .get(&block_url)
                .header("accept", "application/json")
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    // Get block info
                    let block_data: Value = response.json().await.unwrap_or_default();
                    let block_val = block_data.get("block").unwrap_or(&block_data);
                    let txn_count = count_transactions(block_val);
                    let timestamp_secs = block_val["ts"].as_u64().unwrap_or(0);
                    let formatted_time = format_timestamp(timestamp_secs);
                    let proposer = block_val["cert"]["prop"]["addr"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string();
                    let seed = block_val["seed"].as_str().unwrap_or("unknown").to_string();

                    // Create BlockInfo
                    Ok(Some(BlockInfo {
                        id: round,
                        timestamp: formatted_time,
                        txn_count,
                        proposer,
                        seed,
                    }))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    // Search for an asset
    async fn search_asset(&self, asset_id_str: &str) -> Result<Option<AssetInfo>> {
        if let Ok(asset_id) = asset_id_str.parse::<u64>() {
            // Check if the asset exists
            let asset_url = format!("{}/v2/assets/{}", self.network.indexer_url(), asset_id);

            match self
                .client
                .get(&asset_url)
                .header("accept", "application/json")
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    // Get asset info
                    let asset_data: Value = response.json().await.unwrap_or_default();
                    let params = &asset_data["asset"]["params"];

                    let name = params["name"].as_str().unwrap_or("").to_string();
                    let unit_name = params["unit-name"].as_str().unwrap_or("").to_string();
                    let creator = params["creator"].as_str().unwrap_or("unknown").to_string();
                    let total = params["total"].as_u64().unwrap_or(0);
                    let decimals = params["decimals"].as_u64().unwrap_or(0);
                    let url = params["url"].as_str().unwrap_or("").to_string();

                    // Create AssetInfo
                    Ok(Some(AssetInfo {
                        id: asset_id,
                        name,
                        unit_name,
                        creator,
                        total,
                        decimals,
                        url,
                    }))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    // Search for an account
    async fn search_address(&self, address: &str) -> Result<Option<AccountInfo>> {
        // Direct account lookup
        let account_url = format!("{}/v2/accounts/{}", self.network.indexer_url(), address);

        match self
            .client
            .get(&account_url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                // Get account info
                let account_data: Value = response.json().await.unwrap_or_default();
                let account = &account_data["account"];

                let balance = account["amount"].as_u64().unwrap_or(0);
                let pending_rewards = account["pending-rewards"].as_u64().unwrap_or(0);
                let reward_base = account["reward-base"].as_u64().unwrap_or(0);
                let status = account["status"].as_str().unwrap_or("unknown").to_string();

                // Extract assets count
                let assets_count = account["assets"]
                    .as_array()
                    .map_or(0, |assets| assets.len());

                // Extract created assets count
                let created_assets_count = account["created-assets"]
                    .as_array()
                    .map_or(0, |assets| assets.len());

                // Create AccountInfo
                Ok(Some(AccountInfo {
                    address: address.to_string(),
                    balance,
                    pending_rewards,
                    reward_base,
                    status,
                    assets_count,
                    created_assets_count,
                }))
            }
            _ => Ok(None),
        }
    }

    // Helper function to fetch transactions from a URL
    async fn fetch_transactions_from_url(&self, url: &str) -> Result<Vec<Transaction>> {
        let response = match self
            .client
            .get(url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => resp,
            _ => return Ok(Vec::new()),
        };

        let json: Value = match response.json().await {
            Ok(data) => data,
            Err(_) => return Ok(Vec::new()),
        };

        let empty_vec = Vec::new();
        let transactions_array = json["transactions"].as_array().unwrap_or(&empty_vec);
        let mut transactions = Vec::with_capacity(transactions_array.len());

        for txn_json in transactions_array {
            let id = txn_json["id"].as_str().unwrap_or("unknown").to_string();
            let txn_type = determine_transaction_type(txn_json);
            let from = txn_json["sender"].as_str().unwrap_or("unknown").to_string();
            let to = extract_receiver(txn_json, &txn_type);

            // Extract additional transaction details
            let timestamp = txn_json["round-time"]
                .as_u64()
                .map(format_timestamp)
                .unwrap_or_else(|| "Unknown".to_string());

            let block = txn_json["confirmed-round"].as_u64().unwrap_or(0);
            let fee = txn_json["fee"].as_u64().unwrap_or(0);

            let note = txn_json["note"]
                .as_str()
                .map(|n| n.to_string())
                .unwrap_or_else(|| {
                    txn_json["note"]
                        .as_array()
                        .map(|bytes| format!("{:?}", bytes))
                        .unwrap_or_else(|| "None".to_string())
                });

            // Extract amount based on transaction type
            let (amount, asset_id) = match txn_type {
                TxnType::Payment => {
                    let amount = txn_json["payment-transaction"]["amount"]
                        .as_u64()
                        .unwrap_or(0);
                    (amount, None)
                }
                TxnType::AssetTransfer => {
                    let amount = txn_json["asset-transfer-transaction"]["amount"]
                        .as_u64()
                        .unwrap_or(0);
                    let asset_id = txn_json["asset-transfer-transaction"]["asset-id"].as_u64();
                    (amount, asset_id)
                }
                _ => (0, None),
            };

            transactions.push(Transaction {
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
            });
        }

        Ok(transactions)
    }
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

fn extract_receiver(txn_json: &Value, txn_type: &TxnType) -> String {
    match txn_type {
        TxnType::Payment => txn_json["payment-transaction"]["receiver"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        TxnType::AssetTransfer => txn_json["asset-transfer-transaction"]["receiver"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        TxnType::AssetConfig => {
            if txn_json["asset-config-transaction"]["params"].is_object() {
                txn_json["asset-config-transaction"]["params"]["manager"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                "unknown".to_string()
            }
        }
        TxnType::AssetFreeze => txn_json["asset-freeze-transaction"]["address"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        TxnType::AppCall => {
            if txn_json["application-transaction"]["application-id"]
                .as_u64()
                .unwrap_or(0)
                > 0
            {
                txn_json["application-transaction"]["application-id"].to_string()
            } else {
                txn_json["created-application-index"].to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

fn format_timestamp(timestamp_secs: u64) -> String {
    if timestamp_secs == 0 {
        return "Timestamp not available".to_string();
    }

    let datetime =
        chrono::DateTime::from_timestamp(timestamp_secs as i64, 0).unwrap_or_else(chrono::Utc::now);

    datetime.format("%a, %d %b %Y %H:%M:%S").to_string()
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
