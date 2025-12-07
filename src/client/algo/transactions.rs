//! Transaction fetching and parsing methods for AlgoClient.

use color_eyre::Result;
use serde_json::Value;

use super::AlgoClient;
use crate::domain::{AlgoError, Transaction};

impl AlgoClient {
    /// Fetch a single transaction by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or JSON parsing fails.
    pub async fn get_transaction_by_id(&self, txid: &str) -> Result<Option<Transaction>> {
        let url = format!("{}/v2/transactions/{}", self.indexer_url, txid);
        let response = self
            .build_indexer_request(&url)
            .send()
            .await
            .map_err(AlgoError::Network)?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let json: Value = response
            .json()
            .await
            .map_err(|_| AlgoError::parse("Failed to parse transaction JSON").into_report())?;

        let txn_json = match json.get("transaction") {
            Some(txn) => txn,
            None => return Ok(None),
        };

        Transaction::from_json(txn_json)
            .map(Some)
            .map_err(AlgoError::into_report)
    }

    /// Fetch the latest transactions from the network
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails.
    pub async fn get_latest_transactions(&self, limit: usize) -> Result<Vec<Transaction>> {
        let status_url = format!("{}/v2/status", self.algod_url);
        let status_response = self.build_algod_request(&status_url).send().await?;

        let status: Value = status_response.json().await?;

        let latest_round = status["last-round"].as_u64().ok_or_else(|| {
            AlgoError::parse("algod status response missing 'last-round'").into_report()
        })?;
        if latest_round == 0 {
            return Ok(Vec::new());
        }

        let min_round = latest_round.saturating_sub(20);
        let url = format!(
            "{}/v2/transactions?limit={}&min-round={}&max-round={}&order=desc",
            self.indexer_url, limit, min_round, latest_round
        );

        let response = self.build_indexer_request(&url).send().await?;

        let json: Value = response.json().await?;

        let mut transactions = parse_transactions_array(&json)?;
        transactions.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(transactions)
    }

    /// Search for a transaction by ID.
    pub(crate) async fn search_transaction(&self, txid: &str) -> Result<Vec<Transaction>> {
        if txid.is_empty() {
            return Err(AlgoError::invalid_input("Transaction ID cannot be empty").into_report());
        }

        if txid.len() < 40 || txid.len() > 60 {
            return Err(AlgoError::invalid_input(
                "Invalid transaction ID format. Transaction IDs are typically 52 characters long.",
            )
            .into_report());
        }

        let url = format!("{}/v2/transactions/{}", self.indexer_url, txid);

        let response = self.build_indexer_request(&url).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(json) = resp.json::<Value>().await
                    && let Some(txn_json) = json.get("transaction")
                {
                    let transaction =
                        Transaction::from_json(txn_json).map_err(AlgoError::into_report)?;
                    return Ok(vec![transaction]);
                }
            }
            Ok(resp) => {
                let status = resp.status();
                if status.as_u16() != 404 {
                    tracing::debug!("Transaction lookup returned status {status}, trying search");
                }
            }
            Err(e) => {
                tracing::debug!("Transaction lookup failed, trying search: {e}");
            }
        }

        let search_url = format!(
            "{}/v2/transactions?txid={}&limit=10",
            self.indexer_url, txid
        );

        let search_results = self.fetch_transactions_from_url(&search_url).await?;

        if search_results.is_empty() {
            return Err(AlgoError::not_found("transaction", txid).into_report());
        }

        Ok(search_results)
    }

    pub(crate) async fn fetch_transactions_from_url(&self, url: &str) -> Result<Vec<Transaction>> {
        let response = match self.build_indexer_request(url).send().await {
            Ok(resp) if resp.status().is_success() => resp,
            _ => return Ok(Vec::new()),
        };

        let json: Value = match response.json().await {
            Ok(data) => data,
            Err(_) => return Ok(Vec::new()),
        };

        parse_transactions_array(&json)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse an array of transactions from JSON response
///
/// This helper function extracts the "transactions" array from the JSON
/// and parses each transaction using `Transaction::from_json()`.
pub(crate) fn parse_transactions_array(json: &Value) -> Result<Vec<Transaction>> {
    let empty_vec = Vec::new();
    let transactions_array = json["transactions"].as_array().unwrap_or(&empty_vec);
    let mut transactions = Vec::with_capacity(transactions_array.len());

    for txn_json in transactions_array {
        match Transaction::from_json(txn_json) {
            Ok(txn) => transactions.push(txn),
            Err(_) => {
                // Skip malformed transactions but continue processing
                continue;
            }
        }
    }

    Ok(transactions)
}
