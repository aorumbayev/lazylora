# Worker A - Stage 0: Theme & Constants

## Task Overview
- **Worker**: A
- **Stage**: 0 (Foundation)
- **Duration**: 2 days
- **Risk Level**: Low
- **Status**: NOT_STARTED

## Prerequisites
- [ ] Fresh branch from `main`: `refactor/stage0-worker-a-theme`
- [ ] Rust toolchain working (`cargo build` succeeds)
- [ ] Read existing color usage in `ui.rs` and `widgets.rs`

## Deliverables
| File | Lines | Status |
|------|-------|--------|
| `src/theme.rs` | ~150 | NOT_STARTED |
| `src/constants.rs` | ~50 | NOT_STARTED |

## DO NOT TOUCH
- `src/ui.rs`
- `src/widgets.rs`
- `src/algorand.rs`
- `src/app_state.rs`
- Any other existing files

---

## Task 1: Create `src/theme.rs`

### Status: NOT_STARTED

### 1.1 Extract Color Definitions
- [ ] Search for `Color::Rgb` in `ui.rs` - document all colors found
- [ ] Search for `Color::Rgb` in `widgets.rs` - document all colors found
- [ ] Identify duplicate color definitions
- [ ] Create `pub mod colors` with deduplicated constants

**Colors to extract:**
```
Location: ui.rs
- [ ] Line ___: Color::Rgb(___, ___, ___) - Purpose: ___________
- [ ] Line ___: Color::Rgb(___, ___, ___) - Purpose: ___________
(Add more as discovered)

Location: widgets.rs
- [ ] Line ___: Color::Rgb(___, ___, ___) - Purpose: ___________
- [ ] Line ___: Color::Rgb(___, ___, ___) - Purpose: ___________
(Add more as discovered)
```

### 1.2 Create Color Constants
- [ ] Define brand colors (PRIMARY, SECONDARY)
- [ ] Define semantic colors (SUCCESS, ERROR, WARNING, INFO)
- [ ] Define transaction type colors (TXN_PAYMENT, TXN_ASSET_TRANSFER, etc.)
- [ ] Define UI element colors (BORDER_ACTIVE, BORDER_INACTIVE, TEXT_*, etc.)

### 1.3 Create Style Helpers
- [ ] Create `pub mod styles` submodule
- [ ] Implement `title() -> Style`
- [ ] Implement `border_active() -> Style`
- [ ] Implement `border_inactive() -> Style`
- [ ] Implement `text_primary() -> Style`
- [ ] Implement `text_secondary() -> Style`
- [ ] Implement `highlight() -> Style`
- [ ] Implement `success() -> Style`
- [ ] Implement `error() -> Style`
- [ ] Add `#[must_use]` attributes to all style functions

### 1.4 Create Layout Constants
- [ ] Create `pub mod layout` submodule
- [ ] Define MIN_WIDTH, MIN_HEIGHT
- [ ] Define PADDING
- [ ] Define BLOCK_LIST_WIDTH_PCT, TXN_LIST_WIDTH_PCT
- [ ] Define HEADER_HEIGHT, FOOTER_HEIGHT

### 1.5 Documentation
- [ ] Add module-level `//!` documentation
- [ ] Add `///` docs for each public constant
- [ ] Add `///` docs for each public function with `# Returns` section

### 1.6 Verification
- [ ] File compiles: `cargo check`
- [ ] No warnings: `cargo clippy --all-features -- -D warnings`
- [ ] Formatting correct: `cargo fmt -- --check`

---

## Task 2: Create `src/constants.rs`

### Status: NOT_STARTED

### 2.1 Extract API Constants
- [ ] Search for timeout values in `algorand.rs`
- [ ] Search for retry counts in `algorand.rs`
- [ ] Create `pub mod api` with REQUEST_TIMEOUT_SECS, MAX_RETRIES, RETRY_DELAY_MS

### 2.2 Create App Metadata
- [ ] Create `pub mod app` submodule
- [ ] Define NAME (from Cargo.toml)
- [ ] Define VERSION using `env!("CARGO_PKG_VERSION")`
- [ ] Define CONFIG_FILE name

### 2.3 Create Pagination Constants
- [ ] Search for page size values in codebase
- [ ] Create `pub mod pagination` with DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE

### 2.4 Create Format Constants
- [ ] Search for address truncation lengths
- [ ] Create `pub mod format` with ADDRESS_PREFIX_LEN, ADDRESS_SUFFIX_LEN
- [ ] Add TXID_PREFIX_LEN, TXID_SUFFIX_LEN

### 2.5 Documentation
- [ ] Add module-level `//!` documentation
- [ ] Add `///` docs for each constant

### 2.6 Verification
- [ ] File compiles: `cargo check`
- [ ] No warnings: `cargo clippy --all-features -- -D warnings`
- [ ] Formatting correct: `cargo fmt -- --check`

---

## Task 3: Write Unit Tests

### Status: NOT_STARTED

### 3.1 Theme Tests
- [ ] Create `#[cfg(test)] mod tests` in theme.rs
- [ ] Test that style functions return non-default styles
- [ ] Test color constants are valid RGB values

### 3.2 Constants Tests
- [ ] Create `#[cfg(test)] mod tests` in constants.rs
- [ ] Test VERSION is not empty
- [ ] Test pagination values are sensible (DEFAULT < MAX)

---

## Task 4: Final Checklist

### Status: NOT_STARTED

- [ ] All files created and compile
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

**Blocked issues:**
- 

**Notes for coordinator:**
- 
