//! Algorand Indexer API client.
//!
//! The indexer provides historical data about transactions, accounts, assets, and applications.

use super::http::{HttpClient, HttpConfig, LOCALNET_API_TOKEN};

// ============================================================================
// Constants
// ============================================================================

/// Header name for indexer API token
#[allow(dead_code)] // Public API not yet integrated
pub const INDEXER_TOKEN_HEADER: &str = "X-Indexer-API-Token";

// ============================================================================
// Indexer Client
// ============================================================================

/// Algorand Indexer client
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API not yet integrated
pub struct IndexerClient {
    http: HttpClient,
    base_url: String,
}

impl IndexerClient {
    /// Create a new indexer client
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: HttpClient::new(),
            base_url: base_url.into(),
        }
    }

    /// Create a new indexer client for LocalNet
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn localnet() -> Self {
        Self {
            http: HttpClient::with_config(HttpConfig {
                use_localnet_auth: true,
                ..Default::default()
            }),
            base_url: "http://localhost:8980".to_string(),
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
            req = req.header(INDEXER_TOKEN_HEADER, LOCALNET_API_TOKEN);
        }

        req
    }
}
