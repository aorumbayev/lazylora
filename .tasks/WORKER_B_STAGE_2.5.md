# Worker B - Stage 2.5: UI Popups & Components

## Task Overview
- **Worker**: B
- **Stage**: 2.5 (UI Split)
- **Duration**: 3 days
- **Risk Level**: Medium
- **Status**: NOT_STARTED
- **Depends On**: Stage 2 Sync Complete

## Prerequisites
- [ ] Stage 2 sync complete (widgets/ and state/ modules ready)
- [ ] Fresh branch from post-sync main: `refactor/stage2.5-worker-b-ui-popups`
- [ ] `src/theme.rs` available
- [ ] `src/widgets/` available
- [ ] `src/state/` available (especially PopupState enum)
- [ ] Read `src/ui.rs` popup-related code

## Deliverables
| File | Lines | Status |
|------|-------|--------|
| `src/ui/popups/mod.rs` | ~80 | NOT_STARTED |
| `src/ui/popups/search.rs` | ~200 | NOT_STARTED |
| `src/ui/popups/network.rs` | ~150 | NOT_STARTED |
| `src/ui/popups/help.rs` | ~150 | NOT_STARTED |
| `src/ui/popups/message.rs` | ~80 | NOT_STARTED |
| `src/ui/popups/search_results.rs` | ~150 | NOT_STARTED |
| `src/ui/components/mod.rs` | ~30 | NOT_STARTED |
| `src/ui/components/toast.rs` | ~80 | NOT_STARTED |
| `src/ui/components/tabs.rs` | ~100 | NOT_STARTED |
| `src/ui/components/scrollbar.rs` | ~80 | NOT_STARTED |

## DO NOT TOUCH
- `src/ui.rs` (will be deleted at Stage 3)
- `src/ui/layout.rs` (Worker A)
- `src/ui/header.rs` (Worker A)
- `src/ui/footer.rs` (Worker A)
- `src/ui/panels/*` (Worker A)

---

## Task 1: Create Directory Structure

### Status: NOT_STARTED

- [ ] Create `src/ui/popups/` directory
- [ ] Create `src/ui/components/` directory
- [ ] Create placeholder `mod.rs` files

---

## Task 2: Analyze Popup Code in `ui.rs`

### Status: NOT_STARTED

### 2.1 Document Popup Functions
- [ ] List all popup rendering functions
- [ ] Note line numbers and sizes
- [ ] Identify shared popup patterns

**Functions found:**
```
Popups:
- [ ] render_search_popup (line ___) - ___ lines
- [ ] render_network_selector (line ___) - ___ lines
- [ ] render_help_popup (line ___) - ___ lines
- [ ] render_message_popup (line ___) - ___ lines
- [ ] render_search_results (line ___) - ___ lines

Components:
- [ ] render_toast (line ___) - ___ lines
- [ ] render_tabs (line ___) - ___ lines
```

### 2.2 Identify Shared Patterns
- [ ] Popup border/title pattern
- [ ] Centered positioning
- [ ] Clear background
- [ ] Input handling hints

---

## Task 3: Create Popup Trait in `src/ui/popups/mod.rs`

### Status: NOT_STARTED

### 3.1 Define Popup Trait
```rust
use ratatui::Frame;
use ratatui::layout::Rect;
use crate::state::App;

/// Trait for popup components
pub trait Popup {
    /// Render the popup content
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);
    
    /// Get the popup title
    fn title(&self) -> &str;
    
    /// Get preferred size as (width_percent, height_percent)
    fn size(&self) -> (u16, u16) {
        (60, 50) // Default size
    }
}

/// Render a popup with standard frame
fn render_popup_frame(frame: &mut Frame, popup: &dyn Popup, app: &App) {
    let size = popup.size();
    let area = crate::ui::layout::centered_rect(size.0, size.1, frame.area());
    
    // Clear background
    frame.render_widget(Clear, area);
    
    // Draw border with title
    let block = Block::default()
        .title(popup.title())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(crate::theme::styles::border_active());
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    // Render popup content
    popup.render(frame, inner, app);
}
```

### 3.2 Implement Popup Dispatcher
```rust
mod help;
mod message;
mod network;
mod search;
mod search_results;

pub use help::HelpPopup;
pub use message::MessagePopup;
pub use network::NetworkPopup;
pub use search::SearchPopup;
pub use search_results::SearchResultsPopup;

/// Render the currently active popup if any
pub fn render_active(frame: &mut Frame, app: &App) {
    match &app.ui.popup {
        PopupState::None => {}
        PopupState::Help => render_popup_frame(frame, &HelpPopup, app),
        PopupState::NetworkSelect(idx) => {
            render_popup_frame(frame, &NetworkPopup::new(*idx), app)
        }
        PopupState::SearchWithType(query, search_type) => {
            render_popup_frame(frame, &SearchPopup::new(query, *search_type), app)
        }
        PopupState::SearchResults(results) => {
            render_popup_frame(frame, &SearchResultsPopup::new(results), app)
        }
        PopupState::Message(msg) => {
            render_popup_frame(frame, &MessagePopup::new(msg), app)
        }
    }
}
```

---

## Task 4: Create `src/ui/popups/search.rs`

### Status: NOT_STARTED

### 4.1 Extract Search Popup
- [ ] Locate search popup code in ui.rs
- [ ] Extract input handling display
- [ ] Extract search type selector

### 4.2 Implement SearchPopup
```rust
pub struct SearchPopup<'a> {
    query: &'a str,
    search_type: SearchType,
}

impl<'a> SearchPopup<'a> {
    pub fn new(query: &'a str, search_type: SearchType) -> Self {
        Self { query, search_type }
    }
}

impl Popup for SearchPopup<'_> {
    fn title(&self) -> &str {
        "Search"
    }
    
    fn size(&self) -> (u16, u16) {
        (50, 20)
    }
    
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Search type tabs
        // Input field with query
        // Placeholder text
        // Keybinding hints
    }
}
```

### 4.3 Components
- [ ] Search type tabs (Transaction, Account, Asset, Block)
- [ ] Text input field
- [ ] Cursor display
- [ ] Help text

### 4.4 Documentation & Verification
- [ ] Clippy passes

---

## Task 5: Create `src/ui/popups/network.rs`

### Status: NOT_STARTED

### 5.1 Extract Network Selector
- [ ] Locate network selector code in ui.rs
- [ ] Extract network list display

### 5.2 Implement NetworkPopup
```rust
pub struct NetworkPopup {
    selected_index: usize,
}

impl NetworkPopup {
    pub fn new(selected_index: usize) -> Self {
        Self { selected_index }
    }
}

impl Popup for NetworkPopup {
    fn title(&self) -> &str {
        "Select Network"
    }
    
    fn size(&self) -> (u16, u16) {
        (40, 15)
    }
    
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        // List of networks
        // Highlight selected
        // Show current network indicator
    }
}
```

### 5.3 Network List
- [ ] MainNet with description
- [ ] TestNet with description
- [ ] LocalNet with description
- [ ] Highlight current network
- [ ] Highlight selected option

### 5.4 Documentation & Verification
- [ ] Clippy passes

---

## Task 6: Create `src/ui/popups/help.rs`

### Status: NOT_STARTED

### 6.1 Extract Help Popup
- [ ] Locate help/keybinding popup in ui.rs
- [ ] Extract keybinding display

### 6.2 Implement HelpPopup
```rust
pub struct HelpPopup;

impl Popup for HelpPopup {
    fn title(&self) -> &str {
        "Help - Keybindings"
    }
    
    fn size(&self) -> (u16, u16) {
        (70, 80)
    }
    
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Navigation keys
        // Action keys
        // View keys
        // Search keys
    }
}
```

### 6.3 Keybinding Sections
- [ ] Navigation (j/k, arrows, g/G, etc.)
- [ ] Actions (Enter, Esc, etc.)
- [ ] Views (Tab, ?, etc.)
- [ ] Search (/, etc.)

### 6.4 Documentation & Verification
- [ ] Clippy passes

---

## Task 7: Create `src/ui/popups/message.rs`

### Status: NOT_STARTED

### 7.1 Implement MessagePopup
```rust
pub struct MessagePopup<'a> {
    message: &'a str,
}

impl<'a> MessagePopup<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }
}

impl Popup for MessagePopup<'_> {
    fn title(&self) -> &str {
        "Message"
    }
    
    fn size(&self) -> (u16, u16) {
        (50, 20)
    }
    
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Message text (wrapped)
        // Dismiss hint
    }
}
```

### 7.2 Documentation & Verification
- [ ] Clippy passes

---

## Task 8: Create `src/ui/popups/search_results.rs`

### Status: NOT_STARTED

### 8.1 Extract Search Results Display
- [ ] Locate search results popup in ui.rs
- [ ] Extract results list rendering

### 8.2 Implement SearchResultsPopup
```rust
pub struct SearchResultsPopup<'a> {
    results: &'a [(usize, SearchResultItem)],
}

impl<'a> SearchResultsPopup<'a> {
    pub fn new(results: &'a [(usize, SearchResultItem)]) -> Self {
        Self { results }
    }
}

impl Popup for SearchResultsPopup<'_> {
    fn title(&self) -> &str {
        "Search Results"
    }
    
    fn size(&self) -> (u16, u16) {
        (70, 60)
    }
    
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Results count
        // Selectable list
        // Result type badges
        // Preview of selected
    }
}
```

### 8.3 Documentation & Verification
- [ ] Clippy passes

---

## Task 9: Create `src/ui/components/toast.rs`

### Status: NOT_STARTED

### 9.1 Extract Toast Notification
- [ ] Locate toast rendering in ui.rs
- [ ] Extract positioning logic

### 9.2 Implement Toast Component
```rust
pub struct Toast<'a> {
    message: &'a str,
    level: ToastLevel,
}

pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl<'a> Toast<'a> {
    pub fn new(message: &'a str, level: ToastLevel) -> Self {
        Self { message, level }
    }
    
    pub fn render(&self, frame: &mut Frame) {
        // Position at bottom-right
        // Style based on level
        // Auto-dismiss hint
    }
}

/// Render toast if present and not expired
pub fn render_toast(frame: &mut Frame, app: &App) {
    if let Some((message, created_at)) = &app.ui.toast_message {
        if created_at.elapsed() < Duration::from_secs(3) {
            Toast::new(message, ToastLevel::Info).render(frame);
        }
    }
}
```

### 9.3 Documentation & Verification
- [ ] Clippy passes

---

## Task 10: Create `src/ui/components/tabs.rs`

### Status: NOT_STARTED

### 10.1 Implement Tabs Component
```rust
pub struct Tabs<'a> {
    titles: &'a [&'a str],
    selected: usize,
}

impl<'a> Tabs<'a> {
    pub fn new(titles: &'a [&'a str], selected: usize) -> Self {
        Self { titles, selected }
    }
}

impl Widget for Tabs<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render tab bar
        // Highlight selected tab
        // Use theme colors
    }
}
```

### 10.2 Documentation & Verification
- [ ] Clippy passes

---

## Task 11: Create `src/ui/components/scrollbar.rs`

### Status: NOT_STARTED

### 11.1 Implement Scrollbar Helper
```rust
/// Render a scrollbar for a list
pub fn render_scrollbar(
    frame: &mut Frame,
    area: Rect,
    total_items: usize,
    visible_items: usize,
    offset: usize,
) {
    if total_items <= visible_items {
        return;
    }
    
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    
    let mut state = ScrollbarState::new(total_items)
        .position(offset);
    
    frame.render_stateful_widget(scrollbar, area, &mut state);
}
```

### 11.2 Documentation & Verification
- [ ] Clippy passes

---

## Task 12: Create `src/ui/components/mod.rs`

### Status: NOT_STARTED

```rust
mod scrollbar;
mod tabs;
mod toast;

pub use scrollbar::render_scrollbar;
pub use tabs::Tabs;
pub use toast::{render_toast, Toast, ToastLevel};
```

---

## Task 13: Write Tests

### Status: NOT_STARTED

### 13.1 Popup Tests
- [ ] Test popup sizing
- [ ] Test popup rendering doesn't panic

### 13.2 Component Tests
- [ ] Test tabs selection
- [ ] Test scrollbar calculations

---

## Task 14: Final Checklist

### Status: NOT_STARTED

- [ ] All 10 files created
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

**Popup trait design:**
- 

**Blocked issues:**
- 

**Notes for coordinator:**
- Worker A creates ui/panels/ and ui/layout.rs in parallel
- Final ui/mod.rs needs to call popups::render_active()
- Toast component should be rendered after popups
