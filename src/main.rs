mod block;
mod blockchain;
pub mod custom_error;
pub mod messages;
mod networking;
mod transaction; // Declare the modules

use blockchain::Blockchain;
use networking::try_connect_peer;

use std::{env, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex, time::sleep, time::Duration};
use transaction::Transaction;

use crate::networking::connect_to_peers;

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
    let port_for_server = port.clone(); // Clone for the server
    let port_for_peers = port.clone(); // Clone for the peers

    // Start the server (this should keep running to listen for incoming connections)
    let server_handle = tokio::spawn(networking::start_server(
        port_for_server,
        blockchain.clone(),
    ));

    // Use a timer to periodically attempt connections to known peers
    let peer_connection_handle = tokio::spawn(async move {
        const PEER_REFRESH_INTERVAL: u64 = 60; // Example: Try to connect to peers every 60 seconds.

        loop {
            let current_node_address = format!("127.0.0.1:{}", port_for_peers.clone());
            connect_to_peers(current_node_address).await;
            sleep(Duration::from_secs(PEER_REFRESH_INTERVAL)).await;
        }
    });

    // Await both tasks to completion (they likely won't complete under normal circumstances unless there's an error)
    let _ = tokio::try_join!(server_handle, peer_connection_handle);
}

async fn broadcast_transaction(transaction: &Transaction, blockchain: Arc<Mutex<Blockchain>>) {
    let blockchain_guard = blockchain.lock().await;
    for peer_address in &blockchain_guard.peers {
        if let Ok(mut peer_stream) = TcpStream::connect(peer_address).await {
            networking::write_message(
                &mut peer_stream,
                &messages::Message::NewTransaction(transaction.clone()),
            )
            .await
            .unwrap();
        }
    }
}

async fn broadcast_mined_block(block: &block::Block, blockchain: Arc<Mutex<Blockchain>>) {
    let blockchain_guard = blockchain.lock().await;
    for peer_address in &blockchain_guard.peers {
        if let Ok(mut peer_stream) = TcpStream::connect(peer_address).await {
            networking::write_message(
                &mut peer_stream,
                &messages::Message::BroadcastBlock(block.clone()),
            )
            .await
            .unwrap();
        }
    }
}

const SEED_NODES: [&str; 2] = ["127.0.0.1:8000", "127.0.0.1:8001"];

// When a node starts:
async fn start_node(port: &str, blockchain: Arc<Mutex<Blockchain>>) {
    // Start the server to listen for incoming connections.
    let server_handle = tokio::spawn(networking::start_server(
        port.to_string(),
        blockchain.clone(),
    ));

    // Try connecting to seed nodes
    for seed in SEED_NODES.iter() {
        // Don't connect to yourself
        if seed != &format!("127.0.0.1:{}", port) {
            if let Err(err) = try_connect_peer(seed).await {
                eprintln!("Failed to connect to seed node {}: {}", seed, err);
            }
        }
    }

    // Wait for the server to finish (it probably won't, since it should keep listening)
    let _ = server_handle.await;
}
