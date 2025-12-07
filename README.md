```
██╗      █████╗ ███████╗██╗   ██╗██╗      ██████╗ ██████╗  █████╗ 
██║     ██╔══██╗╚══███╔╝╚██╗ ██╔╝██║     ██╔═══██╗██╔══██╗██╔══██╗
██║     ███████║  ███╔╝  ╚████╔╝ ██║     ██║   ██║██████╔╝███████║
██║     ██╔══██║ ███╔╝    ╚██╔╝  ██║     ██║   ██║██╔══██╗██╔══██║
███████╗██║  ██║███████╗   ██║   ███████╗╚██████╔╝██║  ██║██║  ██║
╚══════╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
```

> Terminal UI for Algorand blockchain exploration

[![GitHub release (latest by date)](https://img.shields.io/github/v/release/aorumbayev/lazylora)](https://github.com/aorumbayev/lazylora/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

LazyLora is a terminal user interface for exploring the Algorand blockchain. Browse blocks, transactions, accounts, assets, and applications - all from your terminal.

![LazyLora Screenshot](assets/lazylora.png)

## Install

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/aorumbayev/lazylora/main/install.sh | bash
```

### Windows

```powershell
iwr -useb https://raw.githubusercontent.com/aorumbayev/lazylora/main/install.ps1 | iex
```

> [!NOTE]
> Ensure you have [Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170) installed.

## What You Can Do

- **Browse** the latest blocks and transactions in real-time
- **Search** by transaction ID, account address, block number, asset ID, or NFD name
- **Inspect** transactions with visual graph view showing inner transactions and asset flows
- **Explore** accounts (balances, assets, apps), assets (supply, metadata), and applications (state, programs)
- **Export** transaction graphs as SVG files
- **Copy** transaction IDs, addresses, or raw JSON to clipboard
- **Open** any entity directly in your browser (Lora explorer)
- **Switch** between MainNet, TestNet, and LocalNet

## Usage

```bash
lazylora                    # Launch TUI (MainNet)
lazylora -n testnet         # Connect to TestNet
lazylora -t <TXID>          # Look up transaction
lazylora -a <ADDRESS>       # Look up account
lazylora -b <BLOCK>         # Look up block
lazylora -s <ASSET_ID>      # Look up asset
lazylora -t <TXID> -g       # Open transaction in graph view
lazylora version            # Show version
lazylora update             # Check for updates
lazylora update --install   # Install update
```

## Key Bindings

| Key | Action |
|-----|--------|
| `q` | Quit |
| `r` | Refresh |
| `?` | Help |
| `f` | Search |
| `n` | Switch network |
| `Space` | Toggle live updates |
| `Tab` | Cycle panels / Switch view |
| `j`/`k` | Navigate |
| `Enter` | Open details |
| `Esc` | Close |
| `c` | Copy ID |
| `y` | Copy JSON |
| `o` | Open in browser |
| `s` | Export SVG (graph view) |

## CLI Reference

| Option | Short | Description |
|--------|-------|-------------|
| `--tx <TXID>` | `-t` | Transaction lookup |
| `--account <ADDRESS>` | `-a` | Account lookup |
| `--block <NUMBER>` | `-b` | Block lookup |
| `--asset <ID>` | `-s` | Asset lookup |
| `--network <NETWORK>` | `-n` | Network (mainnet, testnet, localnet) |
| `--graph` | `-g` | Open in graph view |

| Subcommand | Description |
|------------|-------------|
| `version` | Show version |
| `update` | Check for updates |
| `update --install` | Install update |

## Building from Source

```bash
git clone https://github.com/aorumbayev/lazylora.git
cd lazylora
cargo build --release
```

## Contributing

Contributions welcome! See the [documentation](https://aorumbayev.github.io/lazylora/) for details.

## License

MIT - see [LICENSE](LICENSE)
