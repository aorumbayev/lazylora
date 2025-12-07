# Networks

LazyLora connects to Algorand networks. Press `n` to switch.

## MainNet

Production network. Real ALGO, real transactions. This is the default.

```bash
lazylora              # defaults to mainnet
lazylora -n mainnet   # explicit
```

## TestNet

Test network for development. Free test ALGO from the [faucet](https://bank.testnet.algorand.network/).

```bash
lazylora -n testnet
```

## LocalNet

Your local Algorand node. Useful with [AlgoKit LocalNet](https://github.com/algorandfoundation/algokit-cli).

```bash
# Start LocalNet first
algokit localnet start

# Then connect
lazylora -n localnet
```

LocalNet connects to `http://localhost:4001` (algod) and `http://localhost:8980` (indexer).

## Current Network

The header shows which network you're connected to. Live updates indicator shows connection status.
