//! NFD (NFDomains) API methods for AlgoClient.

use color_eyre::Result;
use serde_json::Value;

use super::AlgoClient;
use crate::domain::NfdInfo;

impl AlgoClient {
    /// Look up an NFD (NFDomains) by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or JSON parsing fails.
    pub async fn get_nfd_by_name(&self, name: &str) -> Result<Option<NfdInfo>> {
        let Some(nfd_url) = self.nfd_api_url() else {
            return Ok(None); // NFD not supported on this network
        };

        // Normalize the name - ensure it ends with .algo
        let normalized_name = if name.ends_with(".algo") {
            name.to_string()
        } else {
            format!("{}.algo", name)
        };

        let url = format!("{}/nfd/{}?view=brief", nfd_url, normalized_name);

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let json: Value = resp.json().await?;
                    Ok(Some(NfdInfo::from_json(&json)))
                } else {
                    Ok(None) // NFD not found or other errors
                }
            }
            Err(_) => Ok(None), // Network errors, treat as not found
        }
    }

    /// Reverse lookup - get the primary NFD for an Algorand address.
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or JSON parsing fails.
    pub async fn get_nfd_for_address(&self, address: &str) -> Result<Option<NfdInfo>> {
        let Some(nfd_url) = self.nfd_api_url() else {
            return Ok(None); // NFD not supported on this network
        };

        // Validate address format first
        if address.len() != 58
            || !address
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return Ok(None);
        }

        let url = format!(
            "{}/nfd/lookup?address={}&view=brief&allowUnverified=true",
            nfd_url, address
        );

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let json: Value = resp.json().await?;
                    // The response is a map of address -> NFD info
                    if let Some(nfd_data) = json.get(address) {
                        Ok(Some(NfdInfo::from_json(nfd_data)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None) // 404 or other errors
                }
            }
            Err(_) => Ok(None), // Network errors
        }
    }

    /// Check if a query string looks like an NFD name.
    /// NFD names end with .algo or could be just the name part.
    #[must_use]
    pub fn looks_like_nfd_name(query: &str) -> bool {
        let trimmed = query.trim().to_lowercase();

        // Must have at least 1 character before .algo or be a simple name
        if trimmed.is_empty() {
            return false;
        }

        // If it ends with .algo, check the part before it
        if let Some(name_part) = trimmed.strip_suffix(".algo") {
            // NFD names must be at least 1 char and contain only valid chars
            !name_part.is_empty()
                && name_part
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        } else {
            // Could be just the name without .algo suffix
            // It's likely an NFD if it contains alphanumeric chars and isn't a valid address/number
            trimmed
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                && trimmed.parse::<u64>().is_err()
                && trimmed.len() < 58 // Not an Algorand address
        }
    }
}
