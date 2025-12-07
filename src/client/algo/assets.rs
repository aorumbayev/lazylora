//! Asset fetching methods for AlgoClient.

use color_eyre::Result;
use serde_json::Value;

use super::AlgoClient;
use crate::domain::{AlgoError, AssetDetails, AssetInfo};

impl AlgoClient {
    /// Search for an asset by ID.
    pub(crate) async fn search_asset(&self, asset_id_str: &str) -> Result<Option<AssetInfo>> {
        let asset_id = asset_id_str.parse::<u64>().map_err(|_| {
            AlgoError::invalid_input(format!(
                "Invalid asset ID '{}'. Please enter a valid positive integer.",
                asset_id_str
            ))
            .into_report()
        })?;

        let asset_url = format!("{}/v2/assets/{}", self.indexer_url, asset_id);

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

    /// Get detailed asset information from indexer
    ///
    /// # Errors
    ///
    /// Returns an error if the asset doesn't exist or network request fails.
    pub async fn get_asset_details(&self, asset_id: u64) -> Result<AssetDetails> {
        let asset_url = format!("{}/v2/assets/{}", self.indexer_url, asset_id);
        let response = self.build_indexer_request(&asset_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(AlgoError::not_found("asset", asset_id.to_string()).into_report());
            }
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(color_eyre::eyre::eyre!(
                "Failed to fetch asset details: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let asset_data: Value = response.json().await?;
        Ok(Self::parse_asset_details(&asset_data, asset_id))
    }

    #[must_use]
    fn parse_asset_details(data: &Value, asset_id: u64) -> AssetDetails {
        let asset = &data["asset"];
        let params = &asset["params"];

        AssetDetails {
            id: asset_id,
            name: params["name"].as_str().unwrap_or("").to_string(),
            unit_name: params["unit-name"].as_str().unwrap_or("").to_string(),
            creator: params["creator"].as_str().unwrap_or("").to_string(),
            total: params["total"].as_u64().unwrap_or(0),
            decimals: params["decimals"].as_u64().unwrap_or(0),
            url: params["url"].as_str().unwrap_or("").to_string(),
            metadata_hash: params["metadata-hash"].as_str().map(String::from),
            default_frozen: params["default-frozen"].as_bool().unwrap_or(false),
            manager: params["manager"].as_str().map(String::from),
            reserve: params["reserve"].as_str().map(String::from),
            freeze: params["freeze"].as_str().map(String::from),
            clawback: params["clawback"].as_str().map(String::from),
            deleted: asset["deleted"].as_bool().unwrap_or(false),
            created_at_round: asset["created-at-round"].as_u64(),
        }
    }
}
