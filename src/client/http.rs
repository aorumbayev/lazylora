//! HTTP client abstraction for Algorand API requests.

use reqwest::Client;
use std::time::Duration;

// ============================================================================
// Constants
// ============================================================================

/// Default timeout for HTTP requests in seconds
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default timeout for health check requests in seconds
pub const HEALTH_CHECK_TIMEOUT_SECS: u64 = 2;

/// LocalNet API token (used for development)
pub const LOCALNET_API_TOKEN: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

// ============================================================================
// Configuration
// ============================================================================

/// HTTP client configuration
#[derive(Debug, Clone)]
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

impl HttpConfig {
    /// Create config for LocalNet
    #[must_use]
    pub fn localnet() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            use_localnet_auth: true,
        }
    }

    /// Create config with custom timeout
    #[must_use]
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            use_localnet_auth: false,
        }
    }
}

// ============================================================================
// Traits
// ============================================================================

/// Trait for HTTP request building (to be implemented by specific clients)
pub trait RequestBuilder {
    /// Build a GET request with appropriate headers
    fn build_get(&self, url: &str) -> reqwest::RequestBuilder;
}

// ============================================================================
// HTTP Client
// ============================================================================

/// Base HTTP client wrapper
#[derive(Debug, Clone)]
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
    pub fn with_config(config: HttpConfig) -> Self {
        Self {
            inner: Client::new(),
            config,
        }
    }

    /// Get the inner reqwest client
    #[must_use]
    pub fn inner(&self) -> &Client {
        &self.inner
    }

    /// Get the configuration
    #[must_use]
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }

    /// Build a GET request with standard headers
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
