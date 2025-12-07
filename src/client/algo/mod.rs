//! Algorand API client for interacting with Algorand networks.
//!
//! This module provides the unified `AlgoClient` for making requests to:
//! - Algorand Node (algod) - for current blockchain state
//! - Algorand Indexer - for historical data queries
//! - NFD API - for human-readable address names
//!
//! # Example
//!
//! ```ignore
//! use crate::client::AlgoClient;
//! use crate::domain::Network;
//!
//! let client = AlgoClient::new(Network::MainNet)?;
//! let blocks = client.get_latest_blocks(10).await?;
//! ```

use reqwest::Client;
use std::time::Duration;

use crate::domain::{AlgoError, Network};

mod accounts;
mod applications;
mod assets;
mod blocks;
mod nfd;
mod search;
mod transactions;

#[cfg(test)]
mod tests;

// ============================================================================
// Algorand API Client
// ============================================================================

#[derive(Debug, Clone)]
pub struct AlgoClient {
    /// The indexer API URL.
    pub(crate) indexer_url: String,
    /// The algod API URL.
    pub(crate) algod_url: String,
    /// Optional indexer API token.
    indexer_token: Option<String>,
    /// Optional algod API token.
    algod_token: Option<String>,
    /// The NFD API URL (optional).
    nfd_api_url: Option<String>,
    /// Whether NFD lookups are allowed (built-in MainNet/TestNet only).
    allow_nfd: bool,
    /// Whether this is a LocalNet (affects auth headers).
    pub(crate) is_localnet: bool,
    /// HTTP client for requests.
    pub(crate) client: Client,
}

impl AlgoClient {
    /// Creates a new client for a built-in network.
    ///
    /// # Errors
    ///
    /// Returns `AlgoError::ClientInit` if the HTTP client fails to initialize
    /// (e.g., TLS backend unavailable).
    #[allow(dead_code)] // Kept for callers that want built-in shortcut
    pub fn new(network: Network) -> Result<Self, AlgoError> {
        let client = Self::build_http_client()?;

        Ok(Self {
            indexer_url: network.indexer_url().to_string(),
            algod_url: network.algod_url().to_string(),
            indexer_token: None,
            algod_token: None,
            nfd_api_url: network.nfd_api_url().map(String::from),
            allow_nfd: matches!(network, Network::MainNet | Network::TestNet),
            is_localnet: network == Network::LocalNet,
            client,
        })
    }

    /// Supports both built-in networks and custom user-defined networks.
    ///
    /// # Errors
    ///
    /// Returns `AlgoError::ClientInit` if the HTTP client fails to initialize.
    #[allow(dead_code)] // Will be used when custom networks are fully supported
    pub fn from_config(config: &crate::domain::NetworkConfig) -> Result<Self, AlgoError> {
        use crate::domain::NetworkConfig;

        let client = Self::build_http_client()?;

        Ok(match config {
            NetworkConfig::BuiltIn(network) => Self {
                indexer_url: network.indexer_url().to_string(),
                algod_url: network.algod_url().to_string(),
                indexer_token: None,
                algod_token: None,
                nfd_api_url: network.nfd_api_url().map(String::from),
                allow_nfd: matches!(
                    network,
                    crate::domain::Network::MainNet | crate::domain::Network::TestNet
                ),
                is_localnet: *network == Network::LocalNet,
                client,
            },
            NetworkConfig::Custom(custom) => Self {
                indexer_url: custom.indexer_url.clone(),
                algod_url: custom.algod_url.clone(),
                indexer_token: custom.indexer_token.clone(),
                algod_token: custom.algod_token.clone(),
                nfd_api_url: custom.nfd_api_url.clone(),
                allow_nfd: false,
                is_localnet: false,
                client,
            },
        })
    }

    /// Build the HTTP client with connection pooling.
    fn build_http_client() -> Result<Client, AlgoError> {
        Client::builder()
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| AlgoError::client_init(e.to_string()))
    }

    #[must_use]
    #[allow(dead_code)] // Public API
    pub fn indexer_url(&self) -> &str {
        &self.indexer_url
    }

    #[must_use]
    #[allow(dead_code)] // Public API
    pub fn algod_url(&self) -> &str {
        &self.algod_url
    }

    #[must_use]
    pub fn nfd_api_url(&self) -> Option<&str> {
        self.nfd_api_url.as_deref()
    }

    #[must_use]
    pub fn supports_nfd(&self) -> bool {
        self.allow_nfd && self.nfd_api_url.is_some()
    }

    pub(crate) fn build_algod_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.get(url).header("accept", "application/json");

        if let Some(token) = &self.algod_token {
            request = request.header("X-Algo-API-Token", token);
        } else if self.is_localnet {
            request = request.header(
                "X-Algo-API-Token",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            );
        }

        request
    }

    pub(crate) fn build_indexer_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.get(url).header("accept", "application/json");

        if let Some(token) = &self.indexer_token {
            request = request.header("X-Indexer-API-Token", token);
        } else if self.is_localnet {
            request = request.header(
                "X-Indexer-API-Token",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            );
        }

        request
    }

    /// Check the health status of the network's algod and indexer services.
    ///
    /// # Errors
    ///
    /// Returns an error if algod or indexer (LocalNet only) is unreachable.
    pub async fn get_network_status(&self) -> std::result::Result<(), String> {
        let algod_url = format!("{}/health", self.algod_url);
        let indexer_url = format!("{}/health", self.indexer_url);

        let (algod_result, indexer_result) = tokio::join!(
            self.build_algod_request(&algod_url)
                .timeout(Duration::from_secs(2))
                .send(),
            self.build_indexer_request(&indexer_url)
                .timeout(Duration::from_secs(2))
                .send()
        );

        if let Err(e) = algod_result {
            return Err(format!(
                "Unable to connect to algod at {}. Error: {}",
                self.algod_url, e
            ));
        }

        if self.is_localnet && indexer_result.is_err() {
            return Err(format!(
                "Unable to connect to indexer at {}. Algod is running but indexer is not available.",
                self.indexer_url
            ));
        }

        Ok(())
    }
}
