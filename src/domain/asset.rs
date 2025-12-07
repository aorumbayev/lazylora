//! Asset types for Algorand blockchain.
//!
//! This module defines asset-related types including basic asset info
//! for search results and detailed asset information for popups.

// ============================================================================
// Asset Info
// ============================================================================

/// Basic asset info for search results display.
///
/// Contains essential asset metadata for display in search results.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetInfo {
    /// The asset ID.
    pub id: u64,
    /// Asset name.
    pub name: String,
    /// Asset unit name (ticker symbol).
    pub unit_name: String,
    /// Creator address.
    pub creator: String,
    /// Total supply in base units.
    pub total: u64,
    /// Number of decimal places for display formatting.
    pub decimals: u64,
    /// URL with asset metadata.
    pub url: String,
}

impl AssetInfo {
    /// Create a new `AssetInfo` with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `id` - The asset ID
    /// * `name` - Asset name
    /// * `unit_name` - Asset unit name (ticker)
    /// * `creator` - Creator address
    /// * `total` - Total supply in base units
    /// * `decimals` - Number of decimal places
    /// * `url` - Metadata URL
    ///
    /// # Returns
    ///
    /// A new `AssetInfo` instance.
    #[must_use]
    #[allow(dead_code)]
    pub fn new(
        id: u64,
        name: String,
        unit_name: String,
        creator: String,
        total: u64,
        decimals: u64,
        url: String,
    ) -> Self {
        Self {
            id,
            name,
            unit_name,
            creator,
            total,
            decimals,
            url,
        }
    }

    /// Returns the formatted total supply accounting for decimals.
    ///
    /// # Returns
    ///
    /// The total supply as a formatted floating point number.
    #[must_use]
    #[allow(dead_code)]
    pub fn formatted_total(&self) -> f64 {
        if self.decimals == 0 {
            self.total as f64
        } else {
            self.total as f64 / 10_f64.powi(self.decimals as i32)
        }
    }

    /// Returns a display string for the asset.
    ///
    /// # Returns
    ///
    /// A string in the format "Name (UNIT)" or just "Name" if no unit name.
    #[must_use]
    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        if self.unit_name.is_empty() {
            self.name.clone()
        } else {
            format!("{} ({})", self.name, self.unit_name)
        }
    }
}

// ============================================================================
// Asset Details
// ============================================================================

/// Detailed asset information for popup display.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AssetDetails {
    /// The asset ID.
    pub id: u64,
    /// Asset name.
    pub name: String,
    /// Asset unit name (ticker symbol).
    pub unit_name: String,
    /// Creator address.
    pub creator: String,
    /// Total supply in base units.
    pub total: u64,
    /// Number of decimal places for display formatting.
    pub decimals: u64,
    /// URL with asset metadata.
    pub url: String,
    /// Metadata hash (Base64 encoded).
    pub metadata_hash: Option<String>,
    /// Whether holdings are frozen by default.
    pub default_frozen: bool,
    /// Manager address - can change asset configuration.
    pub manager: Option<String>,
    /// Reserve address - holds non-minted units.
    pub reserve: Option<String>,
    /// Freeze address - can freeze/unfreeze holdings.
    pub freeze: Option<String>,
    /// Clawback address - can revoke holdings.
    pub clawback: Option<String>,
    /// Whether the asset has been deleted.
    pub deleted: bool,
    /// Round when the asset was created.
    pub created_at_round: Option<u64>,
}

impl AssetDetails {
    /// Returns the formatted total supply accounting for decimals.
    ///
    /// # Returns
    ///
    /// The total supply as a formatted floating point number.
    #[must_use]
    #[allow(dead_code)]
    pub fn formatted_total(&self) -> f64 {
        if self.decimals == 0 {
            self.total as f64
        } else {
            self.total as f64 / 10_f64.powi(self.decimals as i32)
        }
    }

    /// Returns a display string for the asset.
    ///
    /// # Returns
    ///
    /// A string in the format "Name (UNIT)" or just "Name" if no unit name.
    #[must_use]
    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        if self.unit_name.is_empty() {
            self.name.clone()
        } else {
            format!("{} ({})", self.name, self.unit_name)
        }
    }

    /// Returns whether the asset has a manager address.
    ///
    /// # Returns
    ///
    /// `true` if a manager address is set.
    #[must_use]
    #[allow(dead_code)]
    pub fn has_manager(&self) -> bool {
        self.manager.is_some()
    }

    /// Returns whether the asset has clawback capability.
    ///
    /// # Returns
    ///
    /// `true` if a clawback address is set.
    #[must_use]
    #[allow(dead_code)]
    pub fn has_clawback(&self) -> bool {
        self.clawback.is_some()
    }

    /// Returns whether the asset has freeze capability.
    ///
    /// # Returns
    ///
    /// `true` if a freeze address is set.
    #[must_use]
    #[allow(dead_code)]
    pub fn has_freeze(&self) -> bool {
        self.freeze.is_some()
    }

    /// Returns whether the asset is immutable (no manager).
    ///
    /// An immutable asset cannot have its configuration changed.
    ///
    /// # Returns
    ///
    /// `true` if no manager address is set.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_immutable(&self) -> bool {
        self.manager.is_none()
    }

    /// Convert detailed asset info to basic asset info.
    ///
    /// # Returns
    ///
    /// An `AssetInfo` containing the basic fields.
    #[must_use]
    #[allow(dead_code)]
    pub fn to_basic_info(&self) -> AssetInfo {
        AssetInfo {
            id: self.id,
            name: self.name.clone(),
            unit_name: self.unit_name.clone(),
            creator: self.creator.clone(),
            total: self.total,
            decimals: self.decimals,
            url: self.url.clone(),
        }
    }
}

// ============================================================================
// Asset Params (for transaction parsing)
// ============================================================================

/// Asset parameters used in asset configuration transactions.
///
/// This struct is used when parsing asset creation or modification transactions.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub struct AssetParams {
    /// Total supply in base units.
    pub total: Option<u64>,
    /// Number of decimal places.
    pub decimals: Option<u64>,
    /// Whether holdings are frozen by default.
    pub default_frozen: Option<bool>,
    /// Asset name.
    pub name: Option<String>,
    /// Asset unit name (ticker symbol).
    pub unit_name: Option<String>,
    /// URL with asset metadata.
    pub url: Option<String>,
    /// Metadata hash (Base64 encoded).
    pub metadata_hash: Option<String>,
    /// Manager address.
    pub manager: Option<String>,
    /// Reserve address.
    pub reserve: Option<String>,
    /// Freeze address.
    pub freeze: Option<String>,
    /// Clawback address.
    pub clawback: Option<String>,
}

impl AssetParams {
    /// Returns whether these params represent an asset creation.
    ///
    /// Asset creation requires at least total and decimals to be set.
    ///
    /// # Returns
    ///
    /// `true` if this looks like asset creation params.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_creation(&self) -> bool {
        self.total.is_some()
    }

    /// Returns whether any address fields are set.
    ///
    /// # Returns
    ///
    /// `true` if any management address is present.
    #[must_use]
    #[allow(dead_code)]
    pub fn has_any_address(&self) -> bool {
        self.manager.is_some()
            || self.reserve.is_some()
            || self.freeze.is_some()
            || self.clawback.is_some()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_info_new() {
        let info = AssetInfo::new(
            12345,
            "MyToken".to_string(),
            "MTK".to_string(),
            "CREATOR".to_string(),
            1_000_000_000,
            6,
            "https://example.com".to_string(),
        );
        assert_eq!(info.id, 12345);
        assert_eq!(info.name, "MyToken");
        assert_eq!(info.unit_name, "MTK");
        assert_eq!(info.decimals, 6);
    }

    #[test]
    fn test_asset_info_formatted_total() {
        let info = AssetInfo::new(
            1,
            "Test".to_string(),
            "TST".to_string(),
            "CREATOR".to_string(),
            1_000_000_000, // 1 billion base units
            6,             // 6 decimals = 1000.0
            String::new(),
        );
        assert!((info.formatted_total() - 1000.0).abs() < f64::EPSILON);

        let info_no_decimals = AssetInfo::new(
            1,
            "Test".to_string(),
            "TST".to_string(),
            "CREATOR".to_string(),
            1_000_000,
            0,
            String::new(),
        );
        assert!((info_no_decimals.formatted_total() - 1_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_asset_info_display_name() {
        let info = AssetInfo::new(
            1,
            "MyToken".to_string(),
            "MTK".to_string(),
            "CREATOR".to_string(),
            100,
            0,
            String::new(),
        );
        assert_eq!(info.display_name(), "MyToken (MTK)");

        let info_no_unit = AssetInfo::new(
            1,
            "MyToken".to_string(),
            String::new(),
            "CREATOR".to_string(),
            100,
            0,
            String::new(),
        );
        assert_eq!(info_no_unit.display_name(), "MyToken");
    }

    #[test]
    fn test_asset_details_default() {
        let details = AssetDetails::default();
        assert_eq!(details.id, 0);
        assert!(details.name.is_empty());
        assert!(!details.deleted);
        assert!(!details.default_frozen);
    }

    #[test]
    fn test_asset_details_capabilities() {
        let mut details = AssetDetails::default();
        assert!(!details.has_manager());
        assert!(!details.has_clawback());
        assert!(!details.has_freeze());
        assert!(details.is_immutable());

        details.manager = Some("MANAGER".to_string());
        assert!(details.has_manager());
        assert!(!details.is_immutable());

        details.clawback = Some("CLAWBACK".to_string());
        assert!(details.has_clawback());

        details.freeze = Some("FREEZE".to_string());
        assert!(details.has_freeze());
    }

    #[test]
    fn test_asset_details_to_basic_info() {
        let details = AssetDetails {
            id: 12345,
            name: "MyToken".to_string(),
            unit_name: "MTK".to_string(),
            creator: "CREATOR".to_string(),
            total: 1_000_000,
            decimals: 6,
            url: "https://example.com".to_string(),
            metadata_hash: Some("hash".to_string()),
            default_frozen: false,
            manager: Some("MANAGER".to_string()),
            reserve: None,
            freeze: None,
            clawback: None,
            deleted: false,
            created_at_round: Some(100),
        };

        let basic = details.to_basic_info();
        assert_eq!(basic.id, 12345);
        assert_eq!(basic.name, "MyToken");
        assert_eq!(basic.unit_name, "MTK");
        assert_eq!(basic.creator, "CREATOR");
        assert_eq!(basic.total, 1_000_000);
        assert_eq!(basic.decimals, 6);
        assert_eq!(basic.url, "https://example.com");
    }

    #[test]
    fn test_asset_params_is_creation() {
        let mut params = AssetParams::default();
        assert!(!params.is_creation());

        params.total = Some(1_000_000);
        assert!(params.is_creation());
    }

    #[test]
    fn test_asset_params_has_any_address() {
        let mut params = AssetParams::default();
        assert!(!params.has_any_address());

        params.manager = Some("MANAGER".to_string());
        assert!(params.has_any_address());

        params = AssetParams::default();
        params.reserve = Some("RESERVE".to_string());
        assert!(params.has_any_address());

        params = AssetParams::default();
        params.freeze = Some("FREEZE".to_string());
        assert!(params.has_any_address());

        params = AssetParams::default();
        params.clawback = Some("CLAWBACK".to_string());
        assert!(params.has_any_address());
    }
}
