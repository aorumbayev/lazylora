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
