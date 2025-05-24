use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum Task {
    SendTransactions(SendTransactions),
    TransactionResults(TransactionResults),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SendTransactions {
    pub rpc_url: String,
    pub batch_size: usize,
    pub transfers: Vec<Transfer>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TransactionResults {
    pub rpc_url: String,
    pub batch_size: usize,
    pub results: Vec<TransactionResult>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "status", content = "data")]
pub enum TransactionResult {
    FailedToSend { reason: String },
    Failed { signature: String, reason: String },
    Unknown { signature: String, details: String },
    WithStatus { status: String },
    Sent(SentTransaction),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Transfer {
    pub source_base58_private_key: String,
    pub destination_pubkey: String,
    pub lamports: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SentTransaction {
    pub duration_secs: f64,
    pub signature: String,
}
