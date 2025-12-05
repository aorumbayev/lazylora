# LazyLora Refactoring Plan

## Executive Summary

This document outlines a comprehensive refactoring plan for the LazyLora TUI application. The goal is to improve maintainability, testability, and extensibility while preserving all existing functionality.

### Current State Metrics
| File | Lines | Primary Issues |
|------|-------|----------------|
| `widgets.rs` | 4,112 | Mixed concerns (data + rendering + SVG export), multiple widget types |
| `algorand.rs` | 2,783 | Duplicated HTTP handling, mixed API + domain types |
| `ui.rs` | 2,455 | Scattered constants, monolithic render functions |
| `app_state.rs` | 2,054 | Large command handler, embedded platform code |
| `commands.rs` | 827 | Well-structured (keep as-is with minor improvements) |

**Total: ~12,231 lines** across 5 major modules

---

# PARALLEL EXECUTION PLAN

## Visual Timeline (Gantt Chart)

```mermaid
gantt
    title LazyLora Refactoring - Parallel Execution Plan
    dateFormat  YYYY-MM-DD
    
    section Stage 0 - Foundation
    Worker A: Theme + Constants       :a0, 2025-01-01, 2d
    Worker B: Domain Types            :b0, 2025-01-01, 2d
    Worker C: HTTP Client             :c0, 2025-01-01, 2d
    
    section Stage 1 - Sync
    Coordinator: Merge & Update       :crit, s1, after a0 b0 c0, 1d
    
    section Stage 1.5 - Core Split
    Worker A: Widgets List + Common   :a1, after s1, 3d
    Worker B: Widgets Graph + Detail  :b1, after s1, 3d
    Worker C: State Module            :c1, after s1, 3d
    
    section Stage 2 - Sync
    Coordinator: Merge & Cleanup      :crit, s2, after a1 b1 c1, 1d
    
    section Stage 2.5 - UI Split
    Worker A: UI Panels + Layout      :a2, after s2, 3d
    Worker B: UI Popups + Components  :b2, after s2, 3d
    Worker C: Code Review Support     :c2, after s2, 3d
    
    section Stage 3 - Final
    All: Integration & Testing        :crit, s3, after a2 b2, 2d
```

## Dependency Graph

```
                    ┌─────────────────────────────────────────────────────────────┐
                    │                      STAGE 0 (Foundation)                    │
                    │                         ~2 days                              │
                    └─────────────────────────────────────────────────────────────┘
                                               │
                    ┌──────────────────────────┼──────────────────────────┐
                    │                          │                          │
                    ▼                          ▼                          ▼
         ┌──────────────────┐      ┌──────────────────┐      ┌──────────────────┐
         │   Worker A       │      │   Worker B       │      │   Worker C       │
         │   Theme +        │      │   Domain Types   │      │   HTTP Client    │
         │   Constants      │      │   (from algo.rs) │      │   (from algo.rs) │
         │   ~200 lines     │      │   ~990 lines     │      │   ~800 lines     │
         └────────┬─────────┘      └────────┬─────────┘      └────────┬─────────┘
                  │                         │                          │
                  │                         │                          │
                    ┌─────────────────────────────────────────────────────────────┐
                    │                      STAGE 1 (Sync Point)                    │
                    │           Merge & Verify: algorand.rs fully split            │
                    │                         ~1 day                               │
                    └─────────────────────────────────────────────────────────────┘
                                               │
                    ┌──────────────────────────┼──────────────────────────┐
                    │                          │                          │
                    ▼                          ▼                          ▼
         ┌──────────────────┐      ┌──────────────────┐      ┌──────────────────┐
         │   Worker A       │      │   Worker B       │      │   Worker C       │
         │   Widgets:       │      │   Widgets:       │      │   State Module   │
         │   List + Common  │      │   Graph + Detail │      │   (app_state.rs) │
         │   ~700 lines     │      │   ~1200 lines    │      │   ~1330 lines    │
         └────────┬─────────┘      └────────┬─────────┘      └────────┬─────────┘
                  │                         │                          │
                    ┌─────────────────────────────────────────────────────────────┐
                    │                      STAGE 2 (Sync Point)                    │
                    │      Merge: widgets/ complete, state/ complete               │
                    │                         ~1 day                               │
                    └─────────────────────────────────────────────────────────────┘
                                               │
                    ┌──────────────────────────┴──────────────────────────┐
                    │                                                     │
                    ▼                                                     ▼
         ┌──────────────────┐                              ┌──────────────────┐
         │   Worker A       │                              │   Worker B       │
         │   UI: Panels +   │                              │   UI: Popups +   │
         │   Layout         │                              │   Components     │
         │   ~1500 lines    │                              │   ~1080 lines    │
         └────────┬─────────┘                              └────────┬─────────┘
                  │                                                  │
                    ┌─────────────────────────────────────────────────────────────┐
                    │                      STAGE 3 (Final)                         │
                    │              Integration, Testing, Cleanup                   │
                    │                         ~2 days                              │
                    └─────────────────────────────────────────────────────────────┘
```

## Stage Details

### STAGE 0: Foundation Layer (Parallel - 3 Workers)

All three streams work on NEW files only. No conflicts possible.

---

#### 🔵 Worker A: Theme & Constants
**Duration**: 2 days  
**Risk**: Low  
**Creates**:
- `src/theme.rs` (~150 lines)
- `src/constants.rs` (~50 lines)

**Modifies**: NOTHING (yet)  
**DO NOT Touch**: Any existing files  

**Task**: Extract color definitions and constants from ui.rs and widgets.rs into new standalone modules. Do NOT update imports in existing files yet.

**Deliverables**:
- [ ] `theme.rs` with colors, styles, layout modules
- [ ] `constants.rs` with api, app, pagination, format modules
- [ ] Unit tests for style helpers

---

#### 🟢 Worker B: Domain Types
**Duration**: 2 days  
**Risk**: Low-Medium  
**Creates**:
- `src/domain/mod.rs` (~30 lines)
- `src/domain/transaction.rs` (~400 lines)
- `src/domain/block.rs` (~100 lines)
- `src/domain/account.rs` (~150 lines)
- `src/domain/asset.rs` (~100 lines)
- `src/domain/network.rs` (~80 lines)
- `src/domain/nfd.rs` (~50 lines)
- `src/domain/error.rs` (~80 lines)

**Modifies**: NOTHING (yet)  
**DO NOT Touch**: `algorand.rs`, any other existing files  

**Task**: Copy and reorganize type definitions from algorand.rs (lines 14-1000) into domain module. Keep algorand.rs untouched - we'll update imports later.

**Deliverables**:
- [ ] Complete domain module with all types
- [ ] All types compile independently
- [ ] Serialization tests pass

---

#### 🟠 Worker C: HTTP Client Abstraction
**Duration**: 2 days  
**Risk**: Medium  
**Creates**:
- `src/client/mod.rs` (~50 lines)
- `src/client/http.rs` (~200 lines)
- `src/client/indexer.rs` (~300 lines)
- `src/client/node.rs` (~150 lines)
- `src/client/nfd.rs` (~100 lines)

**Modifies**: NOTHING (yet)  
**DO NOT Touch**: `algorand.rs`, any other existing files  

**Task**: Create new HTTP client module with retry logic, extracting patterns from algorand.rs (lines 1000-2783). Reference domain types from original algorand.rs for now.

**Deliverables**:
- [ ] Generic HTTP client with retry/timeout
- [ ] Indexer, Node, NFD clients
- [ ] Integration tests with mocked responses

---

### STAGE 1: Sync Point (1 day)

**Coordinator Task**: Merge all Stage 0 branches and update imports.

**Actions**:
1. Merge Worker A (theme), B (domain), C (client) branches
2. Update `algorand.rs` to re-export from domain/ and client/ (facade pattern)
3. Update `main.rs` to import new modules
4. Run full test suite
5. Verify application still works

**Result**: `algorand.rs` becomes a thin facade, all new modules integrated.

---

### STAGE 1.5: Widget & State Split (Parallel - 3 Workers)

After Stage 1 sync, widgets.rs and app_state.rs can be split in parallel.

---

#### 🔵 Worker A: Widgets - Lists & Common
**Duration**: 3 days  
**Risk**: Medium  
**Creates**:
- `src/widgets/mod.rs` (~80 lines)
- `src/widgets/helpers.rs` (~100 lines)
- `src/widgets/common/mod.rs` (~20 lines)
- `src/widgets/common/badge.rs` (~100 lines)
- `src/widgets/common/amount.rs` (~80 lines)
- `src/widgets/common/address.rs` (~80 lines)
- `src/widgets/list/mod.rs` (~50 lines)
- `src/widgets/list/state.rs` (~150 lines) - Generic ListState<T>
- `src/widgets/list/block_list.rs` (~200 lines)
- `src/widgets/list/txn_list.rs` (~250 lines)

**Modifies**: NOTHING  
**DO NOT Touch**: `widgets.rs` (Worker B handles graph/detail), `ui.rs`, `app_state.rs`  

**Task**: Extract list widgets and common display widgets. Create generic ListState<T> to unify BlockListState and TransactionListState.

**Deliverables**:
- [ ] Generic ListState<T> with full navigation
- [ ] Block and Transaction list widgets
- [ ] Badge, Amount, Address display widgets
- [ ] Widget unit tests

---

#### 🟢 Worker B: Widgets - Graph & Detail
**Duration**: 3 days  
**Risk**: Medium  
**Creates**:
- `src/widgets/graph/mod.rs` (~30 lines)
- `src/widgets/graph/types.rs` (~150 lines)
- `src/widgets/graph/txn_graph.rs` (~350 lines)
- `src/widgets/graph/renderer.rs` (~300 lines)
- `src/widgets/graph/svg_export.rs` (~450 lines)
- `src/widgets/detail/mod.rs` (~30 lines)
- `src/widgets/detail/flow_diagram.rs` (~150 lines)
- `src/widgets/detail/visual_card.rs` (~200 lines)

**Modifies**: NOTHING  
**DO NOT Touch**: `widgets.rs` (will be deleted after merge), `ui.rs`, `app_state.rs`  

**Task**: Extract graph visualization and detail widgets. Separate SVG export from graph data structure.

**Deliverables**:
- [ ] TxnGraph data structure (pure data)
- [ ] TxnGraphWidget (ASCII renderer)
- [ ] SvgExporter (separate concern)
- [ ] Flow diagram and visual card widgets
- [ ] SVG export tests

---

#### 🟠 Worker C: State Module
**Duration**: 3 days  
**Risk**: High  
**Creates**:
- `src/state/mod.rs` (~100 lines)
- `src/state/navigation.rs` (~150 lines)
- `src/state/data.rs` (~200 lines)
- `src/state/ui_state.rs` (~150 lines)
- `src/state/config.rs` (~150 lines)
- `src/state/command_handler.rs` (~400 lines)
- `src/state/platform/mod.rs` (~30 lines)
- `src/state/platform/clipboard.rs` (~100 lines)
- `src/state/platform/paths.rs` (~50 lines)

**Modifies**: NOTHING  
**DO NOT Touch**: `app_state.rs` (will be replaced), `ui.rs`, `commands.rs`  

**Task**: Create new state module structure. Extract platform-specific clipboard code. Decompose command handler into logical groups.

**Deliverables**:
- [ ] Modular state components
- [ ] Platform clipboard abstraction
- [ ] Decomposed command handler
- [ ] Config persistence module
- [ ] State transition tests

---

### STAGE 2: Sync Point (1 day)

**Coordinator Task**: Merge widget and state modules, replace old files.

**Actions**:
1. Merge Worker A (widgets/list), B (widgets/graph), C (state) branches
2. Delete `widgets.rs`, replace with `src/widgets/mod.rs` facade
3. Delete `app_state.rs`, replace with `src/state/mod.rs` facade
4. Update all imports in `ui.rs`, `main.rs`, `commands.rs`
5. Run full test suite
6. Verify application still works

**Result**: widgets.rs and app_state.rs replaced with modular structures.

---

### STAGE 2.5: UI Module Split (Parallel - 2 Workers)

After Stage 2 sync, ui.rs can be split. Only 2 workers needed.

---

#### 🔵 Worker A: UI - Panels & Layout
**Duration**: 3 days  
**Risk**: Medium-High  
**Creates**:
- `src/ui/mod.rs` (~100 lines)
- `src/ui/layout.rs` (~150 lines)
- `src/ui/header.rs` (~100 lines)
- `src/ui/footer.rs` (~80 lines)
- `src/ui/panels/mod.rs` (~30 lines)
- `src/ui/panels/blocks.rs` (~200 lines)
- `src/ui/panels/transactions.rs` (~250 lines)
- `src/ui/panels/details/mod.rs` (~50 lines)
- `src/ui/panels/details/block.rs` (~200 lines)
- `src/ui/panels/details/transaction.rs` (~300 lines)
- `src/ui/panels/details/account.rs` (~200 lines)
- `src/ui/panels/details/asset.rs` (~150 lines)

**Modifies**: NOTHING  
**DO NOT Touch**: `ui.rs` (Worker B handles popups)  

**Task**: Extract main layout, header/footer, and all panel rendering code.

**Deliverables**:
- [ ] Layout calculation module
- [ ] Header and footer renderers
- [ ] All detail panel renderers
- [ ] Panel rendering tests

---

#### 🟢 Worker B: UI - Popups & Components
**Duration**: 3 days  
**Risk**: Medium  
**Creates**:
- `src/ui/popups/mod.rs` (~50 lines)
- `src/ui/popups/search.rs` (~200 lines)
- `src/ui/popups/network.rs` (~150 lines)
- `src/ui/popups/help.rs` (~100 lines)
- `src/ui/popups/error.rs` (~80 lines)
- `src/ui/components/mod.rs` (~30 lines)
- `src/ui/components/scrollbar.rs` (~80 lines)
- `src/ui/components/tabs.rs` (~100 lines)

**Modifies**: NOTHING  
**DO NOT Touch**: `ui.rs` (will be deleted after merge)  

**Task**: Extract all popup rendering and reusable UI components.

**Deliverables**:
- [ ] Popup trait and implementations
- [ ] Search, network, help, error popups
- [ ] Scrollbar and tabs components
- [ ] Popup behavior tests

---

### STAGE 3: Final Integration (2 days)

**All Workers Together**:

1. Merge UI branches
2. Delete `ui.rs`, replace with `src/ui/mod.rs`
3. Update `main.rs` for final structure
4. Delete `algorand.rs` (now fully replaced by domain/ and client/)
5. Full integration testing
6. Performance benchmarking
7. Documentation update

---

## Worker Assignment Summary

| Worker | Stage 0 | Stage 1.5 | Stage 2.5 | Total Files Created |
|--------|---------|-----------|-----------|---------------------|
| **A** | Theme + Constants | Widgets: List + Common | UI: Panels + Layout | 24 files |
| **B** | Domain Types | Widgets: Graph + Detail | UI: Popups + Components | 23 files |
| **C** | HTTP Client | State Module | (Available for review) | 14 files |

## Timeline

| Stage | Duration | Workers | Deliverable |
|-------|----------|---------|-------------|
| Stage 0 | 2 days | 3 parallel | theme.rs, constants.rs, domain/, client/ |
| Stage 1 Sync | 1 day | Coordinator | algorand.rs becomes facade |
| Stage 1.5 | 3 days | 3 parallel | widgets/, state/ |
| Stage 2 Sync | 1 day | Coordinator | widgets.rs, app_state.rs replaced |
| Stage 2.5 | 3 days | 2 parallel | ui/ |
| Stage 3 | 2 days | All | Final integration |
| **Total** | **12 days** | | |

## File Ownership Rules

To prevent merge conflicts, each file is owned by exactly one worker at a time:

### Stage 0
| File/Directory | Owner | Status |
|----------------|-------|--------|
| `src/theme.rs` | Worker A | CREATE |
| `src/constants.rs` | Worker A | CREATE |
| `src/domain/*` | Worker B | CREATE |
| `src/client/*` | Worker C | CREATE |
| All existing files | LOCKED | NO TOUCH |

### Stage 1.5
| File/Directory | Owner | Status |
|----------------|-------|--------|
| `src/widgets/common/*` | Worker A | CREATE |
| `src/widgets/list/*` | Worker A | CREATE |
| `src/widgets/helpers.rs` | Worker A | CREATE |
| `src/widgets/graph/*` | Worker B | CREATE |
| `src/widgets/detail/*` | Worker B | CREATE |
| `src/state/*` | Worker C | CREATE |
| `widgets.rs` | LOCKED | DELETE @ Stage 2 |
| `app_state.rs` | LOCKED | DELETE @ Stage 2 |
| `ui.rs` | LOCKED | NO TOUCH |

### Stage 2.5
| File/Directory | Owner | Status |
|----------------|-------|--------|
| `src/ui/layout.rs` | Worker A | CREATE |
| `src/ui/header.rs` | Worker A | CREATE |
| `src/ui/footer.rs` | Worker A | CREATE |
| `src/ui/panels/*` | Worker A | CREATE |
| `src/ui/popups/*` | Worker B | CREATE |
| `src/ui/components/*` | Worker B | CREATE |
| `ui.rs` | LOCKED | DELETE @ Stage 3 |

---

## Conflict Prevention Checklist

- [ ] Each worker creates only NEW files in their assigned directories
- [ ] No worker modifies existing files until sync points
- [ ] Coordinator handles all import updates at sync points
- [ ] Each sync point includes full test suite run
- [ ] Workers use temporary re-exports from old modules during development

---

## Phase 1: Extract Constants & Theme System (Low Risk)

### 1.1 Create `src/theme.rs` (~150 lines)

Extract all scattered color definitions and create a centralized theme system.

```rust
// src/theme.rs

use ratatui::style::{Color, Modifier, Style};

/// Application color palette
pub mod colors {
    use super::*;
    
    // Brand colors
    pub const PRIMARY: Color = Color::Rgb(255, 165, 2);      // Orange
    pub const SECONDARY: Color = Color::Rgb(100, 100, 100);  // Gray
    
    // Semantic colors
    pub const SUCCESS: Color = Color::Rgb(0, 255, 0);
    pub const ERROR: Color = Color::Rgb(255, 0, 0);
    pub const WARNING: Color = Color::Rgb(255, 255, 0);
    pub const INFO: Color = Color::Rgb(0, 150, 255);
    
    // Transaction type colors
    pub const TXN_PAYMENT: Color = Color::Rgb(0, 150, 255);
    pub const TXN_ASSET_TRANSFER: Color = Color::Rgb(0, 200, 100);
    pub const TXN_APP_CALL: Color = Color::Rgb(200, 100, 255);
    pub const TXN_ASSET_CONFIG: Color = Color::Rgb(255, 200, 0);
    pub const TXN_ASSET_FREEZE: Color = Color::Rgb(0, 200, 200);
    pub const TXN_KEY_REG: Color = Color::Rgb(255, 100, 100);
    pub const TXN_STATE_PROOF: Color = Color::Rgb(150, 150, 150);
    
    // UI element colors
    pub const BORDER_ACTIVE: Color = PRIMARY;
    pub const BORDER_INACTIVE: Color = Color::DarkGray;
    pub const TEXT_PRIMARY: Color = Color::White;
    pub const TEXT_SECONDARY: Color = Color::Gray;
    pub const TEXT_MUTED: Color = Color::DarkGray;
    pub const BACKGROUND: Color = Color::Reset;
    pub const HIGHLIGHT_BG: Color = Color::Rgb(40, 40, 40);
}

/// Pre-composed styles for common UI elements
pub mod styles {
    use super::*;
    
    pub fn title() -> Style {
        Style::default()
            .fg(colors::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn border_active() -> Style {
        Style::default().fg(colors::BORDER_ACTIVE)
    }
    
    pub fn border_inactive() -> Style {
        Style::default().fg(colors::BORDER_INACTIVE)
    }
    
    pub fn text_primary() -> Style {
        Style::default().fg(colors::TEXT_PRIMARY)
    }
    
    pub fn text_secondary() -> Style {
        Style::default().fg(colors::TEXT_SECONDARY)
    }
    
    pub fn highlight() -> Style {
        Style::default()
            .bg(colors::HIGHLIGHT_BG)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn success() -> Style {
        Style::default().fg(colors::SUCCESS)
    }
    
    pub fn error() -> Style {
        Style::default().fg(colors::ERROR)
    }
}

/// Layout constants
pub mod layout {
    /// Minimum terminal width for full layout
    pub const MIN_WIDTH: u16 = 80;
    /// Minimum terminal height for full layout
    pub const MIN_HEIGHT: u16 = 24;
    /// Standard padding
    pub const PADDING: u16 = 1;
    /// Block list default width percentage
    pub const BLOCK_LIST_WIDTH_PCT: u16 = 30;
    /// Transaction list default width percentage  
    pub const TXN_LIST_WIDTH_PCT: u16 = 70;
    /// Header height
    pub const HEADER_HEIGHT: u16 = 3;
    /// Footer height
    pub const FOOTER_HEIGHT: u16 = 1;
}
```

### 1.2 Create `src/constants.rs` (~50 lines)

Application-wide constants that aren't theme-related.

```rust
// src/constants.rs

/// API and network constants
pub mod api {
    /// Default request timeout in seconds
    pub const REQUEST_TIMEOUT_SECS: u64 = 30;
    /// Maximum retry attempts for failed requests
    pub const MAX_RETRIES: u32 = 3;
    /// Delay between retries in milliseconds
    pub const RETRY_DELAY_MS: u64 = 1000;
}

/// Application metadata
pub mod app {
    pub const NAME: &str = "lazylora";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const CONFIG_FILE: &str = "config.json";
}

/// Pagination defaults
pub mod pagination {
    pub const DEFAULT_PAGE_SIZE: usize = 20;
    pub const MAX_PAGE_SIZE: usize = 100;
}

/// Address formatting
pub mod format {
    pub const ADDRESS_PREFIX_LEN: usize = 6;
    pub const ADDRESS_SUFFIX_LEN: usize = 4;
    pub const TXID_PREFIX_LEN: usize = 8;
    pub const TXID_SUFFIX_LEN: usize = 4;
}
```

---

## Phase 2: Extract Domain Types (Low-Medium Risk)

### 2.1 Create `src/domain/` Module Structure

```
src/domain/
├── mod.rs           (~30 lines)  - Module exports
├── transaction.rs   (~400 lines) - Transaction types
├── block.rs         (~100 lines) - Block types
├── account.rs       (~150 lines) - Account types
├── asset.rs         (~100 lines) - Asset types
├── network.rs       (~80 lines)  - Network configuration
├── nfd.rs           (~50 lines)  - NFD types
└── error.rs         (~80 lines)  - Domain errors
```

**Total: ~990 lines** (extracted from `algorand.rs`)

### 2.2 Domain Module Root

```rust
// src/domain/mod.rs

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

### 2.3 Transaction Types Example

```rust
// src/domain/transaction.rs

use serde::{Deserialize, Serialize};

/// Transaction type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    #[serde(rename = "pay")]
    Payment,
    #[serde(rename = "axfer")]
    AssetTransfer,
    #[serde(rename = "appl")]
    ApplicationCall,
    #[serde(rename = "acfg")]
    AssetConfig,
    #[serde(rename = "afrz")]
    AssetFreeze,
    #[serde(rename = "keyreg")]
    KeyRegistration,
    #[serde(rename = "stpf")]
    StateProof,
}

impl TransactionType {
    /// Returns a human-readable label for the transaction type
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Payment => "Payment",
            Self::AssetTransfer => "Asset Transfer",
            Self::ApplicationCall => "App Call",
            Self::AssetConfig => "Asset Config",
            Self::AssetFreeze => "Asset Freeze",
            Self::KeyRegistration => "Key Reg",
            Self::StateProof => "State Proof",
        }
    }
    
    /// Returns a short code for the transaction type
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Payment => "PAY",
            Self::AssetTransfer => "AXFER",
            Self::ApplicationCall => "APPL",
            Self::AssetConfig => "ACFG",
            Self::AssetFreeze => "AFRZ",
            Self::KeyRegistration => "KEYREG",
            Self::StateProof => "STPF",
        }
    }
}

/// Core transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    #[serde(rename = "tx-type")]
    pub tx_type: TransactionType,
    pub sender: String,
    #[serde(rename = "confirmed-round")]
    pub confirmed_round: Option<u64>,
    #[serde(rename = "round-time")]
    pub round_time: Option<u64>,
    pub fee: u64,
    pub note: Option<String>,
    pub group: Option<String>,
    
    // Type-specific details
    #[serde(rename = "payment-transaction")]
    pub payment: Option<PaymentDetails>,
    #[serde(rename = "asset-transfer-transaction")]
    pub asset_transfer: Option<AssetTransferDetails>,
    #[serde(rename = "application-transaction")]
    pub application: Option<ApplicationDetails>,
    #[serde(rename = "asset-config-transaction")]
    pub asset_config: Option<AssetConfigDetails>,
    #[serde(rename = "asset-freeze-transaction")]
    pub asset_freeze: Option<AssetFreezeDetails>,
    #[serde(rename = "keyreg-transaction")]
    pub key_reg: Option<KeyRegDetails>,
    
    // Inner transactions for app calls
    #[serde(rename = "inner-txns")]
    pub inner_txns: Option<Vec<Transaction>>,
}

// ... PaymentDetails, AssetTransferDetails, etc.
```

---

## Phase 3: Create HTTP Client Abstraction (Medium Risk)

### 3.1 Create `src/client/` Module Structure

```
src/client/
├── mod.rs           (~50 lines)  - Module exports
├── http.rs          (~200 lines) - Generic HTTP client with retry logic
├── indexer.rs       (~300 lines) - Indexer API methods
├── node.rs          (~150 lines) - Node API methods
└── nfd.rs           (~100 lines) - NFD API methods
```

**Total: ~800 lines** (refactored from `algorand.rs`)

### 3.2 HTTP Client Trait

```rust
// src/client/http.rs

use color_eyre::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::constants::api;
use crate::domain::AlgoError;

/// Configuration for HTTP requests
#[derive(Debug, Clone)]
pub struct RequestConfig {
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(api::REQUEST_TIMEOUT_SECS),
            max_retries: api::MAX_RETRIES,
            retry_delay: Duration::from_millis(api::RETRY_DELAY_MS),
        }
    }
}

/// Generic HTTP client with retry logic and error handling
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    config: RequestConfig,
}

impl HttpClient {
    /// Creates a new HTTP client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(RequestConfig::default())
    }
    
    /// Creates a new HTTP client with custom configuration
    pub fn with_config(config: RequestConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()?;
        Ok(Self { client, config })
    }
    
    /// Performs a GET request with automatic retry and JSON deserialization
    ///
    /// # Arguments
    /// * `url` - The URL to request
    ///
    /// # Errors
    /// Returns error if request fails after all retries or response cannot be parsed
    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let mut last_error = None;
        
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                tokio::time::sleep(self.config.retry_delay).await;
            }
            
            match self.execute_get::<T>(url).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            AlgoError::ApiError("Request failed with no error details".into()).into()
        }))
    }
    
    async fn execute_get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AlgoError::ApiError(format!(
                "HTTP {}: {}", status, body
            )).into());
        }
        
        let data = response.json::<T>().await?;
        Ok(data)
    }
}
```

### 3.3 Indexer Client

```rust
// src/client/indexer.rs

use color_eyre::Result;

use super::HttpClient;
use crate::domain::{Account, Asset, Block, Network, Transaction};

/// Client for Algorand Indexer API
pub struct IndexerClient {
    http: HttpClient,
    network: Network,
}

impl IndexerClient {
    /// Creates a new indexer client for the given network
    pub fn new(network: Network) -> Result<Self> {
        Ok(Self {
            http: HttpClient::new()?,
            network,
        })
    }
    
    /// Fetches a block by round number
    ///
    /// # Errors
    /// Returns error if the API request fails
    pub async fn get_block(&self, round: u64) -> Result<Block> {
        let url = format!(
            "{}/v2/blocks/{}",
            self.network.indexer_url(),
            round
        );
        self.http.get_json(&url).await
    }
    
    /// Fetches recent blocks
    ///
    /// # Errors
    /// Returns error if the API request fails
    pub async fn get_recent_blocks(&self, limit: usize) -> Result<Vec<Block>> {
        // Implementation
        todo!()
    }
    
    /// Fetches a transaction by ID
    ///
    /// # Errors
    /// Returns error if the API request fails
    pub async fn get_transaction(&self, txid: &str) -> Result<Transaction> {
        let url = format!(
            "{}/v2/transactions/{}",
            self.network.indexer_url(),
            txid
        );
        self.http.get_json(&url).await
    }
    
    /// Fetches account information
    ///
    /// # Errors
    /// Returns error if the API request fails
    pub async fn get_account(&self, address: &str) -> Result<Account> {
        let url = format!(
            "{}/v2/accounts/{}",
            self.network.indexer_url(),
            address
        );
        self.http.get_json(&url).await
    }
    
    /// Fetches asset information
    ///
    /// # Errors
    /// Returns error if the API request fails
    pub async fn get_asset(&self, asset_id: u64) -> Result<Asset> {
        let url = format!(
            "{}/v2/assets/{}",
            self.network.indexer_url(),
            asset_id
        );
        self.http.get_json(&url).await
    }
    
    // ... additional methods
}
```

---

## Phase 4: Widget Module Decomposition (Medium Risk)

### 4.1 Create `src/widgets/` Module Structure

```
src/widgets/
├── mod.rs              (~80 lines)   - Module exports and shared traits
├── common/
│   ├── mod.rs          (~20 lines)   - Common widget exports
│   ├── badge.rs        (~100 lines)  - TxnTypeBadge
│   ├── amount.rs       (~80 lines)   - AmountDisplay
│   └── address.rs      (~80 lines)   - AddressDisplay
├── list/
│   ├── mod.rs          (~50 lines)   - List widget exports and trait
│   ├── state.rs        (~150 lines)  - Generic StatefulList
│   ├── block_list.rs   (~200 lines)  - BlockListWidget
│   └── txn_list.rs     (~250 lines)  - TransactionListWidget
├── graph/
│   ├── mod.rs          (~30 lines)   - Graph exports
│   ├── txn_graph.rs    (~400 lines)  - TxnGraph data structure
│   ├── renderer.rs     (~300 lines)  - TxnGraphWidget (ASCII)
│   └── svg_export.rs   (~450 lines)  - SVG export functionality
├── detail/
│   ├── mod.rs          (~30 lines)   - Detail widget exports
│   ├── flow_diagram.rs (~150 lines)  - TxnFlowDiagram
│   └── visual_card.rs  (~200 lines)  - TxnVisualCard
└── helpers.rs          (~100 lines)  - Formatting helpers
```

**Total: ~2,670 lines** (reorganized from 4,112 lines - reduction through deduplication)

### 4.2 Generic Stateful List

```rust
// src/widgets/list/state.rs

use std::marker::PhantomData;

/// Generic state for scrollable lists
#[derive(Debug, Clone)]
pub struct ListState<T> {
    items: Vec<T>,
    selected: Option<usize>,
    offset: usize,
    viewport_height: usize,
}

impl<T> Default for ListState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            offset: 0,
            viewport_height: 10,
        }
    }
}

impl<T> ListState<T> {
    /// Creates a new list state with the given items
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
        let selected = if items.is_empty() { None } else { Some(0) };
        Self {
            items,
            selected,
            ..Default::default()
        }
    }
    
    /// Returns the currently selected item, if any
    #[must_use]
    pub fn selected_item(&self) -> Option<&T> {
        self.selected.and_then(|i| self.items.get(i))
    }
    
    /// Returns the current selection index
    #[must_use]
    pub const fn selected_index(&self) -> Option<usize> {
        self.selected
    }
    
    /// Moves selection to the next item
    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = Some(
            self.selected
                .map(|i| (i + 1).min(self.items.len() - 1))
                .unwrap_or(0)
        );
        self.adjust_offset();
    }
    
    /// Moves selection to the previous item
    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = Some(
            self.selected
                .map(|i| i.saturating_sub(1))
                .unwrap_or(0)
        );
        self.adjust_offset();
    }
    
    /// Moves selection down by one page
    pub fn page_down(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = Some(
            self.selected
                .map(|i| (i + self.viewport_height).min(self.items.len() - 1))
                .unwrap_or(0)
        );
        self.adjust_offset();
    }
    
    /// Moves selection up by one page
    pub fn page_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = Some(
            self.selected
                .map(|i| i.saturating_sub(self.viewport_height))
                .unwrap_or(0)
        );
        self.adjust_offset();
    }
    
    /// Selects the first item
    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.selected = Some(0);
            self.offset = 0;
        }
    }
    
    /// Selects the last item
    pub fn select_last(&mut self) {
        if !self.items.is_empty() {
            self.selected = Some(self.items.len() - 1);
            self.adjust_offset();
        }
    }
    
    /// Updates the viewport height for pagination calculations
    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height.max(1);
        self.adjust_offset();
    }
    
    /// Replaces all items in the list
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        if self.items.is_empty() {
            self.selected = None;
        } else {
            self.selected = Some(
                self.selected
                    .map(|i| i.min(self.items.len() - 1))
                    .unwrap_or(0)
            );
        }
        self.adjust_offset();
    }
    
    /// Returns a slice of items visible in the current viewport
    #[must_use]
    pub fn visible_items(&self) -> &[T] {
        let end = (self.offset + self.viewport_height).min(self.items.len());
        &self.items[self.offset..end]
    }
    
    /// Returns all items
    #[must_use]
    pub fn items(&self) -> &[T] {
        &self.items
    }
    
    /// Returns the number of items
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    /// Returns true if the list is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    fn adjust_offset(&mut self) {
        if let Some(selected) = self.selected {
            if selected < self.offset {
                self.offset = selected;
            } else if selected >= self.offset + self.viewport_height {
                self.offset = selected - self.viewport_height + 1;
            }
        }
    }
}

/// Trait for items that can be displayed in a list
pub trait ListItem {
    /// Returns the primary display text
    fn display_text(&self) -> String;
    
    /// Returns optional secondary text
    fn secondary_text(&self) -> Option<String> {
        None
    }
}
```

### 4.3 SVG Export Separation

```rust
// src/widgets/graph/svg_export.rs

use crate::domain::Transaction;
use crate::theme::colors;

/// Configuration for SVG export
#[derive(Debug, Clone)]
pub struct SvgExportConfig {
    pub node_width: f64,
    pub node_height: f64,
    pub horizontal_gap: f64,
    pub vertical_gap: f64,
    pub font_size: f64,
    pub include_css: bool,
}

impl Default for SvgExportConfig {
    fn default() -> Self {
        Self {
            node_width: 280.0,
            node_height: 100.0,
            horizontal_gap: 80.0,
            vertical_gap: 40.0,
            font_size: 12.0,
            include_css: true,
        }
    }
}

/// SVG exporter for transaction graphs
pub struct SvgExporter {
    config: SvgExportConfig,
}

impl SvgExporter {
    /// Creates a new SVG exporter with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(SvgExportConfig::default())
    }
    
    /// Creates a new SVG exporter with custom configuration
    #[must_use]
    pub fn with_config(config: SvgExportConfig) -> Self {
        Self { config }
    }
    
    /// Exports a transaction graph to SVG format
    ///
    /// # Arguments
    /// * `graph` - The transaction graph to export
    ///
    /// # Returns
    /// SVG string representation of the graph
    #[must_use]
    pub fn export(&self, graph: &super::TxnGraph) -> String {
        let mut svg = String::new();
        
        // Calculate dimensions
        let (width, height) = self.calculate_dimensions(graph);
        
        // SVG header
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">"#,
            width, height
        ));
        
        // Styles
        if self.config.include_css {
            svg.push_str(&self.generate_styles());
        }
        
        // Render edges first (so they appear behind nodes)
        svg.push_str(&self.render_edges(graph));
        
        // Render nodes
        svg.push_str(&self.render_nodes(graph));
        
        // Close SVG
        svg.push_str("</svg>");
        
        svg
    }
    
    fn calculate_dimensions(&self, graph: &super::TxnGraph) -> (f64, f64) {
        // Implementation based on graph structure
        todo!()
    }
    
    fn generate_styles(&self) -> String {
        format!(
            r#"<style>
                .node {{ fill: #1a1a2e; stroke: #ffa502; stroke-width: 2; }}
                .node-text {{ fill: white; font-family: monospace; font-size: {}px; }}
                .edge {{ stroke: #666; stroke-width: 2; fill: none; }}
                .edge-arrow {{ fill: #666; }}
            </style>"#,
            self.config.font_size
        )
    }
    
    fn render_edges(&self, graph: &super::TxnGraph) -> String {
        // Implementation
        todo!()
    }
    
    fn render_nodes(&self, graph: &super::TxnGraph) -> String {
        // Implementation
        todo!()
    }
}

impl Default for SvgExporter {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Phase 5: UI Module Decomposition (Medium-High Risk)

### 5.1 Create `src/ui/` Module Structure

```
src/ui/
├── mod.rs              (~100 lines)  - Main render dispatcher
├── layout.rs           (~150 lines)  - Layout calculations
├── header.rs           (~100 lines)  - Header rendering
├── footer.rs           (~80 lines)   - Footer/status bar
├── panels/
│   ├── mod.rs          (~30 lines)   - Panel exports
│   ├── blocks.rs       (~200 lines)  - Block list panel
│   ├── transactions.rs (~250 lines)  - Transaction list panel
│   ├── details/
│   │   ├── mod.rs      (~50 lines)   - Details panel router
│   │   ├── block.rs    (~200 lines)  - Block details
│   │   ├── transaction.rs (~300 lines) - Transaction details
│   │   ├── account.rs  (~200 lines)  - Account details
│   │   └── asset.rs    (~150 lines)  - Asset details
├── popups/
│   ├── mod.rs          (~50 lines)   - Popup exports and trait
│   ├── search.rs       (~200 lines)  - Search popup
│   ├── network.rs      (~150 lines)  - Network selector
│   ├── help.rs         (~100 lines)  - Help overlay
│   └── error.rs        (~80 lines)   - Error display
└── components/
    ├── mod.rs          (~30 lines)   - Component exports
    ├── scrollbar.rs    (~80 lines)   - Scrollbar component
    └── tabs.rs         (~100 lines)  - Tab bar component
```

**Total: ~2,580 lines** (reorganized from 2,455 lines)

### 5.2 Render Trait Pattern

```rust
// src/ui/mod.rs

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app_state::App;

mod components;
mod footer;
mod header;
mod layout;
mod panels;
mod popups;

pub use layout::calculate_layout;

/// Trait for renderable UI components
pub trait Render {
    /// Renders the component to the given frame area
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);
}

/// Trait for interactive popups
pub trait Popup: Render {
    /// Returns true if the popup should be displayed
    fn is_visible(&self, app: &App) -> bool;
    
    /// Returns the area the popup should occupy
    fn calculate_area(&self, frame_area: Rect) -> Rect;
}

/// Main render function - dispatches to appropriate renderers
pub fn render(frame: &mut Frame, app: &App) {
    let layout = layout::calculate_layout(frame.area());
    
    // Render main content
    header::render(frame, layout.header, app);
    panels::render(frame, layout.main, app);
    footer::render(frame, layout.footer, app);
    
    // Render popups (layered on top)
    popups::render_active(frame, app);
}
```

### 5.3 Layout Module

```rust
// src/ui/layout.rs

use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app_state::{App, ViewMode};
use crate::theme::layout as layout_const;

/// Calculated layout areas for the UI
#[derive(Debug, Clone, Copy)]
pub struct AppLayout {
    pub header: Rect,
    pub main: Rect,
    pub footer: Rect,
}

/// Layout for the main content area
#[derive(Debug, Clone, Copy)]
pub struct MainLayout {
    pub left_panel: Rect,
    pub right_panel: Rect,
}

/// Layout for detail views
#[derive(Debug, Clone, Copy)]
pub struct DetailLayout {
    pub info: Rect,
    pub content: Rect,
}

/// Calculates the main application layout
#[must_use]
pub fn calculate_layout(area: Rect) -> AppLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(layout_const::HEADER_HEIGHT),
            Constraint::Min(0),
            Constraint::Length(layout_const::FOOTER_HEIGHT),
        ])
        .split(area);
    
    AppLayout {
        header: chunks[0],
        main: chunks[1],
        footer: chunks[2],
    }
}

/// Calculates the two-panel layout for list views
#[must_use]
pub fn calculate_main_layout(area: Rect, app: &App) -> MainLayout {
    let (left_pct, right_pct) = match app.view_mode() {
        ViewMode::Normal => (
            layout_const::BLOCK_LIST_WIDTH_PCT,
            layout_const::TXN_LIST_WIDTH_PCT,
        ),
        ViewMode::BlockDetail | ViewMode::TransactionDetail => (0, 100),
        _ => (30, 70),
    };
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_pct),
            Constraint::Percentage(right_pct),
        ])
        .split(area);
    
    MainLayout {
        left_panel: chunks[0],
        right_panel: chunks[1],
    }
}

/// Calculates layout for detail views with info header
#[must_use]
pub fn calculate_detail_layout(area: Rect) -> DetailLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);
    
    DetailLayout {
        info: chunks[0],
        content: chunks[1],
    }
}

/// Creates a centered popup area
#[must_use]
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

---

## Phase 6: App State Refactoring (High Risk)

### 6.1 Create `src/state/` Module Structure

```
src/state/
├── mod.rs              (~100 lines)  - State exports and App struct
├── navigation.rs       (~150 lines)  - Navigation state
├── data.rs             (~200 lines)  - Data/cache state
├── ui_state.rs         (~150 lines)  - UI-specific state
├── config.rs           (~150 lines)  - Configuration persistence
├── command_handler.rs  (~400 lines)  - Command execution
└── platform/
    ├── mod.rs          (~30 lines)   - Platform exports
    ├── clipboard.rs    (~100 lines)  - Clipboard abstraction
    └── paths.rs        (~50 lines)   - Platform-specific paths
```

**Total: ~1,330 lines** (refactored from 2,054 lines)

### 6.2 Command Handler Decomposition

```rust
// src/state/command_handler.rs

use color_eyre::Result;

use crate::commands::AppCommand;
use super::App;

/// Trait for command handlers
pub trait CommandHandler {
    /// Handles a command and returns whether the app should continue
    fn handle(&mut self, command: AppCommand) -> Result<bool>;
}

impl CommandHandler for App {
    fn handle(&mut self, command: AppCommand) -> Result<bool> {
        match command {
            // Navigation commands
            AppCommand::Quit => return Ok(false),
            AppCommand::Back => self.handle_back(),
            AppCommand::FocusNext => self.handle_focus_next(),
            AppCommand::FocusPrev => self.handle_focus_prev(),
            
            // Selection commands
            AppCommand::SelectNext => self.handle_select_next(),
            AppCommand::SelectPrev => self.handle_select_prev(),
            AppCommand::SelectFirst => self.handle_select_first(),
            AppCommand::SelectLast => self.handle_select_last(),
            AppCommand::PageDown => self.handle_page_down(),
            AppCommand::PageUp => self.handle_page_up(),
            AppCommand::Enter => self.handle_enter(),
            
            // Search commands
            AppCommand::Search => self.handle_search_open(),
            AppCommand::SearchSubmit => self.handle_search_submit(),
            AppCommand::SearchCancel => self.handle_search_cancel(),
            AppCommand::SearchChar(c) => self.handle_search_char(c),
            AppCommand::SearchBackspace => self.handle_search_backspace(),
            
            // View commands
            AppCommand::ToggleHelp => self.handle_toggle_help(),
            AppCommand::ToggleNetworkSelector => self.handle_toggle_network(),
            AppCommand::SelectNetwork(n) => self.handle_select_network(n),
            
            // Data commands
            AppCommand::Refresh => self.handle_refresh(),
            AppCommand::Copy => self.handle_copy(),
            AppCommand::ExportSvg => self.handle_export_svg(),
            
            // Async results
            AppCommand::BlocksLoaded(blocks) => self.handle_blocks_loaded(blocks),
            AppCommand::TransactionsLoaded(txns) => self.handle_txns_loaded(txns),
            AppCommand::Error(msg) => self.handle_error(msg),
            
            _ => {}
        }
        Ok(true)
    }
}

// Private handler implementations
impl App {
    fn handle_back(&mut self) {
        self.navigation.pop();
    }
    
    fn handle_focus_next(&mut self) {
        self.ui_state.focus_next();
    }
    
    fn handle_select_next(&mut self) {
        match self.ui_state.focused_panel() {
            FocusedPanel::Blocks => self.data.blocks.select_next(),
            FocusedPanel::Transactions => self.data.transactions.select_next(),
            _ => {}
        }
    }
    
    // ... other handlers
}
```

### 6.3 Platform Clipboard Abstraction

```rust
// src/state/platform/clipboard.rs

use color_eyre::Result;

/// Clipboard operations abstraction
pub trait Clipboard {
    /// Copies text to the system clipboard
    fn copy(&self, text: &str) -> Result<()>;
    
    /// Reads text from the system clipboard
    fn paste(&self) -> Result<String>;
}

/// Platform-specific clipboard implementation
#[cfg(target_os = "linux")]
pub struct SystemClipboard;

#[cfg(target_os = "linux")]
impl SystemClipboard {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[cfg(target_os = "linux")]
impl Clipboard for SystemClipboard {
    fn copy(&self, text: &str) -> Result<()> {
        use std::io::Write;
        use std::process::{Command, Stdio};
        
        // Try xclip first, then xsel, then wl-copy (Wayland)
        let commands = [
            ("xclip", &["-selection", "clipboard"][..]),
            ("xsel", &["--clipboard", "--input"][..]),
            ("wl-copy", &[][..]),
        ];
        
        for (cmd, args) in commands {
            if let Ok(mut child) = Command::new(cmd)
                .args(args)
                .stdin(Stdio::piped())
                .spawn()
            {
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(text.as_bytes())?;
                }
                if child.wait()?.success() {
                    return Ok(());
                }
            }
        }
        
        Err(color_eyre::eyre::eyre!(
            "No clipboard utility available (tried xclip, xsel, wl-copy)"
        ))
    }
    
    fn paste(&self) -> Result<String> {
        use std::process::Command;
        
        let commands = [
            ("xclip", &["-selection", "clipboard", "-o"][..]),
            ("xsel", &["--clipboard", "--output"][..]),
            ("wl-paste", &[][..]),
        ];
        
        for (cmd, args) in commands {
            if let Ok(output) = Command::new(cmd).args(args).output() {
                if output.status.success() {
                    return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
                }
            }
        }
        
        Err(color_eyre::eyre::eyre!(
            "No clipboard utility available"
        ))
    }
}

#[cfg(target_os = "macos")]
pub struct SystemClipboard;

#[cfg(target_os = "macos")]
impl Clipboard for SystemClipboard {
    fn copy(&self, text: &str) -> Result<()> {
        use std::io::Write;
        use std::process::{Command, Stdio};
        
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())?;
        }
        
        child.wait()?;
        Ok(())
    }
    
    fn paste(&self) -> Result<String> {
        let output = std::process::Command::new("pbpaste").output()?;
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

#[cfg(target_os = "windows")]
pub struct SystemClipboard {
    // Use clipboard-win crate
}
```

---

## Phase 7: Final Module Structure

### 7.1 Complete `src/` Layout

```
src/
├── main.rs             (~100 lines)  - Entry point
├── lib.rs              (~50 lines)   - Library exports (optional)
├── constants.rs        (~50 lines)   - App-wide constants
├── theme.rs            (~150 lines)  - Colors, styles, layout constants
├── tui.rs              (~200 lines)  - Terminal setup (unchanged)
├── boot_screen.rs      (~150 lines)  - Boot animation (unchanged)
├── updater.rs          (~200 lines)  - Self-updater (unchanged)
├── commands.rs         (~850 lines)  - Input handling (minor changes)
│
├── domain/             (~990 lines)  - Domain types
│   ├── mod.rs
│   ├── transaction.rs
│   ├── block.rs
│   ├── account.rs
│   ├── asset.rs
│   ├── network.rs
│   ├── nfd.rs
│   └── error.rs
│
├── client/             (~800 lines)  - API clients
│   ├── mod.rs
│   ├── http.rs
│   ├── indexer.rs
│   ├── node.rs
│   └── nfd.rs
│
├── widgets/            (~2,670 lines) - UI widgets
│   ├── mod.rs
│   ├── helpers.rs
│   ├── common/
│   │   ├── mod.rs
│   │   ├── badge.rs
│   │   ├── amount.rs
│   │   └── address.rs
│   ├── list/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   ├── block_list.rs
│   │   └── txn_list.rs
│   ├── graph/
│   │   ├── mod.rs
│   │   ├── txn_graph.rs
│   │   ├── renderer.rs
│   │   └── svg_export.rs
│   └── detail/
│       ├── mod.rs
│       ├── flow_diagram.rs
│       └── visual_card.rs
│
├── ui/                 (~2,580 lines) - UI rendering
│   ├── mod.rs
│   ├── layout.rs
│   ├── header.rs
│   ├── footer.rs
│   ├── panels/
│   │   ├── mod.rs
│   │   ├── blocks.rs
│   │   ├── transactions.rs
│   │   └── details/
│   │       ├── mod.rs
│   │       ├── block.rs
│   │       ├── transaction.rs
│   │       ├── account.rs
│   │       └── asset.rs
│   ├── popups/
│   │   ├── mod.rs
│   │   ├── search.rs
│   │   ├── network.rs
│   │   ├── help.rs
│   │   └── error.rs
│   └── components/
│       ├── mod.rs
│       ├── scrollbar.rs
│       └── tabs.rs
│
└── state/              (~1,330 lines) - Application state
    ├── mod.rs
    ├── navigation.rs
    ├── data.rs
    ├── ui_state.rs
    ├── config.rs
    ├── command_handler.rs
    └── platform/
        ├── mod.rs
        ├── clipboard.rs
        └── paths.rs
```

### 7.2 Line Count Summary

| Module | Before | After | Change |
|--------|--------|-------|--------|
| Domain types | (in algorand.rs) | 990 | New |
| API clients | (in algorand.rs) | 800 | New |
| algorand.rs | 2,783 | 0 | Removed |
| Widgets | 4,112 | 2,670 | -35% |
| UI | 2,455 | 2,580 | +5% (more structure) |
| State | 2,054 | 1,330 | -35% |
| Commands | 827 | 850 | +3% |
| Theme/Constants | 0 | 200 | New |
| Other (unchanged) | ~400 | ~400 | 0% |
| **Total** | **~12,631** | **~9,820** | **-22%** |

---

## Migration Order & Risk Assessment

### Phase 1: Extract Constants & Theme (Week 1)
- **Risk**: Low
- **Dependencies**: None
- **Testing**: Visual regression only
- **Rollback**: Easy - revert imports

### Phase 2: Extract Domain Types (Week 1-2)
- **Risk**: Low-Medium
- **Dependencies**: Phase 1
- **Testing**: Unit tests for serialization
- **Rollback**: Medium - requires import changes

### Phase 3: HTTP Client Abstraction (Week 2)
- **Risk**: Medium
- **Dependencies**: Phase 2
- **Testing**: Integration tests with mock server
- **Rollback**: Medium - API layer changes

### Phase 4: Widget Decomposition (Week 3-4)
- **Risk**: Medium
- **Dependencies**: Phases 1, 2
- **Testing**: Widget unit tests, visual regression
- **Rollback**: Hard - many file moves

### Phase 5: UI Decomposition (Week 4-5)
- **Risk**: Medium-High
- **Dependencies**: Phases 1, 4
- **Testing**: Integration tests, manual testing
- **Rollback**: Hard - render pipeline changes

### Phase 6: App State Refactoring (Week 5-6)
- **Risk**: High
- **Dependencies**: All previous phases
- **Testing**: Full integration tests
- **Rollback**: Very hard - core state changes

---

## Testing Strategy

### Unit Tests
- Domain type serialization/deserialization
- HTTP client retry logic (with mocked responses)
- Widget state management (ListState)
- Layout calculations
- Formatting helpers

### Integration Tests
- API client against test network
- Full command flow tests
- State persistence round-trips

### Visual Regression
- Screenshot comparison for key views
- Terminal size edge cases
- Popup overlay rendering

### Manual Testing Checklist
- [ ] All keybindings work
- [ ] Network switching works
- [ ] Search functionality works
- [ ] Copy to clipboard works
- [ ] SVG export works
- [ ] Transaction graph renders correctly
- [ ] All detail views display correctly
- [ ] Error states handled gracefully

---

## Implementation Notes

### Backward Compatibility
- Keep `algorand.rs` as a re-export facade during migration
- Deprecate old paths with `#[deprecated]` attributes
- Maintain existing public API surface

### Documentation Requirements
- Update module-level docs for all new modules
- Add architecture diagram to README
- Document public traits and their implementations
- Add examples for common use cases

### CI/CD Updates
- Add module-specific test targets
- Update code coverage requirements
- Add visual regression tests (optional)

---

## Appendix: Key Refactoring Patterns

### A. Extract Method
Used extensively in command handler decomposition.

### B. Extract Class/Module
Primary pattern for splitting large files.

### C. Replace Conditional with Polymorphism
Used for transaction type rendering.

### D. Introduce Parameter Object
Used for SVG export configuration.

### E. Template Method
Used for list widget rendering.

### F. Strategy Pattern
Used for platform-specific clipboard handling.
