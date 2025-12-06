//! Search-related types and auto-detection logic.
//!
//! This module contains the search type enumeration and heuristics
//! for auto-detecting what kind of search to perform based on user input.

// ============================================================================
// Search Type
// ============================================================================

/// The type of search to perform.
///
/// This determines how search queries are interpreted and which
/// API endpoints are called.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchType {
    /// Search for transactions by ID.
    #[default]
    Transaction,
    /// Search for assets by ID.
    Asset,
    /// Search for accounts by address or NFD name.
    Account,
    /// Search for blocks by round number.
    Block,
    /// Search for applications by ID.
    Application,
}

impl SearchType {
    /// Returns the display string for this search type.
    ///
    /// # Returns
    ///
    /// A static string describing the search type.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Transaction => "Transaction",
            Self::Asset => "Asset",
            Self::Account => "Account",
            Self::Block => "Block",
            Self::Application => "Application",
        }
    }

    /// Cycles to the next search type.
    ///
    /// # Returns
    ///
    /// The next search type in the cycle.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Transaction => Self::Block,
            Self::Block => Self::Account,
            Self::Account => Self::Asset,
            Self::Asset => Self::Application,
            Self::Application => Self::Transaction,
        }
    }
}

// ============================================================================
// Search Type Auto-Detection
// ============================================================================

/// Auto-detect the search type based on input pattern.
///
/// Uses the following heuristics:
/// - 52-char uppercase alphanumeric → Transaction ID
/// - Pure digits → Block number (small) or Asset ID (large)
/// - 58-char uppercase alphanumeric → Account address
/// - Contains ".algo" or looks like NFD name → Account (NFD)
/// - Otherwise → None (unknown format)
#[must_use]
pub fn detect_search_type(query: &str) -> Option<SearchType> {
    let trimmed = query.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Check for transaction ID (52 chars, uppercase alphanumeric)
    if trimmed.len() == 52
        && trimmed
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Some(SearchType::Transaction);
    }

    // Check for valid Algorand address (58 chars, uppercase alphanumeric)
    if trimmed.len() == 58
        && trimmed
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Some(SearchType::Account);
    }

    // Check for NFD name (contains .algo or looks like name)
    if looks_like_nfd_name(trimmed) {
        return Some(SearchType::Account);
    }

    // Check for pure integer (block or asset)
    if let Ok(num) = trimmed.parse::<u64>() {
        // Use heuristic: blocks are typically < 100M, assets can be much larger
        // But really, both are valid - we'll default to Block for simplicity
        // since blocks are more commonly searched by number
        if num < 100_000_000 {
            return Some(SearchType::Block);
        }
        return Some(SearchType::Asset);
    }

    // Partial transaction ID (40-60 chars, mostly uppercase)
    if (40..=60).contains(&trimmed.len())
        && trimmed.chars().filter(|c| c.is_ascii_uppercase()).count() > trimmed.len() / 2
    {
        return Some(SearchType::Transaction);
    }

    None
}

/// Check if a query string looks like an NFD name.
#[must_use]
fn looks_like_nfd_name(query: &str) -> bool {
    let trimmed = query.trim().to_lowercase();

    if trimmed.is_empty() {
        return false;
    }

    // If it ends with .algo, it's definitely an NFD name
    if let Some(name_part) = trimmed.strip_suffix(".algo") {
        return !name_part.is_empty()
            && name_part
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
    }

    // Could be just the name without .algo suffix
    // It's likely an NFD if it's a short alphanumeric string that isn't a number
    trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && trimmed.parse::<u64>().is_err()
        && trimmed.len() < 30  // NFD names are typically short
        && trimmed.len() >= 2 // At least 2 chars for a name
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_type_cycle_behavior() {
        // Default is Transaction, cycles through all 5 types
        let mut current = SearchType::default();
        assert_eq!(current, SearchType::Transaction);

        let expected_cycle = [
            SearchType::Block,
            SearchType::Account,
            SearchType::Asset,
            SearchType::Application,
            SearchType::Transaction, // Back to start
        ];

        for expected in expected_cycle {
            current = current.next();
            assert_eq!(current, expected);
        }

        // All types have string representations
        assert!(!SearchType::Transaction.as_str().is_empty());
        assert!(!SearchType::Block.as_str().is_empty());
        assert!(!SearchType::Account.as_str().is_empty());
        assert!(!SearchType::Asset.as_str().is_empty());
        assert!(!SearchType::Application.as_str().is_empty());
    }

    #[test]
    fn test_detect_search_type_all_cases() {
        // Empty string
        assert_eq!(detect_search_type(""), None);
        assert_eq!(detect_search_type("   "), None);

        // Transaction ID (52 chars)
        let txn_id = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        assert_eq!(txn_id.len(), 52);
        assert_eq!(detect_search_type(txn_id), Some(SearchType::Transaction));

        // Account address (58 chars)
        let address = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        assert_eq!(address.len(), 58);
        assert_eq!(detect_search_type(address), Some(SearchType::Account));

        // Block number (small integer)
        assert_eq!(detect_search_type("12345"), Some(SearchType::Block));
        assert_eq!(detect_search_type("1000000"), Some(SearchType::Block));

        // Asset ID (large integer)
        assert_eq!(detect_search_type("100000000"), Some(SearchType::Asset));
        assert_eq!(detect_search_type("999999999"), Some(SearchType::Asset));

        // NFD name
        assert_eq!(detect_search_type("alice.algo"), Some(SearchType::Account));
        assert_eq!(detect_search_type("bob"), Some(SearchType::Account));
        assert_eq!(detect_search_type("my-nfd"), Some(SearchType::Account));
    }
}
