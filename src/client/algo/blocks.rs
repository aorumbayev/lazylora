//! Block fetching methods for AlgoClient.

use color_eyre::Result;
use serde_json::Value;
use tokio::task::JoinSet;

use super::AlgoClient;
use crate::domain::{
    AlgoBlock, AlgoError, BlockDetails, BlockInfo, count_transactions, format_timestamp,
};

impl AlgoClient {
    /// Fetch the latest blocks from the network
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails.
    pub async fn get_latest_blocks(&self, limit: usize) -> Result<Vec<AlgoBlock>> {
        let status_url = format!("{}/v2/status", self.algod_url);
        let status_response = self.build_algod_request(&status_url).send().await?;

        let status: Value = status_response.json().await?;
        let latest_round = status["last-round"].as_u64().ok_or_else(|| {
            AlgoError::parse("algod status response missing 'last-round'").into_report()
        })?;

        if latest_round == 0 {
            return Ok(Vec::new());
        }

        // Fetch blocks in parallel using JoinSet (std lib over external crate)
        let mut join_set = JoinSet::new();
        let num_blocks = limit.min(latest_round as usize);

        for round in (0..num_blocks).map(|i| latest_round - i as u64) {
            let block_url = format!("{}/v2/blocks/{}", self.algod_url, round);
            let request = self.build_algod_request(&block_url).send();

            join_set.spawn(async move {
                let response = request
                    .await
                    .inspect_err(|e| tracing::debug!("Block {round} fetch failed: {e}"))
                    .ok()?
                    .error_for_status()
                    .inspect_err(|e| tracing::debug!("Block {round} HTTP error: {e}"))
                    .ok()?;
                let block_data: Value = response
                    .json()
                    .await
                    .inspect_err(|e| tracing::debug!("Block {round} JSON parse error: {e}"))
                    .ok()?;

                let block = block_data.get("block").unwrap_or(&block_data);
                let timestamp_secs = block["ts"].as_u64().unwrap_or(0);
                let formatted_time = format_timestamp(timestamp_secs);
                let txn_count = count_transactions(block);

                Some(AlgoBlock {
                    id: round,
                    txn_count,
                    timestamp: formatted_time,
                })
            });
        }

        // Collect results using iterator chain (iterators over manual loops)
        let mut blocks: Vec<AlgoBlock> = join_set.join_all().await.into_iter().flatten().collect();

        // Sort by block ID descending (newest first)
        blocks.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(blocks)
    }

    /// Search for a block by round number.
    pub(crate) async fn search_block(&self, round_str: &str) -> Result<Option<BlockInfo>> {
        let round = round_str.parse::<u64>().map_err(|_| {
            AlgoError::invalid_input(format!(
                "Invalid block number '{}'. Please enter a valid positive integer.",
                round_str
            ))
            .into_report()
        })?;

        let block_url = format!("{}/v2/blocks/{}", self.algod_url, round);

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

        // Try multiple paths: 'prp' (algod v2), 'proposer' (indexer), 'cert.prop.addr' (legacy)
        let proposer = block_val["prp"]
            .as_str()
            .or_else(|| block_val["proposer"].as_str())
            .or_else(|| block_val["cert"]["prop"]["addr"].as_str())
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

    /// Get detailed block information including all transactions
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or parsing fails.
    pub async fn get_block_details(&self, round: u64) -> Result<Option<BlockDetails>> {
        use super::transactions::parse_transactions_array;

        // First, get the basic block info
        let block_info = match self.search_block(&round.to_string()).await? {
            Some(info) => info,
            None => return Ok(None),
        };

        // Fetch transactions for this round using the indexer
        let txns_url = format!("{}/v2/transactions?round={}", self.indexer_url, round);

        let response = self.build_indexer_request(&txns_url).send().await?;

        let transactions = if response.status().is_success() {
            let json: Value = response.json().await?;
            parse_transactions_array(&json)?
        } else {
            // If we can't get transactions, return empty list
            Vec::new()
        };

        // Compute transaction type counts
        let mut txn_type_counts = std::collections::HashMap::new();
        for txn in &transactions {
            *txn_type_counts.entry(txn.txn_type).or_insert(0) += 1;
        }

        Ok(Some(BlockDetails {
            info: block_info,
            transactions,
            txn_type_counts,
        }))
    }
}
