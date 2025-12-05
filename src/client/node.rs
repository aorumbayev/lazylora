//! Algorand Node (algod) API client.
//!
//! The algod client provides access to the current state of the blockchain
//! including the latest blocks, account balances, and network status.

use super::http::{HttpClient, HttpConfig, LOCALNET_API_TOKEN};

// ============================================================================
// Constants
// ============================================================================

/// Header name for algod API token
#[allow(dead_code)] // Public API not yet integrated
pub const ALGOD_TOKEN_HEADER: &str = "X-Algo-API-Token";

// ============================================================================
// Node Client
// ============================================================================

/// Algorand Node (algod) client
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API not yet integrated
pub struct NodeClient {
    http: HttpClient,
    base_url: String,
}

impl NodeClient {
    /// Create a new node client
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: HttpClient::new(),
            base_url: base_url.into(),
        }
    }

    /// Create a new node client for LocalNet
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn localnet() -> Self {
        Self {
            http: HttpClient::with_config(HttpConfig {
                use_localnet_auth: true,
                ..Default::default()
            }),
            base_url: "http://localhost:4001".to_string(),
        }
    }

    /// Get the base URL
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build a request with appropriate authentication
    #[allow(dead_code)] // Public API not yet integrated
    pub fn request(&self, endpoint: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut req = self.http.get(&url);

        if self.http.config().use_localnet_auth {
            req = req.header(ALGOD_TOKEN_HEADER, LOCALNET_API_TOKEN);
        }

        req
    }
}
