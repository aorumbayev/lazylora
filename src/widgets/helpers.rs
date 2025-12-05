//! Helper functions for formatting and displaying Algorand data.
//!
//! This module contains utility functions used across various widgets for:
//! - Address truncation and formatting
//! - Amount formatting (ALGO and ASA)
//! - Transaction type icons and codes

#![allow(dead_code)] // Transitional phase - items will be used after integration

use crate::domain::TxnType;

// ============================================================================
// Constants
// ============================================================================

/// Algorand symbol for display
pub const ALGO_SYMBOL: &str = "◈";

/// Asset symbol for display
pub const ASSET_SYMBOL: &str = "◆";

/// Number of microAlgos per Algo
pub const MICROALGOS_PER_ALGO: f64 = 1_000_000.0;

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

    #[test]
    fn test_truncate_address_short() {
        let addr = "ABCDEFGH";
        assert_eq!(truncate_address(addr, 20), "ABCDEFGH");
    }

    #[test]
    fn test_truncate_address_exact() {
        let addr = "ABCDEFGHIJ";
        assert_eq!(truncate_address(addr, 10), "ABCDEFGHIJ");
    }

    #[test]
    fn test_truncate_address_long() {
        let addr = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let result = truncate_address(addr, 20);
        assert_eq!(result.len(), 20);
        assert!(result.contains("..."));
        assert!(result.starts_with("AAAA"));
        assert!(result.ends_with("AAAA"));
    }

    #[test]
    fn test_truncate_address_very_short_max() {
        let addr = "ABCDEFGHIJ";
        let result = truncate_address(addr, 5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_format_algo_amount() {
        assert_eq!(format_algo_amount(0), "0.000000 ALGO");
        assert_eq!(format_algo_amount(1_000_000), "1.000000 ALGO");
        assert_eq!(format_algo_amount(5_500_000), "5.500000 ALGO");
        assert_eq!(format_algo_amount(123_456), "0.123456 ALGO");
    }

    #[test]
    fn test_format_asset_amount_no_decimals() {
        assert_eq!(format_asset_amount(1000, None), "1,000");
        assert_eq!(format_asset_amount(1_000_000, None), "1,000,000");
    }

    #[test]
    fn test_format_asset_amount_with_decimals() {
        assert_eq!(format_asset_amount(100_000_000, Some(6)), "100.000000");
        assert_eq!(format_asset_amount(1_500_000, Some(6)), "1.500000");
    }

    #[test]
    fn test_format_with_commas() {
        assert_eq!(format_with_commas(0), "0");
        assert_eq!(format_with_commas(999), "999");
        assert_eq!(format_with_commas(1000), "1,000");
        assert_eq!(format_with_commas(1_000_000), "1,000,000");
        assert_eq!(format_with_commas(1_234_567_890), "1,234,567,890");
    }

    #[test]
    fn test_txn_type_icon() {
        assert_eq!(txn_type_icon(TxnType::Payment), "[$]");
        assert_eq!(txn_type_icon(TxnType::AppCall), "[A]");
        assert_eq!(txn_type_icon(TxnType::AssetTransfer), "[>]");
        assert_eq!(txn_type_icon(TxnType::AssetConfig), "[*]");
        assert_eq!(txn_type_icon(TxnType::AssetFreeze), "[#]");
        assert_eq!(txn_type_icon(TxnType::KeyReg), "[K]");
        assert_eq!(txn_type_icon(TxnType::StateProof), "[S]");
        assert_eq!(txn_type_icon(TxnType::Heartbeat), "[H]");
        assert_eq!(txn_type_icon(TxnType::Unknown), "[?]");
    }

    #[test]
    fn test_txn_type_code() {
        assert_eq!(txn_type_code(TxnType::Payment), "PAY");
        assert_eq!(txn_type_code(TxnType::AppCall), "APP");
        assert_eq!(txn_type_code(TxnType::AssetTransfer), "AXF");
        assert_eq!(txn_type_code(TxnType::Unknown), "???");
    }
}
