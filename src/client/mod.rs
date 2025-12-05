//! HTTP clients for Algorand network APIs.
//!
//! This module provides typed clients for interacting with:
//! - Algorand Node (algod) - current blockchain state
//! - Algorand Indexer - historical data queries
//! - NFD API - human-readable address names
//!
//! # Example
//!
//! ```ignore
//! use crate::client::{NodeClient, IndexerClient, NfdClient};
//!
//! // Create clients for MainNet
//! let node = NodeClient::new("https://mainnet-api.algonode.cloud");
//! let indexer = IndexerClient::new("https://mainnet-idx.algonode.cloud");
//! let nfd = NfdClient::mainnet();
//! ```

#![allow(unused_imports)]

pub mod algo;
pub mod http;
pub mod indexer;
pub mod nfd;
pub mod node;

// ============================================================================
// Re-exports
// ============================================================================

pub use algo::AlgoClient;
pub use http::{HttpClient, HttpConfig};
pub use indexer::IndexerClient;
pub use nfd::NfdClient;
pub use node::NodeClient;
