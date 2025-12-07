//! Account fetching methods for AlgoClient.

use color_eyre::Result;
use serde_json::Value;

use super::AlgoClient;
use crate::domain::{
    AccountAssetHolding, AccountDetails, AccountInfo, AlgoError, AppLocalState, CreatedAppInfo,
    CreatedAssetInfo, ParticipationInfo,
};

impl AlgoClient {
    /// Search for an address (validates format or resolves NFD).
    pub(crate) async fn search_address(&self, query: &str) -> Result<Option<AccountInfo>> {
        let trimmed = query.trim();

        // First, check if it's a valid Algorand address
        if trimmed.len() == 58
            && trimmed
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            // It's a valid address format, search directly
            return self.search_address_direct(trimmed).await;
        }

        // Check if NFD is supported and the query looks like an NFD name
        if self.supports_nfd() && Self::looks_like_nfd_name(trimmed) {
            // Try to resolve as NFD name
            if let Ok(Some(nfd_info)) = self.get_nfd_by_name(trimmed).await {
                // Get the deposit account from NFD
                let address = nfd_info
                    .deposit_account
                    .as_ref()
                    .or(nfd_info.owner.as_ref());

                if let Some(addr) = address {
                    // Search for the resolved address
                    return self.search_address_direct(addr).await;
                }
            }

            // NFD not found
            return Err(AlgoError::not_found("NFD", trimmed).into_report());
        }

        // Not a valid address and not an NFD name
        Err(AlgoError::invalid_input(
            "Invalid input. Enter a 58-character Algorand address or an NFD name (e.g., alice.algo)."
        ).into_report())
    }

    /// Search for an address directly (after validation or NFD resolution)
    async fn search_address_direct(&self, address: &str) -> Result<Option<AccountInfo>> {
        let indexer_result = self.search_address_via_indexer(address).await;

        match indexer_result {
            Ok(Some(account)) => {
                return Ok(Some(account));
            }
            Ok(None) => {}
            Err(e) => {
                tracing::debug!("Indexer lookup failed, falling back to algod: {e}");
            }
        }

        let algod_result = self.search_address_via_algod(address).await;

        match algod_result {
            Ok(Some(account)) => Ok(Some(account)),
            Ok(None) => Err(AlgoError::not_found("account", address).into_report()),
            Err(e) => Err(color_eyre::eyre::eyre!(
                "Failed to fetch account information for '{}': {}",
                address,
                e
            )),
        }
    }

    async fn search_address_via_indexer(&self, address: &str) -> Result<Option<AccountInfo>> {
        let account_url = format!("{}/v2/accounts/{}", self.indexer_url, address);

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
            Ok(Some(Self::parse_account_info(account, address)))
        } else {
            Err(AlgoError::parse("Invalid indexer response format").into_report())
        }
    }

    async fn search_address_via_algod(&self, address: &str) -> Result<Option<AccountInfo>> {
        let account_url = format!("{}/v2/accounts/{}", self.algod_url, address);

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

        Ok(Some(Self::parse_account_info(&account_data, address)))
    }

    #[must_use]
    fn parse_account_info(account: &Value, address: &str) -> AccountInfo {
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

    /// Get detailed account information from algod.
    ///
    /// # Errors
    ///
    /// Returns an error if the address format is invalid, account not found, or network fails.
    pub async fn get_account_details(&self, address: &str) -> Result<AccountDetails> {
        // Validate address format
        if address.len() != 58
            || !address
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return Err(AlgoError::invalid_input("Invalid Algorand address format").into_report());
        }

        let account_url = format!("{}/v2/accounts/{}", self.algod_url, address);
        let response = self.build_algod_request(&account_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(AlgoError::not_found("account", address).into_report());
            }
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(color_eyre::eyre::eyre!(
                "Failed to fetch account details: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let account_data: Value = response.json().await?;
        let mut account_details = Self::parse_account_details(&account_data, address);

        // Fetch NFD info if supported on this network
        if self.supports_nfd() {
            account_details.nfd = self.get_nfd_for_address(address).await.unwrap_or(None);
        }

        Ok(account_details)
    }

    #[must_use]
    fn parse_account_details(account: &Value, address: &str) -> AccountDetails {
        let balance = account["amount"].as_u64().unwrap_or(0);
        let min_balance = account["min-balance"].as_u64().unwrap_or(0);
        let pending_rewards = account["pending-rewards"].as_u64().unwrap_or(0);
        let rewards = account["rewards"].as_u64().unwrap_or(0);
        let reward_base = account["reward-base"].as_u64().unwrap_or(0);
        let status = account["status"].as_str().unwrap_or("unknown").to_string();

        let total_apps_opted_in = account["total-apps-opted-in"].as_u64().unwrap_or(0) as usize;
        let total_assets_opted_in = account["total-assets-opted-in"].as_u64().unwrap_or(0) as usize;
        let total_created_apps = account["total-created-apps"].as_u64().unwrap_or(0) as usize;
        let total_created_assets = account["total-created-assets"].as_u64().unwrap_or(0) as usize;
        let total_boxes = account["total-boxes"].as_u64().unwrap_or(0) as usize;

        let auth_addr = account["auth-addr"].as_str().map(String::from);

        // Parse participation info if online
        let participation = account.get("participation").map(|part| ParticipationInfo {
            vote_first: part["vote-first-valid"].as_u64().unwrap_or(0),
            vote_last: part["vote-last-valid"].as_u64().unwrap_or(0),
            vote_key_dilution: part["vote-key-dilution"].as_u64().unwrap_or(0),
            selection_key: part["selection-participation-key"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            vote_key: part["vote-participation-key"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            state_proof_key: part["state-proof-key"].as_str().map(String::from),
        });

        // Parse asset holdings (limited to first 10)
        let assets = account["assets"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .map(|a| AccountAssetHolding {
                        asset_id: a["asset-id"].as_u64().unwrap_or(0),
                        amount: a["amount"].as_u64().unwrap_or(0),
                        is_frozen: a["is-frozen"].as_bool().unwrap_or(false),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse created assets (limited to first 10)
        let created_assets = account["created-assets"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .map(|a| {
                        let params = &a["params"];
                        CreatedAssetInfo {
                            asset_id: a["index"].as_u64().unwrap_or(0),
                            name: params["name"].as_str().unwrap_or("").to_string(),
                            unit_name: params["unit-name"].as_str().unwrap_or("").to_string(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse app local states (limited to first 10)
        let apps_local_state = account["apps-local-state"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .map(|a| AppLocalState {
                        app_id: a["id"].as_u64().unwrap_or(0),
                        schema_num_uint: a["schema"]["num-uint"].as_u64().unwrap_or(0),
                        schema_num_byte_slice: a["schema"]["num-byte-slice"].as_u64().unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse created apps (limited to first 10)
        let created_apps = account["created-apps"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .map(|a| CreatedAppInfo {
                        app_id: a["id"].as_u64().unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();

        AccountDetails {
            address: address.to_string(),
            balance,
            min_balance,
            pending_rewards,
            rewards,
            reward_base,
            status,
            total_apps_opted_in,
            total_assets_opted_in,
            total_created_apps,
            total_created_assets,
            total_boxes,
            auth_addr,
            participation,
            assets,
            created_assets,
            apps_local_state,
            created_apps,
            nfd: None, // NFD is set separately after fetching
        }
    }
}
