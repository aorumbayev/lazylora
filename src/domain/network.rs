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

impl Network {
    /// Returns the human-readable name of the network.
    ///
    /// # Returns
    ///
    /// A static string slice with the network name.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::MainNet => "MainNet",
            Self::TestNet => "TestNet",
            Self::LocalNet => "LocalNet",
        }
    }

    /// Returns the indexer API URL for this network.
    ///
    /// The indexer provides historical blockchain data and search capabilities.
    ///
    /// # Returns
    ///
    /// The base URL for the indexer API.
    #[must_use]
    pub const fn indexer_url(&self) -> &str {
        match self {
            Self::MainNet => "https://mainnet-idx.algonode.cloud",
            Self::TestNet => "https://testnet-idx.algonode.cloud",
            Self::LocalNet => "http://localhost:8980",
        }
    }

    /// Returns the algod API URL for this network.
    ///
    /// Algod provides access to current network state and transaction submission.
    ///
    /// # Returns
    ///
    /// The base URL for the algod API.
    #[must_use]
    pub const fn algod_url(&self) -> &str {
        match self {
            Self::MainNet => "https://mainnet-api.algonode.cloud",
            Self::TestNet => "https://testnet-api.algonode.cloud",
            Self::LocalNet => "http://localhost:4001",
        }
    }

    /// Returns the NFD API base URL for the network.
    ///
    /// NFD (Non-Fungible Domains) is only available on MainNet and TestNet.
    ///
    /// # Returns
    ///
    /// `Some` with the NFD API URL if supported, `None` for LocalNet.
    #[must_use]
    pub const fn nfd_api_url(&self) -> Option<&str> {
        match self {
            Self::MainNet => Some("https://api.nf.domains"),
            Self::TestNet => Some("https://api.testnet.nf.domains"),
            Self::LocalNet => None, // NFD not available on LocalNet
        }
    }

    /// Returns whether NFD lookups are supported on this network.
    ///
    /// # Returns
    ///
    /// `true` if NFD is supported, `false` otherwise.
    #[must_use]
    pub const fn supports_nfd(&self) -> bool {
        matches!(self, Self::MainNet | Self::TestNet)
    }

    /// Returns the base URL for the web Lora explorer.
    ///
    /// # Returns
    ///
    /// The base URL for the Lora web explorer for this network.
    #[must_use]
    pub const fn lora_base_url(&self) -> &str {
        match self {
            Self::MainNet => "https://lora.algokit.io/mainnet",
            Self::TestNet => "https://lora.algokit.io/testnet",
            Self::LocalNet => "https://lora.algokit.io/localnet",
        }
    }

    /// Returns the Lora URL for a specific transaction.
    #[must_use]
    pub fn transaction_url(&self, txn_id: &str) -> String {
        format!("{}/transaction/{}", self.lora_base_url(), txn_id)
    }

    /// Returns the Lora URL for a specific account.
    #[must_use]
    pub fn account_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.lora_base_url(), address)
    }

    /// Returns the Lora URL for a specific asset.
    #[must_use]
    pub fn asset_url(&self, asset_id: u64) -> String {
        format!("{}/asset/{}", self.lora_base_url(), asset_id)
    }

    /// Returns the Lora URL for a specific block.
    #[must_use]
    pub fn block_url(&self, round: u64) -> String {
        format!("{}/block/{}", self.lora_base_url(), round)
    }

    /// Returns the Lora URL for a specific application.
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_as_str() {
        assert_eq!(Network::MainNet.as_str(), "MainNet");
        assert_eq!(Network::TestNet.as_str(), "TestNet");
        assert_eq!(Network::LocalNet.as_str(), "LocalNet");
    }

    #[test]
    fn test_network_urls() {
        assert!(Network::MainNet.indexer_url().contains("mainnet"));
        assert!(Network::TestNet.algod_url().contains("testnet"));
        assert!(Network::LocalNet.algod_url().contains("localhost"));
    }

    #[test]
    fn test_nfd_api_url() {
        assert!(Network::MainNet.nfd_api_url().is_some());
        assert!(Network::TestNet.nfd_api_url().is_some());
        assert!(Network::LocalNet.nfd_api_url().is_none());
    }

    #[test]
    fn test_supports_nfd() {
        assert!(Network::MainNet.supports_nfd());
        assert!(Network::TestNet.supports_nfd());
        assert!(!Network::LocalNet.supports_nfd());
    }

    #[test]
    fn test_network_default() {
        assert_eq!(Network::default(), Network::MainNet);
    }

    #[test]
    fn test_network_display() {
        assert_eq!(format!("{}", Network::MainNet), "MainNet");
        assert_eq!(format!("{}", Network::TestNet), "TestNet");
        assert_eq!(format!("{}", Network::LocalNet), "LocalNet");
    }

    #[test]
    fn test_network_serialization() {
        let network = Network::MainNet;
        let serialized = serde_json::to_string(&network).unwrap();
        let deserialized: Network = serde_json::from_str(&serialized).unwrap();
        assert_eq!(network, deserialized);
    }

    #[test]
    fn test_lora_base_url() {
        assert_eq!(
            Network::MainNet.lora_base_url(),
            "https://lora.algokit.io/mainnet"
        );
        assert_eq!(
            Network::TestNet.lora_base_url(),
            "https://lora.algokit.io/testnet"
        );
        assert_eq!(
            Network::LocalNet.lora_base_url(),
            "https://lora.algokit.io/localnet"
        );
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
}
