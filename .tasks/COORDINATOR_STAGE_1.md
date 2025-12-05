# Coordinator - Stage 1 Sync

## Task Overview
- **Role**: Coordinator
- **Stage**: 1 (Sync Point)
- **Duration**: 1 day
- **Risk Level**: Medium
- **Status**: NOT_STARTED
- **Depends On**: All Stage 0 workers complete

## Prerequisites
- [ ] Worker A Stage 0 complete (theme.rs, constants.rs)
- [ ] Worker B Stage 0 complete (domain/)
- [ ] Worker C Stage 0 complete (client/)
- [ ] All three branches pass CI

## Deliverables
- [ ] Merged `main` branch with all Stage 0 work
- [ ] Updated `algorand.rs` as re-export facade
- [ ] Updated `main.rs` with new imports
- [ ] All tests passing
- [ ] Application runs correctly

---

## Task 1: Merge Stage 0 Branches

### Status: NOT_STARTED

### 1.1 Review PRs
- [ ] Review Worker A PR (theme + constants)
- [ ] Review Worker B PR (domain)
- [ ] Review Worker C PR (client)
- [ ] Verify no conflicting changes

### 1.2 Merge Order
- [ ] Merge Worker A (theme + constants) first - no dependencies
- [ ] Merge Worker B (domain) second - no dependencies
- [ ] Merge Worker C (client) third - may reference domain types

### 1.3 Resolve Any Conflicts
- [ ] If conflicts exist, document and resolve
- [ ] Re-run tests after resolution

---

## Task 2: Update `src/algorand.rs` as Facade

### Status: NOT_STARTED

### 2.1 Create Re-exports
Replace algorand.rs content with facade pattern:

```rust
//! Algorand API client and domain types
//!
//! This module re-exports types from the `domain` and `client` modules
//! for backward compatibility.

// Re-export domain types
pub use crate::domain::{
    // Error types
    AlgoError,
    
    // Network
    Network,
    
    // Transaction types
    Transaction, TransactionDetails, TxnType,
    PaymentDetails, AssetTransferDetails, AssetConfigDetails,
    AssetFreezeDetails, ApplicationDetails, KeyRegDetails,
    StateProofDetails, HeartbeatDetails,
    OnComplete, BoxRef, StateSchema,
    
    // Block types
    AlgoBlock, BlockInfo, BlockDetails,
    
    // Account types
    AccountInfo, AccountDetails, ParticipationInfo,
    AccountAssetHolding, AccountAppLocalState,
    
    // Asset types
    AssetInfo, AssetDetails, AssetParams,
    
    // NFD types
    NfdInfo,
};

// Re-export client types
pub use crate::client::{
    HttpClient, RequestConfig,
    IndexerClient, NodeClient, NfdClient,
};

// Keep AlgoClient as main entry point (if unified client exists)
// Or create it here combining the three clients
pub struct AlgoClient {
    pub indexer: IndexerClient,
    pub node: NodeClient,
    pub nfd: Option<NfdClient>,
}

impl AlgoClient {
    pub fn new(network: Network) -> color_eyre::Result<Self> {
        Ok(Self {
            indexer: IndexerClient::for_network(network)?,
            node: NodeClient::for_network(network)?,
            nfd: NfdClient::for_network(network).ok(),
        })
    }
    
    // Delegate methods to appropriate clients
    // ... (migrate existing AlgoClient methods)
}
```

### 2.2 Preserve Existing API
- [ ] Ensure all public types still accessible via `crate::algorand::*`
- [ ] Ensure all public methods still callable
- [ ] No breaking changes to other modules

### 2.3 Verification
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] No warnings about unused imports

---

## Task 3: Update `src/main.rs`

### Status: NOT_STARTED

### 3.1 Add Module Declarations
```rust
mod algorand;  // Existing - now facade
mod app_state;
mod boot_screen;
mod client;    // NEW
mod commands;
mod constants; // NEW
mod domain;    // NEW
mod theme;     // NEW
mod tui;
mod ui;
mod updater;
mod widgets;
```

### 3.2 Update Imports (if needed)
- [ ] Check if main.rs uses any algorand types directly
- [ ] Update to use new module paths if beneficial
- [ ] Or keep using algorand facade

### 3.3 Verification
- [ ] Application starts correctly
- [ ] All functionality works

---

## Task 4: Update Other Files (if needed)

### Status: NOT_STARTED

### 4.1 Check `app_state.rs`
- [ ] Uses `crate::algorand::*` - should still work via facade
- [ ] No changes needed if facade is complete

### 4.2 Check `ui.rs`
- [ ] Uses algorand types - should work via facade
- [ ] May benefit from theme imports (optional - defer to Stage 2.5)

### 4.3 Check `widgets.rs`
- [ ] Uses algorand types - should work via facade
- [ ] May benefit from theme imports (optional - defer to Stage 1.5)

### 4.4 Check `commands.rs`
- [ ] Likely doesn't use algorand types directly
- [ ] No changes expected

---

## Task 5: Run Full Test Suite

### Status: NOT_STARTED

### 5.1 Unit Tests
- [ ] `cargo test --all-features`
- [ ] All domain tests pass
- [ ] All client tests pass
- [ ] All existing tests pass

### 5.2 Clippy
- [ ] `cargo clippy --all-features -- -D warnings`
- [ ] No new warnings

### 5.3 Formatting
- [ ] `cargo fmt -- --check`
- [ ] All files formatted

### 5.4 Build
- [ ] `cargo build`
- [ ] `cargo build --release`

---

## Task 6: Manual Testing

### Status: NOT_STARTED

### 6.1 Basic Functionality
- [ ] Application starts
- [ ] Connects to MainNet
- [ ] Loads blocks
- [ ] Loads transactions
- [ ] Navigation works

### 6.2 Network Switching
- [ ] Switch to TestNet
- [ ] Switch to LocalNet (if available)
- [ ] Switch back to MainNet

### 6.3 Search
- [ ] Search for transaction
- [ ] Search for account
- [ ] Search for asset

### 6.4 Detail Views
- [ ] View block details
- [ ] View transaction details
- [ ] View account details

---

## Task 7: Create Stage 1.5 Branches

### Status: NOT_STARTED

### 7.1 Notify Workers
- [ ] Notify Worker A: Create `refactor/stage1.5-worker-a-widgets-list`
- [ ] Notify Worker B: Create `refactor/stage1.5-worker-b-widgets-graph`
- [ ] Notify Worker C: Create `refactor/stage1.5-worker-c-state`

### 7.2 Document Starting Point
- [ ] Record commit hash of merged main
- [ ] Update REFACTORING_PLAN.md with Stage 0 completion

---

## Task 8: Final Checklist

### Status: NOT_STARTED

- [ ] All Stage 0 PRs merged
- [ ] algorand.rs is now a facade
- [ ] main.rs declares new modules
- [ ] All tests pass
- [ ] Application runs correctly
- [ ] No regressions
- [ ] Workers notified to start Stage 1.5

---

## Progress Log

| Date | Task | Notes |
|------|------|-------|
| | | |

---

## Issues Encountered
(Document any problems and resolutions)

| Issue | Resolution |
|-------|------------|
| | |

---

## Stage 0 Summary

**Worker A deliverables:**
- [ ] src/theme.rs - _____ lines
- [ ] src/constants.rs - _____ lines

**Worker B deliverables:**
- [ ] src/domain/mod.rs
- [ ] src/domain/error.rs
- [ ] src/domain/network.rs
- [ ] src/domain/transaction.rs
- [ ] src/domain/block.rs
- [ ] src/domain/account.rs
- [ ] src/domain/asset.rs
- [ ] src/domain/nfd.rs
- Total: _____ lines

**Worker C deliverables:**
- [ ] src/client/mod.rs
- [ ] src/client/http.rs
- [ ] src/client/indexer.rs
- [ ] src/client/node.rs
- [ ] src/client/nfd.rs
- Total: _____ lines

**Total new code:** _____ lines
**algorand.rs reduction:** from 2783 lines to ~100 lines (facade)
