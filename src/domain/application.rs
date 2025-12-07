//! Application types for Algorand blockchain.
//!
//! This module defines application-related types including basic app info
//! for search results and detailed application information for popups.

// ============================================================================
// Application Info
// ============================================================================

/// Basic application info for search results display.
///
/// Contains essential application metadata for display in search results.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ApplicationInfo {
    /// The application ID.
    pub app_id: u64,
    /// The creator's address.
    pub creator: String,
    /// Whether the application is deleted.
    pub deleted: bool,
}

impl ApplicationInfo {
    /// Create a new `ApplicationInfo`.
    ///
    /// # Arguments
    ///
    /// * `app_id` - The application ID
    /// * `creator` - The creator's address
    /// * `deleted` - Whether the application is deleted
    #[must_use]
    #[allow(dead_code)]
    pub fn new(app_id: u64, creator: String, deleted: bool) -> Self {
        Self {
            app_id,
            creator,
            deleted,
        }
    }
}

// ============================================================================
// Application Details
// ============================================================================

/// Detailed application information for popup display.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ApplicationDetails {
    /// The application ID.
    pub app_id: u64,
    /// The creator's address.
    pub creator: String,
    /// The application account address.
    pub app_address: String,
    /// Whether the application is deleted.
    pub deleted: bool,

    // === State Schema ===
    /// Number of global state byte slices.
    pub global_state_byte: u64,
    /// Number of global state uint64 values.
    pub global_state_uint: u64,
    /// Number of local state byte slices.
    pub local_state_byte: u64,
    /// Number of local state uint64 values.
    pub local_state_uint: u64,
    /// Extra program pages.
    pub extra_program_pages: Option<u64>,

    // === Programs ===
    /// Approval program (Base64 encoded).
    pub approval_program: Option<String>,
    /// Clear state program (Base64 encoded).
    pub clear_state_program: Option<String>,

    // === Global State ===
    /// Global state key-value pairs.
    pub global_state: Vec<AppStateValue>,

    // === Creation Info ===
    /// Round when the application was created.
    pub created_at_round: Option<u64>,
}

impl ApplicationDetails {
    /// Returns the total number of global state entries.
    #[must_use]
    #[allow(dead_code)]
    pub fn total_global_state(&self) -> u64 {
        self.global_state_byte + self.global_state_uint
    }

    /// Returns the total number of local state entries.
    #[must_use]
    #[allow(dead_code)]
    pub fn total_local_state(&self) -> u64 {
        self.local_state_byte + self.local_state_uint
    }
}

// ============================================================================
// App State Value
// ============================================================================

/// Application state key-value pair.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppStateValue {
    /// The key name (decoded from Base64).
    pub key: String,
    /// The value type ("Bytes" or "Uint").
    pub value_type: String,
    /// The value (decoded/formatted).
    pub value: String,
}

impl AppStateValue {
    /// Create a new `AppStateValue`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key name
    /// * `value_type` - The value type
    /// * `value` - The formatted value
    #[must_use]
    #[allow(dead_code)]
    pub fn new(key: String, value_type: String, value: String) -> Self {
        Self {
            key,
            value_type,
            value,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_info_creation() {
        let info = ApplicationInfo::new(12345, "CREATOR_ADDR".to_string(), false);
        assert_eq!(info.app_id, 12345);
        assert_eq!(info.creator, "CREATOR_ADDR");
        assert!(!info.deleted);
    }

    #[test]
    fn test_application_details_state_totals() {
        let details = ApplicationDetails {
            global_state_byte: 3,
            global_state_uint: 5,
            local_state_byte: 2,
            local_state_uint: 10,
            ..Default::default()
        };

        assert_eq!(details.total_global_state(), 8);
        assert_eq!(details.total_local_state(), 12);
    }

    #[test]
    fn test_app_state_value_creation() {
        let state = AppStateValue::new(
            "fee_collector".to_string(),
            "Bytes".to_string(),
            "ADDR123...".to_string(),
        );
        assert_eq!(state.key, "fee_collector");
        assert_eq!(state.value_type, "Bytes");
        assert_eq!(state.value, "ADDR123...");
    }
}
