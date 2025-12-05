# CLI Commands

LazyLora provides several command-line options for launching the explorer and performing direct lookups.

## Basic Usage

```bash
# Run with default settings (mainnet)
lazylora

# Show help
lazylora --help

# Show version
lazylora --version
```

## Direct Search

Launch LazyLora with a specific search query:

```bash
# Transaction lookup
lazylora -t <TXID>
lazylora --tx <TXID>

# Account lookup
lazylora -a <ADDRESS>
lazylora --account <ADDRESS>

# Block lookup
lazylora -b <BLOCK_NUMBER>
lazylora --block <BLOCK_NUMBER>

# Asset lookup
lazylora -s <ASSET_ID>
lazylora --asset <ASSET_ID>
```

## Network Selection

Specify which network to connect to:

```bash
# Connect to mainnet (default)
lazylora -n mainnet

# Connect to testnet
lazylora -n testnet
lazylora --network testnet

# Connect to localnet
lazylora -n localnet
```

## Graph View

Open a transaction directly in graph view:

```bash
lazylora -t <TXID> -g
lazylora --tx <TXID> --graph
```

## Update Commands

```bash
# Check for updates
lazylora update

# Update to the latest version
lazylora update --install
```

## Options Reference

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Display help information |
| `--version` | `-V` | Display version information |
| `--tx <TXID>` | `-t` | Look up a transaction by ID |
| `--account <ADDRESS>` | `-a` | Look up an account by address |
| `--block <NUMBER>` | `-b` | Look up a block by number |
| `--asset <ID>` | `-s` | Look up an asset by ID |
| `--network <NETWORK>` | `-n` | Network to connect to (mainnet, testnet, localnet) |
| `--graph` | `-g` | Open transaction in graph view |

## Examples

```bash
# Look up a transaction on mainnet
lazylora -t TXID123ABC

# Look up an account on testnet
lazylora -n testnet -a ADDR123ABC

# Look up block 12345 on mainnet
lazylora -b 12345

# Look up asset 31566704 (USDC) on mainnet
lazylora -s 31566704

# View transaction graph on testnet
lazylora -n testnet -t TXID123ABC -g
```
