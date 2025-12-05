# Worker B - Stage 0: Domain Types

## Task Overview
- **Worker**: B
- **Stage**: 0 (Foundation)
- **Duration**: 2 days
- **Risk Level**: Low-Medium
- **Status**: NOT_STARTED

## Prerequisites
- [ ] Fresh branch from `main`: `refactor/stage0-worker-b-domain`
- [ ] Rust toolchain working (`cargo build` succeeds)
- [ ] Read `src/algorand.rs` lines 1-1000 (type definitions)

## Deliverables
| File | Lines | Status |
|------|-------|--------|
| `src/domain/mod.rs` | ~30 | NOT_STARTED |
| `src/domain/error.rs` | ~80 | NOT_STARTED |
| `src/domain/network.rs` | ~80 | NOT_STARTED |
| `src/domain/transaction.rs` | ~400 | NOT_STARTED |
| `src/domain/block.rs` | ~100 | NOT_STARTED |
| `src/domain/account.rs` | ~150 | NOT_STARTED |
| `src/domain/asset.rs` | ~100 | NOT_STARTED |
| `src/domain/nfd.rs` | ~50 | NOT_STARTED |

## DO NOT TOUCH
- `src/algorand.rs` (will be updated by coordinator at sync)
- `src/ui.rs`
- `src/widgets.rs`
- `src/app_state.rs`
- Any other existing files

---

## Task 1: Create Directory Structure

### Status: NOT_STARTED

- [ ] Create `src/domain/` directory
- [ ] Create empty `mod.rs` file
- [ ] Verify directory structure exists

---

## Task 2: Create `src/domain/error.rs`

### Status: NOT_STARTED

### 2.1 Extract Error Types from algorand.rs
- [ ] Locate `AlgoError` enum (around line 14-58)
- [ ] Copy error type definition
- [ ] Copy all impl blocks for AlgoError

### 2.2 Implement Error Type
- [ ] Add `use thiserror::Error;`
- [ ] Define `AlgoError` enum with variants:
  - [ ] `Network(#[from] reqwest::Error)`
  - [ ] `Parse { message: String }`
  - [ ] `NotFound { entity: &'static str, id: String }`
  - [ ] `InvalidInput(String)`
- [ ] Implement builder methods: `parse()`, `not_found()`, `invalid_input()`
- [ ] Implement `into_report()` for color_eyre integration
- [ ] Add `#[must_use]` where appropriate

### 2.3 Documentation
- [ ] Add module-level `//!` docs
- [ ] Document each error variant
- [ ] Add usage examples in docs

### 2.4 Verification
- [ ] File compiles standalone
- [ ] Clippy passes

---

## Task 3: Create `src/domain/network.rs`

### Status: NOT_STARTED

### 3.1 Extract Network Types from algorand.rs
- [ ] Locate `Network` enum (around line 64-116)
- [ ] Copy Network enum definition
- [ ] Copy all impl blocks

### 3.2 Implement Network Type
- [ ] Define `Network` enum: `MainNet`, `TestNet`, `LocalNet`
- [ ] Implement `const fn indexer_url(&self) -> &'static str`
- [ ] Implement `const fn algod_url(&self) -> &'static str`
- [ ] Implement `const fn nfd_api_url(&self) -> Option<&'static str>`
- [ ] Implement `const fn supports_nfd(&self) -> bool`
- [ ] Implement `as_str(&self) -> &'static str`
- [ ] Derive: `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default`

### 3.3 Documentation
- [ ] Add module-level docs
- [ ] Document each network variant with URLs

### 3.4 Verification
- [ ] File compiles standalone
- [ ] Clippy passes

---

## Task 4: Create `src/domain/transaction.rs`

### Status: NOT_STARTED

### 4.1 Extract Transaction Types from algorand.rs
- [ ] Locate `TxnType` enum (around line 122)
- [ ] Locate `Transaction` struct 
- [ ] Locate `TransactionDetails` enum
- [ ] Locate all detail structs: `PaymentDetails`, `AssetTransferDetails`, etc.
- [ ] Locate `OnComplete` enum
- [ ] Locate helper structs: `BoxRef`, `StateSchema`

### 4.2 Implement Core Types
- [ ] Define `TxnType` enum with variants:
  - [ ] Payment, AssetTransfer, ApplicationCall, AssetConfig
  - [ ] AssetFreeze, KeyRegistration, StateProof, Heartbeat, Unknown
- [ ] Implement `TxnType::label(&self) -> &'static str`
- [ ] Implement `TxnType::code(&self) -> &'static str`
- [ ] Add serde rename attributes for API compatibility

### 4.3 Implement Transaction Struct
- [ ] Define `Transaction` struct with all fields
- [ ] Add serde rename attributes for JSON mapping
- [ ] Implement `from_json(value: &Value) -> Option<Self>`

### 4.4 Implement Detail Types
- [ ] `PaymentDetails` struct
- [ ] `AssetTransferDetails` struct
- [ ] `AssetConfigDetails` struct
- [ ] `AssetFreezeDetails` struct
- [ ] `ApplicationDetails` struct (AppCallDetails)
- [ ] `KeyRegDetails` struct
- [ ] `StateProofDetails` struct
- [ ] `HeartbeatDetails` struct

### 4.5 Implement TransactionDetails Enum
- [ ] Define enum wrapping all detail types
- [ ] Implement extraction from Transaction

### 4.6 Implement Helper Types
- [ ] `OnComplete` enum
- [ ] `BoxRef` struct
- [ ] `StateSchema` struct

### 4.7 Documentation
- [ ] Module docs with overview
- [ ] Document each type
- [ ] Document JSON field mappings

### 4.8 Verification
- [ ] File compiles standalone
- [ ] Clippy passes

---

## Task 5: Create `src/domain/block.rs`

### Status: NOT_STARTED

### 5.1 Extract Block Types from algorand.rs
- [ ] Locate `AlgoBlock` struct (around line 815)
- [ ] Locate `BlockInfo` struct
- [ ] Locate `BlockDetails` struct

### 5.2 Implement Types
- [ ] `AlgoBlock` struct (id, txn_count, timestamp)
- [ ] `BlockInfo` struct (id, timestamp, txn_count, proposer, seed)
- [ ] `BlockDetails` struct (info, transactions, txn_type_counts)
- [ ] Implement `from_json` methods where needed

### 5.3 Documentation & Verification
- [ ] Add docs
- [ ] Clippy passes

---

## Task 6: Create `src/domain/account.rs`

### Status: NOT_STARTED

### 6.1 Extract Account Types from algorand.rs
- [ ] Locate `AccountInfo` struct (around line 846)
- [ ] Locate `AccountDetails` struct
- [ ] Locate `ParticipationInfo` struct
- [ ] Locate `AccountAssetHolding` struct
- [ ] Locate `AccountAppLocalState` struct

### 6.2 Implement Types
- [ ] `AccountInfo` struct (summary view)
- [ ] `AccountDetails` struct (full details)
- [ ] `ParticipationInfo` struct
- [ ] `AccountAssetHolding` struct
- [ ] `AccountAppLocalState` struct
- [ ] Implement `from_json` methods

### 6.3 Documentation & Verification
- [ ] Add docs
- [ ] Clippy passes

---

## Task 7: Create `src/domain/asset.rs`

### Status: NOT_STARTED

### 7.1 Extract Asset Types from algorand.rs
- [ ] Locate `AssetInfo` struct
- [ ] Locate `AssetDetails` struct
- [ ] Locate `AssetParams` struct

### 7.2 Implement Types
- [ ] `AssetInfo` struct (summary)
- [ ] `AssetDetails` struct (full)
- [ ] `AssetParams` struct
- [ ] Implement `from_json` methods

### 7.3 Documentation & Verification
- [ ] Add docs
- [ ] Clippy passes

---

## Task 8: Create `src/domain/nfd.rs`

### Status: NOT_STARTED

### 8.1 Extract NFD Types from algorand.rs
- [ ] Locate `NfdInfo` struct (around line 959)

### 8.2 Implement Types
- [ ] `NfdInfo` struct (name, deposit_account, owner, avatar_url, is_verified)
- [ ] Implement `from_json` method

### 8.3 Documentation & Verification
- [ ] Add docs
- [ ] Clippy passes

---

## Task 9: Create `src/domain/mod.rs`

### Status: NOT_STARTED

### 9.1 Module Declarations
- [ ] Declare all submodules
- [ ] Re-export all public types

```rust
mod account;
mod asset;
mod block;
mod error;
mod network;
mod nfd;
mod transaction;

pub use account::*;
pub use asset::*;
pub use block::*;
pub use error::*;
pub use network::*;
pub use nfd::*;
pub use transaction::*;
```

### 9.2 Module Documentation
- [ ] Add `//!` module-level docs describing domain module purpose

### 9.3 Verification
- [ ] All submodules compile together
- [ ] `cargo check` succeeds

---

## Task 10: Write Unit Tests

### Status: NOT_STARTED

### 10.1 Serialization Tests
- [ ] Test Transaction JSON parsing
- [ ] Test Block JSON parsing
- [ ] Test Account JSON parsing
- [ ] Test Asset JSON parsing
- [ ] Test NFD JSON parsing

### 10.2 Type Tests
- [ ] Test TxnType label() and code() methods
- [ ] Test Network URL methods
- [ ] Test AlgoError display

### 10.3 Edge Cases
- [ ] Test parsing with missing optional fields
- [ ] Test parsing with unknown transaction types

---

## Task 11: Final Checklist

### Status: NOT_STARTED

- [ ] All 8 files created
- [ ] `cargo build` succeeds
- [ ] `cargo test --all-features` passes
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] No modifications to existing files
- [ ] Branch ready for PR

---

## Progress Log

| Date | Task | Notes |
|------|------|-------|
| | | |

---

## Handoff Notes
(To be filled when complete)

**Files created:**
- 

**Types extracted:**
- 

**Blocked issues:**
- 

**Notes for coordinator:**
- 
