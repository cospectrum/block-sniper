use futures::future::join_all;
use solana_client::nonblocking::rpc_client::RpcClient;

use super::types::*;

pub async fn check_transactions(input: &TransactionResults) -> TransactionResults {
    let client = RpcClient::new(input.rpc_url.clone());
    let mut results = Vec::new();
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
    let Ok(status) = client.get_signature_status(&tx.signature).await else {
        return TransactionResult::Failed("failed to get signature status".to_owned());
    };
    match status {
        Some(res) => match res {
            Err(_) => TransactionResult::Failed("tx failed".to_owned()),
            Ok(_) => TransactionResult::Processed,
        },
        None => TransactionResult::InProcess,
    }
}
