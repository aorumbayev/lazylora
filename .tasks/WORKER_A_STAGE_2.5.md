# Worker A - Stage 2.5: UI Panels & Layout

## Task Overview
- **Worker**: A
- **Stage**: 2.5 (UI Split)
- **Duration**: 3 days
- **Risk Level**: Medium-High
- **Status**: NOT_STARTED
- **Depends On**: Stage 2 Sync Complete

## Prerequisites
- [ ] Stage 2 sync complete (widgets/ and state/ modules ready)
- [ ] Fresh branch from post-sync main: `refactor/stage2.5-worker-a-ui-panels`
- [ ] `src/theme.rs` available
- [ ] `src/widgets/` available
- [ ] `src/state/` available
- [ ] Read `src/ui.rs` thoroughly

## Deliverables
| File | Lines | Status |
|------|-------|--------|
| `src/ui/mod.rs` | ~100 | NOT_STARTED |
| `src/ui/layout.rs` | ~150 | NOT_STARTED |
| `src/ui/header.rs` | ~100 | NOT_STARTED |
| `src/ui/footer.rs` | ~80 | NOT_STARTED |
| `src/ui/panels/mod.rs` | ~30 | NOT_STARTED |
| `src/ui/panels/blocks.rs` | ~200 | NOT_STARTED |
| `src/ui/panels/transactions.rs` | ~250 | NOT_STARTED |
| `src/ui/panels/details/mod.rs` | ~50 | NOT_STARTED |
| `src/ui/panels/details/block.rs` | ~200 | NOT_STARTED |
| `src/ui/panels/details/transaction.rs` | ~300 | NOT_STARTED |
| `src/ui/panels/details/account.rs` | ~200 | NOT_STARTED |
| `src/ui/panels/details/asset.rs` | ~150 | NOT_STARTED |

## DO NOT TOUCH
- `src/ui.rs` (will be deleted at Stage 3)
- `src/ui/popups/*` (Worker B)
- `src/ui/components/*` (Worker B)

---

## Task 1: Create Directory Structure

### Status: NOT_STARTED

- [ ] Create `src/ui/` directory
- [ ] Create `src/ui/panels/` directory
- [ ] Create `src/ui/panels/details/` directory
- [ ] Create placeholder `mod.rs` files

---

## Task 2: Analyze `ui.rs` Structure

### Status: NOT_STARTED

### 2.1 Document Render Functions
- [ ] List all `render_*` functions in ui.rs
- [ ] Categorize by: header, footer, panels, details, popups
- [ ] Note line numbers and sizes

**Functions found:**
```
Header/Footer:
- [ ] render_header (line ___) - ___ lines
- [ ] render_footer (line ___) - ___ lines

Panels:
- [ ] render_blocks_panel (line ___) - ___ lines
- [ ] render_transactions_panel (line ___) - ___ lines

Details:
- [ ] render_block_details (line ___) - ___ lines
- [ ] render_transaction_details (line ___) - ___ lines
- [ ] render_account_details (line ___) - ___ lines
- [ ] render_asset_details (line ___) - ___ lines

Popups (Worker B):
- [ ] render_search_popup (line ___) - ___ lines
- [ ] render_network_popup (line ___) - ___ lines
- [ ] render_help_popup (line ___) - ___ lines
```

### 2.2 Document Layout Functions
- [ ] Find main layout calculation code
- [ ] Find popup centering helpers
- [ ] Find constraint definitions

---

## Task 3: Create `src/ui/layout.rs`

### Status: NOT_STARTED

### 3.1 Define Layout Structs
```rust
/// Main application layout areas
#[derive(Debug, Clone, Copy)]
pub struct AppLayout {
    pub header: Rect,
    pub main: Rect,
    pub footer: Rect,
}

/// Two-panel layout for main content
#[derive(Debug, Clone, Copy)]
pub struct MainLayout {
    pub left_panel: Rect,
    pub right_panel: Rect,
}

/// Detail view layout
#[derive(Debug, Clone, Copy)]
pub struct DetailLayout {
    pub header: Rect,
    pub content: Rect,
    pub tabs: Option<Rect>,
}
```

### 3.2 Implement Layout Functions
- [ ] `pub fn calculate_app_layout(area: Rect) -> AppLayout`
- [ ] `pub fn calculate_main_layout(area: Rect, app: &App) -> MainLayout`
- [ ] `pub fn calculate_detail_layout(area: Rect) -> DetailLayout`
- [ ] `pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect`

### 3.3 Use Theme Constants
- [ ] Import `crate::theme::layout::*`
- [ ] Use HEADER_HEIGHT, FOOTER_HEIGHT, etc.

### 3.4 Documentation & Verification
- [ ] Document layout calculations
- [ ] Clippy passes

---

## Task 4: Create `src/ui/header.rs`

### Status: NOT_STARTED

### 4.1 Extract Header Rendering
- [ ] Locate header rendering code in ui.rs
- [ ] Extract to separate function

### 4.2 Implement Header
```rust
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    // Logo/title
    // Network status
    // Live indicator
    // Current round
}
```

### 4.3 Components
- [ ] App title with logo
- [ ] Network indicator (MainNet/TestNet/LocalNet)
- [ ] Live updates indicator
- [ ] Current block round

### 4.4 Documentation & Verification
- [ ] Document header layout
- [ ] Clippy passes

---

## Task 5: Create `src/ui/footer.rs`

### Status: NOT_STARTED

### 5.1 Extract Footer Rendering
- [ ] Locate footer/status bar code in ui.rs
- [ ] Extract keybinding hints

### 5.2 Implement Footer
```rust
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    // Context-sensitive keybinding hints
    // Status messages
}
```

### 5.3 Keybinding Hints
- [ ] Show relevant keys for current context
- [ ] Different hints for different views

### 5.4 Documentation & Verification
- [ ] Clippy passes

---

## Task 6: Create `src/ui/panels/blocks.rs`

### Status: NOT_STARTED

### 6.1 Extract Block Panel Rendering
- [ ] Locate block list panel code in ui.rs
- [ ] Extract rendering logic

### 6.2 Implement Block Panel
```rust
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    // Panel title/border
    // Block list widget
    // Scrollbar
    // Empty state
}
```

### 6.3 Use Widgets
- [ ] Import `crate::widgets::BlockListWidget`
- [ ] Import `crate::widgets::ListState`
- [ ] Use theme styles for borders

### 6.4 Documentation & Verification
- [ ] Clippy passes

---

## Task 7: Create `src/ui/panels/transactions.rs`

### Status: NOT_STARTED

### 7.1 Extract Transaction Panel Rendering
- [ ] Locate transaction list panel code in ui.rs

### 7.2 Implement Transaction Panel
```rust
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    // Panel title/border
    // Transaction list widget
    // Scrollbar
    // Empty state
}
```

### 7.3 Use Widgets
- [ ] Import `crate::widgets::TransactionListWidget`
- [ ] Use theme styles

### 7.4 Documentation & Verification
- [ ] Clippy passes

---

## Task 8: Create Detail Panel Files

### Status: NOT_STARTED

### 8.1 Create `src/ui/panels/details/block.rs`
- [ ] Extract block detail rendering from ui.rs
- [ ] Implement `pub fn render(frame: &mut Frame, area: Rect, block: &BlockDetails, app: &App)`
- [ ] Handle tabs (Info, Transactions)
- [ ] Use widgets for display

### 8.2 Create `src/ui/panels/details/transaction.rs`
- [ ] Extract transaction detail rendering
- [ ] Implement `pub fn render(frame: &mut Frame, area: Rect, txn: &Transaction, app: &App)`
- [ ] Use TxnVisualCard widget
- [ ] Use TxnFlowDiagram widget
- [ ] Handle different transaction types

### 8.3 Create `src/ui/panels/details/account.rs`
- [ ] Extract account detail rendering
- [ ] Implement `pub fn render(frame: &mut Frame, area: Rect, account: &AccountDetails, app: &App)`
- [ ] Show balance, assets, apps

### 8.4 Create `src/ui/panels/details/asset.rs`
- [ ] Extract asset detail rendering
- [ ] Implement `pub fn render(frame: &mut Frame, area: Rect, asset: &AssetDetails, app: &App)`
- [ ] Show asset parameters

### 8.5 Create `src/ui/panels/details/mod.rs`
```rust
mod account;
mod asset;
mod block;
mod transaction;

pub use account::render as render_account;
pub use asset::render as render_asset;
pub use block::render as render_block;
pub use transaction::render as render_transaction;

/// Dispatch to appropriate detail renderer
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    match app.current_detail_view() {
        DetailView::Block(block) => block::render(frame, area, block, app),
        DetailView::Transaction(txn) => transaction::render(frame, area, txn, app),
        DetailView::Account(acc) => account::render(frame, area, acc, app),
        DetailView::Asset(asset) => asset::render(frame, area, asset, app),
        DetailView::None => {}
    }
}
```

---

## Task 9: Create `src/ui/panels/mod.rs`

### Status: NOT_STARTED

```rust
pub mod blocks;
pub mod details;
pub mod transactions;

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::state::App;
use super::layout::{calculate_main_layout, MainLayout};

/// Render the main panel area
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let layout = calculate_main_layout(area, app);
    
    if app.is_showing_details() {
        details::render(frame, area, app);
    } else {
        blocks::render(frame, layout.left_panel, app);
        transactions::render(frame, layout.right_panel, app);
    }
}
```

---

## Task 10: Create `src/ui/mod.rs` (Partial)

### Status: NOT_STARTED

### 10.1 Module Structure
```rust
//! UI rendering for LazyLora TUI
//!
//! This module handles all terminal rendering using ratatui.

mod footer;
mod header;
pub mod layout;
pub mod panels;
// pub mod popups;     // Worker B
// pub mod components; // Worker B

use ratatui::Frame;
use crate::state::App;

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    let layout = layout::calculate_app_layout(frame.area());
    
    header::render(frame, layout.header, app);
    panels::render(frame, layout.main, app);
    footer::render(frame, layout.footer, app);
    
    // Popups rendered on top (Worker B will add)
    // popups::render_active(frame, app);
}
```

### 10.2 Note for Worker B
- [ ] Leave comment for popup integration point
- [ ] Worker B will add popups module import

---

## Task 11: Write Tests

### Status: NOT_STARTED

### 11.1 Layout Tests
- [ ] Test AppLayout calculations
- [ ] Test centered_rect helper
- [ ] Test edge cases (small terminals)

### 11.2 Render Tests (if possible)
- [ ] Test render functions don't panic
- [ ] Test with mock App state

---

## Task 12: Final Checklist

### Status: NOT_STARTED

- [ ] All 12 files created
- [ ] `cargo build` succeeds
- [ ] `cargo test --all-features` passes
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] No modifications to `ui.rs`
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

**Layout system:**
- 

**Blocked issues:**
- 

**Notes for coordinator:**
- Worker B creates ui/popups/ and ui/components/ in parallel
- Final ui/mod.rs needs to import both workers' modules
- Main render() function dispatches to all submodules
