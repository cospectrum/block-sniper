# first

## Getting Started

1. Create `config.yml`:
```yml
rpc_url: <solana url>
batch_size: <usize>
wallets:
    - <addr1>
    - <addr2>
    ...
```


2. Run the task:
```rust
CONFIG=./config.yml cargo run
```
