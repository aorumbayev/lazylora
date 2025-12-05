# Worker A - Stage 1.5: Widgets List & Common

## Task Overview
- **Worker**: A
- **Stage**: 1.5 (Core Split)
- **Duration**: 3 days
- **Risk Level**: Medium
- **Status**: NOT_STARTED
- **Depends On**: Stage 1 Sync Complete

## Prerequisites
- [ ] Stage 0 complete and merged
- [ ] Stage 1 sync complete (algorand.rs is now facade)
- [ ] Fresh branch from post-sync main: `refactor/stage1.5-worker-a-widgets-list`
- [ ] `src/theme.rs` available (from Stage 0)
- [ ] `src/domain/` available (from Stage 0)

## Deliverables
| File | Lines | Status |
|------|-------|--------|
| `src/widgets/mod.rs` | ~80 | NOT_STARTED |
| `src/widgets/helpers.rs` | ~100 | NOT_STARTED |
| `src/widgets/common/mod.rs` | ~20 | NOT_STARTED |
| `src/widgets/common/badge.rs` | ~100 | NOT_STARTED |
| `src/widgets/common/amount.rs` | ~80 | NOT_STARTED |
| `src/widgets/common/address.rs` | ~80 | NOT_STARTED |
| `src/widgets/list/mod.rs` | ~50 | NOT_STARTED |
| `src/widgets/list/state.rs` | ~150 | NOT_STARTED |
| `src/widgets/list/block_list.rs` | ~200 | NOT_STARTED |
| `src/widgets/list/txn_list.rs` | ~250 | NOT_STARTED |

## DO NOT TOUCH
- `src/widgets.rs` (will be deleted at Stage 2 sync)
- `src/widgets/graph/*` (Worker B)
- `src/widgets/detail/*` (Worker B)
- `src/ui.rs`
- `src/app_state.rs`

---

## Task 1: Create Directory Structure

### Status: NOT_STARTED

- [ ] Create `src/widgets/` directory
- [ ] Create `src/widgets/common/` directory
- [ ] Create `src/widgets/list/` directory
- [ ] Create placeholder `mod.rs` files

---

## Task 2: Create `src/widgets/helpers.rs`

### Status: NOT_STARTED

### 2.1 Extract Helper Functions from widgets.rs
- [ ] Locate `truncate_address` function
- [ ] Locate `format_amount` function
- [ ] Locate `format_timestamp` function
- [ ] Locate `format_microalgos` function
- [ ] Locate any other formatting helpers

**Functions found in widgets.rs:**
```
- [ ] Line ___: truncate_address
- [ ] Line ___: format_amount
- [ ] Line ___: _______________
(Add more as discovered)
```

### 2.2 Implement Helper Functions
- [ ] `pub fn truncate_address(address: &str, prefix: usize, suffix: usize) -> String`
- [ ] `pub fn format_microalgos(microalgos: u64) -> String`
- [ ] `pub fn format_timestamp(unix_ts: u64) -> String`
- [ ] `pub fn format_amount(amount: u64, decimals: u8) -> String`
- [ ] Add `#[must_use]` to all pure functions

### 2.3 Use Constants
- [ ] Import `crate::constants::format::*`
- [ ] Use ADDRESS_PREFIX_LEN, ADDRESS_SUFFIX_LEN constants

### 2.4 Documentation & Tests
- [ ] Document each function
- [ ] Write unit tests for edge cases

---

## Task 3: Create Generic `ListState<T>` in `src/widgets/list/state.rs`

### Status: NOT_STARTED

### 3.1 Analyze Existing List States
- [ ] Locate `BlockListState` in widgets.rs (around line 1328)
- [ ] Locate `TransactionListState` in widgets.rs (around line 1535)
- [ ] Document common fields and methods
- [ ] Identify differences

**Common elements:**
```
Fields:
- [ ] selected_index: Option<usize>
- [ ] scroll_position: u16
- [ ] _______________

Methods:
- [ ] new()
- [ ] select()
- [ ] selected()
- [ ] _______________
```

### 3.2 Design Generic ListState
- [ ] Create `ListState<T>` struct
- [ ] Add `items: Vec<T>` field
- [ ] Add `selected: Option<usize>` field
- [ ] Add `offset: usize` field (scroll position)
- [ ] Add `viewport_height: usize` field

### 3.3 Implement Navigation Methods
- [ ] `pub fn new(items: Vec<T>) -> Self`
- [ ] `pub fn select_next(&mut self)`
- [ ] `pub fn select_previous(&mut self)`
- [ ] `pub fn select_first(&mut self)`
- [ ] `pub fn select_last(&mut self)`
- [ ] `pub fn page_down(&mut self)`
- [ ] `pub fn page_up(&mut self)`

### 3.4 Implement Accessor Methods
- [ ] `pub fn selected_item(&self) -> Option<&T>`
- [ ] `pub fn selected_index(&self) -> Option<usize>`
- [ ] `pub fn visible_items(&self) -> &[T]`
- [ ] `pub fn items(&self) -> &[T]`
- [ ] `pub fn len(&self) -> usize`
- [ ] `pub fn is_empty(&self) -> bool`

### 3.5 Implement Update Methods
- [ ] `pub fn set_items(&mut self, items: Vec<T>)`
- [ ] `pub fn set_viewport_height(&mut self, height: usize)`
- [ ] Private `fn adjust_offset(&mut self)`

### 3.6 Implement Traits
- [ ] `impl<T> Default for ListState<T>`
- [ ] Consider `impl<T: Clone> Clone for ListState<T>`

### 3.7 Create ListItem Trait (Optional)
- [ ] Define `pub trait ListItem`
- [ ] Method `fn display_text(&self) -> String`
- [ ] Method `fn secondary_text(&self) -> Option<String>`

### 3.8 Documentation & Tests
- [ ] Document all methods
- [ ] Test navigation at boundaries
- [ ] Test empty list handling
- [ ] Test viewport scrolling

---

## Task 4: Create `src/widgets/common/badge.rs` (TxnTypeBadge)

### Status: NOT_STARTED

### 4.1 Extract TxnTypeBadge from widgets.rs
- [ ] Locate `TxnTypeBadge` struct (around line 184)
- [ ] Locate Widget impl (around line 248)
- [ ] Document rendering logic

### 4.2 Implement TxnTypeBadge
- [ ] Define struct with transaction type info
- [ ] Implement `new()` constructor
- [ ] Import colors from `crate::theme::colors`

### 4.3 Implement Widget Trait
- [ ] `impl Widget for TxnTypeBadge`
- [ ] Implement `fn render(self, area: Rect, buf: &mut Buffer)`
- [ ] Use theme colors for transaction types

### 4.4 Add Color Mapping
- [ ] Map TxnType to theme colors
- [ ] Add icon/emoji for each type

### 4.5 Documentation & Verification
- [ ] Document widget usage
- [ ] Clippy passes

---

## Task 5: Create `src/widgets/common/amount.rs` (AmountDisplay)

### Status: NOT_STARTED

### 5.1 Extract AmountDisplay from widgets.rs
- [ ] Locate `AmountDisplay` struct (around line 522)
- [ ] Locate Widget impl (around line 605)

### 5.2 Implement AmountDisplay
- [ ] Define struct for amount display
- [ ] Support ALGO and ASA amounts
- [ ] Support decimals configuration

### 5.3 Implement Widget Trait
- [ ] Render formatted amount
- [ ] Use appropriate colors

### 5.4 Documentation & Verification
- [ ] Add docs
- [ ] Clippy passes

---

## Task 6: Create `src/widgets/common/address.rs` (AddressDisplay)

### Status: NOT_STARTED

### 6.1 Extract AddressDisplay from widgets.rs
- [ ] Locate `AddressDisplay` struct (around line 643)
- [ ] Locate Widget impl (around line 703)

### 6.2 Implement AddressDisplay
- [ ] Define struct for address display
- [ ] Support optional label
- [ ] Support truncation

### 6.3 Implement Widget Trait
- [ ] Render truncated address
- [ ] Show label if present

### 6.4 Documentation & Verification
- [ ] Add docs
- [ ] Clippy passes

---

## Task 7: Create `src/widgets/list/block_list.rs`

### Status: NOT_STARTED

### 7.1 Extract BlockListWidget from widgets.rs
- [ ] Locate `BlockListWidget` struct (around line 1384)
- [ ] Locate StatefulWidget impl (around line 1435)

### 7.2 Implement BlockListWidget
- [ ] Define widget struct
- [ ] Accept `&[AlgoBlock]` items
- [ ] Accept focus state

### 7.3 Implement StatefulWidget Trait
- [ ] Use `ListState<AlgoBlock>` as state type
- [ ] Render block list with selection
- [ ] Handle empty state
- [ ] Add scrollbar

### 7.4 Styling
- [ ] Use theme colors for borders
- [ ] Highlight selected item
- [ ] Show focus state

### 7.5 Documentation & Verification
- [ ] Document usage
- [ ] Clippy passes

---

## Task 8: Create `src/widgets/list/txn_list.rs`

### Status: NOT_STARTED

### 8.1 Extract TransactionListWidget from widgets.rs
- [ ] Locate `TransactionListWidget` struct (around line 1593)
- [ ] Locate StatefulWidget impl (around line 1644)

### 8.2 Implement TransactionListWidget
- [ ] Define widget struct
- [ ] Accept `&[Transaction]` items
- [ ] Accept focus state

### 8.3 Implement StatefulWidget Trait
- [ ] Use `ListState<Transaction>` as state type
- [ ] Render transaction list with selection
- [ ] Include TxnTypeBadge for each item
- [ ] Handle empty state
- [ ] Add scrollbar

### 8.4 Styling
- [ ] Use theme colors
- [ ] Show transaction type badges
- [ ] Highlight selected

### 8.5 Documentation & Verification
- [ ] Document usage
- [ ] Clippy passes

---

## Task 9: Create Module Files

### Status: NOT_STARTED

### 9.1 Create `src/widgets/common/mod.rs`
```rust
mod address;
mod amount;
mod badge;

pub use address::AddressDisplay;
pub use amount::AmountDisplay;
pub use badge::TxnTypeBadge;
```

### 9.2 Create `src/widgets/list/mod.rs`
```rust
mod block_list;
mod state;
mod txn_list;

pub use block_list::BlockListWidget;
pub use state::{ListState, ListItem};
pub use txn_list::TransactionListWidget;
```

### 9.3 Create `src/widgets/mod.rs`
```rust
pub mod common;
pub mod helpers;
pub mod list;

// Re-export commonly used items
pub use common::*;
pub use helpers::*;
pub use list::*;
```

---

## Task 10: Final Checklist

### Status: NOT_STARTED

- [ ] All 10 files created
- [ ] `cargo build` succeeds
- [ ] `cargo test --all-features` passes
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] No modifications to existing files (especially `widgets.rs`)
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

**Generic ListState benefits:**
- 

**Blocked issues:**
- 

**Notes for coordinator:**
- Worker B needs to create widgets/graph/ and widgets/detail/ in parallel
- Final widgets/mod.rs will need to import from both workers' directories
