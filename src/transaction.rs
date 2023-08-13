#[derive(Clone, Debug)]
pub struct Transaction {
    // Define the fields of a transaction here
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    // Add any other fields that make sense for your use case
}

impl Transaction {
    // You can define methods related to the transaction here
}
