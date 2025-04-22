use std::time::Duration;

// UI Layout
pub const HEADER_HEIGHT: u16 = 3;
pub const FOOTER_HEIGHT: u16 = 1;
pub const TITLE_HEIGHT: u16 = 3;
pub const BLOCK_ITEM_HEIGHT: u16 = 3;
pub const TXN_ITEM_HEIGHT: u16 = 4;

// Data Fetching
pub const BLOCK_FETCH_INTERVAL: Duration = Duration::from_secs(5);
pub const TXN_FETCH_INTERVAL: Duration = Duration::from_secs(5);
pub const NETWORK_CHECK_INTERVAL: Duration = Duration::from_secs(10);
pub const MAX_BLOCKS_TO_KEEP: usize = 100;
pub const MAX_TXNS_TO_KEEP: usize = 100;

// Interaction
pub const TICK_RATE: Duration = Duration::from_millis(100);
