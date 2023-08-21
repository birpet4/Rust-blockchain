use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{block::Block, transaction::Transaction};
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    difficulty: usize,
}

impl Blockchain {
    // Creates a blockchain with a genesis block
    pub fn new() -> Self {
        let mut chain = Vec::new();
        let genesis_block = Blockchain::create_genesis_block();
        chain.push(genesis_block);

        Blockchain {
            chain,
            pending_transactions: vec![],
            difficulty: 4,
        }
    }

    fn create_genesis_block() -> Block {
        // Define the genesis block with index 0 and a hardcoded previous hash
        Block::new(0, 0, 0, String::from("0"), Vec::new())
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        // TODO: Add validation
        self.pending_transactions.push(transaction);
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let previous_block = self.chain.last().unwrap();
        let index = previous_block.index + 1;
        let timestamp = Utc::now().timestamp();
        let nonce = 0;
        let previous_hash = previous_block.hash.clone();
        let mut block = Block::new(index, timestamp, nonce, previous_hash, transactions);

        block.mine_block(); // Mine the block using proof of work

        self.chain.push(block);
    }

    pub fn is_chain_valid(&self) -> bool {
        for (i, current_block) in self.chain[1..].iter().enumerate() {
            let previous_block = &self.chain[i];
            if current_block.hash != current_block.calculate_hash()
                || current_block.previous_hash != previous_block.hash
            {
                return false;
            }
        }
        true
    }

    pub fn from(&self, blocks: Vec<Block>) -> Self {
        Blockchain {
            chain: blocks,
            difficulty: self.difficulty,
            pending_transactions: self.pending_transactions,
        }
    }

    // You can add other methods like mining, resolving conflicts, etc., here
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_block() {
        let mut blockchain = Blockchain::new();
        let transactions = Vec::new(); // Define some transactions...

        let original_length = blockchain.chain.len();
        blockchain.add_block(transactions);

        assert_eq!(blockchain.chain.len(), original_length + 1);
    }

    #[test]
    fn test_is_chain_valid() {
        let mut blockchain = Blockchain::new();
        let transactions = Vec::new(); // Define some transactions...

        blockchain.add_block(transactions);
        assert!(blockchain.is_chain_valid());

        // Tamper with the chain
        blockchain.chain[1].hash = String::from("tampered");
        assert!(!blockchain.is_chain_valid());
    }

    // Add more tests for the blockchain...
}
