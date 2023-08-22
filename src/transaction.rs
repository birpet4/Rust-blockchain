use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    // Define the fields of a transaction here
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    // Add any other fields that make sense for your use case
}

impl Transaction {
    pub fn verify(&self) -> bool {
        // Check if amount is positive
        if self.amount <= 0.0 {
            return false;
        }

        // Check if sender and receiver are not empty and are different
        if self.sender.is_empty() || self.receiver.is_empty() || self.sender == self.receiver {
            return false;
        }

        // TODO: Add more checks if needed.

        true
    }
}
