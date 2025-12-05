# Worker C - Stage 1.5: State Module

## Task Overview
- **Worker**: C
- **Stage**: 1.5 (Core Split)
- **Duration**: 3 days
- **Risk Level**: High
- **Status**: COMPLETE
- **Depends On**: Stage 1 Sync Complete

## Prerequisites
- [ ] Stage 0 complete and merged
- [ ] Stage 1 sync complete
- [ ] Fresh branch from post-sync main: `refactor/stage1.5-worker-c-state`
- [ ] `src/domain/` available (from Stage 0)
- [ ] `src/client/` available (from Stage 0)
- [ ] Read `src/app_state.rs` thoroughly

## Deliverables
| File | Lines | Status |
|------|-------|--------|
| `src/state/mod.rs` | ~100 | NOT_STARTED |
| `src/state/navigation.rs` | ~150 | NOT_STARTED |
| `src/state/data.rs` | ~200 | NOT_STARTED |
| `src/state/ui_state.rs` | ~150 | NOT_STARTED |
| `src/state/config.rs` | ~150 | NOT_STARTED |
| `src/state/command_handler.rs` | ~400 | NOT_STARTED |
| `src/state/platform/mod.rs` | ~30 | NOT_STARTED |
| `src/state/platform/clipboard.rs` | ~100 | NOT_STARTED |
| `src/state/platform/paths.rs` | ~50 | NOT_STARTED |

## DO NOT TOUCH
- `src/app_state.rs` (will be replaced at Stage 2 sync)
- `src/commands.rs`
- `src/ui.rs`
- `src/widgets.rs`

---

## Task 1: Create Directory Structure

### Status: NOT_STARTED

- [ ] Create `src/state/` directory
- [ ] Create `src/state/platform/` directory
- [ ] Create placeholder `mod.rs` files

---

## Task 2: Analyze `app_state.rs` Structure

### Status: NOT_STARTED

### 2.1 Document Current State Decomposition
- [ ] Identify `NavigationState` struct and fields
- [ ] Identify `DataState` struct and fields
- [ ] Identify `UiState` struct and fields
- [ ] Identify `App` struct top-level fields

**NavigationState fields:**
```
- [ ] selected_block_index
- [ ] selected_transaction_index
- [ ] selected_block_id (stable ID)
- [ ] selected_transaction_id (stable ID)
- [ ] show_block_details
- [ ] show_transaction_details
- [ ] block_scroll, transaction_scroll
- [ ] (Add more)
```

**DataState fields:**
```
- [ ] blocks: Vec<AlgoBlock>
- [ ] transactions: Vec<Transaction>
- [ ] current_block_details
- [ ] current_transaction_details
- [ ] (Add more)
```

**UiState fields:**
```
- [ ] focus: Focus enum
- [ ] popup_state: PopupState enum
- [ ] detail_view_mode
- [ ] (Add more)
```

### 2.2 Document Command Handler Size
- [ ] Count command variants handled
- [ ] Group commands by category
- [ ] Identify command handler size (~220 lines)

---

## Task 3: Create `src/state/navigation.rs`

### Status: NOT_STARTED

### 3.1 Define NavigationState
- [ ] Extract from app_state.rs
- [ ] Keep selection indices and stable IDs
- [ ] Keep scroll positions
- [ ] Keep detail view flags

```rust
#[derive(Debug, Default)]
pub struct NavigationState {
    // Block selection
    pub selected_block_index: Option<usize>,
    pub selected_block_id: Option<u64>,
    pub block_scroll: u16,
    
    // Transaction selection
    pub selected_transaction_index: Option<usize>,
    pub selected_transaction_id: Option<String>,
    pub transaction_scroll: u16,
    
    // Detail view navigation
    pub show_block_details: bool,
    pub show_transaction_details: bool,
    // ...
}
```

### 3.2 Implement Navigation Methods
- [ ] `pub fn select_block(&mut self, index: Option<usize>, id: Option<u64>)`
- [ ] `pub fn select_transaction(&mut self, index: Option<usize>, id: Option<String>)`
- [ ] `pub fn push_view(&mut self, view: ViewState)` (if using view stack)
- [ ] `pub fn pop_view(&mut self) -> Option<ViewState>`
- [ ] `pub fn clear_selection(&mut self)`

### 3.3 Implement Sync Methods
- [ ] `pub fn sync_block_selection(&mut self, blocks: &[AlgoBlock])`
- [ ] `pub fn sync_transaction_selection(&mut self, txns: &[Transaction])`

### 3.4 Documentation & Verification
- [ ] Document state transitions
- [ ] Clippy passes

---

## Task 4: Create `src/state/data.rs`

### Status: NOT_STARTED

### 4.1 Define DataState
- [ ] Extract data containers from app_state.rs

```rust
#[derive(Debug, Default)]
pub struct DataState {
    // Block data
    pub blocks: Vec<AlgoBlock>,
    pub current_round: Option<u64>,
    pub block_details: Option<BlockDetails>,
    
    // Transaction data
    pub transactions: Vec<Transaction>,
    pub transaction_details: Option<Box<Transaction>>,
    
    // Account/Asset data
    pub account_details: Option<Box<AccountDetails>>,
    pub asset_details: Option<Box<AssetDetails>>,
    
    // Search
    pub search_results: Vec<SearchResultItem>,
    pub filtered_search_results: Vec<(usize, SearchResultItem)>,
}
```

### 4.2 Implement Data Methods
- [ ] `pub fn update_blocks(&mut self, blocks: Vec<AlgoBlock>)`
- [ ] `pub fn merge_blocks(&mut self, new_blocks: Vec<AlgoBlock>)`
- [ ] `pub fn update_transactions(&mut self, txns: Vec<Transaction>)`
- [ ] `pub fn find_block_index(&self, id: u64) -> Option<usize>`
- [ ] `pub fn find_transaction_index(&self, id: &str) -> Option<usize>`

### 4.3 Implement Search Methods
- [ ] `pub fn set_search_results(&mut self, results: Vec<SearchResultItem>)`
- [ ] `pub fn filter_search_results(&mut self, query: &str)`
- [ ] `pub fn clear_search(&mut self)`

### 4.4 Documentation & Verification
- [ ] Document data flow
- [ ] Clippy passes

---

## Task 5: Create `src/state/ui_state.rs`

### Status: NOT_STARTED

### 5.1 Define Focus Enum
- [ ] Extract from app_state.rs

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Blocks,
    Transactions,
}
```

### 5.2 Define PopupState Enum
- [ ] Extract all popup variants

```rust
#[derive(Debug, Clone, Default)]
pub enum PopupState {
    #[default]
    None,
    NetworkSelect(usize),
    SearchWithType(String, SearchType),
    SearchResults(Vec<(usize, SearchResultItem)>),
    Message(String),
    Help,
}
```

### 5.3 Define UiState
```rust
#[derive(Debug, Default)]
pub struct UiState {
    pub focus: Focus,
    pub popup: PopupState,
    pub detail_view_mode: DetailViewMode,
    pub block_detail_tab: BlockDetailTab,
    pub toast_message: Option<(String, Instant)>,
    // ...
}
```

### 5.4 Implement UI Methods
- [ ] `pub fn focus_next(&mut self)`
- [ ] `pub fn focus_prev(&mut self)`
- [ ] `pub fn open_popup(&mut self, popup: PopupState)`
- [ ] `pub fn close_popup(&mut self)`
- [ ] `pub fn show_toast(&mut self, message: String)`
- [ ] `pub fn is_popup_open(&self) -> bool`

### 5.5 Documentation & Verification
- [ ] Document popup states
- [ ] Clippy passes

---

## Task 6: Create `src/state/config.rs`

### Status: NOT_STARTED

### 6.1 Extract Config from app_state.rs
- [ ] Locate `AppConfig` struct
- [ ] Locate `load()` and `save()` methods
- [ ] Locate config file path logic

### 6.2 Define AppConfig
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub network: Network,
    pub show_live: bool,
    // Add other persisted settings
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: Network::MainNet,
            show_live: true,
        }
    }
}
```

### 6.3 Implement Persistence
- [ ] `pub fn config_path() -> Option<PathBuf>`
- [ ] `pub fn load() -> Self`
- [ ] `pub fn save(&self) -> Result<()>`

### 6.4 Use Platform Paths
- [ ] Import from `platform::paths`
- [ ] Handle missing directories

### 6.5 Documentation & Verification
- [ ] Document config file location
- [ ] Test save/load round-trip
- [ ] Clippy passes

---

## Task 7: Create `src/state/platform/clipboard.rs`

### Status: NOT_STARTED

### 7.1 Extract Clipboard Code from app_state.rs
- [ ] Locate platform-specific clipboard code (~50 lines)
- [ ] Identify Linux clipboard tools (xclip, xsel, wl-copy)
- [ ] Identify macOS clipboard (pbcopy)

### 7.2 Define Clipboard Trait
```rust
pub trait Clipboard {
    fn copy(&self, text: &str) -> Result<()>;
    fn paste(&self) -> Result<String>;
}
```

### 7.3 Implement SystemClipboard for Linux
```rust
#[cfg(target_os = "linux")]
pub struct SystemClipboard;

#[cfg(target_os = "linux")]
impl Clipboard for SystemClipboard {
    fn copy(&self, text: &str) -> Result<()> {
        // Try xclip, then xsel, then wl-copy
    }
    
    fn paste(&self) -> Result<String> {
        // Try xclip -o, then xsel -o, then wl-paste
    }
}
```

### 7.4 Implement SystemClipboard for macOS
```rust
#[cfg(target_os = "macos")]
impl Clipboard for SystemClipboard {
    fn copy(&self, text: &str) -> Result<()> {
        // Use pbcopy
    }
    
    fn paste(&self) -> Result<String> {
        // Use pbpaste
    }
}
```

### 7.5 Implement SystemClipboard for Windows (Stub)
```rust
#[cfg(target_os = "windows")]
impl Clipboard for SystemClipboard {
    // Use clipboard crate or Windows API
}
```

### 7.6 Documentation & Verification
- [ ] Document platform requirements
- [ ] Clippy passes

---

## Task 8: Create `src/state/platform/paths.rs`

### Status: NOT_STARTED

### 8.1 Implement Path Helpers
```rust
/// Returns the application config directory
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("lazylora"))
}

/// Returns the application data directory
pub fn data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join("lazylora"))
}

/// Returns the application cache directory
pub fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join("lazylora"))
}

/// Ensures a directory exists, creating it if necessary
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
```

### 8.2 Documentation & Verification
- [ ] Document directory locations per platform
- [ ] Clippy passes

---

## Task 9: Create `src/state/command_handler.rs`

### Status: NOT_STARTED

### 9.1 Analyze Command Categories
- [ ] Navigation commands (Quit, Back, Focus*)
- [ ] Selection commands (SelectNext, SelectPrev, Enter, etc.)
- [ ] Search commands (Search, SearchSubmit, SearchChar, etc.)
- [ ] View commands (ToggleHelp, ToggleNetwork, etc.)
- [ ] Data commands (Refresh, Copy, ExportSvg)
- [ ] Async result commands (BlocksLoaded, Error, etc.)

### 9.2 Define CommandHandler Trait
```rust
pub trait CommandHandler {
    /// Handles a command, returns false if app should exit
    fn handle_command(&mut self, command: AppCommand) -> Result<bool>;
}
```

### 9.3 Implement Command Handler
- [ ] Implement for App struct
- [ ] Group handlers by category

```rust
impl CommandHandler for App {
    fn handle_command(&mut self, command: AppCommand) -> Result<bool> {
        match command {
            // Navigation
            AppCommand::Quit => return Ok(false),
            AppCommand::Back => self.handle_back(),
            
            // Selection
            cmd if cmd.is_selection() => self.handle_selection(cmd),
            
            // Search
            cmd if cmd.is_search() => self.handle_search(cmd),
            
            // View
            cmd if cmd.is_view() => self.handle_view(cmd),
            
            // Data
            cmd if cmd.is_data() => self.handle_data(cmd),
            
            // Async results
            cmd if cmd.is_async_result() => self.handle_async_result(cmd),
            
            _ => {}
        }
        Ok(true)
    }
}
```

### 9.4 Implement Handler Groups
- [ ] `fn handle_back(&mut self)`
- [ ] `fn handle_selection(&mut self, cmd: AppCommand)`
- [ ] `fn handle_search(&mut self, cmd: AppCommand)`
- [ ] `fn handle_view(&mut self, cmd: AppCommand)`
- [ ] `fn handle_data(&mut self, cmd: AppCommand)`
- [ ] `fn handle_async_result(&mut self, cmd: AppCommand)`

### 9.5 Consider Command Pattern Extension
- [ ] Add `is_selection()`, `is_search()` etc. to AppCommand
- [ ] Or use nested enums for categories

### 9.6 Documentation & Verification
- [ ] Document command flow
- [ ] Clippy passes

---

## Task 10: Create `src/state/mod.rs`

### Status: NOT_STARTED

### 10.1 Module Declarations
```rust
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
pub use ui_state::{Focus, PopupState, UiState};
```

### 10.2 Re-export App Struct (Placeholder)
- [ ] Define minimal App struct that composes substates
- [ ] Will be completed at Stage 2 sync

### 10.3 Module Documentation
- [ ] Document state module architecture

---

## Task 11: Create Platform Module File

### Status: NOT_STARTED

### 11.1 Create `src/state/platform/mod.rs`
```rust
mod clipboard;
mod paths;

pub use clipboard::{Clipboard, SystemClipboard};
pub use paths::*;
```

---

## Task 12: Write Tests

### Status: NOT_STARTED

### 12.1 Navigation Tests
- [ ] Test selection sync
- [ ] Test view stack (if implemented)

### 12.2 Data Tests
- [ ] Test block merging
- [ ] Test search filtering

### 12.3 Config Tests
- [ ] Test save/load round-trip
- [ ] Test default values

### 12.4 Clipboard Tests (Platform-specific)
- [ ] Test copy functionality
- [ ] Handle missing clipboard tools gracefully

---

## Task 13: Final Checklist

### Status: NOT_STARTED

- [ ] All 9 files created
- [ ] `cargo build` succeeds
- [ ] `cargo test --all-features` passes
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] No modifications to existing files
- [ ] Platform clipboard abstracted
- [ ] Command handler decomposed
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

**State decomposition:**
- NavigationState: handles...
- DataState: handles...
- UiState: handles...

**Command handler categories:**
- 

**Platform support:**
- Linux: 
- macOS: 
- Windows: 

**Blocked issues:**
- 

**Notes for coordinator:**
- App struct will need to be reconstructed at Stage 2 sync
- Commands.rs may need minor updates for command categories
- Clipboard trait allows for testing with mock implementations
