//! Platform-specific path helpers for configuration and data directories.
//!
//! This module provides utilities for locating application-specific directories
//! across different platforms following platform conventions.
//!
//! # Directory Locations
//!
//! | Platform | Config Dir | Data Dir | Cache Dir |
//! |----------|------------|----------|-----------|
//! | Linux | `~/.config/lazylora` | `~/.local/share/lazylora` | `~/.cache/lazylora` |
//! | macOS | `~/Library/Application Support/lazylora` | Same as config | `~/Library/Caches/lazylora` |
//! | Windows | `%APPDATA%/lazylora` | `%LOCALAPPDATA%/lazylora` | `%LOCALAPPDATA%/lazylora/cache` |
//!
//! # Example
//!
//! ```ignore
//! use crate::state::platform::paths::AppPaths;
//!
//! let paths = AppPaths::new();
//!
//! // Get config directory (creates if needed)
//! if let Ok(config_dir) = paths.config_dir() {
//!     println!("Config: {}", config_dir.display());
//! }
//!
//! // Get specific file paths
//! if let Ok(config_file) = paths.config_file() {
//!     println!("Config file: {}", config_file.display());
//! }
//! ```

// TODO: Remove after full integration in Stage 2
#![allow(dead_code)]

use color_eyre::Result;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// Constants
// ============================================================================

/// Application name used for directory naming.
pub const APP_NAME: &str = "lazylora";

/// Default configuration file name.
pub const CONFIG_FILE_NAME: &str = "config.json";

/// Default log file name.
pub const LOG_FILE_NAME: &str = "lazylora.log";

// ============================================================================
// AppPaths
// ============================================================================

/// Provides platform-specific path resolution for application directories and files.
///
/// This struct encapsulates the logic for locating configuration, data, and cache
/// directories across different operating systems.
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// The application name used for directory creation.
    app_name: String,
}

impl Default for AppPaths {
    fn default() -> Self {
        Self::new()
    }
}

impl AppPaths {
    /// Creates a new `AppPaths` instance with the default application name.
    #[must_use]
    pub fn new() -> Self {
        Self {
            app_name: APP_NAME.to_string(),
        }
    }

    /// Creates a new `AppPaths` instance with a custom application name.
    ///
    /// # Arguments
    ///
    /// * `app_name` - The application name to use for directory creation
    #[must_use]
    pub fn with_app_name(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }

    // ========================================================================
    // Directory Methods
    // ========================================================================

    /// Returns the configuration directory, creating it if necessary.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be determined or created.
    ///
    /// # Returns
    ///
    /// The path to the configuration directory.
    pub fn config_dir(&self) -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not find config directory"))?;
        path.push(&self.app_name);
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// Returns the data directory, creating it if necessary.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be determined or created.
    ///
    /// # Returns
    ///
    /// The path to the data directory.
    pub fn data_dir(&self) -> Result<PathBuf> {
        let mut path = dirs::data_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not find data directory"))?;
        path.push(&self.app_name);
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// Returns the cache directory, creating it if necessary.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be determined or created.
    ///
    /// # Returns
    ///
    /// The path to the cache directory.
    pub fn cache_dir(&self) -> Result<PathBuf> {
        let mut path = dirs::cache_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not find cache directory"))?;
        path.push(&self.app_name);
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// Returns the home directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined.
    ///
    /// # Returns
    ///
    /// The path to the home directory.
    pub fn home_dir() -> Result<PathBuf> {
        dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))
    }

    // ========================================================================
    // File Methods
    // ========================================================================

    /// Returns the path to the configuration file.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined or created.
    ///
    /// # Returns
    ///
    /// The path to the configuration file.
    pub fn config_file(&self) -> Result<PathBuf> {
        let mut path = self.config_dir()?;
        path.push(CONFIG_FILE_NAME);
        Ok(path)
    }

    /// Returns the path to the log file.
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory cannot be determined or created.
    ///
    /// # Returns
    ///
    /// The path to the log file.
    pub fn log_file(&self) -> Result<PathBuf> {
        let mut path = self.data_dir()?;
        path.push(LOG_FILE_NAME);
        Ok(path)
    }

    /// Returns a path to a file in the configuration directory.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the file
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined or created.
    ///
    /// # Returns
    ///
    /// The path to the file.
    pub fn config_file_path(&self, filename: &str) -> Result<PathBuf> {
        let mut path = self.config_dir()?;
        path.push(filename);
        Ok(path)
    }

    /// Returns a path to a file in the data directory.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the file
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory cannot be determined or created.
    ///
    /// # Returns
    ///
    /// The path to the file.
    pub fn data_file_path(&self, filename: &str) -> Result<PathBuf> {
        let mut path = self.data_dir()?;
        path.push(filename);
        Ok(path)
    }

    /// Returns a path to a file in the cache directory.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the file
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be determined or created.
    ///
    /// # Returns
    ///
    /// The path to the file.
    pub fn cache_file_path(&self, filename: &str) -> Result<PathBuf> {
        let mut path = self.cache_dir()?;
        path.push(filename);
        Ok(path)
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Ensures all application directories exist.
    ///
    /// # Errors
    ///
    /// Returns an error if any directory cannot be created.
    pub fn ensure_directories(&self) -> Result<()> {
        self.config_dir()?;
        self.data_dir()?;
        self.cache_dir()?;
        Ok(())
    }

    /// Returns the application name.
    #[must_use]
    pub fn app_name(&self) -> &str {
        &self.app_name
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Returns the configuration directory using default application name.
///
/// # Errors
///
/// Returns an error if the directory cannot be determined or created.
pub fn config_dir() -> Result<PathBuf> {
    AppPaths::new().config_dir()
}

/// Returns the data directory using default application name.
///
/// # Errors
///
/// Returns an error if the directory cannot be determined or created.
pub fn data_dir() -> Result<PathBuf> {
    AppPaths::new().data_dir()
}

/// Returns the cache directory using default application name.
///
/// # Errors
///
/// Returns an error if the directory cannot be determined or created.
pub fn cache_dir() -> Result<PathBuf> {
    AppPaths::new().cache_dir()
}

/// Returns the path to the configuration file using default application name.
///
/// # Errors
///
/// Returns an error if the directory cannot be determined or created.
pub fn config_file() -> Result<PathBuf> {
    AppPaths::new().config_file()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_paths_new() {
        let paths = AppPaths::new();
        assert_eq!(paths.app_name(), APP_NAME);
    }

    #[test]
    fn test_app_paths_with_custom_name() {
        let paths = AppPaths::with_app_name("custom_app");
        assert_eq!(paths.app_name(), "custom_app");
    }

    #[test]
    fn test_default_is_same_as_new() {
        let paths1 = AppPaths::new();
        let paths2 = AppPaths::default();
        assert_eq!(paths1.app_name(), paths2.app_name());
    }

    #[test]
    fn test_config_dir_contains_app_name() {
        if let Ok(dir) = AppPaths::new().config_dir() {
            let dir_name = dir.file_name().and_then(|n| n.to_str());
            assert_eq!(dir_name, Some(APP_NAME));
        }
        // Skip if config dir unavailable (CI environments)
    }

    #[test]
    fn test_config_file_has_json_extension() {
        if let Ok(path) = AppPaths::new().config_file() {
            let extension = path.extension().and_then(|e| e.to_str());
            assert_eq!(extension, Some("json"));

            let filename = path.file_name().and_then(|n| n.to_str());
            assert_eq!(filename, Some(CONFIG_FILE_NAME));
        }
        // Skip if config dir unavailable
    }

    #[test]
    fn test_log_file_has_log_extension() {
        if let Ok(path) = AppPaths::new().log_file() {
            let extension = path.extension().and_then(|e| e.to_str());
            assert_eq!(extension, Some("log"));

            let filename = path.file_name().and_then(|n| n.to_str());
            assert_eq!(filename, Some(LOG_FILE_NAME));
        }
        // Skip if data dir unavailable
    }

    #[test]
    fn test_config_file_path_custom() {
        if let Ok(path) = AppPaths::new().config_file_path("custom.toml") {
            let filename = path.file_name().and_then(|n| n.to_str());
            assert_eq!(filename, Some("custom.toml"));

            // Should be in config directory
            let parent = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str());
            assert_eq!(parent, Some(APP_NAME));
        }
        // Skip if config dir unavailable
    }

    #[test]
    fn test_data_file_path_custom() {
        if let Ok(path) = AppPaths::new().data_file_path("data.db") {
            let filename = path.file_name().and_then(|n| n.to_str());
            assert_eq!(filename, Some("data.db"));
        }
        // Skip if data dir unavailable
    }

    #[test]
    fn test_cache_file_path_custom() {
        if let Ok(path) = AppPaths::new().cache_file_path("temp.bin") {
            let filename = path.file_name().and_then(|n| n.to_str());
            assert_eq!(filename, Some("temp.bin"));
        }
        // Skip if cache dir unavailable
    }

    #[test]
    fn test_home_dir_exists() {
        // Home directory should exist on most systems
        if let Ok(home) = AppPaths::home_dir() {
            assert!(home.exists());
        }
        // Skip if home dir unavailable
    }

    #[test]
    fn test_convenience_functions_match_methods() {
        let paths = AppPaths::new();

        // These should produce the same results
        if let (Ok(dir1), Ok(dir2)) = (paths.config_dir(), config_dir()) {
            assert_eq!(dir1, dir2);
        }

        if let (Ok(dir1), Ok(dir2)) = (paths.data_dir(), data_dir()) {
            assert_eq!(dir1, dir2);
        }

        if let (Ok(dir1), Ok(dir2)) = (paths.cache_dir(), cache_dir()) {
            assert_eq!(dir1, dir2);
        }

        if let (Ok(file1), Ok(file2)) = (paths.config_file(), config_file()) {
            assert_eq!(file1, file2);
        }
    }

    #[test]
    fn test_ensure_directories_does_not_panic() {
        // Just verify it doesn't panic
        let paths = AppPaths::new();
        let _ = paths.ensure_directories();
    }
}
