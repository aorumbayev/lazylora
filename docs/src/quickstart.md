# Quick Start

## Launch

```bash
lazylora
```

This opens the TUI connected to MainNet.

## Your First Session

1. **Browse blocks** - The left panel shows recent blocks. Use `j`/`k` or arrow keys to navigate.

2. **View transactions** - Press `Tab` to switch to the transactions panel. Press `Enter` on any transaction to see details.

3. **Try the graph view** - In transaction details, press `Tab` to toggle between Table and Visual (graph) mode. The graph shows inner transactions and asset flows.

4. **Search for something** - Press `f` to focus the search bar. Type an address, transaction ID, or block number. The search type is auto-detected.

5. **Toggle live updates** - Press `Space` to pause/resume live block updates.

6. **Get help** - Press `?` to see all keybindings.

7. **Quit** - Press `q` to exit.

## Quick Lookups from CLI

Jump straight to what you need:

```bash
# Look up a transaction
lazylora -t <TXID>

# Look up an account
lazylora -a <ADDRESS>

# Look up a block
lazylora -b <BLOCK_NUMBER>

# Look up an asset
lazylora -s <ASSET_ID>

# Open transaction in graph view
lazylora -t <TXID> -g
```

## Switch Networks

Press `n` in the app to switch between MainNet, TestNet, and LocalNet. Or specify at launch:

```bash
lazylora -n testnet
lazylora -n localnet
```
