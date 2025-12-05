//! Algorand Node (algod) API client.
//!
//! The algod client provides access to the current state of the blockchain
//! including the latest blocks, account balances, and network status.

use super::http::{HttpClient, HttpConfig, LOCALNET_API_TOKEN};

// ============================================================================
// Constants
// ============================================================================

/// Header name for algod API token
pub const ALGOD_TOKEN_HEADER: &str = "X-Algo-API-Token";

// ============================================================================
// Node Client
// ============================================================================

/// Algorand Node (algod) client
#[derive(Debug, Clone)]
pub struct NodeClient {
    http: HttpClient,
    base_url: String,
}

impl NodeClient {
    /// Create a new node client
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: HttpClient::new(),
            base_url: base_url.into(),
        }
    }

    /// Create a new node client for LocalNet
    #[must_use]
    pub fn localnet() -> Self {
        Self {
            http: HttpClient::with_config(HttpConfig::localnet()),
            base_url: "http://localhost:4001".to_string(),
        }
    }

    /// Get the base URL
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build a request with appropriate authentication
    pub fn request(&self, endpoint: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut req = self.http.get(&url);

        if self.http.config().use_localnet_auth {
            req = req.header(ALGOD_TOKEN_HEADER, LOCALNET_API_TOKEN);
        }

        req
    }
}
