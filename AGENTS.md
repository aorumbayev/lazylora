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

## Code Principles

### Guidelines

```
# Simplicity
1.  Delete code over adding abstraction.
2.  Fewer files; split at ~500 lines.
3.  Hardcode first, configure later (if ever).
4.  Abstract after 2-3 duplications.
5.  Static dispatch over dynamic; simple `if` over clever patterns.

# Rust Idioms
6.  `?` chains over nested match arms.
7.  Iterators over index loops; `collect` over `push`.
8.  `expect("reason")` over bare `unwrap()`.
9.  Owned types in structs; borrows in functions.
10. Enums over type flags; newtypes over primitives.

# Dependencies
11. std > well-maintained crate > DIY.
12. One way to do X. Pick it. Document it. Move on.

# File Operations
13. Use Unix CLI tools (`sed`, `awk`, `head`, `tail`) for bulk file ops.

# Documentation
14. Document WHY, not WHAT. Update or delete stale docs immediately.

# TUI Architecture
15. State is data; UI is `fn(&State) -> Frame`.
16. Commands for input, Messages for async.
17. Layout at top, pass `Rect` down, widgets are ephemeral.
18. Async spawns outside render; sync inside.
19. Target 80x24 first.

# Testing
20. Snapshot tests over many cell assertions.
21. rstest fixtures over copy-pasted setup.

# Meta
22. Ship it.
23. Naming is design. If you can't name it simply, redesign it.
```

### Code Smells

**Over-engineered:**
- Trait with one implementation
- Builder for struct with ≤3 fields
- `Arc<Mutex<RefCell<T>>>`
- More than 2 type parameters
- Module tree deeper than 3 levels

**Unidiomatic Rust:**
- `for i in 0..vec.len()` instead of `.iter()`
- `.clone()` to satisfy borrow checker
- `&String` or `&Vec<T>` in parameters (use `&str`, `&[T]`)
- `unsafe` without `// SAFETY:` comment

**Bad error handling:**
- Bare `unwrap()` outside tests
- Errors swallowed with `let _ = ...`
- Error messages missing context

**Unnecessary docs:**
- Function name already says what it does
- Restating the type signature
- Private helper called from one place

**Docs needed when:**
- Non-obvious invariants exist
- Subtle gotchas or panics
- Public API

**Over-specified tests:**
- Tests break when visual spacing changes
- 10+ tests for same widget with minor variations

**Under-engineered tests:**
- Setup code copy-pasted across 3+ tests
- Tests hit the network unnecessarily

---

## Code Style

- **Rust edition**: 2024
- **Imports**: std first, external crates, local modules
- **Naming**: PascalCase types, snake_case functions, SCREAMING_SNAKE_CASE constants
- **Error handling**: `color_eyre::Result`, custom errors via `thiserror`
- **Attributes**: `#[must_use]` on pure functions returning values
- **Testing**: Tests in same file with `#[cfg(test)]`; use rstest for fixtures

---

## Architecture

### State Decomposition (TEA-inspired)

```rust
pub struct App {
    pub navigation: NavigationState,
    pub data: DataState,
    pub ui: UiState,
}
```

Each state type has clear responsibilities. Avoids monolithic state.

### Command Pattern for Input

```rust
pub enum Command {
    Quit,
    MoveUp,
    MoveDown,
    Select,
    CycleFocus,
}
```

Decouples key bindings from behavior.

### Message Passing for Async

```rust
pub enum Message {
    BlocksLoaded(Vec<Block>),
    TransactionLoaded(Transaction),
    Error(String),
}
```

Separates async I/O from sync state updates.

### Builder-Lite Pattern

Ratatui methods consume `self` and return modified `Self`:

```rust
// Correct
let paragraph = Paragraph::new("text")
    .block(Block::bordered())
    .centered();

// Wrong - result discarded
let mut paragraph = Paragraph::new("text");
paragraph.centered();
```

### Keybindings

```rust
match key.code {
    KeyCode::Char('q') | KeyCode::Esc => self.quit(),
    KeyCode::Char('j') | KeyCode::Down => self.next(),
    KeyCode::Char('k') | KeyCode::Up => self.prev(),
    KeyCode::Enter => self.select(),
    _ => {}
}
```

### TUI Anti-Patterns

```rust
// Bad: Widget holds state
struct App {
    list: List<'static>,
}

// Good: State in App, widgets ephemeral
struct App {
    items: Vec<String>,
    list_state: ListState,
}
```

```rust
// Bad: Blocks render loop
let data = reqwest::blocking::get(url)?;

// Good: Spawn async task
self.start_fetch();
```

---

## Testing

### Decision Tree

```
Visual? → Snapshot test (insta)
Pure function? → Unit test (rstest parametrized or table-driven)
State/logic? → Test transitions, not getters
Otherwise → Probably don't need a test
```

### rstest Example

```rust
#[fixture]
fn test_terminal() -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(80, 24)).unwrap()
}

#[rstest]
#[case::empty_graph(TxnGraph::new(), true, 8)]
#[case::custom_width(TxnGraph::new().with_column_width(12), true, 12)]
fn test_graph_config(
    #[case] graph: TxnGraph,
    #[case] expected_empty: bool,
    #[case] expected_width: usize,
) {
    assert_eq!(graph.is_empty(), expected_empty);
    assert_eq!(graph.column_width, expected_width);
}
```

### Snapshot Testing

```rust
#[rstest]
fn test_widget_snapshot(test_terminal: Terminal<TestBackend>) {
    let mut terminal = test_terminal;
    terminal.draw(|frame| {
        render_widget(frame, frame.area());
    }).unwrap();
    assert_snapshot!(terminal.backend());
}
```

Use consistent sizes: `80x24` or `120x40`.

### Test Data: Mother Pattern

```rust
pub struct TransactionMother;

impl TransactionMother {
    pub fn payment() -> Transaction {
        Transaction {
            id: "PAY123".to_string(),
            txn_type: TxnType::Payment,
            amount: 5_000_000,
            ..Default::default()
        }
    }

    pub fn app_call() -> Transaction {
        Transaction {
            txn_type: TxnType::AppCall,
            ..Self::payment()
        }
    }
}
```

### What NOT to Test

- Ratatui internals
- Trivial getters/setters
- Network calls in unit tests

### Running Tests

```bash
cargo t --all-features          # Run all
cargo t <test_name>             # Run specific test
cargo insta review              # Review snapshot changes
```

---

## Quick Reference

### Prefer

```rust
let data = fetch_data()?;
let file = File::open(path).expect("config file should exist");
let sum: u64 = items.iter().map(|x| x.amount).sum();
if let Some(block) = selected_block { render_block(block); }
let widget = MyWidget::new(data).focused(true);
```

### Avoid

```rust
match result { Ok(v) => v, Err(e) => return Err(e) }
for i in 0..vec.len() { sum += vec[i]; }
Arc<Mutex<RefCell<Option<T>>>>
thing.unwrap();
```

---

## Before Adding Abstraction

1. Can I delete this instead?
2. Will this have 2+ implementations within 6 months?
3. Would a new team member understand this in 5 minutes?
4. Am I solving a real problem or an imaginary one?

If any answer is "no" - don't abstract.

---

## CI Checklist

1. `cargo t --all-features` - Tests pass
2. `cargo clippy --all-features -- -D warnings` - No warnings
3. `cargo fmt -- --check` - Formatted
4. Snapshot files committed
