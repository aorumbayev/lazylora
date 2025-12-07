# Contributing

Contributions welcome!

## Setup

```bash
git clone https://github.com/YOUR_USERNAME/lazylora.git
cd lazylora

# Install nextest for faster tests
cargo install cargo-nextest
```

## Workflow

1. Fork the repo
2. Create a branch: `git checkout -b my-feature`
3. Make changes
4. Test: `cargo t --all-features`
5. Lint: `cargo clippy --all-features -- -D warnings`
6. Format: `cargo fmt`
7. Submit PR

## Testing

```bash
cargo t --all-features          # All tests
cargo t <test_name>             # Single test
cargo insta review              # Review snapshot changes
```

## Guidelines

- Add tests for new functionality
- Update docs if user-facing behavior changes
- Keep commits focused

## Issues

Found a bug? [Open an issue](https://github.com/aorumbayev/lazylora/issues/new).

## License

Contributions are MIT licensed.
