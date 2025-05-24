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
    let TransactionResult::Sent(tx) = result else {
        return result.clone();
    };
    let Ok(signature) = Signature::from_str(&tx.signature) else {
        return TransactionResult::Failed {
            signature: tx.signature.to_owned(),
            reason: "invalid signature".to_owned(),
        };
    };
    let Ok(mut statuses) = client
        .get_signature_statuses_with_history(&[signature])
        .await
    else {
        return TransactionResult::Unknown {
            signature: tx.signature.to_owned(),
            details: "failed to get signature status".to_owned(),
        };
    };
    let status = statuses.value[0].take();
    match status {
        Some(res) => {
            let status = format!("{:?}", res.confirmation_status());
            TransactionResult::WithStatus { status }
        }
        None => TransactionResult::Unknown {
            signature: tx.signature.to_owned(),
            details: "failed to get signature status".to_owned(),
        },
    }
}
