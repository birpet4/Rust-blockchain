use crate::blockchain::Blockchain;
use crate::custom_error::CustomError;
use crate::messages::Message;

use once_cell::sync::Lazy;
use std::{sync::Arc, collections::HashSet};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

// Static list of initial seed nodes
const SEED_NODES: &[&str] = &["127.0.0.1:8000", "127.0.0.1:8001"];

pub static PEERS: Lazy<Arc<Mutex<Vec<String>>>> = Lazy::new(|| {
    let initial_peers = SEED_NODES.iter().map(|&s| s.to_string()).collect();
    Arc::new(Mutex::new(initial_peers))
});

pub static ACTIVE_PEERS: Lazy<Arc<Mutex<HashSet<String>>>> = Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));

pub async fn add_peer(address: String) {
    let mut peers = PEERS.lock().await;
    if !peers.contains(&address) {
        peers.push(address);
    }
}

pub async fn get_peers() -> Vec<String> {
    let peers: tokio::sync::MutexGuard<'_, Vec<String>> = PEERS.lock().await;
    peers.clone()
}

pub async fn connect_to_peers(current_node_address: String) {
    let peers = get_peers().await;
    let active_peers = ACTIVE_PEERS.lock().await;

    for peer in peers {
        if peer != current_node_address && !active_peers.contains(&peer) {
            if let Err(err) = try_connect_peer(&peer).await {
                eprintln!("Failed to connect to {}: {}", peer, err);
            }
        }
    }
}

pub async fn try_connect_peer(address: &str) -> Result<(), CustomError> {
    let mut stream = TcpStream::connect(address).await?;

    // Logging a successful connection
    println!("Successfully connected to peer: {}", address);

    // Add to the active peers list
    let mut active_peers = ACTIVE_PEERS.lock().await;
    active_peers.insert(address.to_string());

    let message = Message::RequestBlockchain;
    let serialized_message = serde_json::to_string(&message)?;
    stream.write_all(serialized_message.as_bytes()).await?;
    Ok(())
}

pub async fn start_server(
    port: String,
    blockchain: Arc<Mutex<Blockchain>>,
) -> Result<(), CustomError> {
    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(address).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let blockchain_clone = blockchain.clone(); // Clone the Arc
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, blockchain_clone).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

pub async fn write_message(stream: &mut TcpStream, message: &Message) -> Result<(), CustomError> {
    let serialized_message = serde_json::to_string(&message)?;
    stream.write_all(serialized_message.as_bytes()).await?;
    Ok(())
}

// Responsible for managing the communication with another node (peer) in the P2P network once a connection is established.
// This communication is centered around messages, and the content and type of each message dictate the action taken.
// Throughout this function, the shared instance of the blockchain is accessed using the Arc and Mutex wrappers to ensure safe concurrent access across multiple threads/tasks.
pub async fn handle_connection(
    mut stream: TcpStream,
    blockchain: Arc<Mutex<Blockchain>>,
) -> Result<(), CustomError> {
    // Reads incoming data from the connection stream into the buffer. n is the number of bytes read.
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;

    // // Gets the address of the remote peer (i.e., the node you're communicating with) and adds it to the list of known peers.
    // let remote_addr = stream.peer_addr().unwrap().to_string();
    // add_peer(remote_addr).await;

    // Converts the incoming bytes into a Message type using serde_json for deserialization.
    let message: Message = serde_json::from_slice(&buffer[0..n])?;

    match message {
        // If the incoming message is a request for the blockchain, this part sends back the entire blockchain to the requester.
        Message::RequestBlockchain => {
            let blockchain_data = blockchain.lock().await;
            let serialized_blockchain = serde_json::to_string(&*blockchain_data)?;
            stream.write_all(serialized_blockchain.as_bytes()).await?;
        }

        // If another peer sends its blockchain, this part checks if the received blockchain is valid and longer than the current blockchain.
        // If both conditions are met, it replaces the current blockchain with the received one.
        Message::SendBlockchain(blocks) => {
            let mut blockchain_data = blockchain.lock().await;
            let received_blockchain = Blockchain::from(&blockchain_data, blocks);
            if received_blockchain.is_chain_valid()
                && received_blockchain.chain.len() > blockchain_data.chain.len()
            {
                blockchain_data.chain = received_blockchain.chain;
                println!("Blockchain replaced with a longer valid chain from a peer.");
            }
        }

        // Upon receiving a new transaction, the transaction is added to a new block (this may not be the best approach in a real-world scenario, but it works for the sake of this example).
        // After adding, it broadcasts this transaction to all known peers.
        Message::NewTransaction(transaction) => {
            let mut blockchain_data = blockchain.lock().await;
            blockchain_data.add_block(vec![transaction.clone()]);

            for peer_address in &blockchain_data.peers {
                if let Ok(mut peer_stream) = TcpStream::connect(peer_address).await {
                    write_message(
                        &mut peer_stream,
                        &Message::BroadcastTransaction(transaction.clone()),
                    )
                    .await?;
                }
            }
        }

        // If a transaction is being broadcasted from another peer, this code validates the transaction.
        // If valid, it's added to the transaction pool and then re-broadcasted to all other known peers.
        Message::BroadcastTransaction(transaction) => {
            let mut blockchain_data = blockchain.lock().await;

            // 1. Validate the transaction
            if blockchain_data.validate_transaction(&transaction) {
                // 2. Add the transaction to the transaction pool
                blockchain_data
                    .pending_transactions
                    .push(transaction.clone());

                // 3. Broadcast the transaction to all other known peers
                for peer_address in &blockchain_data.peers {
                    if peer_address != &stream.peer_addr().unwrap().to_string() {
                        // Avoid sending back to the sender
                        if let Ok(mut peer_stream) = TcpStream::connect(peer_address).await {
                            write_message(
                                &mut peer_stream,
                                &Message::BroadcastTransaction(transaction.clone()),
                            )
                            .await?;
                        }
                    }
                }
            }
        }

        // When a block is broadcasted from another peer, this code attempts to add the block's transactions to the blockchain.
        // If successful, the block is then broadcasted to all other peers.
        Message::BroadcastBlock(block) => {
            let mut blockchain_data = blockchain.lock().await;
            // Extract transactions from the block
            let transactions = block.transactions.clone();

            match blockchain_data.add_block(transactions) {
                Ok(_) => {
                    // Broadcast the block to all other known peers
                    for peer_address in &blockchain_data.peers {
                        if peer_address != &stream.peer_addr().unwrap().to_string() {
                            if let Ok(mut peer_stream) = TcpStream::connect(peer_address).await {
                                write_message(
                                    &mut peer_stream,
                                    &Message::BroadcastBlock(block.clone()),
                                )
                                .await?;
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Failed to add block: {}", err);
                }
            }
        }
    }
    Ok(())
}
