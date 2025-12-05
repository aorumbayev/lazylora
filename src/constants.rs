//! Application constants for the LazyLora TUI.
//!
//! This module provides centralized constants for UI dimensions, display symbols,
//! and numeric values used throughout the application. Grouping these constants
//! improves maintainability and ensures consistency across the codebase.

// TODO: Remove these allows after full integration in Stage 2
#![allow(dead_code)]
#![allow(unused_imports)]

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
/// The header contains the logo and network status indicator.
pub const HEADER_HEIGHT: u16 = 3;

/// Height of the section title area (in rows).
///
/// Used for the "Explore" section title and live mode toggle.
pub const TITLE_HEIGHT: u16 = 3;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    /// Height of each block item in the blocks list.
    pub block_height: u16,
    /// Height of each transaction item in the transactions list.
    pub txn_height: u16,
    /// Height of the application header.
    pub header_height: u16,
    /// Height of section titles.
    pub title_height: u16,
}

impl Dimensions {
    /// Creates a new `Dimensions` instance with custom values.
    ///
    /// # Arguments
    ///
    /// * `block_height` - Height for block list items
    /// * `txn_height` - Height for transaction list items
    /// * `header_height` - Height for the application header
    /// * `title_height` - Height for section titles
    ///
    /// # Returns
    ///
    /// A new `Dimensions` instance.
    #[must_use]
    pub const fn new(
        block_height: u16,
        txn_height: u16,
        header_height: u16,
        title_height: u16,
    ) -> Self {
        Self {
            block_height,
            txn_height,
            header_height,
            title_height,
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
            title_height: TITLE_HEIGHT,
        }
    }
}

// ============================================================================
// Display Symbols
// ============================================================================

/// Unicode symbol for Algorand currency display.
///
/// This diamond-like symbol (◈) represents the Algo currency in the UI.
pub const ALGO_SYMBOL: &str = "◈";

/// Unicode symbol for Algorand Standard Asset display.
///
/// This filled diamond symbol (◆) represents ASAs in the UI.
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
pub const MICROALGOS_PER_ALGO_U64: u64 = 1_000_000;

// ============================================================================
// Formatting Constants
// ============================================================================

/// Default decimal places for Algo amount display.
pub const ALGO_DECIMALS: u32 = 6;

/// Maximum address length before truncation.
///
/// Algorand addresses are 58 characters. This constant defines when
/// to apply truncation for display purposes.
pub const MAX_ADDRESS_DISPLAY_LENGTH: usize = 58;

/// Default truncated address length for compact displays.
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

    #[test]
    fn test_dimension_constants() {
        assert_eq!(BLOCK_HEIGHT, 3);
        assert_eq!(TXN_HEIGHT, 4);
        assert_eq!(HEADER_HEIGHT, 3);
        assert_eq!(TITLE_HEIGHT, 3);
    }

    #[test]
    fn test_dimensions_default() {
        let dims = Dimensions::default();
        assert_eq!(dims.block_height, BLOCK_HEIGHT);
        assert_eq!(dims.txn_height, TXN_HEIGHT);
        assert_eq!(dims.header_height, HEADER_HEIGHT);
        assert_eq!(dims.title_height, TITLE_HEIGHT);
    }

    #[test]
    fn test_dimensions_new() {
        let dims = Dimensions::new(5, 6, 4, 2);
        assert_eq!(dims.block_height, 5);
        assert_eq!(dims.txn_height, 6);
        assert_eq!(dims.header_height, 4);
        assert_eq!(dims.title_height, 2);
    }

    #[test]
    fn test_blocks_per_page() {
        let dims = Dimensions::default();
        assert_eq!(dims.blocks_per_page(30), 10); // 30 / 3 = 10
        assert_eq!(dims.blocks_per_page(15), 5); // 15 / 3 = 5
        assert_eq!(dims.blocks_per_page(2), 0); // 2 / 3 = 0 (integer division)
    }

    #[test]
    fn test_transactions_per_page() {
        let dims = Dimensions::default();
        assert_eq!(dims.transactions_per_page(40), 10); // 40 / 4 = 10
        assert_eq!(dims.transactions_per_page(20), 5); // 20 / 4 = 5
        assert_eq!(dims.transactions_per_page(3), 0); // 3 / 4 = 0 (integer division)
    }

    #[test]
    fn test_display_symbols() {
        assert_eq!(ALGO_SYMBOL, "◈");
        assert_eq!(ASSET_SYMBOL, "◆");
    }

    #[test]
    fn test_numeric_constants() {
        assert_eq!(MICROALGOS_PER_ALGO, 1_000_000.0);
        assert_eq!(MICROALGOS_PER_ALGO_U64, 1_000_000);
        assert_eq!(ALGO_DECIMALS, 6);
    }

    #[test]
    fn test_formatting_constants() {
        assert_eq!(MAX_ADDRESS_DISPLAY_LENGTH, 58);
        assert_eq!(DEFAULT_TRUNCATED_ADDRESS_LENGTH, 20);
    }

    #[test]
    fn test_microalgos_to_algos() {
        assert_eq!(microalgos_to_algos(0), 0.0);
        assert_eq!(microalgos_to_algos(1_000_000), 1.0);
        assert_eq!(microalgos_to_algos(5_500_000), 5.5);
        assert_eq!(microalgos_to_algos(123_456), 0.123456);
    }

    #[test]
    fn test_algos_to_microalgos() {
        assert_eq!(algos_to_microalgos(0.0), 0);
        assert_eq!(algos_to_microalgos(1.0), 1_000_000);
        assert_eq!(algos_to_microalgos(5.5), 5_500_000);
        assert_eq!(algos_to_microalgos(0.123456), 123_456);
    }

    #[test]
    fn test_format_algo() {
        assert_eq!(format_algo(0), "0.000000 ALGO");
        assert_eq!(format_algo(1_000_000), "1.000000 ALGO");
        assert_eq!(format_algo(5_500_000), "5.500000 ALGO");
        assert_eq!(format_algo(123_456), "0.123456 ALGO");
    }

    #[test]
    fn test_format_algo_with_symbol() {
        assert_eq!(format_algo_with_symbol(0), "◈ 0.000000");
        assert_eq!(format_algo_with_symbol(1_000_000), "◈ 1.000000");
        assert_eq!(format_algo_with_symbol(5_000_000), "◈ 5.000000");
    }

    #[test]
    fn test_conversion_roundtrip() {
        // Test that converting back and forth preserves the value
        let original = 12_345_678_u64;
        let algos = microalgos_to_algos(original);
        let back = algos_to_microalgos(algos);
        assert_eq!(original, back);
    }
}
