use crate::app_state::SearchType;
use color_eyre::Result;
use ratatui::style::Color;
use reqwest::Client;
use serde_json::Value;

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
            Self::LocalNet => "http://localhost:4001",
        }
    }
}

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

    fn build_algod_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.get(url).header("accept", "application/json");

        if self.network == Network::LocalNet {
            request = request.header(
                "X-Algo-API-Token",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            );
        }

        request
    }

    fn build_indexer_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.get(url).header("accept", "application/json");

        if self.network == Network::LocalNet {
            request = request.header(
                "X-Indexer-API-Token",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            );
        }

        request
    }

    pub async fn get_network_status(&self) -> Result<(), String> {
        let algod_url = format!("{}/health", self.network.algod_url());
        let indexer_url = format!("{}/health", self.network.indexer_url());

        let algod_result = self
            .build_algod_request(&algod_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;

        let indexer_result = self
            .build_indexer_request(&indexer_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;

        if let Err(e) = algod_result {
            return Err(format!(
                "Unable to connect to algod at {}. Error: {}",
                self.network.algod_url(),
                e
            ));
        }

        if self.network == Network::LocalNet && indexer_result.is_err() {
            return Err(format!(
                "Unable to connect to indexer at {}. Algod is running but indexer is not available.",
                self.network.indexer_url()
            ));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_transaction_by_id(&self, txid: &str) -> Result<Option<Transaction>> {
        let url = format!("{}/v2/transactions/{}", self.network.indexer_url(), txid);
        let response = self
            .build_indexer_request(&url)
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
        let status_response = self.build_algod_request(&status_url).send().await?;

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

            let response = match self.build_algod_request(&block_url).send().await {
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
        let status_response = match self.build_algod_request(&status_url).send().await {
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

        let response = match self.build_indexer_request(&url).send().await {
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

    pub async fn search_by_query(
        &self,
        query: &str,
        search_type: SearchType,
    ) -> Result<Vec<SearchResultItem>> {
        let results = match search_type {
            SearchType::Transaction => {
                let txns = self.search_transaction(query).await?;
                txns.into_iter()
                    .map(SearchResultItem::Transaction)
                    .collect()
            }
            SearchType::Account => match self.search_address(query).await {
                Ok(Some(account)) => {
                    vec![SearchResultItem::Account(account)]
                }
                Ok(None) => {
                    vec![]
                }
                Err(e) => {
                    return Err(e);
                }
            },
            SearchType::Block => match self.search_block(query).await? {
                Some(block) => vec![SearchResultItem::Block(block)],
                None => {
                    return Err(color_eyre::eyre::eyre!(
                        "Block '{}' not found. Please enter a valid block number.",
                        query
                    ));
                }
            },
            SearchType::Asset => match self.search_asset(query).await? {
                Some(asset) => vec![SearchResultItem::Asset(asset)],
                None => {
                    return Err(color_eyre::eyre::eyre!(
                        "Asset '{}' not found. Please enter a valid asset ID.",
                        query
                    ));
                }
            },
        };

        Ok(results)
    }

    async fn search_transaction(&self, txid: &str) -> Result<Vec<Transaction>> {
        if txid.is_empty() {
            return Err(color_eyre::eyre::eyre!("Transaction ID cannot be empty"));
        }

        if txid.len() < 40 || txid.len() > 60 {
            return Err(color_eyre::eyre::eyre!(
                "Invalid transaction ID format. Transaction IDs are typically 52 characters long."
            ));
        }

        let url = format!("{}/v2/transactions/{}", self.network.indexer_url(), txid);

        let response = self.build_indexer_request(&url).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(txn_json) = json.get("transaction") {
                        let transaction = self.parse_transaction(txn_json)?;
                        return Ok(vec![transaction]);
                    }
                }
            }
            Ok(resp) => {
                let status = resp.status();
                if status.as_u16() != 404 {
                    // Log non-404 errors silently, continue with search
                }
            }
            Err(_) => {
                // Log error silently, continue with search
            }
        }

        let search_url = format!(
            "{}/v2/transactions?txid={}&limit=10",
            self.network.indexer_url(),
            txid
        );

        let search_results = self.fetch_transactions_from_url(&search_url).await?;

        if search_results.is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "Transaction '{}' not found. Please verify the transaction ID is correct and exists on the {} network.",
                txid,
                self.network.as_str()
            ));
        }

        Ok(search_results)
    }

    fn parse_transaction(&self, txn_json: &Value) -> Result<Transaction> {
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
                let asset_id = txn_json["asset-transfer-transaction"]["asset-id"].as_u64();
                (amount, asset_id)
            }
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

    async fn search_block(&self, round_str: &str) -> Result<Option<BlockInfo>> {
        let round = round_str.parse::<u64>().map_err(|_| {
            color_eyre::eyre::eyre!(
                "Invalid block number '{}'. Please enter a valid positive integer.",
                round_str
            )
        })?;

        let block_url = format!("{}/v2/blocks/{}", self.network.algod_url(), round);

        let response = self.build_algod_request(&block_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 404 {
                return Ok(None);
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "Failed to fetch block #{}: HTTP {} - {}",
                    round,
                    status,
                    error_text
                ));
            }
        }

        let block_data: Value = response.json().await?;
        let block_val = block_data.get("block").unwrap_or(&block_data);

        let txn_count = count_transactions(block_val);
        let timestamp_secs = block_val["ts"].as_u64().unwrap_or(0);
        let formatted_time = format_timestamp(timestamp_secs);

        let proposer = block_val["cert"]["prop"]["addr"]
            .as_str()
            .or_else(|| block_val["proposer"].as_str())
            .unwrap_or("unknown")
            .to_string();

        let seed = block_val["seed"].as_str().unwrap_or("unknown").to_string();

        Ok(Some(BlockInfo {
            id: round,
            timestamp: formatted_time,
            txn_count,
            proposer,
            seed,
        }))
    }

    async fn search_asset(&self, asset_id_str: &str) -> Result<Option<AssetInfo>> {
        let asset_id = asset_id_str.parse::<u64>().map_err(|_| {
            color_eyre::eyre::eyre!(
                "Invalid asset ID '{}'. Please enter a valid positive integer.",
                asset_id_str
            )
        })?;

        let asset_url = format!("{}/v2/assets/{}", self.network.indexer_url(), asset_id);

        let response = self.build_indexer_request(&asset_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 404 {
                return Ok(None);
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "Failed to fetch asset #{}: HTTP {} - {}",
                    asset_id,
                    status,
                    error_text
                ));
            }
        }

        let asset_data: Value = response.json().await?;
        let params = &asset_data["asset"]["params"];

        let name = params["name"].as_str().unwrap_or("").to_string();
        let unit_name = params["unit-name"].as_str().unwrap_or("").to_string();
        let creator = params["creator"].as_str().unwrap_or("unknown").to_string();
        let total = params["total"].as_u64().unwrap_or(0);
        let decimals = params["decimals"].as_u64().unwrap_or(0);
        let url = params["url"].as_str().unwrap_or("").to_string();

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

    async fn search_address(&self, address: &str) -> Result<Option<AccountInfo>> {
        if address.len() != 58
            || !address
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return Err(color_eyre::eyre::eyre!(
                "Invalid Algorand address format. Address must be 58 characters long and contain only uppercase letters and numbers."
            ));
        }

        let indexer_result = self.search_address_via_indexer(address).await;

        match indexer_result {
            Ok(Some(account)) => {
                return Ok(Some(account));
            }
            Ok(None) => {
                // Try algod as fallback
            }
            Err(_) => {
                // Try algod as fallback
            }
        }

        let algod_result = self.search_address_via_algod(address).await;

        match algod_result {
            Ok(Some(account)) => Ok(Some(account)),
            Ok(None) => Err(color_eyre::eyre::eyre!(
                "Account '{}' not found. Please verify the address is correct and the account exists on the {} network.",
                address,
                self.network.as_str()
            )),
            Err(e) => Err(color_eyre::eyre::eyre!(
                "Failed to fetch account information for '{}': {}",
                address,
                e
            )),
        }
    }

    async fn search_address_via_indexer(&self, address: &str) -> Result<Option<AccountInfo>> {
        let account_url = format!("{}/v2/accounts/{}", self.network.indexer_url(), address);

        let response = self.build_indexer_request(&account_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 404 {
                return Ok(None);
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "Indexer request failed with status {}: {}",
                    status,
                    error_text
                ));
            }
        }

        let account_data: Value = response.json().await?;

        if let Some(account) = account_data.get("account") {
            Ok(Some(self.parse_account_info(account, address)))
        } else {
            Err(color_eyre::eyre::eyre!("Invalid indexer response format"))
        }
    }

    async fn search_address_via_algod(&self, address: &str) -> Result<Option<AccountInfo>> {
        let account_url = format!("{}/v2/accounts/{}", self.network.algod_url(), address);

        let response = self.build_algod_request(&account_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 404 {
                return Ok(None);
            } else {
                return Err(color_eyre::eyre::eyre!(
                    "Algod request failed with status {}: {}",
                    status,
                    error_text
                ));
            }
        }

        let account_data: Value = response.json().await?;

        Ok(Some(self.parse_account_info(&account_data, address)))
    }

    fn parse_account_info(&self, account: &Value, address: &str) -> AccountInfo {
        let balance = account["amount"].as_u64().unwrap_or(0);
        let pending_rewards = account["pending-rewards"].as_u64().unwrap_or(0);
        let reward_base = account["reward-base"].as_u64().unwrap_or(0);
        let status = account["status"].as_str().unwrap_or("unknown").to_string();

        let assets_count = account["assets"]
            .as_array()
            .map_or(0, |assets| assets.len());

        let created_assets_count = account["created-assets"]
            .as_array()
            .map_or(0, |assets| assets.len());

        AccountInfo {
            address: address.to_string(),
            balance,
            pending_rewards,
            reward_base,
            status,
            assets_count,
            created_assets_count,
        }
    }

    async fn fetch_transactions_from_url(&self, url: &str) -> Result<Vec<Transaction>> {
        let response = match self.build_indexer_request(url).send().await {
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

    pub fn get_search_suggestions(query: &str, search_type: SearchType) -> String {
        let trimmed = query.trim();

        match search_type {
            SearchType::Account => {
                if trimmed.is_empty() {
                    "Enter an Algorand address (58 characters, uppercase letters and numbers)"
                        .to_string()
                } else if trimmed.len() < 58 {
                    format!(
                        "Address too short ({} chars). Algorand addresses are 58 characters long.",
                        trimmed.len()
                    )
                } else if trimmed.len() > 58 {
                    format!(
                        "Address too long ({} chars). Algorand addresses are 58 characters long.",
                        trimmed.len()
                    )
                } else if !trimmed
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                {
                    "Address contains invalid characters. Use only uppercase letters and numbers."
                        .to_string()
                } else {
                    "Valid address format. Press Enter to search.".to_string()
                }
            }
            SearchType::Transaction => {
                if trimmed.is_empty() {
                    "Enter a transaction ID (typically 52 characters)".to_string()
                } else if trimmed.len() < 40 {
                    format!(
                        "Transaction ID too short ({} chars). Most transaction IDs are 52 characters.",
                        trimmed.len()
                    )
                } else if trimmed.len() > 60 {
                    format!(
                        "Transaction ID too long ({} chars). Most transaction IDs are 52 characters.",
                        trimmed.len()
                    )
                } else {
                    "Valid transaction ID format. Press Enter to search.".to_string()
                }
            }
            SearchType::Block => {
                if trimmed.is_empty() {
                    "Enter a block number (positive integer)".to_string()
                } else if trimmed.parse::<u64>().is_err() {
                    "Block number must be a positive integer".to_string()
                } else {
                    "Valid block number. Press Enter to search.".to_string()
                }
            }
            SearchType::Asset => {
                if trimmed.is_empty() {
                    "Enter an asset ID (positive integer)".to_string()
                } else if trimmed.parse::<u64>().is_err() {
                    "Asset ID must be a positive integer".to_string()
                } else {
                    "Valid asset ID. Press Enter to search.".to_string()
                }
            }
        }
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

#[derive(Debug, Clone, PartialEq)]
pub struct AlgoBlock {
    pub id: u64,
    pub txn_count: u16,
    pub timestamp: String,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct BlockInfo {
    pub id: u64,
    pub timestamp: String,
    pub txn_count: u16,
    pub proposer: String,
    pub seed: String,
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum SearchResultItem {
    Transaction(Transaction),
    Block(BlockInfo),
    Account(AccountInfo),
    Asset(AssetInfo),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_suggestions() {
        assert!(
            AlgoClient::get_search_suggestions("", SearchType::Account)
                .contains("Enter an Algorand address")
        );

        assert!(
            AlgoClient::get_search_suggestions("ABC", SearchType::Account).contains("too short")
        );

        assert!(
            AlgoClient::get_search_suggestions(
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                SearchType::Account
            )
            .contains("Valid address format")
        );

        assert!(
            AlgoClient::get_search_suggestions("", SearchType::Transaction)
                .contains("Enter a transaction ID")
        );

        assert!(
            AlgoClient::get_search_suggestions("ABC", SearchType::Transaction)
                .contains("too short")
        );

        assert!(
            AlgoClient::get_search_suggestions("", SearchType::Block)
                .contains("Enter a block number")
        );

        assert!(
            AlgoClient::get_search_suggestions("123456", SearchType::Block)
                .contains("Valid block number")
        );

        assert!(
            AlgoClient::get_search_suggestions("abc", SearchType::Block)
                .contains("must be a positive integer")
        );

        assert!(
            AlgoClient::get_search_suggestions("", SearchType::Asset).contains("Enter an asset ID")
        );

        assert!(
            AlgoClient::get_search_suggestions("123", SearchType::Asset).contains("Valid asset ID")
        );
    }
}
