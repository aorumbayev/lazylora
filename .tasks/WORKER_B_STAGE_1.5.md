# Worker B - Stage 1.5: Widgets Graph & Detail

## Task Overview
- **Worker**: B
- **Stage**: 1.5 (Core Split)
- **Duration**: 3 days
- **Risk Level**: Medium
- **Status**: NOT_STARTED
- **Depends On**: Stage 1 Sync Complete

## Prerequisites
- [ ] Stage 0 complete and merged
- [ ] Stage 1 sync complete (algorand.rs is now facade)
- [ ] Fresh branch from post-sync main: `refactor/stage1.5-worker-b-widgets-graph`
- [ ] `src/theme.rs` available (from Stage 0)
- [ ] `src/domain/` available (from Stage 0)

## Deliverables
| File | Lines | Status |
|------|-------|--------|
| `src/widgets/graph/mod.rs` | ~30 | NOT_STARTED |
| `src/widgets/graph/types.rs` | ~150 | NOT_STARTED |
| `src/widgets/graph/txn_graph.rs` | ~350 | NOT_STARTED |
| `src/widgets/graph/renderer.rs` | ~300 | NOT_STARTED |
| `src/widgets/graph/svg_export.rs` | ~450 | NOT_STARTED |
| `src/widgets/detail/mod.rs` | ~30 | NOT_STARTED |
| `src/widgets/detail/flow_diagram.rs` | ~150 | NOT_STARTED |
| `src/widgets/detail/visual_card.rs` | ~200 | NOT_STARTED |

## DO NOT TOUCH
- `src/widgets.rs` (will be deleted at Stage 2 sync)
- `src/widgets/common/*` (Worker A)
- `src/widgets/list/*` (Worker A)
- `src/widgets/helpers.rs` (Worker A)
- `src/ui.rs`
- `src/app_state.rs`

---

## Task 1: Create Directory Structure

### Status: NOT_STARTED

- [ ] Create `src/widgets/graph/` directory
- [ ] Create `src/widgets/detail/` directory
- [ ] Create placeholder `mod.rs` files

---

## Task 2: Create `src/widgets/graph/types.rs`

### Status: NOT_STARTED

### 2.1 Extract Graph Types from widgets.rs
- [ ] Locate `GraphEntityType` enum (around line 1790)
- [ ] Locate `GraphColumn` struct (around line 1823)
- [ ] Locate `GraphRepresentation` enum (around line 1871)
- [ ] Locate `GraphRow` struct (around line 1882)

### 2.2 Implement GraphEntityType
- [ ] Define enum variants: Account, Application, Asset
- [ ] Add color mapping method
- [ ] Add icon/label method

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphEntityType {
    Account,
    Application,
    Asset,
}
```

### 2.3 Implement GraphColumn
- [ ] Define struct with entity info
- [ ] Add index, entity_type, entity_id fields
- [ ] Add label field
- [ ] Implement constructors for each entity type

### 2.4 Implement GraphRepresentation
- [ ] Define enum for visual representations
- [ ] Variants: Arrow, Line, Box, etc.

### 2.5 Implement GraphRow
- [ ] Define struct for transaction row
- [ ] Add transaction reference
- [ ] Add source/target column indices
- [ ] Add flow direction
- [ ] Add amount/action label

### 2.6 Documentation & Verification
- [ ] Document all types
- [ ] Clippy passes

---

## Task 3: Create `src/widgets/graph/txn_graph.rs`

### Status: NOT_STARTED

### 3.1 Extract TxnGraph from widgets.rs
- [ ] Locate `TxnGraph` struct (around line 1911)
- [ ] Locate all impl methods (around line 1921-2630)
- [ ] Identify data structure methods vs rendering methods

### 3.2 Implement TxnGraph Struct
- [ ] Define struct with columns and rows
- [ ] Add scroll position fields

```rust
pub struct TxnGraph {
    pub columns: Vec<GraphColumn>,
    pub rows: Vec<GraphRow>,
    pub scroll_x: usize,
    pub scroll_y: usize,
}
```

### 3.3 Implement Construction Methods
- [ ] `pub fn new() -> Self`
- [ ] `pub fn from_transaction(txn: &Transaction) -> Self`
- [ ] `pub fn from_transactions(txns: &[Transaction]) -> Self`

### 3.4 Implement Column Management
- [ ] `fn get_or_create_account_column(&mut self, address: &str) -> usize`
- [ ] `fn get_or_create_app_column(&mut self, app_id: u64) -> usize`
- [ ] `fn get_or_create_asset_column(&mut self, asset_id: u64) -> usize`
- [ ] Deduplicate the three similar methods into generic helper

### 3.5 Implement Row Building
- [ ] `fn add_transaction(&mut self, txn: &Transaction)`
- [ ] Handle different transaction types
- [ ] Handle inner transactions recursively

### 3.6 Implement Navigation
- [ ] `pub fn scroll_left(&mut self)`
- [ ] `pub fn scroll_right(&mut self)`
- [ ] `pub fn scroll_up(&mut self)`
- [ ] `pub fn scroll_down(&mut self)`

### 3.7 Implement Default Trait
- [ ] `impl Default for TxnGraph`

### 3.8 Documentation & Verification
- [ ] Document graph construction
- [ ] Add usage examples
- [ ] Clippy passes

---

## Task 4: Create `src/widgets/graph/renderer.rs` (TxnGraphWidget)

### Status: NOT_STARTED

### 4.1 Extract ASCII Renderer from widgets.rs
- [ ] Locate `TxnGraphWidget` struct (around line 2637)
- [ ] Locate Widget impl (around line 3088)
- [ ] Identify ASCII rendering logic

### 4.2 Implement TxnGraphWidget
- [ ] Define widget struct
- [ ] Accept `&TxnGraph` reference
- [ ] Configure cell dimensions

```rust
pub struct TxnGraphWidget<'a> {
    graph: &'a TxnGraph,
    column_width: u16,
    row_height: u16,
}
```

### 4.3 Implement Widget Trait
- [ ] `impl Widget for TxnGraphWidget<'_>`
- [ ] Render columns as headers
- [ ] Render rows with ASCII art connections
- [ ] Handle scrolling/viewport

### 4.4 ASCII Art Rendering
- [ ] Draw boxes for entities
- [ ] Draw arrows between columns
- [ ] Draw flow lines
- [ ] Add labels for amounts/actions

### 4.5 Styling
- [ ] Use theme colors for entity types
- [ ] Highlight active/selected elements
- [ ] Use Unicode box-drawing characters

### 4.6 Documentation & Verification
- [ ] Document rendering approach
- [ ] Clippy passes

---

## Task 5: Create `src/widgets/graph/svg_export.rs`

### Status: NOT_STARTED

### 5.1 Extract SVG Export from widgets.rs
- [ ] Locate SVG generation methods in TxnGraph (approximately 400 lines)
- [ ] Identify all SVG-specific code

### 5.2 Create SvgExportConfig
- [ ] Define configuration struct

```rust
pub struct SvgExportConfig {
    pub node_width: f64,
    pub node_height: f64,
    pub horizontal_gap: f64,
    pub vertical_gap: f64,
    pub font_size: f64,
    pub font_family: String,
    pub include_css: bool,
}
```

- [ ] Implement Default with sensible values

### 5.3 Create SvgExporter
- [ ] Define exporter struct with config

```rust
pub struct SvgExporter {
    config: SvgExportConfig,
}
```

### 5.4 Implement Export Methods
- [ ] `pub fn new() -> Self`
- [ ] `pub fn with_config(config: SvgExportConfig) -> Self`
- [ ] `pub fn export(&self, graph: &TxnGraph) -> String`

### 5.5 Implement SVG Generation
- [ ] `fn calculate_dimensions(&self, graph: &TxnGraph) -> (f64, f64)`
- [ ] `fn generate_styles(&self) -> String`
- [ ] `fn render_nodes(&self, graph: &TxnGraph) -> String`
- [ ] `fn render_edges(&self, graph: &TxnGraph) -> String`
- [ ] `fn render_labels(&self, graph: &TxnGraph) -> String`

### 5.6 Color Mapping
- [ ] Map entity types to SVG colors
- [ ] Use same palette as theme (hex values)

### 5.7 Documentation & Tests
- [ ] Document SVG format
- [ ] Test export produces valid SVG
- [ ] Test dimensions calculation

---

## Task 6: Create `src/widgets/detail/flow_diagram.rs`

### Status: NOT_STARTED

### 6.1 Extract TxnFlowDiagram from widgets.rs
- [ ] Locate `TxnFlowDiagram` struct (around line 291)
- [ ] Locate Widget impl (around line 483)

### 6.2 Implement TxnFlowDiagram
- [ ] Define struct for sender→receiver flow

```rust
pub struct TxnFlowDiagram<'a> {
    sender: &'a str,
    receiver: &'a str,
    amount: Option<String>,
    action: Option<String>,
}
```

### 6.3 Implement Constructor
- [ ] `pub fn new(sender: &str, receiver: &str) -> Self`
- [ ] `pub fn with_amount(mut self, amount: String) -> Self`
- [ ] `pub fn with_action(mut self, action: String) -> Self`

### 6.4 Implement Widget Trait
- [ ] Render ASCII flow diagram
- [ ] Format: `[Sender] ──amount──> [Receiver]`
- [ ] Handle truncation for long addresses

### 6.5 Documentation & Verification
- [ ] Document widget
- [ ] Clippy passes

---

## Task 7: Create `src/widgets/detail/visual_card.rs`

### Status: NOT_STARTED

### 7.1 Extract TxnVisualCard from widgets.rs
- [ ] Locate `TxnVisualCard` struct (around line 741)
- [ ] Locate Widget impl (around line 1280)
- [ ] Note: This is a large component (~560 lines)

### 7.2 Implement TxnVisualCard
- [ ] Define struct for transaction card

```rust
pub struct TxnVisualCard<'a> {
    transaction: &'a Transaction,
    show_details: bool,
}
```

### 7.3 Implement Constructor
- [ ] `pub fn new(transaction: &Transaction) -> Self`
- [ ] `pub fn with_details(mut self, show: bool) -> Self`

### 7.4 Implement Widget Trait
- [ ] Render transaction ID header
- [ ] Render type badge
- [ ] Render flow diagram
- [ ] Render type-specific details

### 7.5 Type-Specific Rendering
- [ ] Payment details section
- [ ] Asset transfer details section
- [ ] App call details section
- [ ] Asset config details section
- [ ] Other type sections

### 7.6 Consider Refactoring
- [ ] Extract detail renderers for each type
- [ ] Or use match with helper methods

### 7.7 Documentation & Verification
- [ ] Document card layout
- [ ] Clippy passes

---

## Task 8: Create Module Files

### Status: NOT_STARTED

### 8.1 Create `src/widgets/graph/mod.rs`
```rust
mod renderer;
mod svg_export;
mod txn_graph;
mod types;

pub use renderer::TxnGraphWidget;
pub use svg_export::{SvgExportConfig, SvgExporter};
pub use txn_graph::TxnGraph;
pub use types::{GraphColumn, GraphEntityType, GraphRepresentation, GraphRow};
```

### 8.2 Create `src/widgets/detail/mod.rs`
```rust
mod flow_diagram;
mod visual_card;

pub use flow_diagram::TxnFlowDiagram;
pub use visual_card::TxnVisualCard;
```

---

## Task 9: Write Tests

### Status: NOT_STARTED

### 9.1 Graph Data Tests
- [ ] Test TxnGraph construction from transaction
- [ ] Test column deduplication
- [ ] Test inner transaction handling

### 9.2 SVG Export Tests
- [ ] Test SVG output is valid XML
- [ ] Test dimensions calculation
- [ ] Test color mapping

### 9.3 Widget Tests
- [ ] Test flow diagram rendering
- [ ] Test visual card for each transaction type

---

## Task 10: Final Checklist

### Status: NOT_STARTED

- [ ] All 8 files created
- [ ] `cargo build` succeeds
- [ ] `cargo test --all-features` passes
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] No modifications to existing files
- [ ] SVG export produces valid output
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

**SVG export separated:**
- 

**Blocked issues:**
- 

**Notes for coordinator:**
- Worker A creates widgets/list/ and widgets/common/ in parallel
- Final widgets/mod.rs will need to merge both workers' modules
- SVG export is now separate concern from graph data
