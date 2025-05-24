use std::str::FromStr;

use futures::future::join_all;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::message::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use solana_sdk::{bs58, system_instruction};

use super::types::*;

pub async fn send_transactions(input: &SendTransactions) -> TransactionResults {
    let client = RpcClient::new(input.rpc_url.clone());
    let mut results = Vec::with_capacity(input.transfers.len());
    for batch in input.transfers.chunks(input.batch_size) {
        results.extend(send_transaction_batch(&client, batch).await);
    }
    TransactionResults {
        rpc_url: input.rpc_url.clone(),
        batch_size: input.batch_size,
        results,
    }
}

async fn send_transaction_batch(client: &RpcClient, batch: &[Transfer]) -> Vec<TransactionResult> {
    let blockhash = match client.get_latest_blockhash().await {
        Ok(hash) => hash,
        Err(e) => {
            let err = TransactionResult::FailedToSend {
                reason: e.to_string(),
            };
            return std::iter::repeat(err).take(batch.len()).collect();
        }
    };
    let tasks = batch
        .iter()
        .map(|transfer| send_transaction(client, blockhash, transfer));
    join_all(tasks).await
}

async fn send_transaction(
    client: &RpcClient,
    blockhash: solana_sdk::hash::Hash,
    transfer: &Transfer,
) -> TransactionResult {
    let tx = match signed_transaction(blockhash, transfer) {
        Ok(tx) => tx,
        Err(e) => {
            return TransactionResult::FailedToSend {
                reason: e.to_string(),
            }
        }
    };
    let clock = std::time::Instant::now();
    match client.send_transaction(&tx).await {
        Ok(signature) => {
            let duration = clock.elapsed();
            let signature = signature.to_string();
            TransactionResult::Sent(SentTransaction {
                duration_secs: duration.as_secs_f64(),
                signature,
            })
        }
        Err(e) => TransactionResult::FailedToSend {
            reason: e.to_string(),
        },
    }
}

fn signed_transaction(
    blockhash: solana_sdk::hash::Hash,
    transfer: &Transfer,
) -> anyhow::Result<Transaction> {
    let sender = keypair_from_base58(&transfer.source_base58_private_key)?;
    let destincation_pubkey = Pubkey::from_str(&transfer.destination_pubkey)?;
    let instruction =
        system_instruction::transfer(&sender.pubkey(), &destincation_pubkey, transfer.lamports);
    let message = Message::new(&[instruction], Some(&sender.pubkey()));
    Ok(Transaction::new(&[sender], message, blockhash))
}

fn keypair_from_base58(private_key: &str) -> anyhow::Result<Keypair> {
    let mut buf = [0u8; ed25519_dalek::KEYPAIR_LENGTH];
    bs58::decode(private_key).onto(&mut buf)?;
    Ok(Keypair::from_bytes(&buf)?)
}
