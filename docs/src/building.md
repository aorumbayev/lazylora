# Building from Source

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- Git

## Clone and Build

```bash
# Clone the repository
git clone https://github.com/aorumbayev/lazylora.git
cd lazylora

# Build in release mode
cargo build --release
```

The binary will be available at `target/release/lazylora`.

## Install Locally

```bash
cargo install --path .
```

## Development Build

```bash
# Build with debug symbols
cargo build

# Run directly
cargo run
```

## Running Tests

```bash
cargo test
```
