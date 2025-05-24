use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::{collections::HashMap, env, str::FromStr};
use tokio_stream::StreamExt;
use tracing::{error, info, warn};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::*;

#[derive(Debug, Deserialize)]
struct Config {
    rpc_url: String,
    recipient_wallet: String,
    transfer_amount_lamports: u64,
    private_key_base58: String,
    geyser_endpoint: String,
    x_token: String,
}

impl Config {
    fn load() -> anyhow::Result<Self> {
        let config_path = env::var("CONFIG").unwrap_or_else(|_| "config.yaml".to_string());
        let content = std::fs::read_to_string(config_path)?;
        Ok(serde_yaml::from_str(&content)?)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load()?;
    info!("Config loaded successfully");

    let rpc_client = RpcClient::new_with_commitment(config.rpc_url, CommitmentConfig::confirmed());
    let keypair = Keypair::from_base58_string(&config.private_key_base58);
    let recipient = Pubkey::from_str(&config.recipient_wallet)?;

    info!("Connecting to Geyser GRPC...");
    let mut client = GeyserGrpcClient::build_from_shared(config.geyser_endpoint)?
        .x_token(Some(config.x_token))?
        .connect()
        .await?;

    let mut blocks = HashMap::new();
    blocks.insert(
        "client".to_string(),
        SubscribeRequestFilterBlocks {
            account_include: vec![],
            include_transactions: Some(false),
            include_accounts: Some(false),
            include_entries: Some(false),
        },
    );

    let mut stream = client
        .subscribe_once(SubscribeRequest {
            blocks,
            slots: HashMap::new(),
            transactions: HashMap::new(),
            transactions_status: HashMap::new(),
            accounts: HashMap::new(),
            entry: HashMap::new(),
            commitment: Some(CommitmentLevel::Confirmed as i32),
            accounts_data_slice: vec![],
            ping: None,
            from_slot: None,
            blocks_meta: HashMap::new(),
        })
        .await?;

    info!("Subscribed to blocks, waiting for new blocks...");

    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                if let Some(subscribe_update::UpdateOneof::Block(block_update)) = msg.update_oneof {
                    info!("New block detected: slot {}", block_update.slot);

                    match send_sol_transfer(
                        &rpc_client,
                        &keypair,
                        &recipient,
                        config.transfer_amount_lamports,
                    )
                    .await
                    {
                        Ok(signature) => info!("SOL transfer sent: {}", signature),
                        Err(e) => error!("Failed to send SOL transfer: {}", e),
                    }
                }
            }
            Err(e) => {
                error!("Stream error: {}", e);
                warn!("Attempting to reconnect...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }

    Ok(())
}

async fn send_sol_transfer(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    recipient: &Pubkey,
    amount_lamports: u64,
) -> anyhow::Result<String> {
    let recent_blockhash = rpc_client.get_latest_blockhash()?;

    let instruction = system_instruction::transfer(&keypair.pubkey(), recipient, amount_lamports);
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&keypair.pubkey()),
        &[keypair],
        recent_blockhash,
    );

    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    Ok(signature.to_string())
}
