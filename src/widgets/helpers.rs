//! Helper functions for formatting and displaying Algorand data.
//!
//! This module contains utility functions used across various widgets for:
//! - Address truncation and formatting
//! - Amount formatting (ALGO and ASA)
//! - Transaction type icons and codes

use crate::domain::TxnType;

// ============================================================================
// Re-exported Constants
// ============================================================================

pub use crate::constants::{ALGO_SYMBOL, ASSET_SYMBOL, MICROALGOS_PER_ALGO};

// ============================================================================
// Address Formatting
// ============================================================================

/// Truncate an address to fit in the given width.
///
/// If the address is longer than `max_len`, it will be truncated with an ellipsis
/// in the middle (e.g., "AAAA...AAAA").
///
/// # Arguments
///
/// * `addr` - The address to truncate
/// * `max_len` - The maximum length of the resulting string
///
/// # Returns
///
/// A truncated address string or the original if it fits
///
/// # Examples
///
/// ```ignore
/// let addr = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
/// let truncated = truncate_address(addr, 20);
/// assert!(truncated.len() <= 20);
/// assert!(truncated.contains("..."));
/// ```
#[must_use]
pub fn truncate_address(addr: &str, max_len: usize) -> String {
    if addr.len() <= max_len {
        return addr.to_string();
    }

    if max_len < 7 {
        // Need at least "A...A" (5 chars) + some buffer
        return addr.chars().take(max_len).collect();
    }

    // Reserve 3 chars for "..."
    let available = max_len - 3;
    let prefix_len = available.div_ceil(2); // Round up for prefix
    let suffix_len = available / 2;

    let prefix: String = addr.chars().take(prefix_len).collect();
    let suffix: String = addr.chars().skip(addr.len() - suffix_len).collect();

    format!("{prefix}...{suffix}")
}

// ============================================================================
// Amount Formatting
// ============================================================================

/// Format microAlgos to Algos with proper decimals.
///
/// # Arguments
///
/// * `microalgos` - The amount in microAlgos
///
/// # Returns
///
/// A formatted string like "5.000000 ALGO"
///
/// # Examples
///
/// ```ignore
/// assert_eq!(format_algo_amount(1_000_000), "1.000000 ALGO");
/// assert_eq!(format_algo_amount(5_500_000), "5.500000 ALGO");
/// ```
#[must_use]
pub fn format_algo_amount(microalgos: u64) -> String {
    let algos = microalgos as f64 / MICROALGOS_PER_ALGO;
    format!("{algos:.6} ALGO")
}

/// Format asset amount with optional decimals.
///
/// # Arguments
///
/// * `amount` - The raw asset amount
/// * `decimals` - Optional decimal places for formatting
///
/// # Returns
///
/// A formatted string with commas for thousands
///
/// # Examples
///
/// ```ignore
/// assert_eq!(format_asset_amount(1000, None), "1,000");
/// assert_eq!(format_asset_amount(100_000_000, Some(6)), "100.000000");
/// ```
#[allow(dead_code)]
#[must_use]
pub fn format_asset_amount(amount: u64, decimals: Option<u64>) -> String {
    match decimals {
        Some(d) if d > 0 => {
            let divisor = 10_u64.pow(d as u32) as f64;
            let formatted = amount as f64 / divisor;
            format_with_commas_f64(formatted, d as usize)
        }
        _ => format_with_commas(amount),
    }
}

/// Format a number with commas for thousands separators.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(format_with_commas(1000), "1,000");
/// assert_eq!(format_with_commas(1_000_000), "1,000,000");
/// ```
#[allow(dead_code)]
#[must_use]
pub fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format a floating point number with commas and specified decimal places.
#[allow(dead_code)]
#[must_use]
pub fn format_with_commas_f64(n: f64, decimals: usize) -> String {
    let int_part = n.trunc() as u64;
    let frac_part = n.fract();

    let int_formatted = format_with_commas(int_part);

    if decimals == 0 {
        int_formatted
    } else {
        let frac_str = format!("{:.prec$}", frac_part, prec = decimals);
        // Skip the "0." prefix
        let frac_digits = &frac_str[2..];
        format!("{int_formatted}.{frac_digits}")
    }
}

// ============================================================================
// Transaction Type Helpers
// ============================================================================

/// Get the ASCII icon for a transaction type.
///
/// Returns ASCII-safe icons that work in all terminals.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(txn_type_icon(TxnType::Payment), "[$]");
/// assert_eq!(txn_type_icon(TxnType::AppCall), "[A]");
/// ```
#[must_use]
pub const fn txn_type_icon(txn_type: TxnType) -> &'static str {
    match txn_type {
        TxnType::Payment => "[$]",
        TxnType::AppCall => "[A]",
        TxnType::AssetTransfer => "[>]",
        TxnType::AssetConfig => "[*]",
        TxnType::AssetFreeze => "[#]",
        TxnType::KeyReg => "[K]",
        TxnType::StateProof => "[S]",
        TxnType::Heartbeat => "[H]",
        TxnType::Unknown => "[?]",
    }
}

/// Get the short code for a transaction type.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(txn_type_code(TxnType::Payment), "PAY");
/// assert_eq!(txn_type_code(TxnType::AppCall), "APP");
/// ```
#[must_use]
#[allow(dead_code)] // Future use
pub const fn txn_type_code(txn_type: TxnType) -> &'static str {
    match txn_type {
        TxnType::Payment => "PAY",
        TxnType::AppCall => "APP",
        TxnType::AssetTransfer => "AXF",
        TxnType::AssetConfig => "ACF",
        TxnType::AssetFreeze => "AFZ",
        TxnType::KeyReg => "KEY",
        TxnType::StateProof => "STP",
        TxnType::Heartbeat => "HBT",
        TxnType::Unknown => "???",
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Table-driven tests for address truncation.
    /// Per commandments: use table tests to reduce duplication.
    #[test]
    fn test_truncate_address() {
        let long_addr = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let cases = [
            // (input, max_len, expected_behavior)
            ("ABCDEFGH", 20, "fits without truncation"),
            ("ABCDEFGHIJ", 10, "exact fit"),
            (long_addr, 20, "truncated with ellipsis"),
            ("ABCDEFGHIJ", 5, "very short max"),
        ];

        for (addr, max_len, desc) in cases {
            let result = truncate_address(addr, max_len);
            assert!(
                result.len() <= max_len,
                "{desc}: result len {} > max {}",
                result.len(),
                max_len
            );
            if addr.len() <= max_len {
                assert_eq!(result, addr, "{desc}: should not truncate");
            } else if max_len >= 7 {
                assert!(result.contains("..."), "{desc}: should have ellipsis");
            }
        }
    }

    /// Table-driven tests for ALGO amount formatting.
    #[test]
    fn test_format_algo_amount() {
        let cases = [
            (0_u64, "0.000000 ALGO"),
            (1_000_000, "1.000000 ALGO"),
            (5_500_000, "5.500000 ALGO"),
            (123_456, "0.123456 ALGO"),
        ];

        for (input, expected) in cases {
            assert_eq!(format_algo_amount(input), expected, "microalgos={input}");
        }
    }

    /// Table-driven tests for asset amount formatting.
    #[test]
    fn test_format_asset_amount() {
        let cases = [
            (1000_u64, None, "1,000"),
            (1_000_000, None, "1,000,000"),
            (100_000_000, Some(6_u64), "100.000000"),
            (1_500_000, Some(6), "1.500000"),
        ];

        for (amount, decimals, expected) in cases {
            assert_eq!(
                format_asset_amount(amount, decimals),
                expected,
                "amount={amount}, decimals={decimals:?}"
            );
        }
    }

    /// Table-driven tests for number formatting with commas.
    #[test]
    fn test_format_with_commas() {
        let cases = [
            (0_u64, "0"),
            (999, "999"),
            (1000, "1,000"),
            (1_000_000, "1,000,000"),
            (1_234_567_890, "1,234,567,890"),
        ];

        for (input, expected) in cases {
            assert_eq!(format_with_commas(input), expected, "input={input}");
        }
    }

    /// Table-driven tests for transaction type icons and codes.
    #[test]
    fn test_txn_type_display() {
        use TxnType::*;

        let cases = [
            (Payment, "[$]", "PAY"),
            (AppCall, "[A]", "APP"),
            (AssetTransfer, "[>]", "AXF"),
            (AssetConfig, "[*]", "ACF"),
            (AssetFreeze, "[#]", "AFZ"),
            (KeyReg, "[K]", "KEY"),
            (StateProof, "[S]", "STP"),
            (Heartbeat, "[H]", "HBT"),
            (Unknown, "[?]", "???"),
        ];

        for (txn_type, expected_icon, expected_code) in cases {
            assert_eq!(txn_type_icon(txn_type), expected_icon, "{txn_type:?} icon");
            assert_eq!(txn_type_code(txn_type), expected_code, "{txn_type:?} code");
        }
    }
}
