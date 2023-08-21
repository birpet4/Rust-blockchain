use serde::{Deserialize, Serialize};

use crate::{block::Block, transaction::Transaction};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    RequestBlockchain,
    SendBlockchain(Vec<Block>),
    NewTransaction(Transaction),
    // ... other message types
}
