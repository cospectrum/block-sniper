use std::str::FromStr;

use futures::future::join_all;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signature;

use super::types::*;

pub async fn check_transactions(input: &TransactionResults) -> TransactionResults {
    let client = RpcClient::new(input.rpc_url.clone());
    let mut results = Vec::with_capacity(input.results.len());
    for batch in input.results.chunks(input.batch_size) {
        results.extend(check_transaction_batch(&client, batch).await);
    }
    TransactionResults {
        rpc_url: input.rpc_url.clone(),
        batch_size: input.batch_size,
        results,
    }
}

async fn check_transaction_batch(
    client: &RpcClient,
    batch: &[TransactionResult],
) -> Vec<TransactionResult> {
    let tasks = batch.iter().map(|result| check_transaction(client, result));
    join_all(tasks).await
}

async fn check_transaction(client: &RpcClient, result: &TransactionResult) -> TransactionResult {
    let tx = match result.clone() {
        TransactionResult::Unknown { tx, details: _ } => tx,
        TransactionResult::WithStatus { tx, status: _ } => tx,
        TransactionResult::Sent(tx) => tx,
        _ => return result.clone(),
    };
    let Ok(signature) = Signature::from_str(&tx.signature) else {
        return TransactionResult::Failed {
            reason: "invalid signature".to_owned(),
            tx,
        };
    };
    let Ok(mut statuses) = client
        .get_signature_statuses_with_history(&[signature])
        .await
    else {
        return TransactionResult::Unknown {
            details: "failed to get signature status".to_owned(),
            tx,
        };
    };
    let status = statuses.value[0].take();
    match status {
        Some(res) => {
            let status = format!("{:?}", res.confirmation_status());
            TransactionResult::WithStatus { tx, status }
        }
        None => TransactionResult::Unknown {
            details: "no transaction status".to_owned(),
            tx,
        },
    }
}
