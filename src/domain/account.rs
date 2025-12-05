//! Account types for Algorand blockchain.
//!
//! This module defines account-related types including basic account info
//! for search results and detailed account information for popups.

use super::nfd::NfdInfo;

// ============================================================================
// Account Info
// ============================================================================

/// Basic account info for search results display.
///
/// Contains essential account metadata for display in search results.
#[derive(Debug, Clone, PartialEq)]
pub struct AccountInfo {
    /// The Algorand address (58 characters).
    pub address: String,
    /// Account balance in microAlgos.
    pub balance: u64,
    /// Pending rewards in microAlgos.
    pub pending_rewards: u64,
    /// Reward base for calculating pending rewards.
    pub reward_base: u64,
    /// Account status (e.g., "Offline", "Online").
    pub status: String,
    /// Number of assets the account holds.
    pub assets_count: usize,
    /// Number of assets created by the account.
    pub created_assets_count: usize,
}

impl AccountInfo {
    /// Create a new `AccountInfo` with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `address` - The Algorand address
    /// * `balance` - Account balance in microAlgos
    /// * `pending_rewards` - Pending rewards in microAlgos
    /// * `reward_base` - Reward base value
    /// * `status` - Account status string
    /// * `assets_count` - Number of asset holdings
    /// * `created_assets_count` - Number of created assets
    ///
    /// # Returns
    ///
    /// A new `AccountInfo` instance.
    #[must_use]
    #[allow(dead_code)] // Part of AccountInfo public API
    pub fn new(
        address: String,
        balance: u64,
        pending_rewards: u64,
        reward_base: u64,
        status: String,
        assets_count: usize,
        created_assets_count: usize,
    ) -> Self {
        Self {
            address,
            balance,
            pending_rewards,
            reward_base,
            status,
            assets_count,
            created_assets_count,
        }
    }

    /// Returns the balance formatted in Algos (not microAlgos).
    ///
    /// # Returns
    ///
    /// The balance in Algos as a floating point number.
    #[must_use]
    #[allow(dead_code)] // Part of AccountInfo public API
    pub fn balance_in_algos(&self) -> f64 {
        self.balance as f64 / 1_000_000.0
    }
}

// ============================================================================
// Account Details
// ============================================================================

/// Detailed account information for popup display.
///
/// Contains comprehensive account data including participation info,
/// asset holdings, and application state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AccountDetails {
    /// The Algorand address (58 characters).
    pub address: String,
    /// Account balance in microAlgos.
    pub balance: u64,
    /// Minimum balance required in microAlgos.
    pub min_balance: u64,
    /// Pending rewards in microAlgos.
    pub pending_rewards: u64,
    /// Total rewards earned in microAlgos.
    pub rewards: u64,
    /// Reward base for calculating pending rewards.
    pub reward_base: u64,
    /// Account status (e.g., "Offline", "Online").
    pub status: String,
    /// Number of apps the account is opted into.
    pub total_apps_opted_in: usize,
    /// Number of assets the account is opted into.
    pub total_assets_opted_in: usize,
    /// Number of apps created by this account.
    pub total_created_apps: usize,
    /// Number of assets created by this account.
    pub total_created_assets: usize,
    /// Number of boxes owned by this account.
    pub total_boxes: usize,
    /// Authorized address if account is rekeyed.
    pub auth_addr: Option<String>,
    /// Participation info for online accounts.
    pub participation: Option<ParticipationInfo>,
    /// Asset holdings (limited to first entries).
    pub assets: Vec<AccountAssetHolding>,
    /// Created assets (limited to first entries).
    pub created_assets: Vec<CreatedAssetInfo>,
    /// App local states (limited to first entries).
    pub apps_local_state: Vec<AppLocalState>,
    /// Created apps (limited to first entries).
    pub created_apps: Vec<CreatedAppInfo>,
    /// NFD name if available (MainNet/TestNet only).
    pub nfd: Option<NfdInfo>,
}

impl AccountDetails {
    /// Returns the balance formatted in Algos (not microAlgos).
    ///
    /// # Returns
    ///
    /// The balance in Algos as a floating point number.
    #[must_use]
    #[allow(dead_code)] // Part of AccountDetails public API
    pub fn balance_in_algos(&self) -> f64 {
        self.balance as f64 / 1_000_000.0
    }

    /// Returns the minimum balance formatted in Algos.
    ///
    /// # Returns
    ///
    /// The minimum balance in Algos as a floating point number.
    #[must_use]
    #[allow(dead_code)] // Part of AccountDetails public API
    pub fn min_balance_in_algos(&self) -> f64 {
        self.min_balance as f64 / 1_000_000.0
    }

    /// Returns whether the account is online (participating in consensus).
    ///
    /// # Returns
    ///
    /// `true` if the account status is "Online", `false` otherwise.
    #[must_use]
    #[allow(dead_code)] // Part of AccountDetails public API
    pub fn is_online(&self) -> bool {
        self.status == "Online"
    }

    /// Returns whether the account is rekeyed.
    ///
    /// # Returns
    ///
    /// `true` if the account has an authorized address, `false` otherwise.
    #[must_use]
    #[allow(dead_code)] // Part of AccountDetails public API
    pub fn is_rekeyed(&self) -> bool {
        self.auth_addr.is_some()
    }
}

// ============================================================================
// Participation Info
// ============================================================================

/// Participation key info for online accounts.
///
/// Contains the participation keys and validity range for consensus participation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParticipationInfo {
    /// First round the participation key is valid for.
    pub vote_first: u64,
    /// Last round the participation key is valid for.
    pub vote_last: u64,
    /// Key dilution parameter.
    pub vote_key_dilution: u64,
    /// Selection participation key (Base64 encoded).
    pub selection_key: String,
    /// Vote participation key (Base64 encoded).
    pub vote_key: String,
    /// State proof key (Base64 encoded), if available.
    pub state_proof_key: Option<String>,
}

impl ParticipationInfo {
    /// Returns whether the participation keys are currently valid.
    ///
    /// # Arguments
    ///
    /// * `current_round` - The current network round
    ///
    /// # Returns
    ///
    /// `true` if current_round is within the valid range.
    #[must_use]
    #[allow(dead_code)] // Part of ParticipationInfo public API
    pub fn is_valid_at(&self, current_round: u64) -> bool {
        current_round >= self.vote_first && current_round <= self.vote_last
    }

    /// Returns the number of rounds remaining until key expiration.
    ///
    /// # Arguments
    ///
    /// * `current_round` - The current network round
    ///
    /// # Returns
    ///
    /// The number of rounds until expiration, or 0 if already expired.
    #[must_use]
    #[allow(dead_code)] // Part of ParticipationInfo public API
    pub fn rounds_remaining(&self, current_round: u64) -> u64 {
        self.vote_last.saturating_sub(current_round)
    }
}

// ============================================================================
// Account Asset Holding
// ============================================================================

/// Asset holding info for an account.
///
/// Represents an asset that the account has opted into and holds.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AccountAssetHolding {
    /// The asset ID.
    pub asset_id: u64,
    /// Amount of the asset held.
    pub amount: u64,
    /// Whether the holding is frozen.
    pub is_frozen: bool,
}

impl AccountAssetHolding {
    /// Create a new `AccountAssetHolding`.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The asset ID
    /// * `amount` - Amount held
    /// * `is_frozen` - Whether the holding is frozen
    ///
    /// # Returns
    ///
    /// A new `AccountAssetHolding` instance.
    #[must_use]
    #[allow(dead_code)] // Part of AccountAssetHolding public API
    pub fn new(asset_id: u64, amount: u64, is_frozen: bool) -> Self {
        Self {
            asset_id,
            amount,
            is_frozen,
        }
    }
}

// ============================================================================
// Created Asset Info
// ============================================================================

/// Created asset summary.
///
/// Brief information about an asset created by an account.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CreatedAssetInfo {
    /// The asset ID.
    pub asset_id: u64,
    /// Asset name.
    pub name: String,
    /// Asset unit name.
    pub unit_name: String,
}

impl CreatedAssetInfo {
    /// Create a new `CreatedAssetInfo`.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The asset ID
    /// * `name` - Asset name
    /// * `unit_name` - Asset unit name
    ///
    /// # Returns
    ///
    /// A new `CreatedAssetInfo` instance.
    #[must_use]
    #[allow(dead_code)] // Part of CreatedAssetInfo public API
    pub fn new(asset_id: u64, name: String, unit_name: String) -> Self {
        Self {
            asset_id,
            name,
            unit_name,
        }
    }
}

// ============================================================================
// App Local State
// ============================================================================

/// App local state summary.
///
/// Brief information about an application's local state for an account.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppLocalState {
    /// The application ID.
    pub app_id: u64,
    /// Number of uint64 values in local state.
    pub schema_num_uint: u64,
    /// Number of byte slice values in local state.
    pub schema_num_byte_slice: u64,
}

impl AppLocalState {
    /// Create a new `AppLocalState`.
    ///
    /// # Arguments
    ///
    /// * `app_id` - The application ID
    /// * `schema_num_uint` - Number of uint64 values
    /// * `schema_num_byte_slice` - Number of byte slice values
    ///
    /// # Returns
    ///
    /// A new `AppLocalState` instance.
    #[must_use]
    #[allow(dead_code)] // Part of AppLocalState public API
    pub fn new(app_id: u64, schema_num_uint: u64, schema_num_byte_slice: u64) -> Self {
        Self {
            app_id,
            schema_num_uint,
            schema_num_byte_slice,
        }
    }

    /// Returns the total number of local state entries.
    ///
    /// # Returns
    ///
    /// The sum of uint and byte slice entries.
    #[must_use]
    #[allow(dead_code)] // Part of AppLocalState public API
    pub fn total_entries(&self) -> u64 {
        self.schema_num_uint + self.schema_num_byte_slice
    }
}

// ============================================================================
// Created App Info
// ============================================================================

/// Created app summary.
///
/// Brief information about an application created by an account.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CreatedAppInfo {
    /// The application ID.
    pub app_id: u64,
}

impl CreatedAppInfo {
    /// Create a new `CreatedAppInfo`.
    ///
    /// # Arguments
    ///
    /// * `app_id` - The application ID
    ///
    /// # Returns
    ///
    /// A new `CreatedAppInfo` instance.
    #[must_use]
    #[allow(dead_code)] // Part of CreatedAppInfo public API
    pub fn new(app_id: u64) -> Self {
        Self { app_id }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_info_new() {
        let info = AccountInfo::new(
            "TESTADDRESS".to_string(),
            1_000_000,
            100,
            50,
            "Online".to_string(),
            5,
            2,
        );
        assert_eq!(info.address, "TESTADDRESS");
        assert_eq!(info.balance, 1_000_000);
        assert_eq!(info.assets_count, 5);
        assert_eq!(info.created_assets_count, 2);
    }

    #[test]
    fn test_account_info_balance_in_algos() {
        let info = AccountInfo::new(
            "TEST".to_string(),
            5_500_000, // 5.5 Algos
            0,
            0,
            "Offline".to_string(),
            0,
            0,
        );
        assert!((info.balance_in_algos() - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_account_details_default() {
        let details = AccountDetails::default();
        assert_eq!(details.address, "");
        assert_eq!(details.balance, 0);
        assert!(!details.is_online());
        assert!(!details.is_rekeyed());
    }

    #[test]
    fn test_account_details_is_online() {
        let mut details = AccountDetails::default();
        details.status = "Offline".to_string();
        assert!(!details.is_online());

        details.status = "Online".to_string();
        assert!(details.is_online());
    }

    #[test]
    fn test_account_details_is_rekeyed() {
        let mut details = AccountDetails::default();
        assert!(!details.is_rekeyed());

        details.auth_addr = Some("AUTHADDR".to_string());
        assert!(details.is_rekeyed());
    }

    #[test]
    fn test_participation_info_is_valid_at() {
        let part = ParticipationInfo {
            vote_first: 100,
            vote_last: 200,
            vote_key_dilution: 10,
            selection_key: "key".to_string(),
            vote_key: "key".to_string(),
            state_proof_key: None,
        };

        assert!(!part.is_valid_at(50));
        assert!(part.is_valid_at(100));
        assert!(part.is_valid_at(150));
        assert!(part.is_valid_at(200));
        assert!(!part.is_valid_at(201));
    }

    #[test]
    fn test_participation_info_rounds_remaining() {
        let part = ParticipationInfo {
            vote_first: 100,
            vote_last: 200,
            vote_key_dilution: 10,
            selection_key: "key".to_string(),
            vote_key: "key".to_string(),
            state_proof_key: None,
        };

        assert_eq!(part.rounds_remaining(150), 50);
        assert_eq!(part.rounds_remaining(200), 0);
        assert_eq!(part.rounds_remaining(250), 0);
    }

    #[test]
    fn test_account_asset_holding_new() {
        let holding = AccountAssetHolding::new(12345, 1000, true);
        assert_eq!(holding.asset_id, 12345);
        assert_eq!(holding.amount, 1000);
        assert!(holding.is_frozen);
    }

    #[test]
    fn test_created_asset_info_new() {
        let asset = CreatedAssetInfo::new(12345, "MyToken".to_string(), "MTK".to_string());
        assert_eq!(asset.asset_id, 12345);
        assert_eq!(asset.name, "MyToken");
        assert_eq!(asset.unit_name, "MTK");
    }

    #[test]
    fn test_app_local_state_new() {
        let state = AppLocalState::new(12345, 5, 3);
        assert_eq!(state.app_id, 12345);
        assert_eq!(state.schema_num_uint, 5);
        assert_eq!(state.schema_num_byte_slice, 3);
        assert_eq!(state.total_entries(), 8);
    }

    #[test]
    fn test_created_app_info_new() {
        let app = CreatedAppInfo::new(12345);
        assert_eq!(app.app_id, 12345);
    }
}
