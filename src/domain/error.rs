//! Error types for Algorand domain operations.
//!
//! This module defines the custom error types used throughout the Algorand
//! client operations, providing structured error handling with helpful messages.

use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Custom error type for Algorand client operations.
///
/// This enum provides specific error variants for different failure modes
/// encountered when interacting with the Algorand network.
#[derive(Debug, Error)]
pub enum AlgoError {
    /// Network-related errors from HTTP requests.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON parsing or data structure errors.
    #[error("Parse error: {message}")]
    Parse {
        /// Description of what failed to parse.
        message: String,
    },

    /// Entity not found on the network.
    #[error("{entity} '{id}' not found")]
    NotFound {
        /// The type of entity that was not found (e.g., "transaction", "account").
        entity: &'static str,
        /// The identifier that was searched for.
        id: String,
    },

    /// Invalid user input.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// HTTP client initialization failed.
    #[error("Failed to initialize HTTP client: {0}")]
    ClientInit(String),
}

impl AlgoError {
    /// Create a new parse error with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - Description of what failed to parse
    ///
    /// # Returns
    ///
    /// A new `AlgoError::Parse` variant.
    #[must_use]
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }

    /// Create a new not found error.
    ///
    /// # Arguments
    ///
    /// * `entity` - The type of entity that was not found
    /// * `id` - The identifier that was searched for
    ///
    /// # Returns
    ///
    /// A new `AlgoError::NotFound` variant.
    #[must_use]
    pub fn not_found(entity: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity,
            id: id.into(),
        }
    }

    /// Create a new invalid input error.
    ///
    /// # Arguments
    ///
    /// * `message` - Description of why the input is invalid
    ///
    /// # Returns
    ///
    /// A new `AlgoError::InvalidInput` variant.
    #[must_use]
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Create a new client initialization error.
    #[must_use]
    pub fn client_init(message: impl Into<String>) -> Self {
        Self::ClientInit(message.into())
    }

    /// Convert to a `color_eyre::Report` for API compatibility.
    ///
    /// This method allows `AlgoError` to be used with color_eyre's error
    /// handling infrastructure while preserving the error message.
    ///
    /// # Returns
    ///
    /// A `color_eyre::Report` containing the error message.
    #[must_use = "this converts the error into a Report for display"]
    pub fn into_report(self) -> color_eyre::Report {
        color_eyre::eyre::eyre!("{}", self)
    }
}

// Tests removed per AGENTS.md commandments:
// - test_algo_error_display: Tests thiserror's Display derive (framework test)
// - test_parse_error_creation: Tests that constructor constructs (trivial)
// - test_not_found_error_creation: Tests that constructor constructs (trivial)
// - test_invalid_input_error_creation: Tests that constructor constructs (trivial)
