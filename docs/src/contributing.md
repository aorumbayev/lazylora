# Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a new branch for your feature
4. Make your changes
5. Submit a Pull Request

## Development Setup

```bash
git clone https://github.com/YOUR_USERNAME/lazylora.git
cd lazylora

# Install cargo-nextest for faster test runs (one-time setup)
cargo install cargo-nextest
# Or faster with binstall: cargo binstall cargo-nextest

cargo build
```

## Running Tests

```bash
cargo t --all-features          # Run all tests (uses nextest)
cargo t <test_name>             # Run specific test
cargo clippy --all-features     # Lint
cargo fmt -- --check            # Check formatting
```

## Guidelines

- Follow Rust best practices and idioms
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

## Reporting Issues

Found a bug or have a feature request? Please [open an issue](https://github.com/aorumbayev/lazylora/issues/new) on GitHub.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
