# Coordinator - Stage 2 Sync

## Task Overview
- **Role**: Coordinator
- **Stage**: 2 (Sync Point)
- **Duration**: 1 day
- **Risk Level**: High
- **Status**: NOT_STARTED
- **Depends On**: All Stage 1.5 workers complete

## Prerequisites
- [ ] Worker A Stage 1.5 complete (widgets/list, widgets/common)
- [ ] Worker B Stage 1.5 complete (widgets/graph, widgets/detail)
- [ ] Worker C Stage 1.5 complete (state/)
- [ ] All three branches pass CI

## Deliverables
- [ ] Merged `main` branch with all Stage 1.5 work
- [ ] `widgets.rs` deleted, replaced by `widgets/` module
- [ ] `app_state.rs` deleted, replaced by `state/` module
- [ ] All imports updated
- [ ] All tests passing
- [ ] Application runs correctly

---

## Task 1: Merge Stage 1.5 Branches

### Status: NOT_STARTED

### 1.1 Review PRs
- [ ] Review Worker A PR (widgets/list + common)
- [ ] Review Worker B PR (widgets/graph + detail)
- [ ] Review Worker C PR (state/)
- [ ] Verify no conflicting changes

### 1.2 Merge Order
- [ ] Merge Worker C (state/) first - independent module
- [ ] Merge Worker A (widgets/list + common) second
- [ ] Merge Worker B (widgets/graph + detail) third - same parent dir as A

### 1.3 Resolve Widget Module Conflicts
- [ ] Workers A and B both create widgets/ subdirectories
- [ ] Merge widgets/mod.rs from both (should be additive)

---

## Task 2: Create Unified `src/widgets/mod.rs`

### Status: NOT_STARTED

### 2.1 Combine Worker A and B Modules
```rust
//! Widget components for the LazyLora TUI
//!
//! This module provides reusable UI widgets for displaying
//! blockchain data in the terminal.

// Common display widgets (Worker A)
pub mod common;

// List widgets (Worker A)
pub mod list;

// Helper functions (Worker A)
pub mod helpers;

// Graph visualization (Worker B)
pub mod graph;

// Detail view widgets (Worker B)
pub mod detail;

// Re-export commonly used items
pub use common::{AddressDisplay, AmountDisplay, TxnTypeBadge};
pub use detail::{TxnFlowDiagram, TxnVisualCard};
pub use graph::{SvgExporter, TxnGraph, TxnGraphWidget};
pub use helpers::*;
pub use list::{BlockListWidget, ListState, TransactionListWidget};
```

### 2.2 Verification
- [ ] All submodules compile together
- [ ] No naming conflicts
- [ ] All re-exports work

---

## Task 3: Delete `src/widgets.rs`

### Status: NOT_STARTED

### 3.1 Backup (Optional)
- [ ] Git history preserves the file
- [ ] No backup needed if confident

### 3.2 Delete File
- [ ] Remove `src/widgets.rs`
- [ ] Verify `src/widgets/mod.rs` takes over

### 3.3 Update References
- [ ] `ui.rs` imports from `crate::widgets::*` - should still work
- [ ] `app_state.rs` imports - should still work
- [ ] Any other files using widgets

---

## Task 4: Create Unified `src/state/mod.rs`

### Status: NOT_STARTED

### 4.1 Finalize State Module
```rust
//! Application state management
//!
//! This module contains the application state and command handling logic.

mod command_handler;
mod config;
mod data;
mod navigation;
pub mod platform;
mod ui_state;

pub use command_handler::CommandHandler;
pub use config::AppConfig;
pub use data::DataState;
pub use navigation::NavigationState;
pub use ui_state::{DetailViewMode, Focus, PopupState, UiState};

use crate::client::AlgoClient;
use crate::domain::Network;
use tokio::sync::{mpsc, watch};

/// Main application state
pub struct App {
    // Sub-states
    pub nav: NavigationState,
    pub data: DataState,
    pub ui: UiState,
    
    // Configuration
    pub config: AppConfig,
    
    // Runtime state
    pub network: Network,
    pub show_live: bool,
    pub exit: bool,
    pub animation_tick: u64,
    
    // Channels
    message_tx: mpsc::UnboundedSender<AppMessage>,
    message_rx: mpsc::UnboundedReceiver<AppMessage>,
    live_updates_tx: watch::Sender<bool>,
    network_tx: watch::Sender<Network>,
    
    // Client
    client: AlgoClient,
}

// Re-export App message types
pub use crate::app_state::AppMessage; // Temporary - migrate this too
```

### 4.2 Migrate App Implementation
- [ ] Move `App::new()` to state module
- [ ] Move `App::run()` to state module
- [ ] Move message processing to state module
- [ ] Use CommandHandler trait for command execution

---

## Task 5: Delete `src/app_state.rs`

### Status: NOT_STARTED

### 5.1 Verify All Code Migrated
- [ ] All structs moved to state/ submodules
- [ ] All methods moved
- [ ] AppMessage enum moved

### 5.2 Delete File
- [ ] Remove `src/app_state.rs`
- [ ] Verify `src/state/mod.rs` takes over

### 5.3 Update References
- [ ] `main.rs` - update imports
- [ ] `ui.rs` - update App imports
- [ ] `commands.rs` - update if needed

---

## Task 6: Update `src/main.rs`

### Status: NOT_STARTED

### 6.1 Update Module Declarations
```rust
mod algorand;   // Facade
mod boot_screen;
mod client;
mod commands;
mod constants;
mod domain;
mod state;      // NEW - replaces app_state
mod theme;
mod tui;
mod ui;
mod updater;
mod widgets;    // Now a directory module
```

### 6.2 Update Imports
```rust
use state::App;  // Was: use app_state::App;
```

### 6.3 Verification
- [ ] main.rs compiles
- [ ] Application starts

---

## Task 7: Update `src/ui.rs`

### Status: NOT_STARTED

### 7.1 Update Imports
```rust
use crate::state::{App, Focus, PopupState, ...};
use crate::widgets::{...};
```

### 7.2 Verify Rendering Still Works
- [ ] All render functions compile
- [ ] Widget types found
- [ ] State types found

---

## Task 8: Update `src/commands.rs`

### Status: NOT_STARTED

### 8.1 Check Imports
- [ ] Update any state type imports
- [ ] Verify InputContext still works

### 8.2 Verification
- [ ] Commands compile
- [ ] Key mapping works

---

## Task 9: Run Full Test Suite

### Status: NOT_STARTED

### 9.1 Unit Tests
- [ ] `cargo test --all-features`
- [ ] All widget tests pass
- [ ] All state tests pass
- [ ] All existing tests pass

### 9.2 Clippy
- [ ] `cargo clippy --all-features -- -D warnings`
- [ ] No new warnings

### 9.3 Formatting
- [ ] `cargo fmt -- --check`

### 9.4 Build
- [ ] `cargo build`
- [ ] `cargo build --release`

---

## Task 10: Manual Testing

### Status: NOT_STARTED

### 10.1 Basic Functionality
- [ ] Application starts
- [ ] Block list renders
- [ ] Transaction list renders
- [ ] Navigation works

### 10.2 Widget Functionality
- [ ] Block list selection
- [ ] Transaction list selection
- [ ] Scrolling works
- [ ] Type badges display

### 10.3 Detail Views
- [ ] Block details render
- [ ] Transaction details render
- [ ] Flow diagram renders
- [ ] Visual card renders

### 10.4 Graph Functionality
- [ ] Transaction graph renders
- [ ] Graph navigation works
- [ ] SVG export works

### 10.5 State Functionality
- [ ] Network switching works
- [ ] Search works
- [ ] Clipboard copy works
- [ ] Config persistence works

---

## Task 11: Create Stage 2.5 Branches

### Status: NOT_STARTED

### 11.1 Notify Workers
- [ ] Notify Worker A: Create `refactor/stage2.5-worker-a-ui-panels`
- [ ] Notify Worker B: Create `refactor/stage2.5-worker-b-ui-popups`
- [ ] Notify Worker C: Available for code review support

### 11.2 Document Starting Point
- [ ] Record commit hash of merged main
- [ ] Update REFACTORING_PLAN.md with Stage 1.5 completion

---

## Task 12: Final Checklist

### Status: NOT_STARTED

- [ ] All Stage 1.5 PRs merged
- [ ] widgets.rs deleted, widgets/ module works
- [ ] app_state.rs deleted, state/ module works
- [ ] main.rs updated
- [ ] ui.rs updated
- [ ] All tests pass
- [ ] Application runs correctly
- [ ] No regressions
- [ ] Workers notified to start Stage 2.5

---

## Progress Log

| Date | Task | Notes |
|------|------|-------|
| | | |

---

## Issues Encountered

| Issue | Resolution |
|-------|------------|
| | |

---

## Stage 1.5 Summary

**Worker A deliverables (widgets/list + common):**
- [ ] widgets/mod.rs (partial)
- [ ] widgets/helpers.rs
- [ ] widgets/common/mod.rs
- [ ] widgets/common/badge.rs
- [ ] widgets/common/amount.rs
- [ ] widgets/common/address.rs
- [ ] widgets/list/mod.rs
- [ ] widgets/list/state.rs
- [ ] widgets/list/block_list.rs
- [ ] widgets/list/txn_list.rs
- Total: ~700 lines

**Worker B deliverables (widgets/graph + detail):**
- [ ] widgets/graph/mod.rs
- [ ] widgets/graph/types.rs
- [ ] widgets/graph/txn_graph.rs
- [ ] widgets/graph/renderer.rs
- [ ] widgets/graph/svg_export.rs
- [ ] widgets/detail/mod.rs
- [ ] widgets/detail/flow_diagram.rs
- [ ] widgets/detail/visual_card.rs
- Total: ~1200 lines

**Worker C deliverables (state/):**
- [ ] state/mod.rs
- [ ] state/navigation.rs
- [ ] state/data.rs
- [ ] state/ui_state.rs
- [ ] state/config.rs
- [ ] state/command_handler.rs
- [ ] state/platform/mod.rs
- [ ] state/platform/clipboard.rs
- [ ] state/platform/paths.rs
- Total: ~1330 lines

**Files deleted:**
- [ ] widgets.rs (4112 lines)
- [ ] app_state.rs (2047 lines)

**Net change:** -6159 old lines + ~3230 new lines = ~2929 lines removed (through deduplication and better organization)
