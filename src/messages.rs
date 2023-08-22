use serde::{Deserialize, Serialize};

use crate::{block::Block, transaction::Transaction};

// Defines the different types of messages that can be sent over the network (e.g., requesting the blockchain, sending the blockchain, creating a new transaction)
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    RequestBlockchain,
    SendBlockchain(Vec<Block>),
    NewTransaction(Transaction),
    BroadcastTransaction(Transaction),
    BroadcastBlock(Block)
    // ... other message types
}
