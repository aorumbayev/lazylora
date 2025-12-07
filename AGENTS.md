# AGENTS.md

## Commands
- `cargo build` / `cargo build --release` - Build project
- `cargo t --all-features` - Run all tests (requires nextest: `cargo binstall cargo-nextest`)
- `cargo t <test_name>` - Run single test
- `cargo fmt -- --check` - Check formatting
- `cargo clippy --all-features -- -D warnings` - Lint

## Code Style
- **Edition**: Rust 2024
- **Imports**: std → external crates → local modules
- **Naming**: PascalCase types, snake_case functions, SCREAMING_SNAKE_CASE constants
- **Errors**: `color_eyre::Result`, `thiserror` for custom errors, `expect("reason")` over `unwrap()`
- **Params**: Use `&str` not `&String`, `&[T]` not `&Vec<T>`
- **Idioms**: `?` chains, iterators over index loops, enums over type flags

## Architecture (TUI)
- State is data; widgets are ephemeral (`fn(&State) -> Frame`)
- Commands for input, Messages for async
- Never block render loop; spawn async tasks

## Testing
- Snapshot tests (insta) for visuals, rstest fixtures for setup
- Tests in same file with `#[cfg(test)]`
- `cargo insta review` for snapshot changes

## Zen
1. Delete > abstract
2. Hardcode > configure
3. `?` > match
4. `.iter()` > `for i in 0..`
5. `expect("why")` > `unwrap()`
6. std > crate > DIY
7. Owned in structs, borrows in functions
8. Split at ~500 lines
9. Abstract after 2-3 duplications
10. Document WHY, not WHAT
11. Target 80x24 first
12. Naming is design
13. Ship it
