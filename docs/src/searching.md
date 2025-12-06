# Searching

LazyLora features an inline search bar in the header that automatically detects the type of search based on your input.

## How to Search

1. Press `f` to focus the search bar in the header
2. Start typing your query - the search type is auto-detected
3. Press `Enter` to execute the search
4. Press `Esc` to cancel and clear the search

## Auto-Detection

The search bar automatically detects what you're searching for:

| Input Pattern | Detected Type | Example |
|--------------|---------------|---------|
| 52-char uppercase | Transaction ID | `AAAAAAA...` |
| 58-char uppercase | Account Address | `AAAAAAAA...` |
| Small integer (<100M) | Block Number | `12345678` |
| Large integer (>=100M) | Asset ID | `123456789` |
| Contains `.algo` | NFD Name | `alice.algo` |
| Short alphanumeric | NFD Name | `alice` |

## Search Types

- **Transaction**: Search by transaction ID (52-character base32 string)
- **Block**: Search by round/block number
- **Account**: Search by Algorand address or NFD name
- **Asset**: Search by ASA (Algorand Standard Asset) ID

## Type Indicators

While typing, you'll see a type indicator showing what was detected:

- `[TXN]` - Transaction ID detected
- `[BLK]` - Block number detected
- `[ACC]` - Account/NFD detected
- `[AST]` - Asset ID detected
- `[???]` - Unknown format (won't search)

## Tips

- The search bar is visible in the header - you can see your query while browsing
- Search results open in a popup - use arrow keys to select and Enter to view details
- Press `r` to refresh and return to the latest data
- If the type can't be detected, complete your input (e.g., full 52-char transaction ID)
