# Searching

Press `f` to focus the search bar. Type your query and press `Enter`.

## Auto-Detection

LazyLora detects what you're searching for:

| You type | Detected as | Example |
|----------|-------------|---------|
| 52-char string | Transaction ID | `AAAAAAA...` (52 chars) |
| 58-char string | Account Address | `AAAAAAAA...` (58 chars) |
| Number < 100M | Block Number | `12345678` |
| Number >= 100M | Asset ID | `31566704` |
| Contains `.algo` | NFD Name | `alice.algo` |
| Short text | NFD Name | `alice` |

## Type Indicator

While typing, a badge shows the detected type:

- `[TXN]` - Transaction
- `[BLK]` - Block
- `[ACC]` - Account/NFD
- `[AST]` - Asset
- `[???]` - Unknown (won't search)

## Search Keys

| Key | Action |
|-----|--------|
| `Enter` | Submit search |
| `Esc` | Cancel |
| `Tab` | Force different search type |
| `Up` / `Down` | Browse history |
| `Left` / `Right` | Move cursor |

## CLI Search

Skip the TUI and go directly to results:

```bash
lazylora -t <TXID>           # Transaction
lazylora -a <ADDRESS>        # Account
lazylora -b <BLOCK>          # Block
lazylora -s <ASSET_ID>       # Asset
lazylora -t <TXID> -g        # Transaction in graph view
```
