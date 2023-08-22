mod block;
mod blockchain;
pub mod messages;
mod networking;
mod transaction; // Declare the modules
pub mod custom_error;

use blockchain::Blockchain;
use chrono::prelude::*;

use std::{sync::Arc, env};
use tokio::{sync::Mutex, net::TcpStream, time::sleep, time::Duration};
use transaction::Transaction;

#[tokio::main]
async fn main() {
    // Create a shared blockchain
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} [port_number]", args[0]);
        return;
    }

    let port = args[1].clone();

    // Start the server (this should keep running to listen for incoming connections)
    let server_handle = tokio::spawn(networking::start_server(port, blockchain.clone()));

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

    // Specify the retry parameters
    const MAX_RETRIES: u32 = 5;  // for example, try 5 times
    const RETRY_DELAY: u64 = 5; // wait for 5 seconds between each try

    // Connect to peers with retry mechanism
    let peer_address = "127.0.0.1:8001";
    let mut retry_count = 0;

    while retry_count < MAX_RETRIES {
        let peer_stream = TcpStream::connect(peer_address).await;

        match peer_stream {
            Ok(stream) => {
                println!("Connected to peer: {}", peer_address);
                let blockchain_clone = blockchain.clone();
                tokio::spawn(async move {
                    networking::handle_connection(stream, blockchain_clone).await.unwrap();
                });
                break; // Exit loop upon successful connection
            }
            Err(e) => {
                retry_count += 1;
                eprintln!(
                    "Failed to connect to peer (Attempt {}/{}): {}",
                    retry_count, MAX_RETRIES, e
                );
                if retry_count < MAX_RETRIES {
                    println!(
                        "Waiting for {} seconds before retrying...",
                        RETRY_DELAY
                    );
                    sleep(Duration::from_secs(RETRY_DELAY)).await;
                } else {
                    eprintln!("Max retry attempts reached. Moving on.");
                }
            }
        }
    }


    let _ = server_handle.await;
}

async fn broadcast_transaction(transaction: &Transaction, blockchain: Arc<Mutex<Blockchain>>) {
    let blockchain_guard = blockchain.lock().await;
    for peer_address in &blockchain_guard.peers {
        if let Ok(mut peer_stream) = TcpStream::connect(peer_address).await {
            networking::write_message(&mut peer_stream, &messages::Message::NewTransaction(transaction.clone())).await.unwrap();
        }
    }
}

async fn broadcast_mined_block(block: &block::Block, blockchain: Arc<Mutex<Blockchain>>) {
    let blockchain_guard = blockchain.lock().await;
    for peer_address in &blockchain_guard.peers {
        if let Ok(mut peer_stream) = TcpStream::connect(peer_address).await {
            networking::write_message(&mut peer_stream, &messages::Message::BroadcastBlock(block.clone())).await.unwrap();
        }
    }
}