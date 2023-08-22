use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{block::Block, transaction::Transaction};

// Manages the entire chain of blocks, adding new blocks, validating the chain, handling transactions, etc...
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub peers: Vec<String>,  // Add this
    pub pending_transactions: Vec<Transaction>,
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
            peers: vec![],
            pending_transactions: vec![],
            difficulty: 4,
        }
    }

    fn create_genesis_block() -> Block {
        // Define the genesis block with index 0 and a hardcoded previous hash
        Block::new(0, 0, 0, String::from("0"), Vec::new())
    }

    /// Validates a list of transactions.
    pub fn validate_transactions(&self, transactions: &[Transaction]) -> bool {
        for tx in transactions {
            if !self.validate_transaction(tx) {
                return false;
            }
        }
        true
    }

    /// Validates a single transaction (e.g., by checking its signature).
    /// This is a stub and would need a lot more logic based on your blockchain's rules.
    pub fn validate_transaction(&self, transaction: &Transaction) -> bool {
        // Assume the Transaction struct has a verify method that checks its signature.
        // This is just an example, your actual verification might look different.
        transaction.verify()
    }

    /// Check if a block's hash meets the difficulty requirement for mining.
    pub fn is_valid_proof(&self, block: &Block) -> bool {
        let num_zeros = self.get_difficulty();  // Example: return 4 for "0000"
        let prefix = "0".repeat(num_zeros);
        block.hash.starts_with(&prefix)
    }

    /// Get the current mining difficulty. This could be a static value or dynamic based on blockchain length or other factors.
    pub fn get_difficulty(&self) -> usize {
        // Just a static example, you might adjust this based on your blockchain's needs.
        4
    }
    pub fn add_transaction(&mut self, transaction: Transaction) {
        // TODO: Add validation
        self.pending_transactions.push(transaction);
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<(), &'static str> {
        // Validate transactions (assuming you've a function for that)
        if !self.validate_transactions(&transactions) {
            return Err("Invalid transactions");
        }
    
        let previous_block = self.chain.last().unwrap();
        let index = previous_block.index + 1;
        let timestamp = Utc::now().timestamp();
        let nonce = 0;
        let previous_hash = previous_block.hash.clone();
        let mut block = Block::new(index, timestamp, nonce, previous_hash, transactions);
    
        block.mine_block();
    
        // Check if mined block's hash meets the difficulty requirement
        if !self.is_valid_proof(&block) {
            return Err("Block did not meet difficulty requirement");
        }
    
        self.chain.push(block);
        Ok(())
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
            peers: self.peers.clone(),
            pending_transactions: self.pending_transactions.clone(),
        }
    }
    // Assuming you might have a method to calculate balance of an address
    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;
        for block in &self.chain {
            for trans in &block.transactions {
                if trans.sender == address {
                    balance -= trans.amount;
                }
                if trans.receiver == address {
                    balance += trans.amount;
                }
            }
        }
        balance
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
