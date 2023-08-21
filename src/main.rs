mod block;
mod blockchain;
pub mod messages;
mod networking;
mod transaction; // Declare the modules

use blockchain::Blockchain;
use chrono::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use transaction::Transaction;

#[tokio::main]
async fn main() {
    // Create a shared blockchain
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    // Create a few transactions
    let transaction1 = Transaction {
        sender: String::from("Alice"),
        receiver: String::from("Bob"),
        amount: 50.0,
    };
    let transaction2 = Transaction {
        sender: String::from("Bob"),
        receiver: String::from("Charlie"),
        amount: 25.0,
    };

    // Add blocks containing the transactions to the shared blockchain
    {
        let mut blockchain_guard = blockchain.lock().await;
        blockchain_guard.add_block(vec![transaction1.clone()]);
        blockchain_guard.add_block(vec![transaction2.clone()]);
    }

    // Start the server (this should keep running to listen for incoming connections)
    networking::start_server(blockchain.clone()).await;

    // The following is just for displaying. If this isn't required, remove it.
    {
        let blockchain_guard = blockchain.lock().await;
        for block in &blockchain_guard.chain {
            println!("Index: {}", block.index);
            println!(
                "Timestamp: {}",
                Utc.timestamp_opt(block.timestamp, 0).unwrap().to_string()
            );
            println!("Nonce: {}", block.nonce);
            println!("Previous Hash: {}", block.previous_hash);
            println!("Hash: {}", block.hash);

            for transaction in &block.transactions {
                println!(
                    "  Transaction: {} -> {} : {}",
                    transaction.sender, transaction.receiver, transaction.amount
                );
            }
            println!("------------------");
        }
    }

    // Again, just for displaying.
    {
        let blockchain_guard = blockchain.lock().await;
        println!("Is chain valid? {}", blockchain_guard.is_chain_valid());
    }
}
