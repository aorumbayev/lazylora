//! HTTP client abstraction for Algorand API requests.

use reqwest::Client;
use std::time::Duration;

// ============================================================================
// Constants
// ============================================================================

/// Default timeout for HTTP requests in seconds
#[allow(dead_code)] // Public API not yet integrated
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default timeout for health check requests in seconds
#[allow(dead_code)] // Public API not yet integrated
pub const HEALTH_CHECK_TIMEOUT_SECS: u64 = 2;

/// LocalNet API token (used for development)
#[allow(dead_code)] // Public API not yet integrated
pub const LOCALNET_API_TOKEN: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

// ============================================================================
// Configuration
// ============================================================================

/// HTTP client configuration
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API not yet integrated
pub struct HttpConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Whether to use LocalNet authentication
    pub use_localnet_auth: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            use_localnet_auth: false,
        }
    }
}

// ============================================================================
// HTTP Client
// ============================================================================

/// Base HTTP client wrapper
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API not yet integrated
pub struct HttpClient {
    inner: Client,
    config: HttpConfig,
}

impl HttpClient {
    /// Create a new HTTP client with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Client::new(),
            config: HttpConfig::default(),
        }
    }

    /// Create a new HTTP client with custom configuration
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn with_config(config: HttpConfig) -> Self {
        Self {
            inner: Client::new(),
            config,
        }
    }

    /// Get the inner reqwest client
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn inner(&self) -> &Client {
        &self.inner
    }

    /// Get the configuration
    #[must_use]
    #[allow(dead_code)] // Public API not yet integrated
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }

    /// Build a GET request with standard headers
    #[allow(dead_code)] // Public API not yet integrated
    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        self.inner
            .get(url)
            .header("accept", "application/json")
            .timeout(self.config.timeout)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
