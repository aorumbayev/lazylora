# AGENTS.md - AI Coding Agent Instructions

## Build/Test/Lint Commands
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release binary
- `cargo test --all-features` - Run all tests
- `cargo test <test_name>` - Run a single test by name
- `cargo fmt -- --check` - Check formatting
- `cargo clippy --all-features -- -D warnings` - Lint with warnings as errors

## Code Style Guidelines
- **Rust edition**: 2024
- **Imports**: Group std lib first, then external crates, then local modules
- **Naming**: PascalCase for types/enums, snake_case for functions, SCREAMING_SNAKE_CASE for constants
- **Error handling**: Use `color_eyre::Result`, custom errors via `thiserror`, document with `# Errors` section
- **Attributes**: Use `#[must_use]` on pure functions returning values
- **Docs**: Use `///` for public API with `# Arguments`, `# Returns`, `# Errors` sections
- **Organization**: Module separators with `// ===...`, state decomposition, command pattern for input
- **Testing**: Tests in same file with `#[cfg(test)]` modules

## Testing Guidelines for LazyLora

This section provides comprehensive guidelines for writing tests in LazyLora, a Ratatui-based TUI application.

### Test Organization

All tests should be placed in the same file as the code they test, within a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Test implementation
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run a specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture

# Run tests matching a pattern
cargo test widget
```

### Types of Tests

#### 1. Unit Tests for Pure Functions

Test utility functions, formatters, and pure logic independently:

```rust
#[test]
fn test_format_algo_amount() {
    assert_eq!(format_algo_amount(0), "0.000000 ALGO");
    assert_eq!(format_algo_amount(1_000_000), "1.000000 ALGO");
    assert_eq!(format_algo_amount(5_500_000), "5.500000 ALGO");
}

#[test]
fn test_truncate_address_long() {
    let addr = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    let result = truncate_address(addr, 20);
    assert_eq!(result.len(), 20);
    assert!(result.contains("..."));
}
```

#### 2. Widget Rendering Tests with TestBackend

Use Ratatui's `TestBackend` to test widget rendering without an actual terminal:

```rust
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn test_widget_renders_correctly() {
    // Create a test terminal with fixed dimensions
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let widget = MyWidget::new(/* test data */);

    terminal.draw(|frame| {
        frame.render_widget(&widget, frame.area());
    }).unwrap();

    // Access the buffer for assertions
    let buffer = terminal.backend().buffer();
    
    // Assert specific content at positions
    assert_eq!(buffer.get(0, 0).symbol(), "E"); // Check cell content
}
```

#### 3. Snapshot Testing with Insta (Recommended for UI)

For comprehensive UI regression testing, use the `insta` crate:

**Setup:**
```bash
# Install cargo-insta CLI tool
cargo install cargo-insta

# Add insta as dev dependency
cargo add insta --dev
```

**Add to Cargo.toml:**
```toml
[dev-dependencies]
insta = "1.40"
```

**Writing Snapshot Tests:**
```rust
use insta::assert_snapshot;
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn test_main_screen_snapshot() {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let app = App::default();

    terminal.draw(|frame| {
        frame.render_widget(&app, frame.area());
    }).unwrap();

    // Captures the entire terminal output as a snapshot
    assert_snapshot!(terminal.backend());
}

#[test]
fn test_transaction_list_snapshot() {
    let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
    let transactions = vec![/* test data */];
    let widget = TransactionList::new(&transactions);

    terminal.draw(|frame| {
        frame.render_widget(&widget, frame.area());
    }).unwrap();

    assert_snapshot!("transaction_list_default", terminal.backend());
}
```

**Managing Snapshots:**
```bash
# Run tests (creates pending snapshots for new tests)
cargo test

# Review and accept/reject snapshot changes
cargo insta review

# Accept all pending snapshots
cargo insta accept

# Reject all pending snapshots
cargo insta reject
```

**Snapshot Best Practices:**
- Use consistent terminal dimensions (e.g., `80x24`, `60x20`) for reproducibility
- Name snapshots descriptively when testing multiple states
- Review snapshots carefully after UI changes
- Commit `.snap` files to version control
- Note: Color assertions are not yet supported in Ratatui snapshots

#### 4. State and Logic Tests

Test application state transitions and business logic separately from rendering:

```rust
#[test]
fn test_animation_phase_transitions() {
    let mut screen = BootScreen::new((80, 24));

    // Test initial state
    assert_eq!(screen.animation_phase, AnimationPhase::Pulsing);

    // Test state transition
    screen.update_animation_phase(Duration::from_millis(2100));
    assert_eq!(screen.animation_phase, AnimationPhase::Complete);
}

#[test]
fn test_command_handling() {
    let mut app_state = AppState::default();
    
    // Simulate command
    let result = app_state.handle_command(Command::NextPage);
    
    assert!(result.is_ok());
    assert_eq!(app_state.current_page, 1);
}
```

#### 5. Domain Model Tests

Test domain types, parsing, and serialization:

```rust
#[test]
fn test_transaction_parsing() {
    let json = r#"{"id": "ABC123", "type": "pay", "amount": 1000000}"#;
    let txn: Transaction = serde_json::from_str(json).unwrap();
    
    assert_eq!(txn.id, "ABC123");
    assert_eq!(txn.txn_type, TxnType::Payment);
}

#[test]
fn test_account_balance_calculation() {
    let account = Account {
        amount: 5_000_000,
        // ...
    };
    
    assert_eq!(account.algo_balance(), 5.0);
}
```

### Testing Patterns for TUI Applications

#### Separation of Concerns

Structure code to separate testable logic from terminal I/O:

```rust
// Good: Pure function that can be unit tested
fn calculate_visible_items(total: usize, page_size: usize, offset: usize) -> Range<usize> {
    let start = offset.min(total);
    let end = (offset + page_size).min(total);
    start..end
}

// Good: Widget that can be snapshot tested
impl Widget for TransactionList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Rendering logic
    }
}

// Keep event handling separate from state updates
fn handle_key_event(key: KeyEvent, state: &mut AppState) -> Option<Action> {
    match key.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Down => {
            state.select_next();
            None
        }
        _ => None,
    }
}
```

#### Testing with Mock Data

Create test fixtures for consistent testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transaction() -> Transaction {
        Transaction {
            id: "TEST123".to_string(),
            txn_type: TxnType::Payment,
            sender: "SENDER...".to_string(),
            receiver: Some("RECEIVER...".to_string()),
            amount: 1_000_000,
            fee: 1000,
            // ...
        }
    }

    fn create_test_block() -> Block {
        Block {
            round: 12345,
            timestamp: 1234567890,
            transactions: vec![create_test_transaction()],
            // ...
        }
    }

    #[test]
    fn test_with_mock_data() {
        let txn = create_test_transaction();
        // Use mock data in tests
    }
}
```

#### Debugging Widget State

Since `println!` and `dbg!` don't work in TUI mode, use these approaches:

1. **Implement Debug toggle for development:**
```rust
#[derive(Debug, Default)]
struct AppState {
    show_debug: bool,
    // ... other fields
}

fn render(frame: &mut Frame, state: &AppState) {
    // Normal rendering...
    
    if state.show_debug {
        let debug_text = Text::from(format!("state: {state:#?}"));
        frame.render_widget(debug_text, debug_area);
    }
}
```

2. **Log to files during testing:**
```rust
use std::fs::OpenOptions;
use std::io::Write;

fn debug_log(msg: &str) {
    if cfg!(test) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/lazylora_test.log")
            .unwrap();
        writeln!(file, "{}", msg).unwrap();
    }
}
```

3. **Use tui-logger crate** for advanced logging in TUI applications.

### Test Coverage Guidelines

When adding new features, ensure tests cover:

1. **Happy path** - Normal expected behavior
2. **Edge cases** - Empty inputs, maximum values, boundary conditions
3. **Error handling** - Invalid inputs, network failures (mocked)
4. **State transitions** - All valid state changes
5. **Rendering** - Widget appearance with various data states

### Example: Complete Test Module

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    // Test fixtures
    fn sample_transactions() -> Vec<Transaction> {
        vec![
            Transaction { id: "TX1".into(), amount: 1_000_000, ..Default::default() },
            Transaction { id: "TX2".into(), amount: 2_000_000, ..Default::default() },
        ]
    }

    // Unit tests
    #[test]
    fn test_format_amount() {
        assert_eq!(format_algo_amount(1_000_000), "1.000000 ALGO");
    }

    // Widget rendering test
    #[test]
    fn test_transaction_list_renders() {
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let txns = sample_transactions();

        terminal.draw(|f| {
            let widget = TransactionListWidget::new(&txns);
            f.render_widget(widget, f.area());
        }).unwrap();

        let buffer = terminal.backend().buffer();
        // Verify header is rendered
        assert!(buffer.get(0, 0).symbol().contains("T"));
    }

    // State transition test
    #[test]
    fn test_selection_wraps_around() {
        let mut state = ListState::default();
        state.items_count = 3;
        state.selected = Some(2);

        state.select_next();
        
        assert_eq!(state.selected, Some(0)); // Wrapped to start
    }

    // Snapshot test (requires insta)
    #[test]
    fn test_transaction_list_snapshot() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let txns = sample_transactions();

        terminal.draw(|f| {
            f.render_widget(TransactionListWidget::new(&txns), f.area());
        }).unwrap();

        insta::assert_snapshot!(terminal.backend());
    }
}
```

### CI Integration

Tests run automatically in CI via GitHub Actions. Ensure:

1. All tests pass locally before pushing: `cargo test --all-features`
2. No clippy warnings: `cargo clippy --all-features -- -D warnings`
3. Code is formatted: `cargo fmt -- --check`
4. Snapshot files are committed if using insta
