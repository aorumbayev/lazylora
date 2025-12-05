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

use crate::domain::Network;

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
/// * `network` - The currently selected Algorand network
/// * `show_live` - Whether live updates are enabled
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    /// The currently selected Algorand network.
    pub network: Network,
    /// Whether live updates are enabled.
    pub show_live: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: Network::MainNet,
            show_live: true,
        }
    }
}

impl AppConfig {
    /// Creates a new `AppConfig` with default values.
    ///
    /// # Returns
    ///
    /// A new `AppConfig` instance with:
    /// - `network`: `Network::MainNet`
    /// - `show_live`: `true`
    #[must_use]
    #[allow(dead_code)] // Part of config API
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `AppConfig` with the specified network.
    ///
    /// # Arguments
    ///
    /// * `network` - The Algorand network to use
    ///
    /// # Returns
    ///
    /// A new `AppConfig` instance with the specified network and live updates enabled.
    #[must_use]
    #[allow(dead_code)] // Part of config API
    pub const fn with_network(network: Network) -> Self {
        Self {
            network,
            show_live: true,
        }
    }

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
        let mut path = dirs::config_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not find config directory"))?;
        path.push(APP_NAME);
        fs::create_dir_all(&path)?;
        path.push(CONFIG_FILE);
        Ok(path)
    }

    /// Returns the path to the configuration directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined.
    ///
    /// # Returns
    ///
    /// The path to the configuration directory.
    #[allow(dead_code)] // Part of config API
    pub fn config_dir() -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not find config directory"))?;
        path.push(APP_NAME);
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
        Self::try_load().unwrap_or_default()
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

    /// Saves the configuration to disk, ignoring any errors.
    ///
    /// This is useful for best-effort saves where failure is acceptable
    /// (e.g., during shutdown).
    #[allow(dead_code)] // Part of config API
    pub fn save_silent(&self) {
        if let Err(e) = self.save() {
            eprintln!("Failed to save configuration: {e}");
        }
    }

    /// Updates the network and saves the configuration.
    ///
    /// # Arguments
    ///
    /// * `network` - The new network to set
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be saved.
    #[allow(dead_code)] // Part of config API
    pub fn set_network(&mut self, network: Network) -> Result<()> {
        self.network = network;
        self.save()
    }

    /// Updates the live updates setting and saves the configuration.
    ///
    /// # Arguments
    ///
    /// * `show_live` - Whether to enable live updates
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be saved.
    #[allow(dead_code)] // Part of config API
    pub fn set_show_live(&mut self, show_live: bool) -> Result<()> {
        self.show_live = show_live;
        self.save()
    }

    /// Toggles the live updates setting and saves the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be saved.
    ///
    /// # Returns
    ///
    /// The new value of `show_live`.
    #[allow(dead_code)] // Part of config API
    pub fn toggle_show_live(&mut self) -> Result<bool> {
        self.show_live = !self.show_live;
        self.save()?;
        Ok(self.show_live)
    }

    /// Checks if the configuration file exists.
    ///
    /// # Returns
    ///
    /// `true` if the configuration file exists, `false` otherwise.
    #[must_use]
    #[allow(dead_code)] // Part of config API
    pub fn exists() -> bool {
        Self::config_path().map(|p| p.exists()).unwrap_or(false)
    }

    /// Deletes the configuration file if it exists.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration path cannot be determined
    /// - The file exists but cannot be deleted
    ///
    /// # Returns
    ///
    /// `Ok(())` on success (including if the file didn't exist).
    #[allow(dead_code)] // Part of config API
    pub fn delete() -> Result<()> {
        let path = Self::config_path()?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.network, Network::MainNet);
        assert!(config.show_live);
    }

    #[test]
    fn test_new_config() {
        let config = AppConfig::new();
        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn test_with_network() {
        let config = AppConfig::with_network(Network::TestNet);
        assert_eq!(config.network, Network::TestNet);
        assert!(config.show_live);
    }

    #[test]
    fn test_serialization() {
        let config = AppConfig {
            network: Network::TestNet,
            show_live: false,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_json_format() {
        let config = AppConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();

        // Verify it's valid JSON with expected fields
        assert!(json.contains("network"));
        assert!(json.contains("show_live"));
    }

    #[test]
    fn test_config_dir_has_app_name() {
        if let Ok(dir) = AppConfig::config_dir() {
            let dir_name = dir.file_name().and_then(|n| n.to_str());
            assert_eq!(dir_name, Some(APP_NAME));
        }
        // Skip test if config dir unavailable (CI environments)
    }

    #[test]
    fn test_config_path_has_json_extension() {
        if let Ok(path) = AppConfig::config_path() {
            let extension = path.extension().and_then(|e| e.to_str());
            assert_eq!(extension, Some("json"));
        }
        // Skip test if config dir unavailable (CI environments)
    }

    #[test]
    fn test_all_networks_serialize() {
        for network in [Network::MainNet, Network::TestNet, Network::LocalNet] {
            let config = AppConfig::with_network(network);
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(config.network, deserialized.network);
        }
    }

    #[test]
    fn test_load_returns_default_on_missing_file() {
        // This tests that load() gracefully handles missing files
        // by returning defaults (it uses try_load internally)
        let config = AppConfig::load();
        // Should get default values - this is the expected behavior
        // when no config file exists
        assert_eq!(config.network, Network::MainNet);
    }
}
