# Worker C - Stage 0: HTTP Client Abstraction

## Task Overview
- **Worker**: C
- **Stage**: 0 (Foundation)
- **Duration**: 2 days
- **Risk Level**: Medium
- **Status**: NOT_STARTED

## Prerequisites
- [ ] Fresh branch from `main`: `refactor/stage0-worker-c-client`
- [ ] Rust toolchain working (`cargo build` succeeds)
- [ ] Read `src/algorand.rs` lines 1000-2783 (API client code)
- [ ] Understand current HTTP patterns and error handling

## Deliverables
| File | Lines | Status |
|------|-------|--------|
| `src/client/mod.rs` | ~50 | NOT_STARTED |
| `src/client/http.rs` | ~200 | NOT_STARTED |
| `src/client/indexer.rs` | ~300 | NOT_STARTED |
| `src/client/node.rs` | ~150 | NOT_STARTED |
| `src/client/nfd.rs` | ~100 | NOT_STARTED |

## DO NOT TOUCH
- `src/algorand.rs` (will be updated by coordinator at sync)
- `src/ui.rs`
- `src/widgets.rs`
- `src/app_state.rs`
- Any other existing files

## Dependencies
- Reference types from `src/algorand.rs` directly (will be updated to use domain/ after sync)

---

## Task 1: Create Directory Structure

### Status: NOT_STARTED

- [ ] Create `src/client/` directory
- [ ] Create empty `mod.rs` file
- [ ] Verify directory structure exists

---

## Task 2: Create `src/client/http.rs`

### Status: NOT_STARTED

### 2.1 Analyze Existing HTTP Patterns
- [ ] Count HTTP request patterns in algorand.rs
- [ ] Document response handling pattern (success/404/error)
- [ ] Document retry patterns (if any)
- [ ] Document timeout usage

**Pattern analysis:**
```
Found patterns:
- [ ] Pattern 1: ___________________________ (count: ___)
- [ ] Pattern 2: ___________________________ (count: ___)
(Add more as discovered)
```

### 2.2 Create RequestConfig Struct
- [ ] Define `RequestConfig` struct
- [ ] Add `timeout: Duration` field
- [ ] Add `max_retries: u32` field
- [ ] Add `retry_delay: Duration` field
- [ ] Implement `Default` trait with sensible values

### 2.3 Create HttpClient Struct
- [ ] Define `HttpClient` struct with `client: reqwest::Client` and `config: RequestConfig`
- [ ] Implement `new() -> Result<Self>`
- [ ] Implement `with_config(config: RequestConfig) -> Result<Self>`

### 2.4 Implement Generic GET Method
- [ ] Implement `async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T>`
- [ ] Add retry logic with exponential backoff
- [ ] Add proper error handling for HTTP errors
- [ ] Add timeout handling
- [ ] Handle 404 responses appropriately

### 2.5 Implement Optional GET Method
- [ ] Implement `async fn get_json_optional<T: DeserializeOwned>(&self, url: &str) -> Result<Option<T>>`
- [ ] Return `None` for 404 responses
- [ ] Return error for other failures

### 2.6 Add Request Building Helpers
- [ ] Method to add headers (for LocalNet API token)
- [ ] Method to set custom timeout per-request

### 2.7 Documentation
- [ ] Module-level docs explaining the HTTP client abstraction
- [ ] Document retry behavior
- [ ] Document error handling
- [ ] Add usage examples in `///` docs

### 2.8 Verification
- [ ] File compiles standalone
- [ ] Clippy passes

---

## Task 3: Create `src/client/indexer.rs`

### Status: NOT_STARTED

### 3.1 Analyze Indexer API Usage
- [ ] List all indexer endpoints used in algorand.rs
- [ ] Document request/response patterns for each

**Endpoints found:**
```
- [ ] GET /v2/transactions/{txid}
- [ ] GET /v2/transactions (with params)
- [ ] GET /v2/accounts/{address}
- [ ] GET /v2/assets/{asset-id}
- [ ] GET /v2/blocks/{round}
(Add more as discovered)
```

### 3.2 Create IndexerClient Struct
- [ ] Define `IndexerClient` struct
- [ ] Add `http: HttpClient` field
- [ ] Add `base_url: String` field
- [ ] Implement `new(base_url: &str) -> Result<Self>`
- [ ] Implement `for_network(network: Network) -> Result<Self>`

### 3.3 Implement Transaction Methods
- [ ] `async fn get_transaction(&self, txid: &str) -> Result<Option<Transaction>>`
- [ ] `async fn get_recent_transactions(&self, limit: usize) -> Result<Vec<Transaction>>`
- [ ] `async fn search_transactions(&self, params: &TransactionSearchParams) -> Result<Vec<Transaction>>`

### 3.4 Implement Block Methods
- [ ] `async fn get_block(&self, round: u64) -> Result<Option<BlockInfo>>`
- [ ] `async fn get_recent_blocks(&self, limit: usize) -> Result<Vec<AlgoBlock>>`

### 3.5 Implement Account Methods
- [ ] `async fn get_account(&self, address: &str) -> Result<Option<AccountDetails>>`
- [ ] `async fn get_account_info(&self, address: &str) -> Result<Option<AccountInfo>>`

### 3.6 Implement Asset Methods
- [ ] `async fn get_asset(&self, asset_id: u64) -> Result<Option<AssetDetails>>`
- [ ] `async fn search_assets(&self, query: &str) -> Result<Vec<AssetInfo>>`

### 3.7 Documentation
- [ ] Module docs
- [ ] Document each method with `# Errors` section
- [ ] Add `#[must_use]` where appropriate

### 3.8 Verification
- [ ] File compiles
- [ ] Clippy passes

---

## Task 4: Create `src/client/node.rs`

### Status: NOT_STARTED

### 4.1 Analyze Algod API Usage
- [ ] List all algod endpoints used in algorand.rs
- [ ] Document LocalNet-specific handling

**Endpoints found:**
```
- [ ] GET /v2/status
- [ ] GET /v2/blocks/{round}
- [ ] GET /v2/accounts/{address}
(Add more as discovered)
```

### 4.2 Create NodeClient Struct
- [ ] Define `NodeClient` struct
- [ ] Add `http: HttpClient` field
- [ ] Add `base_url: String` field
- [ ] Add `api_token: Option<String>` field (for LocalNet)
- [ ] Implement `new(base_url: &str, api_token: Option<String>) -> Result<Self>`
- [ ] Implement `for_network(network: Network) -> Result<Self>`

### 4.3 Implement Status Methods
- [ ] `async fn get_status(&self) -> Result<NodeStatus>`
- [ ] `async fn health_check(&self) -> Result<bool>`

### 4.4 Implement Block Methods
- [ ] `async fn get_block(&self, round: u64) -> Result<Option<BlockDetails>>`
- [ ] `async fn get_current_round(&self) -> Result<u64>`

### 4.5 Implement Account Methods (Algod-specific)
- [ ] `async fn get_account(&self, address: &str) -> Result<Option<AccountInfo>>`

### 4.6 Handle LocalNet Auth
- [ ] Add X-Algo-API-Token header when api_token is set
- [ ] Document LocalNet configuration

### 4.7 Documentation & Verification
- [ ] Module docs
- [ ] Clippy passes

---

## Task 5: Create `src/client/nfd.rs`

### Status: NOT_STARTED

### 5.1 Analyze NFD API Usage
- [ ] List NFD endpoints in algorand.rs
- [ ] Document MainNet/TestNet URL differences

**Endpoints found:**
```
- [ ] GET /nfd/{name}
- [ ] GET /nfd/lookup?address={address}
(Add more as discovered)
```

### 5.2 Create NfdClient Struct
- [ ] Define `NfdClient` struct
- [ ] Add `http: HttpClient` field
- [ ] Add `base_url: String` field
- [ ] Implement `new(base_url: &str) -> Result<Self>`
- [ ] Implement `for_network(network: Network) -> Result<Option<Self>>` (None for LocalNet)

### 5.3 Implement NFD Methods
- [ ] `async fn get_by_name(&self, name: &str) -> Result<Option<NfdInfo>>`
- [ ] `async fn get_for_address(&self, address: &str) -> Result<Option<NfdInfo>>`

### 5.4 Handle Network Support
- [ ] Return appropriate error or None for unsupported networks
- [ ] Document network limitations

### 5.5 Documentation & Verification
- [ ] Module docs
- [ ] Clippy passes

---

## Task 6: Create `src/client/mod.rs`

### Status: NOT_STARTED

### 6.1 Module Declarations
- [ ] Declare all submodules
- [ ] Re-export primary types

```rust
mod http;
mod indexer;
mod nfd;
mod node;

pub use http::{HttpClient, RequestConfig};
pub use indexer::IndexerClient;
pub use nfd::NfdClient;
pub use node::NodeClient;
```

### 6.2 Create Unified AlgoClient (Optional)
- [ ] Consider creating `AlgoClient` that wraps all three clients
- [ ] Or document how to use clients independently

### 6.3 Module Documentation
- [ ] Add `//!` docs describing client module purpose
- [ ] Add usage examples

### 6.4 Verification
- [ ] All submodules compile together
- [ ] `cargo check` succeeds

---

## Task 7: Write Unit Tests

### Status: NOT_STARTED

### 7.1 HTTP Client Tests
- [ ] Test RequestConfig default values
- [ ] Test retry logic with mock failures
- [ ] Test timeout handling
- [ ] Test 404 response handling

### 7.2 Client Construction Tests
- [ ] Test IndexerClient::for_network for all networks
- [ ] Test NodeClient::for_network with LocalNet token
- [ ] Test NfdClient::for_network returns None for LocalNet

### 7.3 Integration Tests (Optional - with mocks)
- [ ] Mock successful responses
- [ ] Mock error responses
- [ ] Verify correct URL construction

---

## Task 8: Final Checklist

### Status: NOT_STARTED

- [ ] All 5 files created
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

**API patterns unified:**
- 

**Blocked issues:**
- 

**Notes for coordinator:**
- Dependencies on algorand.rs types - need import updates at sync
