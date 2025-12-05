//! NFD (Non-Fungible Domains) API client.
//!
//! NFD provides human-readable names for Algorand addresses, similar to ENS on Ethereum.
//! Only available on MainNet and TestNet.

use super::http::{HttpClient, HttpConfig};
use std::time::Duration;

// ============================================================================
// Constants
// ============================================================================

/// Default timeout for NFD API requests
#[allow(dead_code)] // Public API not yet integrated
pub const NFD_TIMEOUT_SECS: u64 = 5;

// ============================================================================
// NFD Client
// ============================================================================

/// NFD API client
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API not yet integrated
pub struct NfdClient {
    http: HttpClient,
    base_url: Option<String>,
}

impl NfdClient {
    /// Create a new NFD client for MainNet
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn mainnet() -> Self {
        Self {
            http: HttpClient::with_config(HttpConfig {
                timeout: Duration::from_secs(NFD_TIMEOUT_SECS),
                ..Default::default()
            }),
            base_url: Some("https://api.nf.domains".to_string()),
        }
    }

    /// Create a new NFD client for TestNet
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn testnet() -> Self {
        Self {
            http: HttpClient::with_config(HttpConfig {
                timeout: Duration::from_secs(NFD_TIMEOUT_SECS),
                ..Default::default()
            }),
            base_url: Some("https://api.testnet.nf.domains".to_string()),
        }
    }

    /// Create a disabled NFD client (for LocalNet)
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn disabled() -> Self {
        Self {
            http: HttpClient::new(),
            base_url: None,
        }
    }

    /// Check if NFD is available
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn is_available(&self) -> bool {
        self.base_url.is_some()
    }

    /// Get the base URL if available
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }

    /// Build a request to the NFD API
    ///
    /// Returns None if NFD is not available for this network.
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn request(&self, endpoint: &str) -> Option<reqwest::RequestBuilder> {
        let base = self.base_url.as_ref()?;
        let url = format!("{}{}", base, endpoint);
        Some(self.http.get(&url))
    }
}
