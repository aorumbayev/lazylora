# AGENTS.md - AI Coding Agent Instructions

## Build/Test/Lint Commands
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release binary
- `cargo t --all-features` - Run all tests (uses nextest)
- `cargo t <test_name>` - Run a single test by name
- `cargo fmt -- --check` - Check formatting
- `cargo clippy --all-features -- -D warnings` - Lint with warnings as errors

**First-time setup:** Install nextest with `cargo install cargo-nextest` (or faster: `cargo binstall cargo-nextest`)

---

## The Zen of LazyLora

Before writing any code, internalize these principles. They are the soul of this codebase.

### The 18 Commandments

```
1.  Short comments over long explanations.
2.  Delete code over adding abstraction.
3.  One file over three "clean" modules.
4.  Match statement over visitor pattern.
5.  Hardcode values over configuration systems.
6.  Copy-paste twice before abstracting once.
7.  Snapshot test over twenty cell assertions.
8.  rstest fixture over manual setup duplication.
9.  Standard library over popular crate over custom solution.
10. Boring code over clever code.
11. `?` operator over manual Result matching.
12. Iterators over manual loops.
13. State decomposition over monolithic state.
14. Command pattern for input, messages for async.
15. `expect()` with context over bare `unwrap()`.
16. Builder-lite pattern over mutable configuration.
17. `let chains` over nested if-let (Rust 2024).
18. Ship it over perfect it.
```

### The Sniff Tests

**Your code is probably over-engineered if:**
- A trait has only one implementation
- You created a `Factory` that creates one thing
- You have a `Builder` for a struct with ≤3 fields
- There's a `Manager` that manages nothing but itself
- Wrapper types just delegate every method
- You're using Strategy pattern where `if` would do
- `Arc<Mutex<RefCell<T>>>` appears anywhere
- More than 2 hops from input to output

**Your comment is probably AI-generated if it contains:**
- "API surface" or "completeness"
- "comprehensive" or "ensuring consistency"
- "provides a complete..." or "maintaining compatibility"
- Multiple justifications for one thing
- More than 2 lines for a lint suppression

**Your tests are probably over-specified if:**
- `assert_eq!(buffer[(x, y)].symbol(), "╭")` appears more than twice
- You're manually calculating expected coordinates
- Tests break when visual spacing changes
- 10+ tests for the same widget with minor variations
- You have tests but users still report visual bugs

**Your tests are probably under-engineered if:**
- Same setup code copy-pasted across 5+ tests
- Test fails and you can't tell which case failed
- Tests hit the network when they don't need to
- Adding a new test case requires 20+ lines of boilerplate

---

## Code Style Guidelines

- **Rust edition**: 2024 (uses `Future`/`IntoFuture` in prelude, `let chains`, new match ergonomics)
- **Imports**: Group std lib first, then external crates, then local modules
- **Naming**: PascalCase for types/enums, snake_case for functions, SCREAMING_SNAKE_CASE for constants
- **Error handling**: Use `color_eyre::Result`, custom errors via `thiserror`, document with `# Errors` section
- **Attributes**: Use `#[must_use]` on pure functions returning values
- **Docs**: Use `///` for public API with `# Arguments`, `# Returns`, `# Errors` sections (keep them SHORT)
- **Organization**: Module separators with `// ===...`, state decomposition, command pattern for input
- **Testing**: Tests in same file with `#[cfg(test)]` modules; use rstest for fixtures/parametrization

---

## Architecture Patterns (How LazyLora Works)

### State Decomposition (TEA-inspired)

The `App` struct decomposes into focused state types, following The Elm Architecture pattern:

```rust
pub struct App {
    pub navigation: NavigationState,  // Focus, selection, panels
    pub data: DataState,              // Blocks, transactions, accounts
    pub ui: UiState,                  // Popups, toasts, search
    // ...
}
```

**Why?** Each state type has clear responsibilities. Avoids 50-field monolithic state. Aligns with Model-Update-View pattern.

### Command Pattern for Input

All key events map to `Command` enum variants:

```rust
pub enum Command {
    Quit,
    MoveUp,
    MoveDown,
    Select,
    CycleFocus,
    // ...
}
```

**Why?** Decouples key bindings from behavior. Easy to test. One place to see all actions. Enables state machine reasoning.

### Message Passing for Async

Background tasks communicate via channels:

```rust
pub enum Message {
    BlocksLoaded(Vec<Block>),
    TransactionLoaded(Transaction),
    Error(String),
    // ...
}
```

**Why?** Clean separation between async I/O and sync state updates. Follows TEA's update-from-message pattern.

### Builder-Lite Pattern for Widgets

Ratatui uses builder-lite pattern - methods consume `self` and return modified `Self`:

```rust
// Good: chained builder-lite
let paragraph = Paragraph::new("text")
    .block(Block::bordered())
    .centered();

// Bad: trying to mutate after creation
let mut paragraph = Paragraph::new("text");
paragraph.centered();  // This returns a NEW value, doesn't mutate!
```

**Why?** Enables fluent API. Methods marked `#[must_use]` catch mistakes at compile time.

---

## Testing Guidelines

### The Golden Rule

**One snapshot test > Twenty cell assertions.**

### Test Decision Tree

```
Is it visual?
├─ Yes → Snapshot test (insta)
│   └─ Multiple states? → One multi-state snapshot
└─ No → Is it a pure function?
    ├─ Yes → Unit test (rstest parametrized or table-driven)
    └─ No → Is it state/logic?
        ├─ Yes → Test transitions, not getters
        └─ No → Probably don't need a test
```

### rstest for Fixtures and Parametrization

Use `rstest` for pytest-like fixtures and parametrized tests:

```rust
use rstest::*;

// Fixture: reusable test setup
#[fixture]
fn test_terminal() -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(80, 24)).unwrap()
}

// Parametrized test: one function, many cases
#[rstest]
#[case::empty_graph(TxnGraph::new(), true, 8, 3)]
#[case::custom_width(TxnGraph::new().with_column_width(12), true, 12, 3)]
fn test_graph_config(
    #[case] graph: TxnGraph,
    #[case] expected_empty: bool,
    #[case] expected_width: usize,
    #[case] expected_spacing: usize,
) {
    assert_eq!(graph.is_empty(), expected_empty);
    assert_eq!(graph.column_width, expected_width);
    assert_eq!(graph.column_spacing, expected_spacing);
}

// Fixture injection
#[rstest]
fn test_widget_renders(test_terminal: Terminal<TestBackend>) {
    // terminal is automatically created by fixture
}

// Async fixture
#[fixture]
async fn mock_app() -> App {
    App::new(StartupOptions::default()).await.unwrap()
}

#[rstest]
#[tokio::test]
async fn test_app_state(#[future] mock_app: App) {
    let app = mock_app.await;
    assert!(app.data.blocks.is_empty());
}
```

**When to use rstest:**
- Same setup code in 3+ tests → Extract to `#[fixture]`
- Testing same logic with different inputs → Use `#[case]`
- Matrix of combinations → Use `#[values]`

**When NOT to use rstest:**
- Single test with unique setup → Plain `#[test]`
- Simple assertions → Table-driven with manual loop is fine

### Snapshot Testing (Preferred for UI)

```rust
use insta::assert_snapshot;
use ratatui::{backend::TestBackend, Terminal};

#[rstest]
fn test_transaction_popup_visual_modes(test_terminal: Terminal<TestBackend>) {
    let mut terminal = test_terminal;
    
    terminal.draw(|frame| {
        // Render widget in multiple states in one snapshot
        render_popup_table_mode(frame, areas[0]);
        render_popup_visual_mode(frame, areas[1]);
    }).unwrap();

    assert_snapshot!(terminal.backend());
}
```

**Why snapshots?**
- Captures entire visual state
- Survives refactors
- Easy regression detection
- Human-reviewable diffs
- Use consistent sizes: `80x24` or `120x40`

**Note:** Color assertions not yet supported in Ratatui snapshots (see ratatui#1402).

### Test Data: Mother Pattern

Use the "Mother" pattern for creating test objects (inspired by algokit-core):

```rust
/// Test data factory - creates common test objects
pub struct TransactionMother;

impl TransactionMother {
    pub fn payment() -> Transaction {
        Transaction {
            id: "PAY123".to_string(),
            txn_type: TxnType::Payment,
            from: "SENDER_ADDR".to_string(),
            to: "RECEIVER_ADDR".to_string(),
            amount: 5_000_000,
            ..Default::default()
        }
    }

    pub fn app_call() -> Transaction {
        Transaction {
            txn_type: TxnType::AppCall,
            to: "12345".to_string(), // App ID
            ..Self::payment()
        }
    }

    pub fn with_inner(parent: Transaction, children: Vec<Transaction>) -> Transaction {
        Transaction { inner_transactions: children, ..parent }
    }
}
```

**Why Mother pattern?**
- Centralized test data creation
- Composable (build complex objects from simple ones)
- Self-documenting (method names describe the variant)
- Easy to update when domain changes

### Table-Driven Tests (Alternative to rstest)

Still valid for simple pure function tests:

```rust
#[test]
fn test_format_algo_amount() {
    let cases = [
        (0, "0.000000 ALGO"),
        (1_000_000, "1.000000 ALGO"),
        (5_500_000, "5.500000 ALGO"),
        (123_456_789, "123.456789 ALGO"),
    ];
    
    for (input, expected) in cases {
        assert_eq!(format_algo_amount(input), expected, "input: {}", input);
    }
}
```

**When to prefer table-driven over rstest:**
- Simple input → output mapping
- No setup/teardown needed
- Cases are trivially similar

### What NOT to Test

- Ratatui internals (don't test the framework)
- Trivial getters/setters
- Layout math you'll never change
- Color-specific assertions (snapshots don't capture colors yet)
- Network calls in unit tests (use static fixtures instead)

### Running Tests

```bash
cargo t --all-features          # Run all (uses nextest via alias)
cargo t <test_name>             # Run specific test
cargo insta review              # Review snapshot changes
cargo insta accept              # Accept all pending
```

**Note:** `cargo t` is aliased to `cargo nextest run` in `.cargo/config.toml`.

---

## Quick Reference: Idiomatic Rust 2024 in LazyLora

### Prefer

```rust
// ? over match for error propagation
let data = fetch_data()?;

// expect() with context over unwrap()
let file = File::open(path).expect("config file should exist");

// Iterators over loops
let sum: u64 = items.iter().map(|x| x.amount).sum();

// if let over match with one arm
if let Some(block) = selected_block {
    render_block(block);
}

// let chains (Rust 2024) over nested if-let
if let Some(x) = foo() && let Some(y) = bar(x) && y > 0 {
    process(y);
}

// impl Into<T> for flexibility
fn set_title(title: impl Into<String>) { ... }

// #[derive] over manual impls
#[derive(Debug, Clone, Default)]
struct Config { ... }

// Builder-lite pattern (Ratatui style)
let widget = MyWidget::new(data).focused(true).title("Header");

// unwrap_or_else for complex defaults
let value = result.unwrap_or_else(|_| compute_default());

// rstest fixtures over manual setup
#[fixture]
fn terminal() -> Terminal<TestBackend> { ... }
```

### Avoid

```rust
// Manual Result matching when ? works
match result {
    Ok(v) => v,
    Err(e) => return Err(e),
}

// Manual loops when iterators work
let mut sum = 0;
for item in items {
    sum += item.amount;
}

// Traits with one impl
trait DataFetcher { ... }
impl DataFetcher for AlgorandFetcher { ... }  // Only impl ever

// Over-nested types
Arc<Mutex<RefCell<Option<T>>>>

// Bare unwrap() in library code
let x = thing.unwrap();  // Use expect() or ? instead

// Forgetting builder-lite returns new value
let widget = Widget::new();
widget.focused(true);  // BUG: result discarded!

// Copy-pasting setup across many tests
fn test_a() { let terminal = Terminal::new(...); ... }
fn test_b() { let terminal = Terminal::new(...); ... }  // Use fixture!
```

---

## The Refactor Test

Before adding abstraction, ask:

1. Can I delete this instead?
2. Will this have 2+ implementations within 6 months?
3. Does the abstraction name describe WHAT, not HOW?
4. Would a new team member understand this in 5 minutes?
5. Am I solving a real problem or an imaginary future one?

**If any answer is "no" — don't abstract. Inline it. Hardcode it. Ship it.**

---

## Example: Complete Test Module

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use ratatui::{backend::TestBackend, Terminal};

    // ============================================================================
    // Fixtures
    // ============================================================================

    #[fixture]
    fn test_terminal() -> Terminal<TestBackend> {
        Terminal::new(TestBackend::new(80, 24)).unwrap()
    }

    #[fixture]
    fn sample_transactions() -> Vec<Transaction> {
        vec![
            Transaction { id: "TX1".into(), amount: 1_000_000, ..Default::default() },
            Transaction { id: "TX2".into(), amount: 2_000_000, ..Default::default() },
        ]
    }

    // ============================================================================
    // Parametrized Tests
    // ============================================================================

    #[rstest]
    #[case::zero(0, "0.000000 ALGO")]
    #[case::one_algo(1_000_000, "1.000000 ALGO")]
    #[case::fractional(5_500_000, "5.500000 ALGO")]
    fn test_format_amount(#[case] input: u64, #[case] expected: &str) {
        assert_eq!(format_algo_amount(input), expected);
    }

    // ============================================================================
    // Snapshot Tests
    // ============================================================================

    #[rstest]
    fn test_transaction_list_snapshot(
        test_terminal: Terminal<TestBackend>,
        sample_transactions: Vec<Transaction>,
    ) {
        let mut terminal = test_terminal;

        terminal.draw(|f| {
            f.render_widget(TransactionListWidget::new(&sample_transactions), f.area());
        }).unwrap();

        insta::assert_snapshot!(terminal.backend());
    }

    // ============================================================================
    // State Transition Tests
    // ============================================================================

    #[rstest]
    #[case::wrap_forward(2, 0)]  // Last item → first
    #[case::normal_forward(0, 1)]
    #[case::middle_forward(1, 2)]
    fn test_selection_next(#[case] start: usize, #[case] expected: usize) {
        let mut state = ListState::default();
        state.items_count = 3;
        state.selected = Some(start);
        state.select_next();
        assert_eq!(state.selected, Some(expected));
    }
}
```

---

## CI Integration

Tests run automatically in CI via GitHub Actions. Before pushing:

1. `cargo t --all-features` - All tests pass
2. `cargo clippy --all-features -- -D warnings` - No warnings
3. `cargo fmt -- --check` - Properly formatted
4. Snapshot files (`.snap`) committed if using insta

---

## Remember

> "10 lines of 'bad' code > 50 lines of 'clean' code"

When in doubt: fewer files, fewer abstractions, more directness. Ship it.
