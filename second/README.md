# block-sniper

Run:
```sh
INPUT=./input.yml OUTPUT=output.yml carg run
```

## SendTransactions

### Input
```yaml
type: SendTransactions
data:
  rpc_url: https://api.mainnet-beta.solana.com
  batch_size: 5
  transfers:
    - source_base58_private_key: <base58-encoded-private-key-1>
      destination_pubkey: 7EcDhSYGxXyscszYEp35KHN8vvw3svAuLKTzXwCFLtV
      lamports: 1
    - source_base58_private_key: <base58-encoded-private-key-2>
      destination_pubkey: DpNXPNWvWoHaZ9P3WtfGCb2ZdLihW8VW1w1Ph4KDH9iG
      lamports: 5
```

## TransactionResults

### Input
Just use the output from `SendTransactions`, which looks like this:
```yaml
type: TransactionResults
data:
  rpc_url: https://api.mainnet-beta.solana.com
  batch_size: 5
  results:
  - status: <status>
    data: ...
  - status: <status>
    data: ...
```
