use crate::blockchain::Blockchain;
use crate::messages::Message;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex
};


#[derive(Debug)]
pub struct CustomError {
    message: String,
}

impl CustomError {
    pub fn new(msg: &str) -> Self {
        CustomError {
            message: msg.to_string(),
        }
    }
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CustomError {}

impl From<std::io::Error> for CustomError {
    fn from(error: std::io::Error) -> Self {
        CustomError::new(&error.to_string())
    }
}

impl From<serde_json::Error> for CustomError {
    fn from(error: serde_json::Error) -> Self {
        CustomError::new(&error.to_string())
    }
}

// Add more conversions as required


pub async fn start_server(blockchain: Arc<Mutex<Blockchain>>) -> Result<(), CustomError> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let blockchain_clone = blockchain.clone(); // Clone the Arc
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, blockchain_clone).await {
                // Here you handle the error, e.g., by logging it.
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}


async fn handle_connection(
    mut stream: TcpStream,
    blockchain: Arc<Mutex<Blockchain>>,
) -> Result<(), CustomError> {
    // Create a buffer to read data into
    let mut buffer = [0; 1024];

    // Read data from the connection
    let n = stream.read(&mut buffer).await?;

    // Deserialize the received data into a Message
    let message: Message = serde_json::from_slice(&buffer[0..n])?;

    // Process the message
    match message {
        Message::RequestBlockchain => {
            let blockchain_data = blockchain.lock().await;
            let serialized_blockchain = serde_json::to_string(&*blockchain_data)?;
            stream.write_all(serialized_blockchain.as_bytes()).await?;
        }
        Message::SendBlockchain(blocks) => {
            let mut blockchain_data = blockchain.lock().await;
            blockchain_data.chain = blocks;
        }
        Message::NewTransaction(transaction) => {
            let mut blockchain_data = blockchain.lock().await;
            blockchain_data.add_transaction(transaction); // Add the new transaction to the blockchain
        }
    }

    // Close the connection or handle other messages as required.
    // For simplicity, this example closes the connection after processing one message.
    Ok(())
}
