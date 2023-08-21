use crate::blockchain::Blockchain;
use crate::messages::Message;
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
// rest of your code

pub fn start_server(blockchain: Arc<Mutex<Blockchain>>) {
    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let blockchain_clone = blockchain.clone(); // Clone the Arc
        tokio::spawn(async move {
            handle_connection(stream, blockchain_clone).await; // Move the clone
        });
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    blockchain: Arc<Mutex<Blockchain>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a buffer to read data into
    let mut buffer = [0; 1024];

    // Read data from the connection
    let n = stream.read(&mut buffer).await?;

    // Deserialize the received data into a Message
    let message: Message = serde_json::from_slice(&buffer[0..n])?;

    // Process the message
    match message {
        Message::RequestBlockchain => {
            let blockchain_data = blockchain.lock().unwrap();
            let serialized_blockchain = serde_json::to_string(&*blockchain_data)?;
            stream.write_all(serialized_blockchain.as_bytes()).await?;
        }
        Message::SendBlockchain(blocks) => {
            let mut blockchain_data = blockchain.lock().unwrap();
            blockchain_data.chain = blocks;
        }
        Message::NewTransaction(transaction) => {
            let mut blockchain_data = blockchain.lock().unwrap();
            blockchain_data.add_transaction(transaction); // Add the new transaction to the blockchain
        }
    }

    // Other networking-related functions
    Ok(())
}
