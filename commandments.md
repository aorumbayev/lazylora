# The Zen of Human Code Comments

## The 10 Commandments

```
1.  Short over long.
2.  One reason, not three.
3.  No philosophy.
4.  Why, not what.
5.  Lint suppression needs five words max.
6.  Section headers, not section essays.
7.  One line or it's wrong.
8.  TODO over "future extensibility."
9.  Explain the weird, not the obvious.
10. Silence is golden.
```

## The Sniff Test

Your comment is probably AI-generated if it contains:
- "API surface"
- "completeness"
- "comprehensive"
- "ensuring consistency"
- "provides a complete..."
- "even if not all... are currently..."
- "maintaining compatibility"
- Multiple justifications for one thing
- More than 2 lines for a lint suppression

---

# The Zen of Human Code

## The Over-Engineering Sniff Test

Your code is probably over-engineered if:

**Abstraction Smell**
- A trait/interface has only one implementation (and no plans for more)
- You have a `Factory` that creates one thing
- A `Builder` for a struct with 3 fields
- A `Manager` that manages nothing but itself
- Wrapper types that just delegate every method

**Pattern Smell**
- Using Strategy pattern where an `if` would do
- Using Command pattern for non-undoable, non-queueable actions
- Event systems for two components that just talk to each other
- Dependency injection frameworks for 5 dependencies

**Layering Smell**
- `Controller` calls `Service` calls `Repository` calls `DAO` for a CRUD app
- More than 2 hops to get from input to output
- "Clean Architecture" with 50 lines of business logic
- Separate modules for "domain", "application", "infrastructure" in a CLI tool

**Type Smell**
- `Result<Result<T, E1>, E2>` - just pick an error type
- `Option<Option<T>>` - rethink your model
- `Arc<Mutex<RefCell<T>>>` - something went wrong
- Generic over things that are never generic in practice

**Future-Proofing Smell**
- "We might need to swap databases" (you won't)
- "What if we need to support XML?" (you won't)
- "This should be configurable" (hardcode it)
- Interfaces for "testability" with no tests

## The Simplicity Principles

```
1.  Delete code > Add abstraction.
2.  One file > Three "clean" modules.
3.  Function > Method > Trait > Framework.
4.  Match statement > Visitor pattern.
5.  Hardcode > Config > Plugin system.
6.  Copy-paste twice > Abstract once.
7.  10 lines of "bad" code > 50 lines of "clean" code.
8.  Standard library > Popular crate > Custom solution.
9.  Boring code > Clever code.
10. Ship it > Perfect it.
```

## Idiomatic Shortcuts by Language

**Rust**
- `?` over manual `match` on `Result`
- `impl Into<T>` over concrete types in args
- `#[derive]` over manual trait impls
- Iterators over manual loops
- `if let` over `match` with one arm

**TypeScript**
- Object spread over `Object.assign`
- Optional chaining over nested ifs
- `Record<K,V>` over `{[key: K]: V}`
- `as const` over manual type narrowing

**Python**
- List comprehension over `map`/`filter`
- `dataclass` over manual `__init__`
- `pathlib` over `os.path`
- Context managers over try/finally
- `f-strings` over `.format()`

## The Refactor Test

Before adding abstraction, ask:
1. Can I delete this instead?
2. Will this have 2+ implementations within 6 months?
3. Does the abstraction name describe WHAT, not HOW?
4. Would a new team member understand this in 5 minutes?
5. Am I solving a real problem or an imaginary future one?

If any answer is "no" — don't abstract. Inline it. Hardcode it. Ship it.

---

# The Zen of TUI Testing

## The Test Consolidation Principles

```
1.  One snapshot test > Twenty cell assertions.
2.  User sees screen > Code sees buffer cells.
3.  Behavior over implementation.
4.  One happy path snapshot > Five edge case unit tests.
5.  TestBackend + insta = elegance.
6.  Test what the user perceives.
7.  Regression prevention > Exhaustive coverage.
8.  Readable snapshots > Clever assertions.
9.  Fixture once, snapshot many.
10. If you're counting cells, stop.
```

## The TUI Test Smell Detection

Your TUI tests are over-specified if:

**Cell-by-Cell Smell**
- `assert_eq!(buffer[(x, y)].symbol(), "╭")` appears more than twice
- You're manually calculating expected coordinates
- Tests break when visual spacing changes
- You have to update 15 assertions for a border change

**Test Proliferation Smell**
- 10+ tests for the same widget with minor data variations
- Separate tests for focused/unfocused that could be one snapshot
- Each key mapping gets its own test function
- Tests duplicate the implementation logic in assertions

**False Confidence Smell**
- Tests pass but users report visual bugs
- 100% line coverage, 0% visual coverage
- Tests check that "something renders" not "renders correctly"
- No snapshot to actually see what was rendered

## The Snapshot-First Approach

### Replace Cell Assertions with Snapshots

**Before (verbose, brittle, 11 tests):**
```rust
#[test]
fn test_create_border_block_unfocused_renders() {
    let backend = TestBackend::new(30, 5);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| {
        let block = create_border_block("Test", false);
        frame.render_widget(block, frame.area());
    }).unwrap();
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(0, 0)].symbol(), "╭");
    assert_eq!(buffer[(29, 0)].symbol(), "╮");
}

#[test]
fn test_create_border_block_focused_renders() { /* another 10 lines */ }

#[test]
fn test_create_border_block_empty_title() { /* another 10 lines */ }
// ... 8 more similar tests
```

**After (elegant, comprehensive, 1 test):**
```rust
#[test]
fn test_border_blocks_all_states() {
    let mut terminal = Terminal::new(TestBackend::new(40, 15)).unwrap();
    
    terminal.draw(|frame| {
        let areas = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ]).split(frame.area());
        
        frame.render_widget(create_border_block("Unfocused", false), areas[0]);
        frame.render_widget(create_border_block("Focused", true), areas[1]);
        frame.render_widget(create_border_block("", false), areas[2]);
        frame.render_widget(create_popup_block("Popup"), areas[3]);
        frame.render_widget(create_border_block("Long Title Here", true), areas[4]);
    }).unwrap();

    insta::assert_snapshot!(terminal.backend());
}
```

### Replace Key Mapping Repetition with Table Tests

**Before (58 individual tests):**
```rust
#[test]
fn test_q_quits() {
    let cmd = KeyMapper::map_key(key_event(KeyCode::Char('q')), &InputContext::Main);
    assert_eq!(cmd, AppCommand::Quit);
}

#[test]
fn test_r_refreshes() {
    let cmd = KeyMapper::map_key(key_event(KeyCode::Char('r')), &InputContext::Main);
    assert_eq!(cmd, AppCommand::Refresh);
}
// ... 56 more identical patterns
```

**After (elegant table test):**
```rust
#[test]
fn test_key_mappings_main_context() {
    let cases = [
        (KeyCode::Char('q'), AppCommand::Quit),
        (KeyCode::Char('r'), AppCommand::Refresh),
        (KeyCode::Char(' '), AppCommand::ToggleLive),
        (KeyCode::Char('f'), AppCommand::OpenSearch),
        (KeyCode::Tab, AppCommand::CycleFocus),
        (KeyCode::Up, AppCommand::MoveUp),
        (KeyCode::Down, AppCommand::MoveDown),
        (KeyCode::Enter, AppCommand::Select),
        (KeyCode::F(1), AppCommand::Noop),
    ];
    
    for (key, expected) in cases {
        let cmd = KeyMapper::map_key(key_event(key), &InputContext::Main);
        assert_eq!(cmd, expected, "Key {:?} should map to {:?}", key, expected);
    }
}

#[test]
fn test_key_mappings_all_contexts_snapshot() {
    // Capture entire key mapping behavior in one readable snapshot
    let contexts = [
        InputContext::Main,
        InputContext::DetailView,
        InputContext::NetworkSelect,
        InputContext::SearchInput,
        InputContext::MessagePopup,
    ];
    
    let keys = [
        KeyCode::Char('q'), KeyCode::Esc, KeyCode::Enter,
        KeyCode::Tab, KeyCode::Up, KeyCode::Down,
    ];
    
    let mut output = String::new();
    for ctx in &contexts {
        output.push_str(&format!("\n=== {:?} ===\n", ctx));
        for key in &keys {
            let cmd = KeyMapper::map_key(key_event(*key), ctx);
            output.push_str(&format!("  {:?} -> {:?}\n", key, cmd));
        }
    }
    
    insta::assert_snapshot!(output);
}
```

### Replace Widget Unit Tests with Visual Verification

**Before (8 tests, no visual confidence):**
```rust
#[test]
fn test_transaction_list_widget_new() { /* checks len() */ }
#[test]
fn test_transaction_list_widget_empty() { /* checks is_empty() */ }
#[test]
fn test_transaction_list_widget_render_empty() { /* checks string contains */ }
#[test]
fn test_transaction_list_widget_render_with_selection() { /* checks "▶" */ }
// ... 4 more
```

**After (2 snapshots, full visual regression coverage):**
```rust
#[test]
fn test_transaction_list_visual_states() {
    let txns = create_sample_transactions();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    
    // Empty state
    let mut buf1 = Buffer::empty(Rect::new(0, 0, 80, 24));
    TransactionListWidget::new(&[])
        .render(Rect::new(0, 0, 80, 24), &mut buf1, &mut TransactionListState::new());
    
    // With data and selection
    let mut buf2 = Buffer::empty(Rect::new(0, 0, 80, 24));
    TransactionListWidget::new(&txns).focused(true)
        .render(Rect::new(0, 0, 80, 24), &mut buf2, &mut TransactionListState::with_selection(1));
    
    insta::assert_snapshot!("txn_list_empty", buf1);
    insta::assert_snapshot!("txn_list_with_selection", buf2);
}
```

## The Consolidation Patterns

### Pattern 1: State Matrix Snapshot

One snapshot captures all state combinations:
```rust
#[test]
fn test_theme_all_styles() {
    let theme = Theme::default();
    let output = format!(
        "primary: {:?}\nborder(focused): {:?}\nborder(unfocused): {:?}\n...",
        theme.primary, theme.border_style(true), theme.border_style(false)
    );
    insta::assert_snapshot!(output);
}
```

### Pattern 2: Navigation State Integration

Test state transitions through outcomes, not internals:
```rust
#[test]
fn test_navigation_state_flows() {
    let mut nav = NavigationState::new();
    
    // Simulate user flow
    nav.select_block(0, &blocks);
    nav.open_block_details();
    nav.cycle_block_detail_tab();
    nav.move_block_txn_down(5, 10);
    
    insta::assert_debug_snapshot!(nav);
}
```

### Pattern 3: Full Screen Snapshots

Test entire layouts, not isolated widgets:
```rust
#[test]
fn test_main_screen_layout() {
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let app = create_test_app_state();
    
    terminal.draw(|f| render_main_screen(f, &app)).unwrap();
    
    insta::assert_snapshot!(terminal.backend());
}
```

## The Consolidation Checklist

Before writing a TUI test, ask:

1. **Can a snapshot replace these 5 assertions?** (Usually yes)
2. **Will a user see this state?** (If not, maybe skip)
3. **Is this testing Ratatui, or my code?** (Don't test the framework)
4. **Would a table test reduce duplication?** (Keys, contexts, configs)
5. **Does this test survive a theme change?** (Avoid color-specific assertions)

## Expected Test Reduction

| Category | Before | After | Notes |
|----------|--------|-------|-------|
| Key mappings | 58 tests | 2-3 table/snapshot tests | Group by context |
| Border blocks | 11 tests | 1 multi-state snapshot | All variants in one |
| Widget rendering | 8+ per widget | 2 snapshots per widget | Empty + populated |
| Theme styles | 16 tests | 1 snapshot | Entire theme at once |
| Navigation state | 19 tests | 3-4 flow tests | Test user journeys |
| Amount formatting | 5 tests | 1 table test | Data-driven |

**Target: 472 tests → ~80 tests, same or better coverage.**

## Snapshot File Hygiene

```
1.  Name snapshots by behavior: "main_screen_with_blocks"
2.  Use consistent terminal sizes: 80x24 or 120x40
3.  Review snapshots on every change: `cargo insta review`
4.  Commit .snap files to git
5.  One snapshot = one user-visible state
```

## The Testing Decision Tree

```
Is it a visual element?
├─ Yes → Snapshot test
│   └─ Multiple states? → One multi-state snapshot
└─ No → Is it a pure function?
    ├─ Yes → Unit test (or table test if multiple cases)
    └─ No → Is it state/logic?
        ├─ Yes → Test transitions, not getters
        └─ No → Probably don't need a test
```
