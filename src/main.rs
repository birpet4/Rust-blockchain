mod block;
mod blockchain;
mod transaction;

use blockchain::Blockchain;
use chrono::prelude::*;
use transaction::Transaction;

fn main() {
    // Create a new blockchain
    let mut blockchain = Blockchain::new();

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

    // Add blocks containing the transactions to the blockchain
    blockchain.add_block(vec![transaction1]);
    blockchain.add_block(vec![transaction2]);

    // Print the blockchain to verify its contents
    for block in &blockchain.chain {
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

    // You could also add code here to test the validity of the chain, etc.
    println!("Is chain valid? {}", blockchain.is_chain_valid());
}
