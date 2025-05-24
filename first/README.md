# first
Get wallet balances.

## Getting Started

1. Create `config.yml`:
```yml
rpc_url: <solana url>
batch_size: <usize>
wallets:
    - <pub_key1>
    - <pub_key2>
    ...
```
The `batch_size` will control how many wallet balance requests are sent concurrently.


2. Run the task:
```rust
CONFIG=./config.yml cargo run
```
