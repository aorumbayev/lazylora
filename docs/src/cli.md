# CLI Commands

## Launch

```bash
lazylora              # MainNet (default)
lazylora -n testnet   # TestNet
lazylora -n localnet  # LocalNet
```

## Direct Lookups

Skip the TUI and go straight to details:

```bash
lazylora -t <TXID>           # Transaction
lazylora -a <ADDRESS>        # Account (or NFD name)
lazylora -b <BLOCK_NUMBER>   # Block
lazylora -s <ASSET_ID>       # Asset
```

## Graph View

Open a transaction directly in visual graph mode:

```bash
lazylora -t <TXID> -g
lazylora --tx <TXID> --graph
```

## Updates

```bash
lazylora update              # Check for new version
lazylora update --install    # Install update
```

## Version

```bash
lazylora version             # Show version info
```

## Options

| Option | Short | Description |
|--------|-------|-------------|
| `--tx <TXID>` | `-t` | Transaction ID |
| `--account <ADDRESS>` | `-a` | Account address or NFD |
| `--block <NUMBER>` | `-b` | Block number |
| `--asset <ID>` | `-s` | Asset ID |
| `--network <NETWORK>` | `-n` | Network: mainnet, testnet, localnet |
| `--graph` | `-g` | Open in graph view |

## Subcommands

| Command | Description |
|---------|-------------|
| `version` | Show version |
| `update` | Check for updates |
| `update --install` | Install update |

## Examples

```bash
# Transaction on MainNet
lazylora -t AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA

# Account on TestNet
lazylora -n testnet -a AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA

# USDC asset details
lazylora -s 31566704

# Block 12345
lazylora -b 12345

# Transaction graph on TestNet
lazylora -n testnet -t TXID -g
```
