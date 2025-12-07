# Building from Source

## Prerequisites

- [Rust](https://rustup.rs/) (stable)

## Build

```bash
git clone https://github.com/aorumbayev/lazylora.git
cd lazylora
cargo build --release
```

Binary: `target/release/lazylora`

## Install Locally

```bash
cargo install --path .
```

## Run Tests

```bash
# Install nextest (faster test runner)
cargo install cargo-nextest

# Run all tests
cargo t --all-features

# Run specific test
cargo t <test_name>
```

## Development

```bash
cargo build          # debug build
cargo run            # run debug build
cargo clippy         # lint
cargo fmt            # format
```
