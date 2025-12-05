//! Platform-specific abstractions for cross-platform functionality.
//!
//! This module provides abstractions for platform-specific operations:
//!
//! - [`clipboard`] - Cross-platform clipboard access
//! - [`paths`] - Configuration and data directory paths
//!
//! # Platform Support
//!
//! The module provides unified APIs that work across:
//! - Linux (X11 and Wayland)
//! - macOS
//! - Windows
//!
//! # Example
//!
//! ```ignore
//! use crate::state::platform::{clipboard, paths};
//!
//! // Copy to clipboard
//! clipboard::copy_text("Hello, world!");
//!
//! // Get config directory
//! let config_dir = paths::config_dir();
//! ```

// TODO: Remove after full integration in Stage 2
#![allow(dead_code)]

pub mod clipboard;
pub mod paths;

// Re-export commonly used items
// Note: Commented out until Stage 2.5 when these are integrated
// pub use clipboard::ClipboardManager;
// pub use paths::AppPaths;
