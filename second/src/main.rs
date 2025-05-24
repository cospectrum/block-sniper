use anyhow::anyhow;
use std::fs;
use std::io::Write;

use second::types;
use second::{check_transactions, send_transactions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let task: types::Task = {
        let input_path = std::env::var("INPUT")?;
        let input_content = fs::read_to_string(&input_path)
            .map_err(|err| anyhow!("failed to read input: {}, {}", input_path, err))?;
        serde_yaml::from_str(&input_content)?
    };
    let output_path = std::env::var("OUTPUT")?;
    let mut output_file = fs::File::create(output_path)?;

    let results = match task {
        types::Task::SendTransactions(input) => {
            types::Task::TransactionResults(send_transactions(&input).await)
        }
        types::Task::TransactionResults(input) => {
            types::Task::TransactionResults(check_transactions(&input).await)
        }
    };
    let output_content = serde_yaml::to_string(&results)?;
    output_file.write_all(output_content.as_bytes())?;
    Ok(())
}
