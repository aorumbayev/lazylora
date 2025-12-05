//! Cross-platform clipboard abstraction.
//!
//! This module provides a unified API for clipboard operations across
//! different platforms (Linux, macOS, Windows).
//!
//! # Platform-Specific Behavior
//!
//! ## Linux
//!
//! On Linux, the module tries multiple clipboard tools in order:
//! 1. `wl-copy` (Wayland)
//! 2. `xclip` (X11)
//! 3. `xsel` (X11 alternative)
//! 4. Falls back to `arboard` crate
//!
//! External tools are preferred because they persist clipboard content
//! after the application exits.
//!
//! ## macOS and Windows
//!
//! Uses the `arboard` crate directly.
//!
//! # Example
//!
//! ```ignore
//! use crate::state::platform::clipboard::ClipboardManager;
//!
//! let mut clipboard = ClipboardManager::new();
//!
//! // Copy text
//! match clipboard.copy_text("Hello, world!") {
//!     Ok(()) => println!("Copied!"),
//!     Err(e) => eprintln!("Failed: {}", e),
//! }
//! ```

// TODO: Remove after full integration in Stage 2
#![allow(dead_code)]

use std::fmt;

// ============================================================================
// Error Type
// ============================================================================

/// Error type for clipboard operations.
#[derive(Debug, Clone)]
pub enum ClipboardError {
    /// Clipboard is not available on this system.
    NotAvailable,
    /// Failed to copy text to clipboard.
    CopyFailed(String),
    /// Failed to read from clipboard.
    ReadFailed(String),
    /// The clipboard is empty.
    Empty,
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAvailable => write!(f, "Clipboard not available"),
            Self::CopyFailed(msg) => write!(f, "Failed to copy: {msg}"),
            Self::ReadFailed(msg) => write!(f, "Failed to read: {msg}"),
            Self::Empty => write!(f, "Clipboard is empty"),
        }
    }
}

impl std::error::Error for ClipboardError {}

/// Result type for clipboard operations.
pub type ClipboardResult<T> = Result<T, ClipboardError>;

// ============================================================================
// Clipboard Manager
// ============================================================================

/// Cross-platform clipboard manager.
///
/// Provides a unified interface for clipboard operations that works
/// across Linux (X11/Wayland), macOS, and Windows.
#[derive(Debug)]
pub struct ClipboardManager {
    /// Whether to prefer external tools on Linux.
    prefer_external_tools: bool,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardManager {
    /// Creates a new clipboard manager.
    ///
    /// On Linux, this will prefer external clipboard tools (wl-copy, xclip, xsel)
    /// over the arboard crate to ensure clipboard persistence.
    #[must_use]
    pub fn new() -> Self {
        Self {
            prefer_external_tools: true,
        }
    }

    /// Creates a clipboard manager that only uses the arboard crate.
    ///
    /// This may be useful in environments where external tools are not available
    /// or not desired.
    #[must_use]
    pub fn arboard_only() -> Self {
        Self {
            prefer_external_tools: false,
        }
    }

    /// Copies text to the clipboard.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to copy
    ///
    /// # Errors
    ///
    /// Returns an error if the clipboard is not available or the copy fails.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    pub fn copy_text(&self, text: &str) -> ClipboardResult<()> {
        #[cfg(target_os = "linux")]
        if self.prefer_external_tools
            && let Ok(()) = self.copy_with_external_tool(text)
        {
            return Ok(());
            // Fall through to arboard if external tools fail
        }

        self.copy_with_arboard(text)
    }

    /// Copies text using the arboard crate.
    fn copy_with_arboard(&self, text: &str) -> ClipboardResult<()> {
        use arboard::Clipboard;

        let mut clipboard = Clipboard::new().map_err(|_| ClipboardError::NotAvailable)?;

        clipboard
            .set_text(text.to_string())
            .map_err(|e| ClipboardError::CopyFailed(e.to_string()))
    }

    /// Copies text using external tools (Linux only).
    #[cfg(target_os = "linux")]
    fn copy_with_external_tool(&self, text: &str) -> ClipboardResult<()> {
        // Try wl-copy first (Wayland)
        if Self::try_tool("wl-copy", &[], text) {
            return Ok(());
        }

        // Try xclip (X11)
        if Self::try_tool("xclip", &["-selection", "clipboard"], text) {
            return Ok(());
        }

        // Try xsel (X11 alternative)
        if Self::try_tool("xsel", &["--clipboard", "--input"], text) {
            return Ok(());
        }

        Err(ClipboardError::NotAvailable)
    }

    /// Tries to copy text using a specific tool.
    #[cfg(target_os = "linux")]
    fn try_tool(tool: &str, args: &[&str], text: &str) -> bool {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let child = Command::new(tool)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        let Ok(mut child) = child else {
            return false;
        };

        let Some(mut stdin) = child.stdin.take() else {
            return false;
        };

        if stdin.write_all(text.as_bytes()).is_err() {
            return false;
        }

        drop(stdin);

        child.wait().map(|s| s.success()).unwrap_or(false)
    }

    /// Checks if clipboard is available on this system.
    ///
    /// # Returns
    ///
    /// `true` if clipboard operations are likely to succeed.
    #[must_use]
    pub fn is_available(&self) -> bool {
        #[cfg(target_os = "linux")]
        if self.prefer_external_tools && Self::has_external_tool() {
            return true;
        }

        // Try to create an arboard clipboard
        arboard::Clipboard::new().is_ok()
    }

    /// Checks if any external clipboard tool is available (Linux only).
    #[cfg(target_os = "linux")]
    fn has_external_tool() -> bool {
        use std::process::Command;

        // Check wl-copy
        if Command::new("which")
            .arg("wl-copy")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return true;
        }

        // Check xclip
        if Command::new("which")
            .arg("xclip")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return true;
        }

        // Check xsel
        if Command::new("which")
            .arg("xsel")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return true;
        }

        false
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Copies text to the clipboard using the default clipboard manager.
///
/// This is a convenience function for simple copy operations.
///
/// # Arguments
///
/// * `text` - The text to copy
///
/// # Errors
///
/// Returns an error if the clipboard is not available or the copy fails.
///
/// # Returns
///
/// `Ok(())` on success.
///
/// # Example
///
/// ```ignore
/// use crate::state::platform::clipboard::copy_text;
///
/// copy_text("Hello, world!").expect("Failed to copy");
/// ```
pub fn copy_text(text: &str) -> ClipboardResult<()> {
    ClipboardManager::new().copy_text(text)
}

/// Checks if clipboard is available on this system.
///
/// # Returns
///
/// `true` if clipboard operations are likely to succeed.
#[must_use]
pub fn is_available() -> bool {
    ClipboardManager::new().is_available()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_manager_creation() {
        let manager = ClipboardManager::new();
        assert!(manager.prefer_external_tools);

        let manager = ClipboardManager::arboard_only();
        assert!(!manager.prefer_external_tools);
    }

    #[test]
    fn test_default_creates_with_external_tools() {
        let manager = ClipboardManager::default();
        assert!(manager.prefer_external_tools);
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            ClipboardError::NotAvailable.to_string(),
            "Clipboard not available"
        );
        assert_eq!(
            ClipboardError::CopyFailed("test".to_string()).to_string(),
            "Failed to copy: test"
        );
        assert_eq!(
            ClipboardError::ReadFailed("test".to_string()).to_string(),
            "Failed to read: test"
        );
        assert_eq!(ClipboardError::Empty.to_string(), "Clipboard is empty");
    }

    #[test]
    fn test_is_available_does_not_panic() {
        // Just verify it doesn't panic
        let _ = is_available();
    }

    // Note: Actual clipboard copy tests are difficult to run in CI
    // because they require a display server. These tests verify the
    // API works without actually testing clipboard functionality.

    #[test]
    fn test_copy_text_returns_result() {
        // This test verifies the function returns a proper Result type
        // Actual clipboard availability depends on the environment
        let result = copy_text("test");
        // Result should be either Ok or Err, not panic
        let _ = result.is_ok();
    }
}
