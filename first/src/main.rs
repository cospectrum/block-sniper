use anyhow::anyhow;
use futures::future::join_all;
use serde::Deserialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::fs;
use std::str::FromStr;

#[derive(Deserialize)]
struct Config {
    rpc_url: String,
    batch_size: usize,
    wallets: Vec<String>,
}

type Balance = u64;
type BalanceResult = anyhow::Result<Balance>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config: Config = {
        let config_path = std::env::var("CONFIG")?;
        let config_content = fs::read_to_string(&config_path)
            .map_err(|err| anyhow!("failed to read config: {}, {}", config_path, err))?;
        serde_yaml::from_str(&config_content)?
    };

    let client = RpcClient::new(config.rpc_url);
    let mut results = Vec::<BalanceResult>::with_capacity(config.wallets.len());

    for batch in config.wallets.chunks(config.batch_size) {
        let tasks = batch.iter().map(|addr| get_wallet_balance(&client, addr));
        results.extend(join_all(tasks).await);
    }

    for (result, addr) in results.iter().zip(config.wallets) {
        println!("{addr}: {result:?}");
    }
    Ok(())
}

async fn get_wallet_balance(client: &RpcClient, wallet_addr: &str) -> anyhow::Result<Balance> {
    let pubkey = Pubkey::from_str(wallet_addr)?;
    Ok(client.get_balance(&pubkey).await?)
}
