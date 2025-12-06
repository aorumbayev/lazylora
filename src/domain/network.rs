//! Network configuration for Algorand networks.
//!
//! This module defines the supported Algorand networks and their
//! associated configuration such as API endpoints.

use serde::{Deserialize, Serialize};

// ============================================================================
// Network Configuration
// ============================================================================

/// Algorand network variants.
///
/// Represents the different Algorand networks that can be connected to,
/// each with its own set of API endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum Network {
    /// Algorand MainNet - the production network.
    #[default]
    MainNet,
    /// Algorand TestNet - the test network for development.
    TestNet,
    /// LocalNet - a local development network.
    LocalNet,
}

/// Custom user-defined network configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomNetwork {
    /// User-defined network name.
    pub name: String,
    /// Indexer API URL.
    pub indexer_url: String,
    /// Algod API URL.
    pub algod_url: String,
    /// Optional Indexer API token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexer_token: Option<String>,
    /// Optional Algod API token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algod_token: Option<String>,
    /// Optional NFD API URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nfd_api_url: Option<String>,
}

impl CustomNetwork {
    /// Creates a new custom network.
    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn new(
        name: impl Into<String>,
        indexer_url: impl Into<String>,
        algod_url: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            indexer_url: indexer_url.into(),
            algod_url: algod_url.into(),
            indexer_token: None,
            algod_token: None,
            nfd_api_url: None,
        }
    }

    /// Sets the NFD API URL.
    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn with_nfd_api(mut self, url: impl Into<String>) -> Self {
        self.nfd_api_url = Some(url.into());
        self
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// NFD lookups only available when `nfd_api_url` is set.
    #[must_use]
    pub fn supports_nfd(&self) -> bool {
        self.nfd_api_url.is_some()
    }
}

/// Network configuration that can be either built-in or custom.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NetworkConfig {
    /// Built-in Algorand network.
    BuiltIn(Network),
    /// Custom user-defined network.
    Custom(CustomNetwork),
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::BuiltIn(Network::MainNet)
    }
}

impl NetworkConfig {
    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn as_str(&self) -> &str {
        match self {
            Self::BuiltIn(network) => network.as_str(),
            Self::Custom(custom) => custom.as_str(),
        }
    }

    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn indexer_url(&self) -> &str {
        match self {
            Self::BuiltIn(network) => network.indexer_url(),
            Self::Custom(custom) => &custom.indexer_url,
        }
    }

    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn algod_url(&self) -> &str {
        match self {
            Self::BuiltIn(network) => network.algod_url(),
            Self::Custom(custom) => &custom.algod_url,
        }
    }

    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn nfd_api_url(&self) -> Option<&str> {
        match self {
            Self::BuiltIn(network) => network.nfd_api_url(),
            Self::Custom(custom) => custom.nfd_api_url.as_deref(),
        }
    }

    #[must_use]
    pub fn indexer_token(&self) -> Option<&str> {
        match self {
            Self::BuiltIn(_) => None,
            Self::Custom(custom) => custom.indexer_token.as_deref(),
        }
    }

    #[must_use]
    pub fn algod_token(&self) -> Option<&str> {
        match self {
            Self::BuiltIn(_) => None,
            Self::Custom(custom) => custom.algod_token.as_deref(),
        }
    }

    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn supports_nfd(&self) -> bool {
        match self {
            Self::BuiltIn(network) => network.supports_nfd(),
            Self::Custom(custom) => custom.supports_nfd(),
        }
    }

    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn lora_base_url(&self) -> &str {
        match self {
            Self::BuiltIn(network) => network.lora_base_url(),
            // Custom networks don't have Lora explorer links
            Self::Custom(_) => "",
        }
    }

    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn transaction_url(&self, txn_id: &str) -> String {
        match self {
            Self::BuiltIn(network) => network.transaction_url(txn_id),
            Self::Custom(_) => String::new(),
        }
    }

    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn account_url(&self, address: &str) -> String {
        match self {
            Self::BuiltIn(network) => network.account_url(address),
            Self::Custom(_) => String::new(),
        }
    }

    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn asset_url(&self, asset_id: u64) -> String {
        match self {
            Self::BuiltIn(network) => network.asset_url(asset_id),
            Self::Custom(_) => String::new(),
        }
    }

    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn block_url(&self, round: u64) -> String {
        match self {
            Self::BuiltIn(network) => network.block_url(round),
            Self::Custom(_) => String::new(),
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn application_url(&self, app_id: u64) -> String {
        match self {
            Self::BuiltIn(network) => network.application_url(app_id),
            Self::Custom(_) => String::new(),
        }
    }
}

impl From<Network> for NetworkConfig {
    fn from(network: Network) -> Self {
        Self::BuiltIn(network)
    }
}

impl From<CustomNetwork> for NetworkConfig {
    fn from(custom: CustomNetwork) -> Self {
        Self::Custom(custom)
    }
}

impl std::fmt::Display for NetworkConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Network {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::MainNet => "MainNet",
            Self::TestNet => "TestNet",
            Self::LocalNet => "LocalNet",
        }
    }

    /// Indexer provides historical blockchain data and search capabilities.
    #[must_use]
    pub const fn indexer_url(&self) -> &str {
        match self {
            Self::MainNet => "https://mainnet-idx.algonode.cloud",
            Self::TestNet => "https://testnet-idx.algonode.cloud",
            Self::LocalNet => "http://localhost:8980",
        }
    }

    /// Algod provides current network state and transaction submission.
    #[must_use]
    pub const fn algod_url(&self) -> &str {
        match self {
            Self::MainNet => "https://mainnet-api.algonode.cloud",
            Self::TestNet => "https://testnet-api.algonode.cloud",
            Self::LocalNet => "http://localhost:4001",
        }
    }

    /// NFD (Non-Fungible Domains) only available on MainNet/TestNet.
    #[must_use]
    pub const fn nfd_api_url(&self) -> Option<&str> {
        match self {
            Self::MainNet => Some("https://api.nf.domains"),
            Self::TestNet => Some("https://api.testnet.nf.domains"),
            Self::LocalNet => None, // NFD not available on LocalNet
        }
    }

    #[must_use]
    pub const fn supports_nfd(&self) -> bool {
        matches!(self, Self::MainNet | Self::TestNet)
    }

    #[must_use]
    pub const fn lora_base_url(&self) -> &str {
        match self {
            Self::MainNet => "https://lora.algokit.io/mainnet",
            Self::TestNet => "https://lora.algokit.io/testnet",
            Self::LocalNet => "https://lora.algokit.io/localnet",
        }
    }

    #[must_use]
    pub fn transaction_url(&self, txn_id: &str) -> String {
        format!("{}/transaction/{}", self.lora_base_url(), txn_id)
    }

    #[must_use]
    pub fn account_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.lora_base_url(), address)
    }

    #[must_use]
    pub fn asset_url(&self, asset_id: u64) -> String {
        format!("{}/asset/{}", self.lora_base_url(), asset_id)
    }

    #[must_use]
    pub fn block_url(&self, round: u64) -> String {
        format!("{}/block/{}", self.lora_base_url(), round)
    }

    #[must_use]
    #[allow(dead_code)] // Reserved for future application detail view
    pub fn application_url(&self, app_id: u64) -> String {
        format!("{}/application/{}", self.lora_base_url(), app_id)
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Display for CustomNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    // Built-in Network tests

    #[rstest]
    #[case::mainnet(Network::MainNet, "MainNet")]
    #[case::testnet(Network::TestNet, "TestNet")]
    #[case::localnet(Network::LocalNet, "LocalNet")]
    fn test_network_as_str(#[case] network: Network, #[case] expected: &str) {
        assert_eq!(network.as_str(), expected);
    }

    #[test]
    fn test_network_urls() {
        assert!(Network::MainNet.indexer_url().contains("mainnet"));
        assert!(Network::TestNet.algod_url().contains("testnet"));
        assert!(Network::LocalNet.algod_url().contains("localhost"));
    }

    #[rstest]
    #[case::mainnet(Network::MainNet, true)]
    #[case::testnet(Network::TestNet, true)]
    #[case::localnet(Network::LocalNet, false)]
    fn test_network_supports_nfd(#[case] network: Network, #[case] expected: bool) {
        assert_eq!(network.supports_nfd(), expected);
        assert_eq!(network.nfd_api_url().is_some(), expected);
    }

    #[test]
    fn test_network_default() {
        assert_eq!(Network::default(), Network::MainNet);
    }

    #[rstest]
    #[case::mainnet(Network::MainNet, "MainNet")]
    #[case::testnet(Network::TestNet, "TestNet")]
    #[case::localnet(Network::LocalNet, "LocalNet")]
    fn test_network_display(#[case] network: Network, #[case] expected: &str) {
        assert_eq!(format!("{network}"), expected);
    }

    #[test]
    fn test_network_serialization() {
        let network = Network::MainNet;
        let serialized = serde_json::to_string(&network).unwrap();
        let deserialized: Network = serde_json::from_str(&serialized).unwrap();
        assert_eq!(network, deserialized);
    }

    #[rstest]
    #[case::mainnet(Network::MainNet, "https://lora.algokit.io/mainnet")]
    #[case::testnet(Network::TestNet, "https://lora.algokit.io/testnet")]
    #[case::localnet(Network::LocalNet, "https://lora.algokit.io/localnet")]
    fn test_network_lora_base_url(#[case] network: Network, #[case] expected: &str) {
        assert_eq!(network.lora_base_url(), expected);
    }

    #[test]
    fn test_lora_entity_urls() {
        let network = Network::MainNet;
        assert_eq!(
            network.transaction_url("ABC123"),
            "https://lora.algokit.io/mainnet/transaction/ABC123"
        );
        assert_eq!(
            network.account_url("ADDR123"),
            "https://lora.algokit.io/mainnet/account/ADDR123"
        );
        assert_eq!(
            network.asset_url(12345),
            "https://lora.algokit.io/mainnet/asset/12345"
        );
        assert_eq!(
            network.block_url(49000000),
            "https://lora.algokit.io/mainnet/block/49000000"
        );
        assert_eq!(
            network.application_url(1234),
            "https://lora.algokit.io/mainnet/application/1234"
        );
    }

    // CustomNetwork tests

    #[test]
    fn test_custom_network_new() {
        let network = CustomNetwork::new(
            "MyNet",
            "https://indexer.example.com",
            "https://algod.example.com",
        );
        assert_eq!(network.name, "MyNet");
        assert_eq!(network.indexer_url, "https://indexer.example.com");
        assert_eq!(network.algod_url, "https://algod.example.com");
        assert_eq!(network.nfd_api_url, None);
        assert!(!network.supports_nfd());
    }

    #[test]
    fn test_custom_network_with_nfd() {
        let network = CustomNetwork::new(
            "MyNet",
            "https://indexer.example.com",
            "https://algod.example.com",
        )
        .with_nfd_api("https://nfd.example.com");

        assert_eq!(
            network.nfd_api_url,
            Some("https://nfd.example.com".to_string())
        );
        assert!(network.supports_nfd());
    }

    #[test]
    fn test_custom_network_display() {
        let network = CustomNetwork::new("MyCustomNet", "http://idx", "http://algod");
        assert_eq!(format!("{network}"), "MyCustomNet");
        assert_eq!(network.as_str(), "MyCustomNet");
    }

    #[test]
    fn test_custom_network_serialization() {
        let network =
            CustomNetwork::new("Test", "http://idx", "http://algod").with_nfd_api("http://nfd");
        let json = serde_json::to_string(&network).unwrap();
        let deserialized: CustomNetwork = serde_json::from_str(&json).unwrap();
        assert_eq!(network, deserialized);
    }

    // NetworkConfig tests

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config, NetworkConfig::BuiltIn(Network::MainNet));
    }

    #[test]
    fn test_network_config_from_network() {
        let config = NetworkConfig::from(Network::TestNet);
        assert_eq!(config, NetworkConfig::BuiltIn(Network::TestNet));
    }

    #[test]
    fn test_network_config_from_custom() {
        let custom = CustomNetwork::new("Test", "http://idx", "http://algod");
        let config = NetworkConfig::from(custom.clone());
        assert_eq!(config, NetworkConfig::Custom(custom));
    }

    #[rstest]
    #[case::builtin_mainnet(
        NetworkConfig::BuiltIn(Network::MainNet),
        "MainNet",
        "https://mainnet-idx.algonode.cloud",
        "https://mainnet-api.algonode.cloud"
    )]
    #[case::builtin_testnet(
        NetworkConfig::BuiltIn(Network::TestNet),
        "TestNet",
        "https://testnet-idx.algonode.cloud",
        "https://testnet-api.algonode.cloud"
    )]
    #[case::custom(
        NetworkConfig::Custom(CustomNetwork::new(
            "MyNet",
            "http://idx.test",
            "http://algod.test"
        )),
        "MyNet",
        "http://idx.test",
        "http://algod.test"
    )]
    fn test_network_config_urls(
        #[case] config: NetworkConfig,
        #[case] expected_name: &str,
        #[case] expected_indexer: &str,
        #[case] expected_algod: &str,
    ) {
        assert_eq!(config.as_str(), expected_name);
        assert_eq!(config.indexer_url(), expected_indexer);
        assert_eq!(config.algod_url(), expected_algod);
    }

    #[rstest]
    #[case::mainnet(NetworkConfig::BuiltIn(Network::MainNet), true)]
    #[case::testnet(NetworkConfig::BuiltIn(Network::TestNet), true)]
    #[case::localnet(NetworkConfig::BuiltIn(Network::LocalNet), false)]
    #[case::custom_without_nfd(
        NetworkConfig::Custom(CustomNetwork::new("Test", "http://i", "http://a")),
        false
    )]
    #[case::custom_with_nfd(
        NetworkConfig::Custom(
            CustomNetwork::new("Test", "http://i", "http://a").with_nfd_api("http://nfd")
        ),
        true
    )]
    fn test_network_config_nfd_support(#[case] config: NetworkConfig, #[case] expected: bool) {
        assert_eq!(config.supports_nfd(), expected);
        assert_eq!(config.nfd_api_url().is_some(), expected);
    }

    #[test]
    fn test_network_config_lora_urls_builtin() {
        let config = NetworkConfig::BuiltIn(Network::MainNet);
        assert_eq!(config.lora_base_url(), "https://lora.algokit.io/mainnet");
        assert!(config.transaction_url("TX123").contains("lora.algokit.io"));
        assert!(config.account_url("ADDR").contains("lora.algokit.io"));
    }

    #[test]
    fn test_network_config_lora_urls_custom() {
        let config = NetworkConfig::Custom(CustomNetwork::new("Test", "http://i", "http://a"));
        assert_eq!(config.lora_base_url(), "");
        assert_eq!(config.transaction_url("TX123"), "");
        assert_eq!(config.account_url("ADDR"), "");
        assert_eq!(config.asset_url(123), "");
        assert_eq!(config.block_url(456), "");
    }

    #[test]
    fn test_network_config_display() {
        let builtin = NetworkConfig::BuiltIn(Network::MainNet);
        assert_eq!(format!("{builtin}"), "MainNet");

        let custom = NetworkConfig::Custom(CustomNetwork::new("MyNet", "http://i", "http://a"));
        assert_eq!(format!("{custom}"), "MyNet");
    }

    #[test]
    fn test_network_config_serialization() {
        let configs = vec![
            NetworkConfig::BuiltIn(Network::MainNet),
            NetworkConfig::Custom(CustomNetwork::new("Test", "http://idx", "http://algod")),
        ];

        for config in configs {
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: NetworkConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(config, deserialized);
        }
    }
}
