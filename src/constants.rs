//! Application constants for the LazyLora TUI.
//!
//! This module provides centralized constants for UI dimensions, display symbols,
//! and numeric values used throughout the application. Grouping these constants
//! improves maintainability and ensures consistency across the codebase.

// ============================================================================
// UI Dimension Constants
// ============================================================================

/// Height of each block item in the blocks list (in rows).
///
/// Each block entry displays:
/// - Line 1: Block ID and transaction count
/// - Line 2: Timestamp
/// - Line 3: Empty spacer
pub const BLOCK_HEIGHT: u16 = 3;

/// Height of each transaction item in the transactions list (in rows).
///
/// Each transaction entry displays:
/// - Line 1: Transaction ID and type badge
/// - Line 2: From address
/// - Line 3: To address
/// - Line 4: Empty spacer
pub const TXN_HEIGHT: u16 = 4;

/// Height of the application header area (in rows).
///
/// The header contains:
/// - Border (top)
/// - Row 1: Logo, Live indicator, Network status
/// - Border (bottom)
pub const HEADER_HEIGHT: u16 = 3;

/// Height of the search bar section (in rows).
///
/// The search bar contains:
/// - Border (top)
/// - Row 1: Search input with type indicator
/// - Border (bottom)
pub const SEARCH_BAR_HEIGHT: u16 = 3;

/// Default terminal width used for layout calculations.
///
/// This is a fallback value when terminal width cannot be determined.
pub const DEFAULT_TERMINAL_WIDTH: u16 = 100;

/// Default number of visible blocks in the blocks list.
///
/// Set to 10 for reasonable scroll behavior on typical 80x24 terminals.
/// Used for scroll calculations when determining visible range.
pub const DEFAULT_VISIBLE_BLOCKS: u16 = 10;

/// Default number of visible transactions in the transactions list.
///
/// Set to 10 to match block list behavior and fit typical terminal height.
/// Used for scroll calculations when determining visible range.
pub const DEFAULT_VISIBLE_TRANSACTIONS: u16 = 10;

// ============================================================================
// Display Symbols
// ============================================================================

/// Unicode symbol for Algorand currency display.
///
/// This diamond-like symbol (◈) represents the Algo currency in the UI.
#[allow(dead_code)]
pub const ALGO_SYMBOL: &str = "◈";

/// Unicode symbol for Algorand Standard Asset display.
///
/// This filled diamond symbol (◆) represents ASAs in the UI.
#[allow(dead_code)]
pub const ASSET_SYMBOL: &str = "◆";

// ============================================================================
// Numeric Constants
// ============================================================================

/// Number of microAlgos per Algo.
///
/// Algorand uses microAlgos as the base unit, where 1 Algo = 1,000,000 microAlgos.
/// This constant is used for converting between display values and raw amounts.
pub const MICROALGOS_PER_ALGO: f64 = 1_000_000.0;

/// Number of microAlgos per Algo as an integer.
///
/// Useful for integer arithmetic without floating-point conversion.
#[allow(dead_code)]
pub const MICROALGOS_PER_ALGO_U64: u64 = 1_000_000;

// ============================================================================
// Helper Functions
// ============================================================================

/// Converts microAlgos to Algos.
///
/// # Arguments
///
/// * `microalgos` - Amount in microAlgos
///
/// # Returns
///
/// The equivalent amount in Algos as a floating-point number.
///
/// # Example
///
/// ```rust
/// use lazylora::constants::microalgos_to_algos;
///
/// let algos = microalgos_to_algos(5_000_000);
/// assert_eq!(algos, 5.0);
/// ```
#[must_use]
#[allow(dead_code)]
pub const fn microalgos_to_algos(microalgos: u64) -> f64 {
    microalgos as f64 / MICROALGOS_PER_ALGO
}

/// Converts Algos to microAlgos.
///
/// # Arguments
///
/// * `algos` - Amount in Algos
///
/// # Returns
///
/// The equivalent amount in microAlgos.
///
/// # Example
///
/// ```rust
/// use lazylora::constants::algos_to_microalgos;
///
/// let microalgos = algos_to_microalgos(5.0);
/// assert_eq!(microalgos, 5_000_000);
/// ```
#[must_use]
#[allow(dead_code)]
pub fn algos_to_microalgos(algos: f64) -> u64 {
    (algos * MICROALGOS_PER_ALGO) as u64
}

/// Formats a microAlgo amount as a human-readable Algo string.
///
/// # Arguments
///
/// * `microalgos` - Amount in microAlgos
///
/// # Returns
///
/// A formatted string like "5.000000 ALGO".
///
/// # Example
///
/// ```rust
/// use lazylora::constants::format_algo;
///
/// let formatted = format_algo(5_500_000);
/// assert_eq!(formatted, "5.500000 ALGO");
/// ```
#[must_use]
#[allow(dead_code)]
pub fn format_algo(microalgos: u64) -> String {
    let algos = microalgos_to_algos(microalgos);
    format!("{algos:.6} ALGO")
}

/// Formats a microAlgo amount with the Algo symbol.
///
/// # Arguments
///
/// * `microalgos` - Amount in microAlgos
///
/// # Returns
///
/// A formatted string like "◈ 5.000000".
///
/// # Example
///
/// ```rust
/// use lazylora::constants::format_algo_with_symbol;
///
/// let formatted = format_algo_with_symbol(5_000_000);
/// assert_eq!(formatted, "◈ 5.000000");
/// ```
#[must_use]
#[allow(dead_code)]
pub fn format_algo_with_symbol(microalgos: u64) -> String {
    let algos = microalgos_to_algos(microalgos);
    format!("{ALGO_SYMBOL} {algos:.6}")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::zero(0, 0.0)]
    #[case::one_algo(1_000_000, 1.0)]
    #[case::five_and_half(5_500_000, 5.5)]
    #[case::fractional(123_456, 0.123456)]
    fn test_microalgos_conversion(#[case] microalgos: u64, #[case] algos: f64) {
        assert_eq!(microalgos_to_algos(microalgos), algos);
        assert_eq!(algos_to_microalgos(algos), microalgos);
    }

    #[rstest]
    #[case::zero(0, "0.000000 ALGO", "◈ 0.000000")]
    #[case::one_algo(1_000_000, "1.000000 ALGO", "◈ 1.000000")]
    #[case::five_and_half(5_500_000, "5.500000 ALGO", "◈ 5.500000")]
    fn test_format_algo_functions(
        #[case] microalgos: u64,
        #[case] expected_plain: &str,
        #[case] expected_symbol: &str,
    ) {
        assert_eq!(format_algo(microalgos), expected_plain);
        assert_eq!(format_algo_with_symbol(microalgos), expected_symbol);
    }
}
