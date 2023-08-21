use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::transaction::Transaction;

const DIFFICULTY: usize = 4; // Adjust to your desired difficulty

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub index: u32,
    pub timestamp: i64,
    pub nonce: u32,
    pub previous_hash: String,
    pub hash: String,
    pub transactions: Vec<Transaction>, // Assume Transaction is defined
}

impl Block {
    // Create a new instance of a Block
    pub fn new(
        index: u32,
        timestamp: i64,
        nonce: u32,
        previous_hash: String,
        transactions: Vec<Transaction>,
    ) -> Self {
        let mut block = Block {
            index,
            timestamp,
            nonce,
            previous_hash,
            hash: String::new(),
            transactions,
        };
        block.hash = block.calculate_hash();
        block
    }

    // Proof of work algorithm
    // &mut self -> mutable reference of self Block
    // &self.hash[0..] - reference slice - checking leading zeros
    // icrement nonce and calculate the hash again
    pub fn mine_block(&mut self) {
        while &self.hash[0..DIFFICULTY] != "0".repeat(DIFFICULTY).as_str() {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block mined: {}", self.hash);
    }

    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{:?}",
            self.index, self.timestamp, self.previous_hash, self.nonce, self.transactions
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        hex::encode(result) // Using the hex crate to convert the hash to a hexadecimal string
    }

    // Other methods like mining can be added here
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_hash_changes_with_nonce() {
        let mut block = Block::new(0, 0, 0, String::from("0"), Vec::new());
        let original_hash = block.hash.clone();
        block.nonce = 1;
        assert_ne!(original_hash, block.calculate_hash());
    }

    // Add more tests for the block...
}
