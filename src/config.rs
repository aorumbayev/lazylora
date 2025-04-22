use crate::algorand::Network;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque; // Using VecDeque for potentially easier addition/removal if needed later

const CONFIG_APP_NAME: &str = "lazylora";
const CONFIG_FILE_NAME: &str = "settings"; // confy uses this by default if None provided, but being explicit can help

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub selected_network: Network,
    // Store only the custom networks. Standard networks are implicitly known.
    pub custom_networks: VecDeque<Network>,
    // Add other persistent settings here in the future if needed
    // e.g., default_search_type: SearchType,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // Default to MainNet on first launch or if config fails to load
            selected_network: Network::MainNet,
            custom_networks: VecDeque::new(),
        }
    }
}

/// Loads the application settings using confy.
/// Returns default settings if loading fails or the config file doesn't exist.
pub fn load_settings() -> Result<AppSettings> {
    match confy::load(CONFIG_APP_NAME, Some(CONFIG_FILE_NAME)) {
        Ok(settings) => Ok(settings),
        Err(e) => {
            // Log the error but return default settings
            eprintln!(
                "Failed to load configuration: {}. Using default settings.",
                e
            );
            // Use Ok here because failure to load isn't a fatal error, we just use defaults.
            Ok(AppSettings::default())
        }
    }
}

/// Saves the application settings using confy.
pub fn save_settings(settings: &AppSettings) -> Result<()> {
    confy::store(CONFIG_APP_NAME, Some(CONFIG_FILE_NAME), settings)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to save configuration: {}", e))
}

// Helper function to add a custom network and save settings
pub fn add_custom_network(
    settings: &mut AppSettings,
    name: String,
    algod_url: String,
    indexer_url: String,
    algod_token: Option<String>,
) -> Result<()> {
    let new_network = Network::Custom {
        name,
        algod_url,
        indexer_url,
        algod_token,
    };
    // Avoid adding duplicates (simple check based on name)
    if !settings
        .custom_networks
        .iter()
        .any(|n| n.as_str() == new_network.as_str())
    {
        settings.custom_networks.push_back(new_network);
        save_settings(settings)?;
    } else {
        // Maybe return an error or indication that it already exists?
        // For now, just don't add it and don't save.
        color_eyre::eyre::bail!(
            "Custom network with name '{}' already exists.",
            new_network.as_str()
        );
    }
    Ok(())
}

// Helper function to update the selected network and save settings
pub fn set_selected_network(settings: &mut AppSettings, network: Network) -> Result<()> {
    settings.selected_network = network;
    save_settings(settings)
}

// Helper function to get a list of all available networks (Standard + Custom)
pub fn get_available_networks(settings: &AppSettings) -> Vec<Network> {
    let mut networks = vec![Network::MainNet, Network::TestNet, Network::LocalNet];
    networks.extend(settings.custom_networks.iter().cloned());
    networks
}
