use color_eyre::Result;
use reqwest::Client;
use serde_json::Value;

// Network types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Network {
    MainNet,
    TestNet,
    LocalNet,
}

impl Network {
    pub fn as_str(&self) -> &str {
        match self {
            Network::MainNet => "MainNet",
            Network::TestNet => "TestNet",
            Network::LocalNet => "LocalNet",
        }
    }

    pub fn indexer_url(&self) -> &str {
        match self {
            Network::MainNet => "https://mainnet-idx.algonode.cloud",
            Network::TestNet => "https://testnet-idx.algonode.cloud",
            Network::LocalNet => "http://localhost:8980",
        }
    }

    pub fn algod_url(&self) -> &str {
        match self {
            Network::MainNet => "https://mainnet-api.algonode.cloud",
            Network::TestNet => "https://testnet-api.algonode.cloud",
            Network::LocalNet => "http://localhost:8080",
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

    pub async fn get_latest_blocks(&self, limit: usize) -> Result<Vec<AlgoBlock>> {
        // Get the current status to find the latest round
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

        let mut blocks = Vec::new();

        // Fetch blocks starting from the latest round and work backwards
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
                Ok(resp) => resp,
                Err(_) => continue,
            };

            if !response.status().is_success() {
                continue;
            }

            let block_data: Value = match response.json().await {
                Ok(data) => data,
                Err(_) => continue,
            };

            // Block structure has outer "block" object in algod response
            let block = match block_data.get("block") {
                Some(block) => block,
                None => &block_data, // Fallback to the entire object if "block" key not found
            };

            // Get timestamp and convert from unix timestamp to readable format
            let timestamp_secs = block["ts"].as_u64().unwrap_or(0);

            // Algorand timestamps are in seconds since Unix epoch
            // If the timestamp is 0, we'll use a fallback message
            let formatted_time = if timestamp_secs == 0 {
                "Timestamp not available".to_string()
            } else {
                // Add the timestamp for the genesis block of Algorand (June 11, 2019)
                // This ensures we don't get January 1, 1970
                let genesis_timestamp: i64 = 1560211200; // June 11, 2019
                let datetime = chrono::DateTime::from_timestamp(timestamp_secs as i64, 0)
                    .unwrap_or_else(|| {
                        chrono::DateTime::from_timestamp(genesis_timestamp, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                    });
                datetime.format("%a, %d %b %Y %H:%M:%S").to_string()
            };

            // Get transaction count - transactions are in the "txns" array
            let txn_count = if let Some(txns) = block.get("txns") {
                if txns.is_array() {
                    txns.as_array().unwrap().len() as u16
                } else if txns.is_object() {
                    // Some endpoints nest transactions in an object with a "transactions" array
                    txns.get("transactions")
                        .and_then(|t| t.as_array())
                        .map_or(0, |arr| arr.len()) as u16
                } else {
                    0
                }
            } else {
                0
            };

            blocks.push(AlgoBlock {
                id: round,
                txn_count,
                timestamp: formatted_time,
            });
        }

        Ok(blocks)
    }

    pub async fn get_latest_transactions(&self, limit: usize) -> Result<Vec<Transaction>> {
        // First, get the current status to find the latest round
        let status_url = format!("{}/v2/status", self.network.algod_url());
        let status_response = match self
            .client
            .get(&status_url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return Ok(Vec::new());
                }
                resp
            }
            Err(_) => return Ok(Vec::new()),
        };

        let status: Value = match status_response.json().await {
            Ok(data) => data,
            Err(_) => return Ok(Vec::new()),
        };

        let latest_round = status["last-round"].as_u64().unwrap_or(0);
        if latest_round == 0 {
            return Ok(Vec::new());
        }

        // Calculate round range - we want the most recent rounds
        // Don't use too large of a range to avoid long queries
        // typically we just need the last 10-20 blocks
        let min_round = if latest_round > 20 {
            latest_round - 20
        } else {
            0
        };

        // Configure URL to get transactions from recent rounds
        let url = format!(
            "{}/v2/transactions?limit={}&min-round={}&max-round={}&order=desc",
            self.network.indexer_url(),
            limit,
            min_round,
            latest_round
        );

        // Make the request
        let response = match self
            .client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return Ok(Vec::new());
                }
                resp
            }
            Err(_) => return Ok(Vec::new()),
        };

        // Parse response
        let json: Value = match response.json().await {
            Ok(data) => data,
            Err(_) => return Ok(Vec::new()),
        };

        // Parse the transactions
        let empty_vec = Vec::new();
        let transactions_array = json["transactions"].as_array().unwrap_or(&empty_vec);
        let mut transactions = Vec::new();

        // Process the first few transactions
        for (i, txn_json) in transactions_array.iter().enumerate() {
            if i < 3 {
                // Log transaction ID and round to help debug
                let _txid = txn_json["id"].as_str().unwrap_or("unknown");
                let _round = txn_json["confirmed-round"].as_u64().unwrap_or(0);
            }

            // Get transaction ID
            let id = txn_json["id"].as_str().unwrap_or("unknown").to_string();

            // Determine transaction type
            let txn_type = if txn_json["payment-transaction"].is_object() {
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
            };

            // Extract sender address
            let from = txn_json["sender"].as_str().unwrap_or("unknown").to_string();

            // Extract receiver address based on transaction type
            let to = if txn_type == TxnType::Payment {
                txn_json["payment-transaction"]["receiver"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string()
            } else if txn_type == TxnType::AssetTransfer {
                txn_json["asset-transfer-transaction"]["receiver"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string()
            } else if txn_type == TxnType::AssetConfig
                && txn_json["asset-config-transaction"]["params"].is_object()
            {
                txn_json["asset-config-transaction"]["params"]["manager"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string()
            } else if txn_type == TxnType::AssetFreeze {
                txn_json["asset-freeze-transaction"]["address"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string()
            } else if txn_type == TxnType::AppCall {
                let app_id = if txn_json["application-transaction"]["application-id"]
                    .as_u64()
                    .unwrap_or(0)
                    > 0
                {
                    txn_json["application-transaction"]["application-id"].to_string()
                } else {
                    txn_json["created-application-index"].to_string()
                };
                app_id
            } else {
                "unknown".to_string()
            };

            transactions.push(Transaction {
                id,
                txn_type,
                from,
                to,
            });
        }

        // Make sure the transactions are in the newest-first order (higher rounds first)
        transactions.sort_by(|a, b| b.id.cmp(&a.id));

        Ok(transactions)
    }
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
            TxnType::Payment => "Payment",
            TxnType::AppCall => "App Call",
            TxnType::AssetTransfer => "Asset Transfer",
            TxnType::AssetConfig => "Asset Config",
            TxnType::AssetFreeze => "Asset Freeze",
            TxnType::KeyReg => "Key Registration",
            TxnType::StateProof => "State Proof",
            TxnType::Heartbeat => "Heartbeat",
            TxnType::Unknown => "Unknown",
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        match self {
            TxnType::Payment => ratatui::style::Color::Green,
            TxnType::AppCall => ratatui::style::Color::Blue,
            TxnType::AssetTransfer => ratatui::style::Color::Yellow,
            TxnType::AssetConfig => ratatui::style::Color::Cyan,
            TxnType::AssetFreeze => ratatui::style::Color::Magenta,
            TxnType::KeyReg => ratatui::style::Color::Red,
            TxnType::StateProof => ratatui::style::Color::Gray,
            TxnType::Heartbeat => ratatui::style::Color::White,
            TxnType::Unknown => ratatui::style::Color::DarkGray,
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
}
