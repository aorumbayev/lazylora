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
/// Used for scroll calculations when determining visible range.
pub const DEFAULT_VISIBLE_BLOCKS: u16 = 10;

/// Default number of visible transactions in the transactions list.
///
/// Used for scroll calculations when determining visible range.
pub const DEFAULT_VISIBLE_TRANSACTIONS: u16 = 10;

// ============================================================================
// UI Dimensions Struct
// ============================================================================

/// Grouped UI dimension constants for layout calculations.
///
/// This struct provides a convenient way to access related dimension constants
/// and can be extended to support different layout configurations.
///
/// # Example
///
/// ```rust
/// use lazylora::constants::Dimensions;
///
/// let dims = Dimensions::default();
/// let items_per_page = area_height / dims.block_height as usize;
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    /// Height of each block item in the blocks list.
    pub block_height: u16,
    /// Height of each transaction item in the transactions list.
    pub txn_height: u16,
    /// Height of the application header (includes search bar).
    pub header_height: u16,
}

impl Dimensions {
    /// Creates a new `Dimensions` instance with custom values.
    ///
    /// # Arguments
    ///
    /// * `block_height` - Height for block list items
    /// * `txn_height` - Height for transaction list items
    /// * `header_height` - Height for the application header
    ///
    /// # Returns
    ///
    /// A new `Dimensions` instance.
    #[must_use]
    #[allow(dead_code)]
    pub const fn new(block_height: u16, txn_height: u16, header_height: u16) -> Self {
        Self {
            block_height,
            txn_height,
            header_height,
        }
    }

    /// Calculates the number of block items that fit in a given height.
    ///
    /// # Arguments
    ///
    /// * `available_height` - The available height in rows
    ///
    /// # Returns
    ///
    /// The number of block items that can be displayed.
    #[must_use]
    #[allow(dead_code)]
    pub const fn blocks_per_page(&self, available_height: u16) -> usize {
        (available_height / self.block_height) as usize
    }

    /// Calculates the number of transaction items that fit in a given height.
    ///
    /// # Arguments
    ///
    /// * `available_height` - The available height in rows
    ///
    /// # Returns
    ///
    /// The number of transaction items that can be displayed.
    #[must_use]
    #[allow(dead_code)]
    pub const fn transactions_per_page(&self, available_height: u16) -> usize {
        (available_height / self.txn_height) as usize
    }
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            block_height: BLOCK_HEIGHT,
            txn_height: TXN_HEIGHT,
            header_height: HEADER_HEIGHT,
        }
    }
}

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
// Formatting Constants
// ============================================================================

/// Default decimal places for Algo amount display.
#[allow(dead_code)]
pub const ALGO_DECIMALS: u32 = 6;

/// Maximum address length before truncation.
///
/// Algorand addresses are 58 characters. This constant defines when
/// to apply truncation for display purposes.
#[allow(dead_code)]
pub const MAX_ADDRESS_DISPLAY_LENGTH: usize = 58;

/// Default truncated address length for compact displays.
#[allow(dead_code)]
pub const DEFAULT_TRUNCATED_ADDRESS_LENGTH: usize = 20;

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

    #[test]
    fn test_dimensions_pagination() {
        let dims = Dimensions::default();

        // Verify default values
        assert_eq!(dims.block_height, BLOCK_HEIGHT);
        assert_eq!(dims.txn_height, TXN_HEIGHT);

        // Test pagination calculations
        assert_eq!(dims.blocks_per_page(30), 10); // 30 / 3
        assert_eq!(dims.transactions_per_page(40), 10); // 40 / 4
        assert_eq!(dims.blocks_per_page(2), 0); // Below threshold
    }

    /// Sanity check that constants haven't drifted.
    #[test]
    fn test_constant_values() {
        assert_eq!(MICROALGOS_PER_ALGO, 1_000_000.0);
        assert_eq!(MICROALGOS_PER_ALGO_U64, 1_000_000);
        assert_eq!(ALGO_DECIMALS, 6);
        assert_eq!(ALGO_SYMBOL, "◈");
        assert_eq!(ASSET_SYMBOL, "◆");
    }
}
