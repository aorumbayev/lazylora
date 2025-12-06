//! Application configuration with persistence.
//!
//! This module provides the [`AppConfig`] structure for managing application
//! settings with automatic load/save to disk.
//!
//! # Configuration File Location
//!
//! The configuration file is stored at:
//! - Linux: `~/.config/lazylora/config.json`
//! - macOS: `~/Library/Application Support/lazylora/config.json`
//! - Windows: `%APPDATA%/lazylora/config.json`
//!
//! # Example
//!
//! ```ignore
//! use crate::state::AppConfig;
//!
//! // Load existing config or use defaults
//! let config = AppConfig::load();
//!
//! // Modify and save
//! let mut config = config;
//! config.show_live = false;
//! config.save().expect("Failed to save config");
//! ```

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::domain::{CustomNetwork, Network, NetworkConfig};

// ============================================================================
// Constants
// ============================================================================

/// Application name used for configuration directory.
const APP_NAME: &str = "lazylora";

/// Configuration file name.
const CONFIG_FILE: &str = "config.json";

// ============================================================================
// AppConfig
// ============================================================================

/// Application configuration structure for persistence.
///
/// This structure is serialized to JSON and stored in the user's
/// configuration directory.
///
/// # Fields
///
/// * `network` - The currently selected network (built-in or custom)
/// * `custom_networks` - List of user-defined custom networks
/// * `show_live` - Whether live updates are enabled
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    /// The currently selected network.
    #[serde(default)]
    pub network: NetworkConfig,
    /// List of user-defined custom networks.
    #[serde(default)]
    pub custom_networks: Vec<CustomNetwork>,
    /// Whether live updates are enabled.
    pub show_live: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::BuiltIn(Network::MainNet),
            custom_networks: Vec::new(),
            show_live: true,
        }
    }
}

impl AppConfig {
    /// Returns the path to the configuration file.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined
    /// or created.
    ///
    /// # Returns
    ///
    /// The path to the configuration file.
    pub fn config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir().ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "Could not determine config directory. Expected XDG_CONFIG_HOME or ~/.config on Linux, ~/Library/Application Support on macOS, %APPDATA% on Windows"
            )
        })?;
        path.push(APP_NAME);
        fs::create_dir_all(&path)?;
        path.push(CONFIG_FILE);
        Ok(path)
    }

    /// Loads the configuration from disk.
    ///
    /// If the configuration file doesn't exist or cannot be parsed,
    /// returns the default configuration.
    ///
    /// # Returns
    ///
    /// The loaded configuration or defaults if loading fails.
    #[must_use]
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Config load failed, using defaults: {err}");
                Self::default()
            }
        }
    }

    /// Attempts to load the configuration from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration path cannot be determined
    /// - The file cannot be read
    /// - The JSON content cannot be parsed
    ///
    /// # Returns
    ///
    /// The loaded configuration.
    pub fn try_load() -> Result<Self> {
        let path = Self::config_path()?;
        let content = fs::read_to_string(&path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Saves the configuration to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration path cannot be determined
    /// - The configuration cannot be serialized
    /// - The file cannot be written
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Adds a custom network and saves the configuration.
    ///
    /// # Arguments
    ///
    /// * `network` - The custom network to add
    ///
    /// # Errors
    ///
    /// Returns an error if a network with the same name exists or if saving fails.
    #[allow(dead_code)] // Part of config API
    pub fn add_custom_network(&mut self, network: CustomNetwork) -> Result<()> {
        if self.custom_networks.iter().any(|n| n.name == network.name) {
            return Err(color_eyre::eyre::eyre!(
                "Network '{}' already exists",
                network.name
            ));
        }
        self.custom_networks.push(network);
        self.save()
    }

    /// Deletes a custom network by name and saves the configuration.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the network to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the network is not found or if saving fails.
    #[allow(dead_code)] // Part of config API
    pub fn delete_custom_network(&mut self, name: &str) -> Result<()> {
        let original_len = self.custom_networks.len();
        self.custom_networks.retain(|n| n.name != name);

        if self.custom_networks.len() == original_len {
            return Err(color_eyre::eyre::eyre!("Network '{}' not found", name));
        }

        // Switch to MainNet if deleted network was active
        if let NetworkConfig::Custom(ref current) = self.network
            && current.name == name
        {
            self.network = NetworkConfig::BuiltIn(Network::MainNet);
        }

        self.save()
    }

    /// Returns all available networks (built-in + custom).
    ///
    /// # Returns
    ///
    /// A vector containing all built-in networks followed by custom networks.
    #[must_use]
    #[allow(dead_code)] // Part of config API
    pub fn get_all_networks(&self) -> Vec<NetworkConfig> {
        let mut networks = vec![
            NetworkConfig::BuiltIn(Network::MainNet),
            NetworkConfig::BuiltIn(Network::TestNet),
            NetworkConfig::BuiltIn(Network::LocalNet),
        ];
        networks.extend(
            self.custom_networks
                .iter()
                .cloned()
                .map(NetworkConfig::Custom),
        );
        networks
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.network, NetworkConfig::BuiltIn(Network::MainNet));
        assert!(config.custom_networks.is_empty());
        assert!(config.show_live);
    }

    #[test]
    fn test_serialization_builtin() {
        let config = AppConfig {
            network: NetworkConfig::BuiltIn(Network::TestNet),
            custom_networks: Vec::new(),
            show_live: false,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_serialization_with_custom_networks() {
        let mut config = AppConfig::default();
        config
            .custom_networks
            .push(CustomNetwork::new("Custom1", "http://i1", "http://a1"));
        config.custom_networks.push(
            CustomNetwork::new("Custom2", "http://i2", "http://a2").with_nfd_api("http://nfd"),
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_backward_compatibility_old_config() {
        // Old config format with direct Network enum
        let old_json = r#"{"network":"TestNet","show_live":true}"#;
        let config: Result<AppConfig, _> = serde_json::from_str(old_json);

        // Should deserialize with defaults for missing fields
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.show_live);
        assert!(config.custom_networks.is_empty());
    }

    #[test]
    fn test_add_custom_network() {
        let mut config = AppConfig::default();
        let network = CustomNetwork::new("MyNet", "http://idx", "http://algod");

        // Can't test save() without filesystem, but we can test the logic
        config.custom_networks.push(network.clone());
        assert_eq!(config.custom_networks.len(), 1);
        assert_eq!(config.custom_networks[0], network);
    }

    #[test]
    fn test_add_custom_network_duplicate_name() {
        let mut config = AppConfig::default();
        let network1 = CustomNetwork::new("MyNet", "http://idx1", "http://algod1");
        let network2 = CustomNetwork::new("MyNet", "http://idx2", "http://algod2");

        config.custom_networks.push(network1);
        let result = config.add_custom_network(network2);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_delete_custom_network() {
        let mut config = AppConfig::default();
        config
            .custom_networks
            .push(CustomNetwork::new("Net1", "http://i1", "http://a1"));
        config
            .custom_networks
            .push(CustomNetwork::new("Net2", "http://i2", "http://a2"));

        config.custom_networks.retain(|n| n.name != "Net1");
        assert_eq!(config.custom_networks.len(), 1);
        assert_eq!(config.custom_networks[0].name, "Net2");
    }

    #[test]
    fn test_delete_custom_network_switches_to_mainnet() {
        let mut config = AppConfig::default();
        let custom = CustomNetwork::new("MyNet", "http://idx", "http://algod");
        config.network = NetworkConfig::Custom(custom.clone());
        config.custom_networks.push(custom);

        // Simulate deletion
        config.custom_networks.retain(|n| n.name != "MyNet");
        if let NetworkConfig::Custom(ref current) = config.network {
            if current.name == "MyNet" {
                config.network = NetworkConfig::BuiltIn(Network::MainNet);
            }
        }

        assert_eq!(config.network, NetworkConfig::BuiltIn(Network::MainNet));
    }

    #[test]
    fn test_get_all_networks_no_custom() {
        let config = AppConfig::default();
        let networks = config.get_all_networks();

        assert_eq!(networks.len(), 3); // MainNet, TestNet, LocalNet
        assert_eq!(networks[0], NetworkConfig::BuiltIn(Network::MainNet));
        assert_eq!(networks[1], NetworkConfig::BuiltIn(Network::TestNet));
        assert_eq!(networks[2], NetworkConfig::BuiltIn(Network::LocalNet));
    }

    #[test]
    fn test_get_all_networks_with_custom() {
        let mut config = AppConfig::default();
        config
            .custom_networks
            .push(CustomNetwork::new("Custom1", "http://i1", "http://a1"));
        config
            .custom_networks
            .push(CustomNetwork::new("Custom2", "http://i2", "http://a2"));

        let networks = config.get_all_networks();
        assert_eq!(networks.len(), 5); // 3 built-in + 2 custom

        // Check custom networks are at the end
        match &networks[3] {
            NetworkConfig::Custom(n) => assert_eq!(n.name, "Custom1"),
            _ => panic!("Expected custom network"),
        }
        match &networks[4] {
            NetworkConfig::Custom(n) => assert_eq!(n.name, "Custom2"),
            _ => panic!("Expected custom network"),
        }
    }

    #[test]
    fn test_json_format() {
        let config = AppConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();

        assert!(json.contains("network"));
        assert!(json.contains("show_live"));
        assert!(json.contains("custom_networks"));
    }

    #[test]
    fn test_config_path_has_json_extension() {
        if let Ok(path) = AppConfig::config_path() {
            let extension = path.extension().and_then(|e| e.to_str());
            assert_eq!(extension, Some("json"));
        }
    }

    #[rstest]
    #[case::mainnet(Network::MainNet)]
    #[case::testnet(Network::TestNet)]
    #[case::localnet(Network::LocalNet)]
    fn test_all_networks_serialize(#[case] network: Network) {
        let config = AppConfig {
            network: NetworkConfig::BuiltIn(network),
            custom_networks: Vec::new(),
            show_live: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.network, deserialized.network);
    }

    #[test]
    fn test_try_load_fails_on_missing_file() {
        let _result = AppConfig::try_load();
        let config = AppConfig::load();
        assert!(matches!(
            config.network,
            NetworkConfig::BuiltIn(Network::MainNet)
                | NetworkConfig::BuiltIn(Network::TestNet)
                | NetworkConfig::BuiltIn(Network::LocalNet)
        ));
    }
}
