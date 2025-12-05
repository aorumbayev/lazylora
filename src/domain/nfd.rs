//! NFD (Non-Fungible Domain) types for Algorand.
//!
//! This module defines types for interacting with NFDomains,
//! a naming service on Algorand that provides human-readable names
//! for Algorand addresses (e.g., "alice.algo").

// Helper methods are part of the public API but not yet all used in the application
#![allow(dead_code)]

use serde_json::Value;

// ============================================================================
// NFD Info
// ============================================================================

/// NFD (Non-Fungible Domain) information from the NFD API.
///
/// This is a simplified view of the NFD data for display purposes.
/// NFD is only available on MainNet and TestNet.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NfdInfo {
    /// The NFD name (e.g., "alice.algo").
    pub name: String,
    /// The deposit account address linked to this NFD.
    pub deposit_account: Option<String>,
    /// The owner address of this NFD.
    pub owner: Option<String>,
    /// Avatar URL if available.
    pub avatar_url: Option<String>,
    /// Whether this is a verified NFD.
    pub is_verified: bool,
}

impl NfdInfo {
    /// Create a new `NfdInfo` with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `name` - The NFD name
    /// * `deposit_account` - Optional deposit account address
    /// * `owner` - Optional owner address
    /// * `avatar_url` - Optional avatar URL
    /// * `is_verified` - Whether the NFD is verified
    ///
    /// # Returns
    ///
    /// A new `NfdInfo` instance.
    #[must_use]
    pub fn new(
        name: String,
        deposit_account: Option<String>,
        owner: Option<String>,
        avatar_url: Option<String>,
        is_verified: bool,
    ) -> Self {
        Self {
            name,
            deposit_account,
            owner,
            avatar_url,
            is_verified,
        }
    }

    /// Create a new NFD info from API response JSON.
    ///
    /// # Arguments
    ///
    /// * `json` - The JSON response from the NFD API
    ///
    /// # Returns
    ///
    /// A new `NfdInfo` parsed from the JSON data.
    #[must_use]
    pub fn from_json(json: &Value) -> Self {
        let name = json["name"].as_str().unwrap_or("").to_string();
        let deposit_account = json["depositAccount"].as_str().map(String::from);
        let owner = json["owner"].as_str().map(String::from);

        // Avatar can be in properties.userDefined.avatar or properties.verified.avatar
        let avatar_url = json["properties"]["verified"]["avatar"]
            .as_str()
            .or_else(|| json["properties"]["userDefined"]["avatar"].as_str())
            .map(String::from);

        // Check if there are verified caAlgo addresses (indicates verification)
        let is_verified = json["caAlgo"].as_array().is_some_and(|arr| !arr.is_empty());

        Self {
            name,
            deposit_account,
            owner,
            avatar_url,
            is_verified,
        }
    }

    /// Returns the primary address associated with this NFD.
    ///
    /// Prefers the deposit account, falls back to owner.
    ///
    /// # Returns
    ///
    /// The primary address, or `None` if neither is set.
    #[must_use]
    pub fn primary_address(&self) -> Option<&str> {
        self.deposit_account.as_deref().or(self.owner.as_deref())
    }

    /// Returns whether this NFD has an avatar.
    ///
    /// # Returns
    ///
    /// `true` if an avatar URL is available.
    #[must_use]
    pub fn has_avatar(&self) -> bool {
        self.avatar_url.is_some()
    }

    /// Returns the short form of the NFD name for display.
    ///
    /// If the name is longer than `max_len`, it will be truncated
    /// with "..." in the middle.
    ///
    /// # Arguments
    ///
    /// * `max_len` - Maximum length of the returned string
    ///
    /// # Returns
    ///
    /// The NFD name, possibly truncated.
    #[must_use]
    pub fn short_name(&self, max_len: usize) -> String {
        if self.name.len() <= max_len {
            self.name.clone()
        } else if max_len < 5 {
            self.name.chars().take(max_len).collect()
        } else {
            let prefix_len = (max_len - 3) / 2;
            let suffix_len = max_len - 3 - prefix_len;
            let prefix: String = self.name.chars().take(prefix_len).collect();
            let suffix: String = self
                .name
                .chars()
                .rev()
                .take(suffix_len)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            format!("{}...{}", prefix, suffix)
        }
    }

    /// Returns the base name without the ".algo" suffix.
    ///
    /// # Returns
    ///
    /// The name without ".algo", or the full name if it doesn't end with ".algo".
    #[must_use]
    pub fn base_name(&self) -> &str {
        self.name.strip_suffix(".algo").unwrap_or(&self.name)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfd_info_new() {
        let nfd = NfdInfo::new(
            "alice.algo".to_string(),
            Some("DEPOSIT_ADDR".to_string()),
            Some("OWNER_ADDR".to_string()),
            Some("https://avatar.example.com".to_string()),
            true,
        );

        assert_eq!(nfd.name, "alice.algo");
        assert_eq!(nfd.deposit_account, Some("DEPOSIT_ADDR".to_string()));
        assert_eq!(nfd.owner, Some("OWNER_ADDR".to_string()));
        assert!(nfd.has_avatar());
        assert!(nfd.is_verified);
    }

    #[test]
    fn test_nfd_info_default() {
        let nfd = NfdInfo::default();
        assert_eq!(nfd.name, "");
        assert!(nfd.deposit_account.is_none());
        assert!(nfd.owner.is_none());
        assert!(nfd.avatar_url.is_none());
        assert!(!nfd.is_verified);
    }

    #[test]
    fn test_nfd_info_from_json() {
        let json = serde_json::json!({
            "name": "alice.algo",
            "depositAccount": "DEPOSIT_ADDR",
            "owner": "OWNER_ADDR",
            "properties": {
                "verified": {
                    "avatar": "https://avatar.example.com"
                }
            },
            "caAlgo": ["VERIFIED_ADDR"]
        });

        let nfd = NfdInfo::from_json(&json);
        assert_eq!(nfd.name, "alice.algo");
        assert_eq!(nfd.deposit_account, Some("DEPOSIT_ADDR".to_string()));
        assert_eq!(nfd.owner, Some("OWNER_ADDR".to_string()));
        assert_eq!(
            nfd.avatar_url,
            Some("https://avatar.example.com".to_string())
        );
        assert!(nfd.is_verified);
    }

    #[test]
    fn test_nfd_info_from_json_user_defined_avatar() {
        let json = serde_json::json!({
            "name": "bob.algo",
            "owner": "OWNER_ADDR",
            "properties": {
                "userDefined": {
                    "avatar": "https://user-avatar.example.com"
                }
            }
        });

        let nfd = NfdInfo::from_json(&json);
        assert_eq!(nfd.name, "bob.algo");
        assert_eq!(
            nfd.avatar_url,
            Some("https://user-avatar.example.com".to_string())
        );
        assert!(!nfd.is_verified);
    }

    #[test]
    fn test_nfd_info_from_json_minimal() {
        let json = serde_json::json!({
            "name": "minimal.algo"
        });

        let nfd = NfdInfo::from_json(&json);
        assert_eq!(nfd.name, "minimal.algo");
        assert!(nfd.deposit_account.is_none());
        assert!(nfd.owner.is_none());
        assert!(nfd.avatar_url.is_none());
        assert!(!nfd.is_verified);
    }

    #[test]
    fn test_nfd_info_primary_address() {
        let mut nfd = NfdInfo::default();
        assert!(nfd.primary_address().is_none());

        nfd.owner = Some("OWNER".to_string());
        assert_eq!(nfd.primary_address(), Some("OWNER"));

        nfd.deposit_account = Some("DEPOSIT".to_string());
        assert_eq!(nfd.primary_address(), Some("DEPOSIT"));
    }

    #[test]
    fn test_nfd_info_has_avatar() {
        let mut nfd = NfdInfo::default();
        assert!(!nfd.has_avatar());

        nfd.avatar_url = Some("https://example.com".to_string());
        assert!(nfd.has_avatar());
    }

    #[test]
    fn test_nfd_info_short_name() {
        let nfd = NfdInfo::new(
            "verylongnamethatisunusual.algo".to_string(),
            None,
            None,
            None,
            false,
        );

        // Full name fits
        assert_eq!(nfd.short_name(50), "verylongnamethatisunusual.algo");

        // Truncated
        let short = nfd.short_name(15);
        assert!(short.contains("..."));
        assert_eq!(short.len(), 15);
    }

    #[test]
    fn test_nfd_info_base_name() {
        let nfd = NfdInfo::new("alice.algo".to_string(), None, None, None, false);
        assert_eq!(nfd.base_name(), "alice");

        let nfd_no_suffix = NfdInfo::new("alice".to_string(), None, None, None, false);
        assert_eq!(nfd_no_suffix.base_name(), "alice");
    }
}
