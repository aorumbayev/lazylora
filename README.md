# 🔎 LazyLora ⛓️‍💥

> Terminal UI for Algorand blockchain exploration

[![GitHub release (latest by date)](https://img.shields.io/github/v/release/aorumbayev/lazylora)](https://github.com/aorumbayev/lazylora/releases/latest)
[![Rust](https://github.com/aorumbayev/lazylora/workflows/Build/badge.svg)](https://github.com/aorumbayev/lazylora/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

![LazyLora Screenshot](assets/screenshot.png)

LazyLora is a terminal user interface for exploring the Algorand blockchain. It provides a simple and intuitive way to browse blocks and transactions.

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/aorumbayev/lazylora/main/install.sh | bash
```

## Features

-   Browse latest blocks and transactions
-   Search transactions by address, transaction ID, or asset ID
-   View detailed transaction information
-   Live updates of new blocks and transactions
-   Support for MainNet, TestNet, and LocalNet

## Usage

```bash
# Run with default settings
lazylora

# Check for updates
lazylora update

# Update to the latest version
lazylora update --install
```

## Key Bindings

-   `q`: Quit the application
-   `r`: Refresh data
-   `f`: Search transactions
-   `n`: Switch network
-   `Space`: Toggle live updates
-   `Tab`: Switch between blocks and transactions
-   `Enter`: View selected item details
-   `Esc`: Close popup or details view

## Building from Source

```bash
# Clone the repository
git clone https://github.com/aorumbayev/lazylora.git
cd lazylora

# Build and install
cargo build --release
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
