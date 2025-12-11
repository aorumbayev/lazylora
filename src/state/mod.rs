//! State management module for the LazyLora TUI application.
//!
//! This module provides a decomposed state architecture, separating concerns into:
//!
//! - [`NavigationState`] - UI navigation (selections, scroll positions, view stack)
//! - [`DataState`] - Application data (blocks, transactions, search results)
//! - [`UiState`] - UI presentation concerns (focus, popups, toasts)
//! - [`AppConfig`] - Persistent configuration with load/save capabilities
//!
//! # Architecture
//!
//! The state is decomposed following the principle of separation of concerns:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                       App                           │
//! ├──────────────┬──────────────┬───────────────────────┤
//! │ NavigationState │  DataState  │       UiState        │
//! │  - selections   │  - blocks   │  - focus            │
//! │  - scroll pos   │  - txns     │  - popups           │
//! │  - view stack   │  - search   │  - toasts           │
//! └──────────────┴──────────────┴───────────────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use crate::state::{App, NavigationState, DataState, UiState, AppConfig};
//!
//! let app = App::new(Network::TestNet);
//! ```

use tokio::sync::{mpsc, watch};

use crate::client::AlgoClient;
use crate::domain::{Network, NetworkConfig};

// ============================================================================
// Module Declarations
// ============================================================================

mod app_lifecycle;

pub mod config;
pub mod data;
pub mod navigation;
pub mod ui_state;

// ============================================================================
// Re-exports
// ============================================================================

// Lifecycle functions
pub use app_lifecycle::prefetch_initial_data;

// Navigation types
pub use navigation::{
    AccountDetailTab, AppDetailTab, BlockDetailTab, DetailViewMode, NavigationState,
};

// Data types
pub use data::DataState;

// UI state types
pub use ui_state::{Focus, PopupState, SearchType, UiState};

// Configuration types
pub use config::AppConfig;

// ============================================================================
// App Message Types
// ============================================================================

/// Messages sent between async tasks and the main app loop.
///
/// These messages are used for async communication between background tasks
/// (e.g., network fetches) and the main application state.
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// New blocks fetched from the network.
    BlocksUpdated(Vec<crate::domain::AlgoBlock>),
    /// New transactions fetched from the network.
    TransactionsUpdated(Vec<crate::domain::Transaction>),
    /// Search completed with results or error.
    SearchCompleted(Result<Vec<crate::domain::SearchResultItem>, String>),
    /// Network error occurred.
    NetworkError(String),
    /// Network connection established.
    NetworkConnected,
    /// Network switch completed successfully.
    NetworkSwitchComplete,
    /// Block details loaded.
    BlockDetailsLoaded(crate::domain::BlockDetails),
    /// Transaction details loaded.
    TransactionDetailsLoaded(Box<crate::domain::Transaction>),
    /// Transaction details fetch failed.
    TransactionDetailsFailed(String),
    /// Account details loaded.
    AccountDetailsLoaded(Box<crate::domain::AccountDetails>),
    /// Account details fetch failed.
    AccountDetailsFailed(String),
    /// Asset details loaded.
    AssetDetailsLoaded(Box<crate::domain::AssetDetails>),
    /// Asset details fetch failed.
    AssetDetailsFailed(String),
    /// Application details loaded.
    ApplicationDetailsLoaded(Box<crate::domain::ApplicationDetails>),
    /// Application details fetch failed.
    ApplicationDetailsFailed(String),
}

// ============================================================================
// Startup Options
// ============================================================================

/// Startup search mode - perform a search immediately on startup.
#[derive(Debug, Clone)]
pub enum StartupSearch {
    /// Search for a transaction by ID.
    Transaction(String),
    /// Search for an account by address.
    Account(String),
    /// Search for a block by round number.
    Block(u64),
    /// Search for an asset by ID.
    Asset(u64),
}

/// Options that can be passed when starting the application.
///
/// These options allow customization of the initial application state,
/// such as pre-selecting a network or performing an initial search.
#[derive(Debug, Clone, Default)]
pub struct StartupOptions {
    /// Network to connect to on startup.
    pub network: Option<Network>,
    /// Search to perform immediately after startup.
    pub search: Option<StartupSearch>,
    /// Start in graph view mode.
    pub graph_view: bool,
}

// ============================================================================
// Main App State
// ============================================================================

/// The main application state container.
///
/// This struct holds all application state including:
/// - Navigation state (selections, scroll positions)
/// - Data state (blocks, transactions, search results)
/// - UI state (focus, popups, toasts)
/// - Network client and configuration
/// - Async communication channels
///
/// # Architecture
///
/// The `App` struct uses a decomposed state architecture where different
/// concerns are separated into sub-modules. This makes the codebase more
/// maintainable and testable.
///
/// # Example
///
/// ```ignore
/// use crate::state::App;
/// use crate::domain::Network;
///
/// let app = App::new(Network::TestNet);
/// ```
#[derive(Debug)]
pub struct App {
    // ========================================================================
    // Sub-states (decomposed concerns)
    // ========================================================================
    /// Navigation state - selections, scroll positions, view stack.
    pub nav: NavigationState,

    /// Data state - blocks, transactions, search results.
    pub data: DataState,

    /// UI state - focus, popups, toasts.
    pub ui: UiState,

    // ========================================================================
    // App-level state
    // ========================================================================
    /// Current network (MainNet, TestNet, LocalNet).
    pub network: Network,

    /// Current network configuration (built-in or custom).
    pub network_config: NetworkConfig,

    /// Cached list of all available networks (built-in + custom).
    pub(crate) available_networks: Vec<NetworkConfig>,

    /// Whether live updates are enabled.
    pub show_live: bool,

    /// Whether the application should exit.
    pub exit: bool,

    /// Animation tick counter for UI animations.
    pub animation_tick: u64,

    // ========================================================================
    // Async Communication Channels
    // ========================================================================
    // NOTE: Channel sends use `let _ = tx.send(...)` throughout this module.
    // This is intentional fire-and-forget: receivers may be dropped during
    // shutdown, and we don't want to propagate those errors.
    /// Sender for app messages (cloned for background tasks).
    pub(crate) message_tx: mpsc::UnboundedSender<AppMessage>,

    /// Receiver for app messages.
    pub(crate) message_rx: mpsc::UnboundedReceiver<AppMessage>,

    /// Watch channel for live updates toggle.
    pub(crate) live_updates_tx: watch::Sender<bool>,

    /// Watch channel for network changes.
    pub(crate) network_tx: watch::Sender<NetworkConfig>,

    // ========================================================================
    // Network Client
    // ========================================================================
    /// Algorand client for network requests.
    pub(crate) client: AlgoClient,

    // ========================================================================
    // Startup Options
    // ========================================================================
    /// Options passed at startup (e.g., initial search).
    pub(crate) startup_options: Option<StartupOptions>,
}

impl App {
    /// Returns the current network configuration.
    #[must_use]
    pub fn current_network_config(&self) -> &NetworkConfig {
        &self.network_config
    }

    /// Returns all available networks (built-in + custom).
    #[must_use]
    pub fn all_networks(&self) -> &[NetworkConfig] {
        &self.available_networks
    }

    /// Returns the index of the current network in the available networks list.
    #[must_use]
    pub fn current_network_index(&self) -> usize {
        self.available_networks
            .iter()
            .position(|n| n == &self.network_config)
            .unwrap_or(0)
    }
}

// ============================================================================
// Implementation Modules
// ============================================================================

// Message processing, data merging
mod app_messages;

// Command execution, input handling
mod app_commands;

// Navigation helpers, selection management
mod app_navigation;

// Search, network, clipboard, browser actions
mod app_actions;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests;
